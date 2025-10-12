pub use github::Github;
pub use gitlab::Gitlab;

mod github;
mod gitlab;

use crate::{USER_AGENT, allowed_signers::ssh::PublicKey};
use async_trait::async_trait;
use reqwest::{
    Client, StatusCode, Url,
    header::{HeaderMap, HeaderValue},
};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
    time::Duration,
};

/// A `Result` alias where the `Err` case is a source [`Error`].
pub(super) type Result<T> = std::result::Result<T, Error>;

/// A source implements a way to get public keys from a Git provider.
#[async_trait]
pub trait Source: Debug + Send + Sync {
    /// Get a users public keys by their username.
    async fn get_keys_by_username(&self, username: &str) -> Result<Vec<PublicKey>>;
}

/// An error that can occur when interacting with a source.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
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
    ClientError(StatusCode),
}

/// Conversion for generic reqwest errors not specific to any `Source`.
///
/// # Panics
///
/// Since the error type is not and enum and cannot be matched exhaustively, this conversion panics
/// as a last resort if an unexpected error is encountered.
impl From<reqwest::Error> for Error {
    #[allow(clippy::panic)]
    fn from(error: reqwest::Error) -> Self {
        if error.is_connect() || error.is_timeout() {
            Error::ConnectionError
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
            Error::ClientError(error.status().unwrap())
        } else if error.is_body() || error.is_decode() {
            ServerError::InvalidResponseBody.into()
        } else {
            panic!("Unexpected reqwest error: {error:?}");
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ServerError {
    #[error("response contains an invalid `{name}` header: {msg}")]
    InvalidResponseHeader { name: String, msg: String },
    #[error("response contains an invalid body")]
    InvalidResponseBody,
    #[error("{0}")]
    StatusCode(StatusCode),
}

fn get_header_value<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>> {
    headers
        .get(name)
        .map(|value| {
            value.to_str().map_err(|e| {
                Error::ServerError(ServerError::InvalidResponseHeader {
                    name: name.to_string(),
                    msg: format!("value is not valid UTF-8: {e}"),
                })
            })
        })
        .transpose()
}

pub(super) fn parse_header_value<T>(headers: &HeaderMap, name: &str) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: Display,
{
    get_header_value(headers, name)?
        .map(|value| {
            value.parse().map_err(|err| {
                Error::ServerError(ServerError::InvalidResponseHeader {
                    name: name.to_string(),
                    msg: format!("value is not valid: {err}"),
                })
            })
        })
        .transpose()
}

/// Parse the next URL from the value of a Link header.
// TODO: Log malformed headers
pub(super) fn parse_link_header_next(header: &HeaderValue) -> Option<Url> {
    let header = header.to_str().ok()?;

    header.split(',').find_map(|segment| {
        let mut parts = segment.trim().split(';');
        let url_part = parts.next()?.trim();
        let url = url_part.strip_prefix('<')?.strip_suffix('>')?;

        let is_next = parts.any(|param| {
            param
                .trim()
                .strip_prefix("rel=")
                .is_some_and(|rel| rel.trim_matches('"') == "next")
        });

        if is_next { Url::parse(url).ok() } else { None }
    })
}

/// The base reqwest Client to be used by sources.
pub(super) fn base_client() -> Client {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(10))
        .use_rustls_tls()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use proptest::prelude::*;
    use rstest::*;

    /// Returns a reqwest error caused by the given status code.
    fn reqwest_status_code_error(status: StatusCode) -> reqwest::Error {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.any_request();
            then.status(status);
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
    /// Returns an `Option` since the error is created by causing an actual connection error,
    /// for which we need to find a free port. If the used port is not free, or we cannot determine
    /// if it is, `None` is returned.
    #[fixture]
    fn reqwest_connection_error() -> Option<reqwest::Error> {
        let test_port = 43286;
        // This should work on macOS and most Linux distributions.
        let test_port_in_use = std::process::Command::new("lsof")
            .args(["-nP", format!("-i:{test_port}").as_str()])
            .status()
            .ok()?
            .success(); // nonzero exit code if the port is not in use
        if test_port_in_use {
            return None;
        }

        let error = reqwest::blocking::get(format!("http://localhost:{test_port}/")).unwrap_err();
        assert!(error.is_connect());
        Some(error)
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
        fn status_code_400()(status in 400..499u16) -> StatusCode {
            let code = StatusCode::from_u16(status).unwrap();
            assert!(code.is_client_error());
            code
        }
    }

    prop_compose! {
        /// A 500 server error status code.
        fn status_code_500()(status in 500..599u16) -> StatusCode {
            let code = StatusCode::from_u16(status).unwrap();
            assert!(code.is_server_error());
            code
        }
    }

    proptest! {
        #[test]
        fn source_error_from_reqwest_400_error_is_client_error(status_code in status_code_400()) {
            let error = reqwest_status_code_error(status_code);
            let expected_conversion = Error::ClientError(status_code);
            assert_eq!(Error::from(error), expected_conversion);
        }

        #[test]
        fn source_error_from_reqwest_500_error_is_server_error(status_code in status_code_500()) {
            let error = reqwest_status_code_error(status_code);
            let expected_conversion = Error::from(ServerError::StatusCode(status_code));
            assert_eq!(Error::from(error), expected_conversion);
        }

    }

    #[rstest]
    fn source_error_from_reqwest_connect_or_timeout_error_is_connection_error(
        #[values(reqwest_connection_error(), reqwest_timeout_error().into())] error: Option<
            reqwest::Error,
        >,
    ) {
        let expected_conversion = Error::ConnectionError;
        // The assertion is skipped if the used fixture failed to create an error.
        if let Some(error) = error {
            assert_eq!(Error::from(error), expected_conversion);
        }
    }

    #[rstest]
    fn source_error_from_reqwest_decode_error_is_server_error_invalid_response_body(
        reqwest_decode_error: reqwest::Error,
    ) {
        let expected_conversion = Error::from(ServerError::InvalidResponseBody);
        assert_eq!(Error::from(reqwest_decode_error), expected_conversion);
    }

    #[rstest]
    #[case(
        r#"<https://api.github.com/repositories/1300192/issues?per_page=2&page=1&before=Y3Vyc29yOnYyOpLPAAABkOs68TjOkOKw1A%3D%3D>; rel="prev""#,
        None,
    )]
    #[case(
        r#"<https://api.github.com/repositories/1300192/issues?per_page=2&after=Y3Vyc29yOnYyOpLPAAABmbe5SzDOz8JUuQ%3D%3D&page=2>; rel="next""#,
        Some("https://api.github.com/repositories/1300192/issues?per_page=2&after=Y3Vyc29yOnYyOpLPAAABmbe5SzDOz8JUuQ%3D%3D&page=2".parse().unwrap())
    )]
    #[case(
        r#"<https://api.github.com/repositories/1300192/issues?page=2>; rel="prev", <https://api.github.com/repositories/1300192/issues?page=4>; rel="next", <https://api.github.com/repositories/1300192/issues?page=515>; rel="last", <https://api.github.com/repositories/1300192/issues?page=1>; rel="first""#,
        Some("https://api.github.com/repositories/1300192/issues?page=4".parse().unwrap())
    )]
    #[case(
        r#"<https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="prev", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="next", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="first", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="last""#,
        Some("https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3".parse().unwrap())
    )]
    fn parse_valid_link_header_returns_correct_url(
        #[case] header: HeaderValue,
        #[case] expected: Option<Url>,
    ) {
        let parsed = parse_link_header_next(&header);
        assert_eq!(parsed, expected);
    }
}
