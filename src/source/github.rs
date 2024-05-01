use super::Source;
use crate::{SshPublicKey, USER_AGENT};
use async_trait::async_trait;
use reqwest::{Client, Result, Url};

#[derive(Debug)]
pub struct Github {
    /// The base URL of the API.
    base_url: Url,
}

impl Github {
    const VERSION: &'static str = "2022-11-28";
    const ACCEPT_HEADER: &'static str = "application/vnd.github+json";

    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }
}

#[async_trait]
impl Source for Github {
    // [API documentation](https://docs.github.com/en/rest/users/ssh-signing-keys?apiVersion=2022-11-28#list-ssh-signing-keys-for-a-user)
    async fn get_keys_by_username(
        &self,
        username: &str,
        client: &Client,
    ) -> Result<Vec<SshPublicKey>> {
        let url = self
            .base_url
            .join(&format!("/users/{username}/ssh_signing_keys"))
            .unwrap();
        let request = client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Accept", Self::ACCEPT_HEADER)
            .header("X-GitHub-Api-Version", Self::VERSION);

        let response = request.send().await?;
        response.json().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use rstest::rstest;

    const API_VERSION: &str = "2022-11-28";
    const API_ACCEPT_HEADER: &str = "application/vnd.github+json";

    /// The API request made to get a users signing keys is correct.
    #[tokio::test]
    async fn api_request_is_correct() {
        let username = "octocat";
        let server = MockServer::start();
        let mock = server.mock(|when, _| {
            when.method(GET)
                .path(format!("/users/{username}/ssh_signing_keys"))
                .header("accept", API_ACCEPT_HEADER)
                .header("x-github-api-version", API_VERSION)
                .header("user-agent", USER_AGENT);
        });

        let client = Client::new();
        let api = Github {
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
                "id": 773452,
                "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS",
                "title": "key-1",
                "created_at": "2023-05-23T09:35:15.638Z"
            },
              {
                "id": 773453,
                "key": "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=",
                "title": "key-2",
                "created_at": "2023-07-22T23:04:29.415Z"
              },
              {
                "id": 773454,
                "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k=",
                "title": "key-3",
                "created_at": "2023-12-04T19:32:23.794Z"
              }
        ]"#,
        vec![
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS".parse().unwrap(),
            "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=".parse().unwrap(),
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k=".parse().unwrap(),
        ]
    )]
    #[tokio::test]
    async fn keys_returned_by_api_deserialized_correctly(
        #[case] body: &str,
        #[case] expected: Vec<SshPublicKey>,
    ) {
        let username = "octocat";
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{username}/ssh_signing_keys"));
            then.status(200)
                .header("Content-Type", "application/json; charset=utf-8")
                .body(body);
        });

        let client = Client::new();
        let api = Github {
            base_url: server.base_url().parse().unwrap(),
        };
        let keys = api.get_keys_by_username(username, &client).await.unwrap();

        assert_eq!(keys, expected);
    }
}
