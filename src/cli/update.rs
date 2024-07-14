//! The update subcommand used to get the latest allowed signers and write them to the output file.
use crate::{user::User, AllowedSignersFile};
use anyhow::{Context, Result};
use std::path::Path;

#[tokio::main]
#[tracing::instrument(skip_all)]
pub(super) async fn update(path: &Path, users: &Vec<User>) -> Result<()> {
    let client = reqwest::Client::new();

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
