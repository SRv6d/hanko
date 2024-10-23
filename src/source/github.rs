use std::ops::Deref;

use async_trait::async_trait;
use reqwest::{Client, Response, StatusCode, Url};
use serde::Deserialize;
use tracing::trace;

use super::main::{base_client, Error, Result, Source};
use crate::{allowed_signers::ssh::PublicKey, USER_AGENT};

#[derive(Debug)]
pub struct Github {
    /// The base URL of the API.
    base_url: Url,
    client: Client,
}

impl Github {
    const VERSION: &'static str = "2022-11-28";
    const ACCEPT_HEADER: &'static str = "application/vnd.github+json";

    #[must_use]
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: base_client(),
        }
    }
}

#[async_trait]
impl Source for Github {
    // [API documentation](https://docs.github.com/en/rest/users/ssh-signing-keys?apiVersion=2022-11-28#list-ssh-signing-keys-for-a-user)
    #[tracing::instrument(level = "trace")]
    async fn get_keys_by_username(&self, username: &str) -> Result<Vec<PublicKey>> {
        let url = self
            .base_url
            .join(&format!("/users/{username}/ssh_signing_keys"))
            .unwrap();
        let request = self
            .client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Accept", Self::ACCEPT_HEADER)
            .header("X-GitHub-Api-Version", Self::VERSION)
            .build()
            .unwrap();

        trace!(?request, "Sending request to GitHub API");
        let response = handle_github_errors(self.client.execute(request).await).await?;
        trace!(?response, "Received response from GitHub API.");
        Ok(response.json().await?)
    }
}

/// A message from the GitHub API.
#[derive(Debug, Deserialize)]
struct Message {
    message: String,
}

impl Deref for Message {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

/// Handle GitHub specific HTTP errors.
/// Takes a reqwest result containing a response, converting it into the `Result` type used in this
/// module which contains either an `Err` variant with a `SourceError` or an `Ok` variant with the
/// response that can be deserialized.
/// If the error is not specific to GitHub, it is converted into a `SourceError` using the
/// more generic `From<reqwest::Error>` implementation.
async fn handle_github_errors(request_result: reqwest::Result<Response>) -> Result<Response> {
    let response = request_result?;

    if let Err(error) = response.error_for_status_ref() {
        let status = error
            .status()
            .expect("Status code error must contain status code");
        let message = response.json::<Message>().await.ok();

        match status {
            StatusCode::NOT_FOUND => return Err(Error::UserNotFound),
            StatusCode::FORBIDDEN
                if message
                    .as_ref()
                    .is_some_and(|m| m.to_lowercase().contains("rate limit exceeded")) =>
            {
                return Err(Error::RatelimitExceeded);
            }
            StatusCode::UNAUTHORIZED
                if message
                    .as_ref()
                    .is_some_and(|m| m.to_lowercase().contains("bad credentials")) =>
            {
                return Err(Error::BadCredentials);
            }
            _ => return Err(Error::from(error)),
        }
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::StatusCode;
    use rstest::*;
    use serde_json::{json, Value as JsonValue};

    const API_VERSION: &str = "2022-11-28";
    const API_ACCEPT_HEADER: &str = "application/vnd.github+json";

    const EXAMPLE_USERNAME: &str = "octocat";

    /// An API instance and a mock server with the APIs base url configured to that of the mock server.
    #[fixture]
    fn api_w_mock_server() -> (Github, MockServer) {
        let server = MockServer::start();
        let api = Github::new(server.base_url().parse().unwrap());
        (api, server)
    }

    /// The API request made to get a users signing keys is correct.
    #[rstest]
    #[tokio::test]
    async fn api_request_is_correct(api_w_mock_server: (Github, MockServer)) {
        let (api, server) = api_w_mock_server;
        let mock = server.mock(|when, _| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"))
                .header("accept", API_ACCEPT_HEADER)
                .header("x-github-api-version", API_VERSION)
                .header("user-agent", USER_AGENT);
        });

        let _ = api.get_keys_by_username(EXAMPLE_USERNAME).await;

        mock.assert();
    }

    /// Keys returned from the API are deserialized correctly.
    #[rstest]
    #[case(json!([]), vec![])]
    #[case(json!(
        [
            {
                "id": 773_452,
                "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS",
                "title": "key-1",
                "created_at": "2023-05-23T09:35:15.638Z"
            },
              {
                "id": 773_453,
                "key": "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=",
                "title": "key-2",
                "created_at": "2023-07-22T23:04:29.415Z"
              },
              {
                "id": 773_454,
                "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k=",
                "title": "key-3",
                "created_at": "2023-12-04T19:32:23.794Z"
              }
        ]),
        vec![
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS".parse().unwrap(),
            "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=".parse().unwrap(),
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k=".parse().unwrap(),
        ]
    )]
    #[tokio::test]
    async fn keys_returned_by_api_deserialized_correctly(
        #[case] body: JsonValue,
        #[case] expected: Vec<PublicKey>,
        api_w_mock_server: (Github, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"));
            then.status(200)
                .header("Content-Type", "application/json; charset=utf-8")
                .json_body(body);
        });

        let keys = api.get_keys_by_username(EXAMPLE_USERNAME).await.unwrap();

        assert_eq!(keys, expected);
    }

    #[test]
    fn json_message_parsed_correctly() {
        let content = "I've Gotta Get a Message to You";
        let json = json!({"message": content});

        let message: Message = serde_json::from_value(json).unwrap();

        assert_eq!(*message, *content);
    }

    /// A HTTP not found status code returns a `SourceError::UserNotFound`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_not_found_returns_user_not_found_error(
        api_w_mock_server: (Github, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"));
            then.status(StatusCode::NOT_FOUND.into());
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::UserNotFound));
    }

    /// A HTTP unauthorized status code along with a body containing a bad credentials message
    /// returns a `SourceError::BadCredentials`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_unauthorized_bad_credentials_returns_bad_credentials(
        api_w_mock_server: (Github, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"));
            then.status(StatusCode::UNAUTHORIZED.into())
                .json_body(json!({"message": "Bad credentials"}));
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::BadCredentials));
    }

    /// A HTTP unauthorized status code without a known error message in the body returns a `SourceError::Other`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_unauthorized_other_returns_client_error(
        api_w_mock_server: (Github, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"));
            then.status(StatusCode::UNAUTHORIZED.into());
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::ClientError(..)));
    }

    /// A HTTP forbidden status code along with a body containing a rate limit exceeded message
    /// returns a `SourceError::RatelimitExceeded`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_forbidden_rate_limit_exceeded_returns_rate_limit_exceeded(
        api_w_mock_server: (Github, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"));
            then.status(StatusCode::FORBIDDEN.into())
                .json_body(json!({"message": "rate limit exceeded"}));
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::RatelimitExceeded));
    }

    /// A HTTP forbidden status code without a known error message in the body returns a `SourceError::ClientError`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_forbidden_other_returns_client_error(
        api_w_mock_server: (Github, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{EXAMPLE_USERNAME}/ssh_signing_keys"));
            then.status(StatusCode::FORBIDDEN.into());
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::ClientError(..)));
    }
}
