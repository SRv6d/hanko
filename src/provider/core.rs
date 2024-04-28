use super::{github::Github, gitlab::Gitlab};
use crate::SshPublicKey;

/// A Git provider.
#[derive(Debug)]
pub enum GitProvider<'a> {
    Github(Github<'a>),
    Gitlab(Gitlab<'a>),
}

impl GitProvider<'_> {
    /// Get the public keys of a user by their username.
    async fn get_keys_by_username(
        &self,
        username: &str,
    ) -> Result<Vec<SshPublicKey>, reqwest::Error> {
        match self {
            Self::Github(provider) => provider.get_keys_by_username(username).await,
            Self::Gitlab(provider) => provider.get_keys_by_username(username).await,
        }
    }
}
