use async_trait::async_trait;
use reqwest::{Client, Request, Response, StatusCode, Url};
use serde::Deserialize;
use tracing::{trace, warn};
use chrono::{DateTime, FixedOffset};

use super::{Error, Result, Source, base_client, next_url_from_link_header};
use crate::{USER_AGENT, allowed_signers::file::PublicKey};

#[derive(Debug)]
pub struct Gitlab {
    /// The base URL of the API.
    base_url: Url,
    client: Client,
}

impl Gitlab {
    const VERSION: &'static str = "v4";
    const ACCEPT_HEADER: &'static str = "application/json";

    #[must_use]
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: base_client(),
        }
    }
}

#[async_trait]
impl Source for Gitlab {
    // [API Documentation](https://docs.gitlab.com/16.10/ee/api/users.html#list-ssh-keys-for-user)
    async fn get_keys_by_username(&self, username: &str) -> Result<Vec<PublicKey>> {
        let mut next_url = Some(
            self.base_url
                .join(&format!(
                    "/api/{version}/users/{username}/keys",
                    version = Self::VERSION,
                ))
                .unwrap(),
        );

        let mut keys = Vec::new();
        while let Some(current_url) = next_url.take() {
            let request = self
                .client
                .get(current_url.clone())
                .header("User-Agent", USER_AGENT)
                .header("Accept", Self::ACCEPT_HEADER)
                .version(reqwest::Version::HTTP_2)
                .build()
                .unwrap();
            let response = make_api_request(request, &self.client).await?;
            let next_page = next_url_from_link_header(response.headers()).unwrap_or_else(|err| {
                warn!("Pagination skipped due to {err}. Keys may be incomplete.");
                None
            });

            let all_keys: Vec<ApiSshKey> = response.json().await?;
            // Get just the signing keys and turn those into public keys.
            let signing_keys = all_keys
                .into_iter()
                .filter(|key| key.usage_type.is_signing())
                .map(PublicKey::from);
            keys.extend(signing_keys);

            match next_page {
                Some(candidate) if candidate != current_url => {
                    next_url = Some(candidate);
                }
                _ => {
                    next_url = None;
                }
            }
        }

        Ok(keys)
    }
}

/// Make an HTTP request to the GitLab API.
async fn make_api_request(request: Request, client: &Client) -> Result<Response> {
    trace!(?request, "Sending request to GitLab API");
    let response = handle_gitlab_errors(client.execute(request).await)?;
    trace!(?response, "Received response from GitLab API.");

    Ok(response)
}

/// Handle GitLab specific HTTP errors.
fn handle_gitlab_errors(request_result: reqwest::Result<Response>) -> Result<Response> {
    let response = request_result?;

    if let Err(error) = response.error_for_status_ref() {
        let status = error
            .status()
            .expect("Status code error must contain status code");

        match status {
            StatusCode::NOT_FOUND => return Err(Error::UserNotFound),
            StatusCode::UNAUTHORIZED => {
                return Err(Error::BadCredentials);
            }
            _ => return Err(Error::from(error)),
        }
    }

    Ok(response)
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum ApiSshKeyUsage {
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "signing")]
    Signing,
    #[serde(rename = "auth_and_signing")]
    AuthAndSigning,
}

impl ApiSshKeyUsage {
    /// Returns true if the key is used for signing.
    pub fn is_signing(&self) -> bool {
        matches!(
            self,
            ApiSshKeyUsage::Signing | ApiSshKeyUsage::AuthAndSigning
        )
    }
}

/// Intermediary representation of a [`PublicKey`] as returned by the GitLab API.
#[derive(Debug, Deserialize)]
struct ApiSshKey {
    key: String,
    expires_at: Option<DateTime<FixedOffset>>,
    usage_type: ApiSshKeyUsage,
}

impl From<ApiSshKey> for PublicKey {
    fn from(api_key: ApiSshKey) -> Self {
        PublicKey {
            blob: api_key.key,
            valid_after: None,
            valid_before: api_key.expires_at
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::StatusCode;
    use rstest::*;
    use serde_json::json;

    const API_ACCEPT_HEADER: &str = "application/json";

    const EXAMPLE_USERNAME: &str = "tanuki";

    /// An API instance and a mock server with the APIs base url configured to that of the mock server.
    #[fixture]
    fn api_w_mock_server() -> (Gitlab, MockServer) {
        let server = MockServer::start();
        let api = Gitlab::new(server.base_url().parse().unwrap());
        (api, server)
    }

    /// The API request made to get a users signing keys is correct.
    #[rstest]
    #[tokio::test]
    async fn api_request_is_correct(api_w_mock_server: (Gitlab, MockServer)) {
        let (api, server) = api_w_mock_server;
        let mock = server.mock(|when, _| {
            when.method(GET)
                .path(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys"))
                .header("accept", API_ACCEPT_HEADER)
                .header("user-agent", USER_AGENT);
        });

        let _ = api.get_keys_by_username(EXAMPLE_USERNAME).await;

        mock.assert();
    }

    /// Keys returned from the API are deserialized correctly.
    #[rstest]
    #[case("[]", vec![])]
    #[case(
        r#"[
              {
                  "id": 1121029,
                  "title": "key-1",
                  "created_at": "2020-08-21T19:43:06.816Z",
                  "expires_at": null,
                  "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS John Doe (gitlab.com)",
                  "usage_type": "auth_and_signing"
              },
              {
                "id": 1121030,
                "title": "key-2",
                "created_at": "2023-07-22T23:04:29.415Z",
                "expires_at": "2025-04-10T00:00:00.000Z",
                "key": "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w= John Doe (gitlab.com)",
                "usage_type": "auth"
              },
              {
                "id": 1121031,
                "title": "key-3",
                "created_at": "2023-12-04T19:32:23.794Z",
                "expires_at": null,
                "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k= John Doe (gitlab.com)",
                "usage_type": "signing"
              }
        ]"#,
        vec![
            PublicKey {
                blob: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS John Doe (gitlab.com)".to_string(),
                valid_after: None,
                valid_before: None
            },
            PublicKey {
                blob: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k= John Doe (gitlab.com)".to_string(),
                valid_after: None,
                valid_before: None
            },
        ]
    )]
    #[tokio::test]
    async fn keys_returned_by_api_deserialized_correctly(
        #[case] body: &str,
        #[case] expected: Vec<PublicKey>,
        api_w_mock_server: (Gitlab, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys"));
            then.status(200)
                .header("Content-Type", "application/json")
                .body(body);
        });

        let keys = api.get_keys_by_username(EXAMPLE_USERNAME).await.unwrap();

        assert_eq!(keys, expected);
    }

    #[rstest]
    #[tokio::test]
    async fn pagination_link_header_next_is_followed(api_w_mock_server: (Gitlab, MockServer)) {
        let (api, server) = api_w_mock_server;

        let next_link = format!(
            "<{}>; rel=\"next\"",
            server.url(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys?page=2"))
        );

        let first_page = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys"))
                .query_param_missing("page");
            then.status(200)
                .header("Content-Type", "application/json")
                .header("Link", next_link.as_str())
                .json_body(json!([]));
        });

        let second_page = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys"))
                .query_param("page", "2");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([]));
        });

        api.get_keys_by_username(EXAMPLE_USERNAME).await.unwrap();

        first_page.assert();
        second_page.assert();
    }

    /// A HTTP not found status code returns a `SourceError::UserNotFound`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_not_found_returns_user_not_found_error(
        api_w_mock_server: (Gitlab, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys"));
            then.status(StatusCode::NOT_FOUND);
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::UserNotFound));
    }

    /// A HTTP unauthorized status code returns a `SourceError::BadCredentials`.
    #[rstest]
    #[tokio::test]
    async fn get_keys_by_username_http_unauthorized_returns_bad_credentials(
        api_w_mock_server: (Gitlab, MockServer),
    ) {
        let (api, server) = api_w_mock_server;
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/v4/users/{EXAMPLE_USERNAME}/keys"));
            then.status(StatusCode::UNAUTHORIZED);
        });

        let error_result = api
            .get_keys_by_username(EXAMPLE_USERNAME)
            .await
            .unwrap_err();

        assert!(matches!(error_result, Error::BadCredentials));
    }
}
