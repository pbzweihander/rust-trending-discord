use crate::Error;
use futures::{future::result, prelude::*};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
pub struct Repo {
    pub author: String,
    pub description: String,
    pub forks: usize,
    pub name: String,
    pub stars: usize,
    pub url: String,
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} ★{}",
            if self.author != self.name {
                format!("{} / {}", self.author, self.name)
            } else {
                self.name.clone()
            },
            self.description,
            self.stars
        )
    }
}

pub fn fetch_repos() -> impl Future<Item = Vec<Repo>, Error = Error> {
    let con = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder().build(con);

    let req =
        Request::get("https://github-trending-api.now.sh/repositories?language=rust&since=daily")
            .body(Body::empty())
            .unwrap();
    let resp = client.request(req);

    resp.and_then(|resp| resp.into_body().concat2())
        .map_err(Into::into)
        .and_then(|body| result(serde_json::from_slice(&body).map_err(Into::into)))
}
