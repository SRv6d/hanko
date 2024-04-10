use crate::USER_AGENT;
use reqwest::{Client, Result, Url};

const API_ACCEPT_HEADER: &str = "application/json";

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
