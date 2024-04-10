use crate::{SshPublicKey, USER_AGENT};
use reqwest::{Client, Result, Url};

const API_ACCEPT_HEADER: &str = "application/json";

/// Make a GET request to the GitLab API at the given URL and return the signing keys contained in the response.
///
/// # GitLab API documentation
/// https://docs.gitlab.com/16.10/ee/api/users.html#list-ssh-keys-for-user
async fn get_signing_keys(url: Url, client: Client) -> Result<Vec<SshPublicKey>> {
    let request = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", API_ACCEPT_HEADER);

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
        let path = "/api/v4/users/tanuki/keys";
        let server = MockServer::start();
        let mock = server.mock(|when, _| {
            when.method(GET)
                .path(path)
                .header("accept", API_ACCEPT_HEADER)
                .header("user-agent", USER_AGENT);
        });

        let url: Url = server.url(path).parse().unwrap();
        let client = Client::new();
        let _ = get_signing_keys(url, client).await;

        mock.assert();
    }
}
