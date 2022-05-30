use rocket::http::CookieJar;
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{Responder, Response, Result};
use std::io::Cursor;

// TODO this won't be needed when askama_rocket updates :/
pub struct Template<T: askama::Template> {
    template: T,
}

impl<T: askama::Template> From<T> for Template<T> {
    fn from(t: T) -> Self {
        Self { template: t }
    }
}

#[rocket::async_trait]
impl<'r, T: askama::Template> Responder<'r, 'static> for Template<T> {
    fn respond_to(self, _: &'r Request<'_>) -> Result<'static> {
        let rsp = self
            .template
            .render()
            .map_err(|_| Status::InternalServerError)?;
        let ctype = ContentType::from_extension("html").ok_or(Status::InternalServerError)?;
        Response::build()
            .header(ctype)
            .sized_body(rsp.len(), Cursor::new(rsp))
            .ok()
    }
}

pub struct HostHeader<'a>(pub &'a str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HostHeader<'r> {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Host") {
            Some(h) => Outcome::Success(HostHeader(h)),
            None => Outcome::Forward(()),
        }
    }
}

pub fn get_theme<'a>(host: Option<HostHeader<'a>>, cookies: &CookieJar<'_>) -> String {
    if let Some(cookie) = cookies.get("theme") {
        if cookie.value() == "light" || cookie.value() == "night" {
            return cookie.value().into();
        }
    }
    match host {
        Some(host) => {
            if host.0.starts_with("light.") || host.0.starts_with("day.") {
                "light"
            } else if host.0.starts_with("night.") || host.0.starts_with("dark.") {
                "night"
            } else {
                ""
            }
        }
        None => "",
    }
    .into()
}
