//! Ensure correct behavior of the update subcommand.
use assert_cmd::Command;
use httpmock::prelude::*;
use indoc::{formatdoc, indoc};
use rstest::*;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

/// A mock github server with preconfigured responses.
#[fixture]
fn mock_github_server() -> MockServer {
    let server = MockServer::start();
    let responses = [
        (
            "jsnow",
            json!(
                [
                    {
                        "id": 773452,
                        "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS",
                        "title": "key-1",
                        "created_at": "2023-05-23T09:35:15.638Z"
                    }
                ]
            ),
        ),
        (
            "imalcom",
            json!(
                [
                    {
                        "id": 773453,
                        "key": "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=",
                        "title": "key-2",
                        "created_at": "2023-07-22T23:04:29.415Z"
                    }
                ]
            ),
        ),
        ("napplic", json!([])),
    ];
    for (user, body) in responses {
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/users/{user}/ssh_signing_keys"));
            then.status(200).json_body(body);
        });
    }

    server
}

/// A mock gitlab server with preconfigured responses.
#[fixture]
fn mock_gitlab_server() -> MockServer {
    let server = MockServer::start();
    let responses = [
        (
            "cwoods",
            json!(
                [
                    {
                        "id": 1121029,
                        "title": "key-1",
                        "created_at": "2020-08-21T19:43:06.816Z",
                        "expires_at": null,
                        "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS John Doe (gitlab.com)",
                        "usage_type": "auth_and_signing"
                    }
                ]
            ),
        ),
        (
            "ernie",
            json!(
                [
                    {
                        "id": 1121031,
                        "title": "key-3",
                        "created_at": "2023-12-04T19:32:23.794Z",
                        "expires_at": null,
                        "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k= John Doe (gitlab.com)",
                        "usage_type": "signing"
                    }
                ]
            ),
        ),
    ];
    for (user, body) in responses {
        server.mock(|when, then| {
            when.method(GET).path(format!("/api/v4/users/{user}/keys"));
            then.status(200).json_body(body);
        });
    }

    server
}

/// When running the update command with an example configuration and mocked endpoints,
/// the expected allowed signers file is written to disk.
#[rstest]
fn update_writes_expected_allowed_signers(
    mock_github_server: MockServer,
    mock_gitlab_server: MockServer,
) {
    let config = {
        let toml = formatdoc! {r#"
            users = [
                {{ name = "jsnow", principals = ["j.snow@wall.com"], sources = ["mock-github"]}},
                {{ name = "imalcom", principals = ["ian.malcom@acme.corp"], sources = ["mock-github"]}},
                {{ name = "cwoods", principals = ["cwoods@universal.exports"], sources = ["mock-gitlab"]}},
                {{ name = "ernie", principals = ["ernie@muppets.com"], sources = ["mock-gitlab"]}},
                {{ name = "napplic", principals = ["not@applicable.com"], sources = ["mock-github"]}}
            ]

            [[sources]]
            name = "mock-github"
            provider = "github"
            url = "{github_url}"

            [[sources]]
            name = "mock-gitlab"
            provider = "gitlab"
            url = "{gitlab_url}"
        "#, github_url = mock_github_server.base_url(), gitlab_url=mock_gitlab_server.base_url()};
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml.as_bytes()).unwrap();
        file
    };
    let allowed_signers = NamedTempFile::new().unwrap();
    let expected_content = indoc! {"
        cwoods@universal.exports ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS John Doe (gitlab.com)
        ernie@muppets.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDDTdEeUFjUX76aMptdG63itqcINvu/tnV5l9RXy/1TS25Ui2r+C2pRjG0vr9lzfz8TGncQt1yKmaZDAAe6mYGFiQlrkh9RJ/MPssRw4uS4slvMTDWhNufO1M3QGkek81lGaZq55uazCcaM5xSOhLBdrWIMROeLgKZ9YkHNqJXTt9V+xNE5ZkB/65i2tCkGdXnQsGJbYFbkuUTvYBuMW9lwmryLTeWwFLWGBP1moZI9etk3snh2hCLTV8+gvmhCTE8sAGBMcJq+TGxnfFoCtnA9Bdy7t+ZMLh1kV7oneUA9YT7qNeUFy55D287DAltB02ntT7CtuG6SBAQ4CQMcCoAX3Os4aVfdILOEC8ghrAj3uTEQuE3nYta0SmqqXcVAxmXUQCawf8n5CJ7QN5aIhCH73MKr6k5puk9dnkAcAFLRM6stvQhnpIqrI3YEbjqs1FGHfbc4+nfEWorxRrd7ur1ckEhuvmAXRKrLzYp9gYWU6TxfRqSxsXh3he0G6i+kC6k= John Doe (gitlab.com)
        ian.malcom@acme.corp ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBCoObGvI0R2SfxLypsqi25QOgiI1lcsAhtL7AqUeVD+4mS0CQ2Nu/C8h+RHtX6tHpd+GhfGjtDXjW598Vr2j9+w=
        j.snow@wall.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS

    "};

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--config")
        .arg(config.path())
        .arg("--file")
        .arg(allowed_signers.path())
        .arg("update")
        .assert()
        .success();
    let content = std::fs::read_to_string(allowed_signers.path()).unwrap();

    assert_eq!(content, expected_content);
}
