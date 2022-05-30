use reqwest::StatusCode;
use serde::Deserialize;

pub const API_SERVER_BASE: &str = "http://localhost:8081";

pub struct Fic {
    pub id: i64,
    pub title: String,
    pub url: String,
}

pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
struct URLs {
    urls: Vec<String>,
}

pub async fn get_fics() -> Vec<Fic> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/{}", API_SERVER_BASE, "v1/urls"))
        .send()
        .await
        .unwrap();
    let status = res.status();
    let body = res.text().await.unwrap();
    eprintln!("got: {:#?}\n{:#?}", status, body);
    match status {
        StatusCode::OK => {
            let urls: URLs = serde_json::from_str(&body).unwrap();
            let urls = urls.urls;
            urls.iter()
                .map(|u| Fic {
                    id: 0,
                    title: u.clone(),
                    url: u.clone(),
                })
                .collect()
        }
        StatusCode::FORBIDDEN => vec![],
        _ => vec![],
    }
}

#[derive(Deserialize)]
struct Tags {
    tags: Vec<String>,
}

pub async fn get_tags() -> Vec<Tag> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/{}", API_SERVER_BASE, "v1/tags"))
        .send()
        .await
        .unwrap();
    let status = res.status();
    let body = res.text().await.unwrap();
    eprintln!("got: {:#?}\n{:#?}", status, body);
    match status {
        StatusCode::OK => {
            let tags: Tags = serde_json::from_str(&body).unwrap();
            let tags = tags.tags;
            tags.iter()
                .map(|t| Tag {
                    id: 0,
                    name: t.clone(),
                })
                .collect()
        }
        StatusCode::FORBIDDEN => vec![],
        _ => vec![],
    }
}
