use crate::SshPublicKey;
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

/// A source implements a way to get public keys from a Git provider.
#[async_trait]
pub trait Source {
    /// Get a users public keys by their username.
    async fn get_keys_by_username(
        &self,
        username: &str,
        client: &reqwest::Client,
    ) -> Result<Vec<SshPublicKey>>;
}

/// A `HashMap` containing named sources.
pub type SourceMap = HashMap<String, Box<dyn Source>>;

/// An error that can occur when interacting with a source.
#[derive(Error, Debug)]
pub enum SourceError {
    #[error("The requested resource was not found.")]
    NotFound,
    #[error("The used credentials are invalid.")]
    BadCredentials,
    #[error("The rate limit has been exceeded.")]
    RatelimitExceeded,
    #[error("A connection error occurred.")]
    ConnectionError,
    #[error("An unknown request error occurred.")]
    Other(reqwest::Error),
}

/// A `Result` alias where the `Err` case is a `SourceError`.
pub type Result<T> = std::result::Result<T, SourceError>;
