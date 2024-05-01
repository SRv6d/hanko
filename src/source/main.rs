use crate::SshPublicKey;
use async_trait::async_trait;
use std::collections::HashMap;

/// A source implements a way to get public keys from a Git provider.
#[async_trait]
pub trait Source {
    /// Get a users public keys by their username.
    async fn get_keys_by_username(
        &self,
        username: &str,
        client: &reqwest::Client,
    ) -> Result<Vec<SshPublicKey>, reqwest::Error>;
}

/// A `HashMap` containing named sources.
pub type SourceMap = HashMap<String, Box<dyn Source>>;
