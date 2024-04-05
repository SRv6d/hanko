use crate::USER_AGENT;
use reqwest::{Client, Result};
use serde::Deserialize;
use std::fmt;

const GH_API_VERSION: &str = "2022-11-28";

#[derive(Debug, Deserialize)]
struct SshSigningKey {
    key: String,
}

impl fmt::Display for SshSigningKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

async fn get_signing_keys(user: &str, client: Client) -> Result<Vec<SshSigningKey>> {
    let url = format!("https://api.github.com/users/{}/ssh_signing_keys", user);
    let request = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", GH_API_VERSION);

    let response = request.send().await?;
    response.json().await
}
