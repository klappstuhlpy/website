#![allow(unused_imports)]

use crate::hosts::{WEB};

use rocket::fs::NamedFile;

#[get("/", rank = 0)]
pub async fn index(_host: WEB) -> Option<NamedFile> {
    /// Serve the index.html file from the frontend directory
    /// This is the main page of the website
    ///
    /// # Arguments
    /// * `_host` - The host header from the request
    ///
    /// # Returns
    /// * `Option<NamedFile>` - The index.html file
    NamedFile::open("templates/index.html").await.ok()
}

#[get("/favicon.ico", rank = 0)]
pub async fn favicon(_host: WEB) -> NamedFile {
    NamedFile::open("static/img/favicon.ico").await.ok().unwrap()
}

#[get("/projects", rank = 0)]
pub async fn projects(_host: WEB) -> Option<NamedFile> {
    /// Serve the projects.html file from the frontend directory
    /// This is the projects page of the website
    ///
    /// # Arguments
    /// * `_host` - The host header from the request
    ///
    /// # Returns
    /// * `Option<NamedFile>` - The projects.html file
    NamedFile::open("templates/projects.html").await.ok()
}
