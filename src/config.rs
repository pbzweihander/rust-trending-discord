use crate::{Error, Repo};

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub redis_url: String,
    pub webhook_url: String,
    pub post_ttl: usize,
    pub fetch_interval: usize,
    pub post_interval: usize,
    pub blacklist: Blacklist,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Blacklist {
    pub names: Vec<String>,
    pub authors: Vec<String>,
}

impl Config {
    pub fn from_file(filename: &str) -> Result<Self, Error> {
        let mut settings = configc::Config::default();
        settings.merge(configc::File::with_name(filename))?;
        Ok(settings.try_into()?)
    }
}

impl Blacklist {
    pub fn is_listed(&self, repo: &Repo) -> bool {
        self.authors.iter().any(|a| &repo.author == a) || self.names.iter().any(|n| &repo.name == n)
    }
}
