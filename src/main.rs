use askama::Template;
use reqwest::StatusCode;
use rocket::form::{Form, FromForm};
use rocket::fs::NamedFile;
use rocket::http::{Cookie, CookieJar};
use rocket::request::{FlashMessage, FromRequest, Outcome, Request};
use rocket::response::{self, Flash, Redirect, Responder, Response};
use rocket::{catch, get, post, State};
use serde::Serialize;
use std::path::{Path, PathBuf};

mod api;
mod template;

use api::{get_fics, get_tags, Fic, Tag, API_SERVER_BASE};

const SESSION_COOKIE_NAME: &str = "FicAiSession";

struct Config {
    domain: &'static str,
}

struct User {
    uid: i64,
}

#[derive(Template)]
#[template(path = "404.html")]
struct FileNotFoundTemplate {
    domain: String,
    theme: String,
}

#[derive(Template)]
#[template(path = "index.html")] // , print = "all"
struct IndexTemplate {
    domain: String,
    theme: String,
    flash_message: Option<String>,
    user: Option<User>,
}

#[derive(Template)]
#[template(path = "fics/index.html")]
struct FicIndexTemplate {
    domain: String,
    theme: String,
    user: Option<User>, // logged in user

    fics: Vec<Fic>,
}

#[derive(Template)]
#[template(path = "fics/template.html")]
struct FicTemplate {
    domain: String,
    theme: String,
    flash_message: Option<String>,

    fic: Fic,
}

#[derive(Template)]
#[template(path = "tags/index.html")]
struct TagIndexTemplate {
    domain: String,
    theme: String,
    flash_message: Option<String>,

    user: Option<User>, // logged in user
    tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "tags/template.html")]
struct TagTemplate {
    domain: String,
    theme: String,

    tag: Tag,
}

#[catch(404)]
async fn not_found<'a>(req: &Request<'_>) -> template::Template<FileNotFoundTemplate> {
    let host = match Option::<template::HostHeader<'_>>::from_request(req).await {
        Outcome::Success(host) => host,
        _ => None,
    };
    // note: always return succesfully
    let cookies = <&CookieJar>::from_request(req).await.unwrap();
    let domain = match req.guard::<&State<Config>>().await {
        Outcome::Success(state) => state.domain,
        _ => "fic.ai",
    }
    .to_string();
    FileNotFoundTemplate {
        domain,
        theme: template::get_theme(host, cookies),
    }
    .into()
}

#[get("/")]
async fn index(
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
    flash: Option<FlashMessage<'_>>,
) -> template::Template<IndexTemplate> {
    IndexTemplate {
        domain: config.domain.to_string(),
        theme: template::get_theme(host, cookies),
        flash_message: flash.map(|flash| format!("{}: {}", flash.kind(), flash.message())),
        user: get_user(cookies).await,
    }
    .into()
}

#[get("/fics")]
async fn fics_index(
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
) -> template::Template<FicIndexTemplate> {
    FicIndexTemplate {
        domain: config.domain.to_string(),
        theme: template::get_theme(host, cookies),
        user: get_user(cookies).await,
        fics: get_fics().await,
    }
    .into()
}

#[get("/fics/<fic_id>")]
async fn fics_by_id(
    fic_id: i64,
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Option<template::Template<FicTemplate>> {
    fics_by_id_with_slug(fic_id, "".to_string(), config, host, cookies, flash).await
}

#[get("/fics/<_fic_id>/<_fic_slug>")]
async fn fics_by_id_with_slug(
    _fic_id: i64,
    _fic_slug: String,
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Option<template::Template<FicTemplate>> {
    Some(
        FicTemplate {
            domain: config.domain.to_string(),
            theme: template::get_theme(host, cookies),
            flash_message: flash.map(|flash| format!("{}: {}", flash.kind(), flash.message())),

            fic: Fic {
                id: 1,
                title: "test title".to_string(),
                url: "https://example.com".to_string(),
            },
        }
        .into(),
    )
}

#[get("/tags")]
async fn tags_index(
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
    flash: Option<FlashMessage<'_>>,
) -> template::Template<TagIndexTemplate> {
    TagIndexTemplate {
        domain: config.domain.to_string(),
        theme: template::get_theme(host, cookies),
        flash_message: flash.map(|flash| format!("{}: {}", flash.kind(), flash.message())),
        user: get_user(cookies).await,

        tags: get_tags().await,
    }
    .into()
}

#[get("/tags/<tag_id>")]
async fn tags_by_id(
    tag_id: i64,
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
) -> Option<template::Template<TagTemplate>> {
    tags_by_id_with_slug(tag_id, "".to_string(), config, host, cookies).await
}

#[get("/tags/<_tag_id>/<_tag_slug>")]
async fn tags_by_id_with_slug(
    _tag_id: i64,
    _tag_slug: String,
    config: &State<Config>,
    host: Option<template::HostHeader<'_>>,
    cookies: &CookieJar<'_>,
) -> Option<template::Template<TagTemplate>> {
    Some(
        TagTemplate {
            domain: config.domain.to_string(),
            theme: template::get_theme(host, cookies),

            tag: Tag {
                id: 1,
                name: "test_tag".to_string(),
            },
        }
        .into(),
    )
}

struct CachedFile(NamedFile);

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request) -> response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", "max-age=86400") //  24h (24*60*60)
            .ok()
    }
}

#[get("/<file..>")]
async fn static_files(file: PathBuf) -> Option<CachedFile> {
    NamedFile::open(Path::new("static/").join(file))
        .await
        .ok()
        .map(CachedFile)
}

fn create_session_cookie(session_id: String, domain: &'static str) -> Cookie {
    Cookie::build(SESSION_COOKIE_NAME, session_id)
        .domain(domain)
        .path("/")
        .secure(true)
        .http_only(true)
        .permanent()
        .finish()
}

async fn clear_session(cookies: &CookieJar<'_>, domain: &'static str) {
    if cookies.get(SESSION_COOKIE_NAME).is_some() {
        cookies.remove(
            Cookie::build(SESSION_COOKIE_NAME, "")
                .domain(domain)
                .path("/")
                .finish(),
        );
    }
}

async fn set_session(cookies: &CookieJar<'_>, session_id: String, domain: &'static str) {
    cookies.add(create_session_cookie(session_id, domain));
}

async fn get_user(cookies: &CookieJar<'_>) -> Option<User> {
    cookies.get(SESSION_COOKIE_NAME).map(|_c| User { uid: 0 })
}

// TODO this should probably be a POST...
#[get("/h0/log_out")]
async fn h0_log_out(config: &State<Config>, cookies: &CookieJar<'_>) -> Redirect {
    clear_session(cookies, config.domain).await;
    Redirect::to("/")
}

#[derive(Debug, FromForm, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterForm {
    email: String,
    password: String,
    #[field(name = "betaKey")]
    beta_key: String,
}

#[post("/h0/register", data = "<q>")]
async fn h0_register(
    q: Form<RegisterForm>,
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> Flash<Redirect> {
    clear_session(cookies, config.domain).await;
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/{}", API_SERVER_BASE, "v1/accounts"))
        .json(&q.into_inner())
        .send()
        .await
        .unwrap();
    let status = res.status();
    let session_cookie = res
        .cookies()
        .find(|c| c.name() == SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());
    let body = res.text().await.unwrap();
    eprintln!("got: {:#?}\n{:#?}", status, body);
    match status {
        StatusCode::CREATED => {
            set_session(cookies, session_cookie.unwrap(), config.domain).await;
            Flash::success(Redirect::to("/"), "Successfully registered.")
        }
        StatusCode::FORBIDDEN => Flash::error(Redirect::to("/"), "Forbidden."),
        StatusCode::CONFLICT => {
            // TODO info leakage?
            Flash::error(Redirect::to("/"), "Account already registered.")
        }
        _ => Flash::error(Redirect::to("/"), "Error."),
    }
}

#[derive(Debug, FromForm, Serialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[post("/h0/log_in", data = "<q>")]
async fn h0_log_in(
    q: Form<LoginForm>,
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> Flash<Redirect> {
    clear_session(cookies, config.domain).await;
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/{}", API_SERVER_BASE, "v1/sessions"))
        .json(&q.into_inner())
        .send()
        .await
        .unwrap();
    let status = res.status();
    let session_cookie = res
        .cookies()
        .find(|c| c.name() == SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());
    let body = res.text().await.unwrap();
    eprintln!("got: {:#?}\n{:#?}", status, body);
    match status {
        StatusCode::NO_CONTENT => {
            set_session(cookies, session_cookie.unwrap(), config.domain).await;
            Flash::success(Redirect::to("/"), "Welcome.")
        }
        StatusCode::FORBIDDEN => Flash::error(Redirect::to("/"), "Forbidden."),
        _ => Flash::error(Redirect::to("/"), "Error."),
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    use dotenv::dotenv;

    dotenv().expect("Failed to read .env file");

    //use rocket::fs::FileServer;
    use rocket::catchers;
    use rocket::routes;
    rocket::build()
        .manage(Config {
            domain: "may.fic.ai",
        })
        .register("/", catchers![not_found])
        .mount(
            "/",
            routes![
                index,
                fics_index,
                fics_by_id,
                fics_by_id_with_slug,
                tags_index,
                tags_by_id,
                tags_by_id_with_slug,
                static_files,
                h0_log_in,
                h0_log_out,
                h0_register,
            ],
        )
    //.mount("/", FileServer::from("static/"))
}
