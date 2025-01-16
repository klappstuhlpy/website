use duplicate::duplicate_item;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};

#[allow(clippy::upper_case_acronyms)]
#[duplicate_item(
Name;
[ IMG ];
[ WEB ];
[ DEF ];
)]
pub struct Name;

#[duplicate_item(
Host    Name;
[ IMG ] [ "cdn.klappstuhl.me" | "cdn.localhost:8000" | "cdn.127.0.0.1:8000" ];
[ WEB ] [ "www.klappstuhl.me" | "klappstuhl.me" | "localhost:8000" | "127.0.0.1:8000" ];
[ DEF ] [ _ ];
)]
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Host {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("host") {
            None => Outcome::Failure((Status::BadRequest, ())),
            Some(Name) => Outcome::Success(Host),
            #[allow(unreachable_patterns)] // DEF
            Some(_) => Outcome::Forward(Default::default()),
            // Some(host) => { // Debug
            //     println!("Host: {host}");
            //     Outcome::Forward(())
            // }
        }
    }
}
