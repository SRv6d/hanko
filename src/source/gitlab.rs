use super::{Result, Source};
use crate::{SshPublicKey, USER_AGENT};
use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Deserialize;

#[derive(Debug)]
pub struct Gitlab {
    /// The base URL of the API.
    base_url: Url,
}

impl Gitlab {
    const VERSION: &'static str = "v4";
    const ACCEPT_HEADER: &'static str = "application/json";

    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }
}

#[async_trait]
impl Source for Gitlab {
    // [API Documentation](https://docs.gitlab.com/16.10/ee/api/users.html#list-ssh-keys-for-user)
    async fn get_keys_by_username(
        &self,
        username: &str,
        client: &Client,
    ) -> Result<Vec<SshPublicKey>> {
        let url = self
            .base_url
            .join(&format!(
                "/api/{version}/users/{username}/keys",
                version = Self::VERSION,
            ))
            .unwrap();
        let request = client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Accept", Self::ACCEPT_HEADER);

        let response = request.send().await?;
        // The API has no way to filter keys by usage type, so this contains all the user's keys.
        let all_keys: Vec<ApiSshKey> = response.json().await?;
        // Filter out the keys that are not used for signing.
        let signing_keys = all_keys
            .into_iter()
            .filter(|key| key.usage_type.is_signing());

        Ok(signing_keys.map(SshPublicKey::from).collect())
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum ApiSshKeyUsage {
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

/// The GitLab API representation of an SSH key.
#[derive(Debug, Deserialize)]
pub struct ApiSshKey {
    pub id: usize,
    pub title: String,
    pub key: String,
    pub usage_type: ApiSshKeyUsage,
}

impl From<ApiSshKey> for SshPublicKey {
    fn from(api_key: ApiSshKey) -> Self {
        api_key.key.parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use rstest::rstest;

    const API_ACCEPT_HEADER: &str = "application/json";

    /// The API request made to get a users signing keys is correct.
    #[tokio::test]
    async fn api_request_is_correct() {
        let username = "tanuki";
        let server = MockServer::start();
        let mock = server.mock(|when, _| {
            when.method(GET)
                .path(format!("/api/v4/users/{username}/keys"))
                .header("accept", API_ACCEPT_HEADER)
                .header("user-agent", USER_AGENT);
        });

        let client = Client::new();
        let api = Gitlab {
            base_url: server.base_url().parse().unwrap(),
        };
        let _ = api.get_keys_by_username(username, &client).await;

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
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS John Doe (gitlab.com)".parse().unwrap(),
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k= John Doe (gitlab.com)".parse().unwrap(),
        ]
    )]
    #[tokio::test]
    async fn keys_returned_by_api_deserialized_correctly(
        #[case] body: &str,
        #[case] expected: Vec<SshPublicKey>,
    ) {
        let username = "tanuki";
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/v4/users/{username}/keys"));
            then.status(200)
                .header("Content-Type", "application/json")
                .body(body);
        });

        let client = Client::new();
        let api = Gitlab {
            base_url: server.base_url().parse().unwrap(),
        };
        let keys = api.get_keys_by_username(username, &client).await.unwrap();

        assert_eq!(keys, expected);
    }
}
