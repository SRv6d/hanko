use crate::{GitProvider, SshPublicKey};
use serde::{Deserialize, Serialize};

/// The main configuration.
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    users: Vec<User>,
    organizations: Vec<Organization>,
    local: Vec<SshPublicKey>,
    sources: Vec<Source>,
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
