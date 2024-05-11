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
#[derive(Error, Debug, PartialEq, Eq)]
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
    ClientError(reqwest::StatusCode),
}

/// Conversion for generic reqwest errors not specific to any `Source`.
impl From<reqwest::Error> for SourceError {
    #[allow(clippy::panic)]
    fn from(error: reqwest::Error) -> Self {
        if error.is_connect() || error.is_timeout() {
            SourceError::ConnectionError
        } else if error.is_status()
            && error
                .status()
                .expect("missing error status code")
                .is_server_error()
        {
            ServerError::StatusCode(error.status().unwrap()).into()
        } else if error.is_status()
            && error
                .status()
                .expect("missing error status code")
                .is_client_error()
        {
            SourceError::ClientError(error.status().unwrap())
        } else if error.is_body() || error.is_decode() {
            ServerError::InvalidResponseBody.into()
        } else {
            panic!("Unexpected reqwest error: {error:?}");
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ServerError {
    #[error("invalid response body")]
    InvalidResponseBody,
    #[error("{0}")]
    StatusCode(reqwest::StatusCode),
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use proptest::prelude::*;
    use rstest::*;

    /// Returns a reqwest error caused by the given status code.
    fn reqwest_status_code_error(status: reqwest::StatusCode) -> reqwest::Error {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.any_request();
            then.status(status.into());
        });
        let error = reqwest::blocking::get(server.base_url())
            .unwrap()
            .error_for_status()
            .unwrap_err();
        assert!(error.is_status());
        error
    }

    /// A reqwest error caused by a timeout.
    #[fixture]
    fn reqwest_timeout_error() -> reqwest::Error {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.any_request();
            then.delay(std::time::Duration::from_millis(1));
        });
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::ZERO)
            .build()
            .unwrap();
        let error = client.get(server.base_url()).send().unwrap_err();
        assert!(error.is_timeout());
        error
    }

    /// A reqwest connection error.
    #[fixture]
    fn reqwest_connection_error() -> reqwest::Error {
        let error = reqwest::blocking::get("http://2001:db8:1/")
            .unwrap()
            .error_for_status()
            .unwrap_err();
        assert!(error.is_connect());
        error
    }

    /// A reqwest decode error.
    #[fixture]
    fn reqwest_decode_error() -> reqwest::Error {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.any_request();
            then.body("not what you think");
        });
        let error = reqwest::blocking::get(server.base_url())
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap_err();
        assert!(error.is_decode());
        error
    }

    prop_compose! {
        /// A 400 client error status code.
        fn status_code_400()(status in 400..499u16) -> reqwest::StatusCode {
            let code = reqwest::StatusCode::from_u16(status).unwrap();
            assert!(code.is_client_error());
            code
        }
    }

    prop_compose! {
        /// A 500 server error status code.
        fn status_code_500()(status in 500..599u16) -> reqwest::StatusCode {
            let code = reqwest::StatusCode::from_u16(status).unwrap();
            assert!(code.is_server_error());
            code
        }
    }

    proptest! {
        #[test]
        fn source_error_from_reqwest_400_error_is_client_error(status_code in status_code_400()) {
            let error = reqwest_status_code_error(status_code);
            let expected_conversion = SourceError::ClientError(status_code);
            assert_eq!(SourceError::from(error), expected_conversion);
        }

        #[test]
        fn source_error_from_reqwest_500_error_is_server_error(status_code in status_code_500()) {
            let error = reqwest_status_code_error(status_code);
            let expected_conversion = SourceError::from(ServerError::StatusCode(status_code));
            assert_eq!(SourceError::from(error), expected_conversion);
        }

    }

    #[rstest]
    fn source_error_from_reqwest_connect_or_timeout_error_is_connection_error(
        #[values(reqwest_connection_error(), reqwest_timeout_error())] error: reqwest::Error,
    ) {
        let expected_conversion = SourceError::ConnectionError;
        assert_eq!(SourceError::from(error), expected_conversion);
    }

    #[rstest]
    fn source_error_from_reqwest_decode_error_is_server_error_invalid_response_body(
        reqwest_decode_error: reqwest::Error,
    ) {
        let expected_conversion = SourceError::from(ServerError::InvalidResponseBody);
        assert_eq!(SourceError::from(reqwest_decode_error), expected_conversion);
    }
}
