#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate futures;

pub use failure::Error;

pub mod config;
mod discord;
mod repo;
mod storage;

pub use crate::config::Config;
use crate::{
    repo::{fetch_repos, Repo},
    storage::Storage,
};

use chrono::Local;
use chrono::{DateTime, Utc};
use futures::{future::ok, prelude::*, stream::iter_ok};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::timer::{Delay, Interval};

fn err_log(e: &Error) {
    eprintln!("At {}", Local::now());
    eprintln!("Error: {}", e);
    eprintln!("Error chain:");
    for c in e.iter_chain() {
        eprintln!("- {}", c);
    }
}

struct TimedStream<S, E>
where
    S: Stream<Error = E>,
    E: From<tokio::timer::Error>,
{
    delay: Delay,
    interval: Duration,
    inner: S,
}

impl<S, E> TimedStream<S, E>
where
    S: Stream<Error = E>,
    E: From<tokio::timer::Error>,
{
    pub fn new(stream: S, at: Instant, interval: Duration) -> Self {
        TimedStream {
            delay: Delay::new(at),
            interval,
            inner: stream,
        }
    }

    pub fn new_interval(stream: S, interval: Duration) -> Self {
        Self::new(stream, Instant::now() + interval, interval)
    }
}

impl<S, E> Stream for TimedStream<S, E>
where
    S: Stream<Error = E>,
    E: From<tokio::timer::Error>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use futures::Async;

        try_ready!(self.delay.poll().map_err(Into::into));

        match self.inner.poll() {
            Ok(Async::Ready(t)) => {
                self.delay.reset(Instant::now() + self.interval);
                Ok(Async::Ready(t))
            }
            other => other,
        }
    }
}

fn post_repo(repo: &Repo, webhook_url: &str) -> impl Future<Item = DateTime<Utc>, Error = Error> {
    use crate::discord::*;

    let webhook = Webhook {
        content: Content::Embeds(vec![Embed {
            title: repo.name.clone(),
            description: repo.description.clone(),
            url: repo.url.clone(),
            footer: Footer {
                text: format!("★{}", repo.stars),
            },
            author: Author {
                name: repo.author.clone(),
                url: format!("https://github.com/{}", repo.author),
                icon_url: format!("https://github.com/{}.png", repo.author),
            },
        }]),
        username: None,
        avatar_url: None,
        tts: None,
    };

    execute_webhook(webhook_url, &webhook).map(|_| Utc::now())
}

pub struct RustTrending {
    config: Config,
    storage: Storage,
}

impl RustTrending {
    pub fn from_config(config: Config) -> Result<Self, Error> {
        let storage = Storage::from_config(&config)?;

        Ok(RustTrending { config, storage })
    }

    pub fn run_loop(self) -> impl Future<Item = (), Error = Error> {
        let fetch_interval = Duration::from_secs(self.config.fetch_interval as u64);
        let post_interval = Duration::from_secs(self.config.post_interval as u64);
        let storage = Arc::new(self.storage);
        let storage1 = storage.clone();
        let webhook_url = Arc::new(self.config.webhook_url + "?wait=true");
        let blacklist = Arc::new(self.config.blacklist);

        let fetch_stream = Interval::new(Instant::now(), fetch_interval)
            .map(move |_| {
                let storage = storage.clone();
                let blacklist = blacklist.clone();
                fetch_repos()
                    .map(iter_ok)
                    .flatten_stream()
                    .and_then(move |r| storage.is_repo_already_posted(&r).map(|b| (r, b)))
                    .filter(|(_, is_repo_already_posted)| !is_repo_already_posted)
                    .map(|(r, _)| r)
                    .filter(move |r| {
                        let blacklist = blacklist.clone();
                        !blacklist.is_listed(&r)
                    })
            })
            .flatten()
            .map_err(|e| e.context("Fetch stream error").into());

        TimedStream::new(fetch_stream, Instant::now(), post_interval)
            .for_each(move |r| {
                let storage = storage1.clone();
                let r1 = r.clone();
                let r2 = r.clone();

                post_repo(&r, &webhook_url)
                    .and_then(move |ts| storage.mark_repo_as_posted(&r1, ts).map(move |_| ts))
                    .map(move |ts| {
                        println!("{}, posted {} - {}", ts, r2.author, r2.name);
                    })
            })
            .map_err(|e| e.context("Post stream error").into())
            .or_else(|e| {
                err_log(&e);
                ok(())
            })
    }
}
