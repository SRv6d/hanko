//! The update subcommand used to get the latest allowed signers and write them to the output file.
use crate::{key::get_public_keys, AllowedSignersEntry, AllowedSignersFile, Configuration};
use anyhow::{Context, Result};
use std::time::Instant;
use tokio::runtime::Runtime;
use tracing::debug;

pub(super) fn update(config: Configuration) -> Result<()> {
    let start = Instant::now();

    let rt = Runtime::new().unwrap();
    let client = reqwest::Client::new();
    let sources = config.get_sources();
    let path = &config.allowed_signers.expect("no default value");

    let mut file = AllowedSignersFile::new(path.clone());
    if let Some(users) = config.users {
        for user in users {
            let sources = user
                .sources
                .iter()
                .map(|name| {
                    sources
                        .get(name)
                        .expect("configuration validated incorrectly")
                })
                .map(AsRef::as_ref);

            let public_keys = rt.block_on(get_public_keys(&user.name, sources, &client))?;
            for public_key in public_keys {
                file.add(AllowedSignersEntry {
                    principals: user.principals.clone(),
                    valid_after: None,
                    valid_before: None,
                    key: public_key,
                });
            }
        }
    }
    file.write().context(format!(
        "Failed to write allowed signers file to {}",
        path.display()
    ))?;

    let duration = start.elapsed();
    debug!(
        "Updated allowed signers file {} in {:?}",
        path.display(),
        duration
    );

    Ok(())
}
