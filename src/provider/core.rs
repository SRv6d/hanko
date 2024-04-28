use super::{github::Github, gitlab::Gitlab};
use crate::SshPublicKey;
use reqwest::{Client, Url};

/// A Git provider.
#[derive(Debug)]
pub enum GitProvider {
    Github(Github),
    Gitlab(Gitlab),
}

impl GitProvider {
    pub fn github(url: Url) -> Self {
        Self::Github(Github::new(url))
    }

    pub fn gitlab(url: Url) -> Self {
        Self::Gitlab(Gitlab::new(url))
    }

    /// Get the public keys of a user by their username.
    async fn get_keys_by_username(
        &self,
        username: &str,
        client: &Client,
    ) -> Result<Vec<SshPublicKey>, reqwest::Error> {
        match self {
            Self::Github(provider) => provider.get_keys_by_username(username, client).await,
            Self::Gitlab(provider) => provider.get_keys_by_username(username, client).await,
        }
    }
}
