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
    #[error("used credentials are invalid")]
    BadCredentials,
    #[error("rate limit has been exceeded")]
    RatelimitExceeded,
    #[error("requested user could not be found")]
    UserNotFound,
    #[error("connection error occurred")]
    ConnectionError,
    #[error("server error occurred")]
    ServerError(#[from] ServerError),
    #[error("client request error")]
    ClientError(reqwest::Error),
}

/// Conversion for generic reqwest errors not specific to any `Source`.
impl From<reqwest::Error> for SourceError {
    #[allow(clippy::panic)]
    fn from(error: reqwest::Error) -> Self {
        if error.is_connect() || error.is_timeout() {
            SourceError::ConnectionError
        } else if error.is_request() {
            SourceError::ClientError(error)
        } else if error.is_status()
            && error
                .status()
                .expect("missing error status code")
                .is_server_error()
        {
            ServerError::StatusCode(error.status().unwrap()).into()
        } else if error.is_body() {
            ServerError::InvalidResponseBody.into()
        } else {
            panic!("Unexpected reqwest error: {error:?}");
        }
    }
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("invalid response body")]
    InvalidResponseBody,
    #[error("{0}")]
    StatusCode(reqwest::StatusCode),
}
