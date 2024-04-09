use crate::USER_AGENT;
use reqwest::{Client, Result, Url};
use serde::Deserialize;
use std::fmt;

const API_URL: &str = "https://api.github.com";
const API_VERSION: &str = "2022-11-28";
const API_ACCEPT_HEADER: &str = "application/vnd.github+json";

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SshSigningKey {
    key: String,
}

impl fmt::Display for SshSigningKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

/// Get the signing keys of a user by their username.
pub async fn get_user_signing_keys(user: &str, client: Client) -> Result<Vec<SshSigningKey>> {
    let url = format!("{API_URL}/users/{user}/ssh_signing_keys")
        .parse()
        .unwrap();
    get_signing_keys(url, client).await
}

/// Make a GET request to the GitHub API at the given URL and return the signing keys contained in the response.
///
/// # Github API documentation
/// https://docs.github.com/en/rest/users/ssh-signing-keys?apiVersion=2022-11-28#list-ssh-signing-keys-for-a-user
async fn get_signing_keys(url: Url, client: Client) -> Result<Vec<SshSigningKey>> {
    let request = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", API_ACCEPT_HEADER)
        .header("X-GitHub-Api-Version", API_VERSION);

    let response = request.send().await?;
    response.json().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use rstest::rstest;

    /// The API request made to get a users signing keys is correct.
    #[tokio::test]
    async fn api_request_is_correct() {
        let path = "/users/octocat/ssh_signing_keys";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(path)
                .header("accept", API_ACCEPT_HEADER)
                .header("x-github-api-version", API_VERSION)
                .header("user-agent", USER_AGENT);
        });

        let url: Url = server.url(path).parse().unwrap();
        let client = Client::new();
        let _ = get_signing_keys(url, client).await;

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
            SshSigningKey {
                key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS".to_string(),
            },
            SshSigningKey {
                key: "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=".to_string(),
            },
            SshSigningKey {
                key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k=".to_string(),
            },
        ]
    )]
    #[tokio::test]
    async fn keys_returned_by_api_deserialized_correctly(
        #[case] body: &str,
        #[case] expected: Vec<SshSigningKey>,
    ) {
        let path = "/users/octocat/ssh_signing_keys";
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(path);
            then.status(200)
                .header("Content-Type", "application/json; charset=utf-8")
                .body(body);
        });

        let client = Client::new();
        let keys = get_signing_keys(server.url(path).parse().unwrap(), client)
            .await
            .unwrap();

        assert_eq!(keys, expected);
    }
}
