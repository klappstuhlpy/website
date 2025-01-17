#![allow(unused_doc_comments)]

#[macro_use]
extern crate rocket;

use crate::routes::main;
use crate::routes::images;

use rocket::fs::FileServer;
use rocket::routes;

use rocket_db_pools::Database;

mod hosts;
mod routes;
mod errors;


#[derive(Database)]
#[database("website")]
pub struct DB(rocket_db_pools::sqlx::PgPool);


#[rocket::main]
async fn main() {
    /// Initialize the Rocket configuration
    ///
    /// Additionaly use a custom config to set the file size limit to 5MB (5 * 1024 * 1024 bytes)
    /// This needs to be done to prevent 413 errors when uploading large files to the databse
    rocket::build()
        .mount("/", routes![
            main::index, main::projects,
            images::index, images::favicon, images::upload, images::gallery,
            images::delete
        ])
        .mount("/templates", FileServer::from("templates"))
        .mount("/static/img", FileServer::from("static/img"))
        .register("/", catchers![errors::catch_all_errors, errors::not_found])
        .attach(DB::init())
        .launch()
        .await
        .expect("Rocket failed to launch :/");
}