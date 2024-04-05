use crate::USER_AGENT;
use reqwest::{Client, Result, Url};
use serde::Deserialize;
use std::fmt;

const GH_ACCEPT: &str = "application/vnd.github+json";
const GH_API_VERSION: &str = "2022-11-28";
const GH_API_URL: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
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
    let url = format!("{GH_API_URL}/users/{user}/ssh_signing_keys")
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
        .header("Accept", GH_ACCEPT)
        .header("X-GitHub-Api-Version", GH_API_VERSION);

    let response = request.send().await?;
    response.json().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    /// The API request made to get a users signing keys is correct.
    #[tokio::test]
    async fn api_request_is_correct() {
        let path = "/users/octocat/ssh_signing_keys";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(path)
                .header("accept", GH_ACCEPT)
                .header("x-github-api-version", GH_API_VERSION)
                .header("user-agent", USER_AGENT);
        });

        let url: Url = server.url(path).parse().unwrap();
        let client = Client::new();
        let _ = get_signing_keys(url, client).await;

        mock.assert();
    }
}
