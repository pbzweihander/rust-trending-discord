extern crate chrono;
extern crate futures;
extern crate redis;

use self::chrono::prelude::*;
use self::futures::future::{ok, Either};
use self::futures::prelude::*;

use crate::{Config, Error, Repo};

#[derive(Clone, Debug)]
pub struct Storage {
    config: Config,
    client: redis::Client,
}

impl Storage {
    pub fn from_config(config: &Config) -> Result<Self, Error> {
        let client = redis::Client::open(config.redis_url.as_ref())?;

        Ok(Storage {
            config: config.clone(),
            client,
        })
    }

    pub fn mark_repo_as_posted(
        &self,
        repo: &Repo,
        timestamp: DateTime<Utc>,
    ) -> impl Future<Item = (), Error = Error> {
        let repo_name = repo.name.clone();
        let post_ttl = self.config.post_ttl;
        let ts = timestamp.timestamp();
        self.client
            .get_async_connection()
            .and_then(move |con| {
                let repo_name1 = repo_name.clone();
                redis::cmd("SETNX")
                    .arg(repo_name1)
                    .arg(ts)
                    .query_async::<_, usize>(con)
                    .and_then(move |(con, val)| {
                        let repo_name2 = repo_name.clone();
                        if val == 1 {
                            Either::A(
                                redis::cmd("EXPIRE")
                                    .arg(repo_name2)
                                    .arg(post_ttl)
                                    .query_async::<_, usize>(con)
                                    .map(|_| ()),
                            )
                        } else {
                            Either::B(ok(()))
                        }
                    })
            })
            .map_err(Into::<Error>::into)
            .map_err(|e| e.context("storage error").into())
    }

    pub fn is_repo_already_posted(&self, repo: &Repo) -> impl Future<Item = bool, Error = Error> {
        let repo_name = repo.name.clone();
        self.client
            .get_async_connection()
            .and_then(move |con| {
                redis::cmd("EXISTS")
                    .arg(repo_name)
                    .query_async::<_, bool>(con)
            })
            .map(|(_, b)| b)
            .map_err(Into::<Error>::into)
            .map_err(|e| e.context("storage error").into())
    }
}
