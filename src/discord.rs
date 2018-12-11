use crate::Error;
use futures::prelude::*;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;

#[derive(Clone, Serialize, Debug)]
pub struct Footer {
    pub text: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub icon_url: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct Embed {
    pub title: String,
    pub description: String,
    pub url: String,
    pub footer: Footer,
    pub author: Author,
}

#[derive(Clone, Serialize, Debug)]
pub enum Content {
    #[serde(rename = "content")]
    Content(String),
    #[serde(rename = "embeds")]
    Embeds(Vec<Embed>),
}

#[derive(Clone, Serialize, Debug)]
pub struct Webhook {
    #[serde(flatten)]
    pub content: Content,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub tts: Option<bool>,
}

pub fn execute_webhook(url: &str, webhook: &Webhook) -> impl Future<Item = String, Error = Error> {
    let con = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder().build(con);

    let body = serde_json::to_string_pretty(webhook).unwrap();

    let req = Request::post(url)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = client.request(req);

    resp.and_then(|resp| resp.into_body().concat2())
        .map_err(Into::into)
        .map(|chunk| String::from_utf8_lossy(&chunk.into_bytes()).to_string())
}
