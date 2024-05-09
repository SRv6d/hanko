use crate::SshPublicKey;
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

/// A `Result` alias where the `Err` case is a `SourceError`.
pub type Result<T> = std::result::Result<T, SourceError>;

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
    #[error("The used credentials are invalid.")]
    BadCredentials,
    #[error("The rate limit has been exceeded.")]
    RatelimitExceeded,
    #[error("The requested user could not be found.")]
    UserNotFound,
    #[error("A connection error occurred.")]
    ConnectionError,
    #[error("An unknown request error occurred.")]
    Other(Box<dyn std::error::Error>),
}

/// A fallback conversion for generic reqwest errors.
impl From<reqwest::Error> for SourceError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_connect() || error.is_timeout() {
            SourceError::ConnectionError
        } else {
            SourceError::Other(Box::new(error))
        }
    }
}
