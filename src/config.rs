use crate::{GitProvider, SshPublicKey};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The main configuration.
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    users: Vec<User>,
    organizations: Vec<Organization>,
    local: Vec<SshPublicKey>,
    sources: Vec<Source>,
}

impl Config {
    /// Load the configuration from a TOML file at the given path.
    fn load(path: PathBuf) -> figment::Result<Self> {
        Figment::from(Toml::file(path)).extract()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    name: String,
    sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Organization {
    name: String,
    sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Source {
    name: String,
    provider: GitProvider,
    url: String,
}
