#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(renamed_and_removed_lints)]

extern crate tree_magic;
use crate::DB;
use crate::hosts::IMG;
use crate::errors::CustomError;

use std::borrow::ToOwned;
use std::fmt::format;
use nanoid::nanoid;
use rocket::form::Form;
use rocket::fs::{NamedFile, TempFile};
use rocket::http::{ContentType, Header, Status};
use rocket::request::{FromRequest, Outcome as RequestOutcome};
use rocket::data::{self, ByteUnit, Data, FromData, Outcome as DataOutcome};
use rocket::response::{Redirect, Responder};
use rocket::{Either, Request, Response};
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection};
use std::fs::{copy, create_dir_all, File, read, remove_file};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::string::ToString;
use once_cell::sync::Lazy;
use dotenv;
use base64::{engine::general_purpose, Engine as _};
use rocket::http::private::Array;

pub struct Authorization(String);

#[derive(Debug)]
pub enum AuthorizationError {
    Missing,
    Invalid,
}

static AUTH_KEY: Lazy<String> = Lazy::new(|| { dotenv::var("AUTH_KEY").unwrap() }.to_string());

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authorization {
    /// Check if the Authorization header is present and contains the correct key
    ///
    /// # Arguments
    /// * `request` - The request to check
    ///
    /// # Returns
    /// * A RequestOutcome containing the Authorization header or an error
    /// * If the header is missing, a 401 error is returned
    /// * If the header is present but contains the wrong key, a 401 error is returned
    /// * If the header is present and contains the correct key, the Authorization header is returned
    type Error = AuthorizationError;

    async fn from_request(request: &'r Request<'_>) -> RequestOutcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(key) if key.to_string() == *AUTH_KEY => {
                RequestOutcome::Success(Authorization(key.to_owned()))
            }
            Some(_) => RequestOutcome::Failure((Status::Unauthorized, AuthorizationError::Invalid)),
            None => RequestOutcome::Failure((Status::Unauthorized, AuthorizationError::Missing)),
        }
    }
}

#[get("/", rank = 1)]
pub async fn index(_host: IMG) -> &'static str {
    "Image Database by Klappstuhl65"
}

#[derive(Debug)]
pub struct ImageResponse(Vec<u8>, String, String);

impl<'r> Responder<'r, 'static> for ImageResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let response = Response::build()
            .header(Header::new("Content-Type", self.2))
            .status(Status::Ok)
            .raw_header("Content-Disposition", "inline")
            .sized_body(self.1.len(), Cursor::new(self.1))
            .streamed_body(Cursor::new(self.0))
            .finalize();

        Ok(response)
    }
}

#[derive(Debug, Responder)]
pub enum GalleryResponse {
    ImageResponse(ImageResponse),
    Redirect(Redirect),
    Error(CustomError),
}

#[get("/gallery/<filename>?<size>", rank = 1)]
pub async fn gallery(
    mut db: Connection<DB>,
    filename: &str,
    size: Option<String>,
    _host: IMG,
) -> GalleryResponse {
    /// Get an image from the database and return it as a response
    ///
    /// # Arguments
    /// * `db` - The database connection
    /// * `filename` - The name of the image to get
    /// * `size` - The size of the image to get (Optional)
    /// * `host` - The host header
    ///
    /// # Returns
    /// * A GalleryResponse enum containing the image response, a redirect or an error
    /// * If the image could not be found in the database, a 404 error is returned
    /// * If the image could be found in the database, the image is returned
    /// * If the filename was provided without the file extension, a redirect to the correct filename is returned
    ///
    /// # Response
    /// * The image is returned as a HTML response to display the image in the browser
    /// * The image data is streamed to the client

    let split: Vec<&str> = filename.split(".").collect();
    let name = split[0];
    let resp = sqlx::query("SELECT * FROM images WHERE id = $1")
        .bind(name)
        .fetch_optional(&mut *db)
        .await;

    match resp {
        Ok(Some(row)) => {
            let content_type: &str = row.try_get("mimetype").unwrap_or("image/png");
            let ext = content_type.split("/").collect::<Vec<&str>>()[1];
            let image_bytes = row.try_get::<Vec<u8>, _>("image_data").unwrap();

            let mut resolved_size: Vec<i32> = Vec::new();

            match size {
                Some(size_parts) => {
                    let size_parts: Vec<&str> = size_parts.split('x').collect();
                    if size_parts.len() == 1 {
                        let width = size_parts[0].parse::<i32>().unwrap();
                        // Handle cases like ?size=512
                        resolved_size.push(width);
                        resolved_size.push(width);
                    } else if size_parts.len() == 2 {
                        let width = size_parts[0].parse::<i32>().unwrap();
                        let height = size_parts[1].parse::<i32>().unwrap();
                        resolved_size.push(width);
                        resolved_size.push(height);
                    }
                }
                _ => {
                    match imagesize::blob_size(&image_bytes) {
                        Ok(size) => {
                            resolved_size.push(size.width as i32);
                            resolved_size.push(size.height as i32);
                        }
                        Err(e) => {
                            return GalleryResponse::Error(CustomError(
                                Status::NotFound, format!("Failed to get Image dimensions of `{}`: {:?}", filename, e)));
                        }
                    }
                }
            }

            let final_filename = format!("{}.{}", name, ext);

            if filename != final_filename {
                let redirect_url = format!("/gallery/{}", final_filename);
                return GalleryResponse::Redirect(Redirect::to(redirect_url))
            }

            let html_content = format!(
                r#"
                <html>
                    <head>
                        <style>
                            body {{
                                background: url("chrome://global/skin/media/imagedoc-darknoise.png") fixed;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                height: 100vh;
                                margin: 0;
                            }}
                            img {{
                                max-width: 100%;
                                max-height: 100%;
                            }}
                        </style>
                    </head>
                    <body>
                        <img src="{}" alt="Image" width="{}" height="{}"/>
                    </body>
                </html>
                "#,
                format!("data:{};base64,{}", content_type, general_purpose::STANDARD.encode(&image_bytes)),
                resolved_size[0], resolved_size[1]
            );

            return GalleryResponse::ImageResponse(ImageResponse(image_bytes, html_content, content_type.into()))
        }
        _ => GalleryResponse::Error(CustomError(Status::NotFound,
            format!("Image with name `{}` could not be found.", filename))),
    }
}

fn get_id() -> String {
    /// Generate a random id for the image
    /// The id is 10 characters long and contains only lowercase and uppercase letters
    let chars: [char; 52] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
        'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    nanoid!(10, &chars)
}

#[derive(FromForm)]
pub struct Upload<'f> {
    pub file: TempFile<'f>
}

#[post("/upload", format = "multipart/form-data", data = "<form>", rank = 1)]
pub async fn upload(
    mut form: Form<Upload<'_>>,
    mut db: Connection<DB>,
    _auth: Authorization,
    _host: IMG,
) -> Result<(Status, (ContentType, String)), CustomError> {
    /// Save the image to the database and return the id and the url
    /// The id is used to access the image in the database
    /// The url is used to access the image in the browser
    ///
    /// # Arguments
    /// * `form` - The form data containing the image
    /// * `db` - The database connection
    /// * `auth` - The authorization header
    /// * `host` - The host header
    ///
    /// # Returns
    /// * A tuple containing the status code and the response
    /// * The response is a JSON object containing the id and the url of the image
    /// * If the image could not be saved to the database, a 500 error is returned
    ///
    /// # Example Response
    /// ```json
    /// {
    ///    "id": "abc123",
    ///    "url": "https://cdn.klappstuhl.me/gallery/abc123.png"
    /// }
    /// ```
    // Check if the file is too large or missing
    if form.file.len() == 0 {
        return Err(CustomError::new(Status::BadRequest, "File is empty or incomplete".to_owned()));
    }

    let id = get_id();
    let path = format!("temp/{}.{:?}", id, form.file.content_type().unwrap().extension().unwrap());

    match create_dir_all("temp") {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{:?}", e);
            return Err(CustomError(Status::InternalServerError, "Failed to create temp directory.".to_owned()))
        }
    }
    form.file.persist_to(&path)
        .await
        .expect("Something went wrong when creating tempfile.");
    let image_data = read(&path).unwrap();
    let mimetype = tree_magic::from_u8(&image_data);

    // Check if the mimetype as an image, if not return 400
    if !mimetype.starts_with("image/") {
        return Err(CustomError(
            Status::BadRequest, "The file you uploaded is not an image.".to_owned(),
        ))
    }

    let resp = sqlx::query("INSERT INTO images (id, image_data, mimetype) VALUES ($1, $2, $3);")
        .bind(id.clone())
        .bind(image_data)
        .bind(mimetype.to_string())
        .execute(&mut *db)
        .await;

    match resp {
        Ok(_) => {
            match remove_file(path) {
                _ => (),
            };
            Ok((
                Status::Ok,
                (
                    ContentType::JSON,
                    format!(
                        r#"{{"id": "{}", "url": "https://cdn.klappstuhl.me/gallery/{}.{}"}}"#,
                        id, id, mimetype.split("/").collect::<Vec<&str>>()[1]
                    ).to_owned(),
                ),
            ))
        }
        Err(e) => {
            eprintln!("{:?}", e);
            Err(CustomError(Status::InternalServerError, "Failed to insert image into database.".to_owned()))
        }
    }
}

#[delete("/delete?<id>", format = "application/json", rank = 1)]
pub async fn delete(
    mut db: Connection<DB>,
    _auth: Authorization,
    id: String,
    _host: IMG,
) -> Result<(Status, (ContentType, String)), CustomError> {
    /// Delete an image from the database
    ///
    /// # Arguments
    /// * `db` - The database connection
    /// * `auth` - The authorization header
    /// * `id` - The id of the image to delete
    ///
    /// # Returns
    /// * A tuple containing the status code and the response
    /// * The response is a JSON object containing the id of the deleted image
    /// * If the image could not be deleted from the database, a 500 error is returned
    /// * If the image could not be found in the database, a 404 error is returned
    ///
    /// # Example Response
    /// ```json
    /// {
    ///   "id": "abc123"
    /// }
    /// ```
    let id = id.split(".").collect::<Vec<&str>>()[0];

    let resp = sqlx::query("DELETE FROM images WHERE id = $1;")
        .bind(id)
        .execute(&mut *db)
        .await;

    match resp {
        Ok(_) => {
            Ok((
                Status::Ok,
                (
                    ContentType::JSON,
                    format!(r#"{{"id": "{}"}}"#, id).to_owned(),
                ),
            ))
        }
        Err(e) => {
            eprintln!("{:?}", e);
            Err(CustomError(Status::InternalServerError, "Failed to delete image from database.".to_owned()))
        }
    }
}