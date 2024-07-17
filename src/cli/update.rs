//! The update subcommand used to get the latest allowed signers and write them to the output file.
use crate::{user::User, AllowedSignersFile, USER_AGENT};
use anyhow::{Context, Result};
use std::{path::Path, time::Duration};

#[tokio::main]
#[tracing::instrument(skip_all)]
pub(super) async fn update(path: &Path, users: &Vec<User>) -> Result<()> {
    let client = get_client();

    let mut entries = Vec::new();
    for user in users {
        let allowed_signers = user.get_allowed_signers(&client).await?;
        entries.extend(allowed_signers);
    }

    let file = AllowedSignersFile::with_signers(path.into(), entries);
    file.write().context(format!(
        "Failed to write allowed signers file to {}",
        path.display()
    ))?;

    Ok(())
}

/// Configure and return a reqwest Client.
fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}
