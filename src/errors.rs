use std::fmt::Display;
use std::io::Cursor;
use rocket::http::{ContentType, Status};
use rocket::{Request, Response};
use rocket::fs::NamedFile;
use rocket::response::{Responder, Result as ResponderResult};
use serde_json::json;

#[derive(Debug, Responder)]
pub enum NotFoundResponse {
    #[response(status = 404)]
    Html(NamedFile),
    #[response(status = 404)]
    Json(CustomError),
}

#[catch(404)]
pub async fn not_found(req: &Request<'_>) -> NotFoundResponse {
    /// Check the Accept header to determine the response type
    /// If the header contains "text/html" return a HTML response
    /// Otherwise return a JSON response

    let content_type = req.headers().get_one("Accept").unwrap_or("text/html").to_string();

    match content_type.contains("text/html") {
        true => {
            NotFoundResponse::Html(NamedFile::open("templates/404.html").await.ok().expect("404.html not found :/"))
        }
        false => {
            NotFoundResponse::Json(CustomError(Status::NotFound, "Not Found".to_string()))
        }
    }
}

#[catch(default)]
pub async fn catch_all_errors(status: Status, _req: &Request<'_>) -> CustomError {
    CustomError(status, status.reason().unwrap_or("Unknown Error").to_string())
}


#[derive(Debug, Clone)]
pub struct CustomError(pub Status, pub String);

impl CustomError {
    #[allow(dead_code)]
    pub fn new(status: Status, message: String) -> Self {
        CustomError(status, message.to_string())
    }
}

impl Display for CustomError {
    /// Formats the value using the given formatter.
    ///
    /// self.0 is the status code
    /// self.1 is the error message

    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "Error {}: {}", self.0, self.1)
    }
}

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, _req: &'r Request<'_>) -> ResponderResult<'static> {
        let response = json!({
            "error": {
                "code": self.0.code,
                "message": self.1
            }
        }
        ).to_string();

        Response::build()
            .status(self.0)
            .header(ContentType::JSON)
            .sized_body(response.len(), Cursor::new(response))
            .ok()
    }
}