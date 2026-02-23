pub(super) mod file;

use std::{path::Path, sync::Arc};

use anyhow::Context;
use tokio::task::JoinSet;
use tracing::{debug, error, warn};

/// The outcome of an [`update`].
pub enum Outcome {
    /// The allowed signers file was written.
    Written,
    /// No signer entries were collected, the file was not written.
    SkippedNoEntries,
}

/// Update the allowed signers file.
pub async fn update<S>(path: &Path, signers: S) -> anyhow::Result<Outcome>
where
    S: IntoIterator<Item = Signer>,
{
    let entries = get_entries(signers).await?;

    if entries.is_empty() {
        return Ok(Outcome::SkippedNoEntries);
    }

    let file = File::from_entries(path.to_path_buf(), entries);
    file.write().context(format!(
        "Failed to write allowed signers file to {}",
        path.display()
    ))?;
    Ok(Outcome::Written)
}

pub use file::{Entry, File, PublicKey};

use crate::{Error, source::Source};

/// An allowed signer.
#[derive(Debug)]
pub struct Signer {
    pub name: String,
    pub principals: Vec<String>,
    pub sources: Vec<Arc<Box<dyn Source>>>,
}

impl Signer {
    /// Get the signers public keys from all of it's sources.
    #[tracing::instrument(skip_all, fields(username=self.name), level = "debug")]
    async fn get_keys(&self) -> Result<Vec<PublicKey>, Error> {
        let mut set: JoinSet<_> = self
            .sources
            .iter()
            .map(|source| {
                let source = source.clone();
                let username = self.name.clone();
                async move {
                    debug!(
                        ?source,
                        "Requesting keys from source for signer {}", &username
                    );
                    match source.get_keys_by_username(&username).await {
                        Ok(keys) => {
                            if keys.is_empty() {
                                warn!(
                                    ?source,
                                    "User {} does not have any signing keys configured on source",
                                    &username
                                );
                            }
                            Ok(keys)
                        }
                        Err(Error::UserNotFound) => {
                            warn!(?source, "User {} does not exist on source", &username);
                            Ok(vec![])
                        }
                        Err(Error::ConnectionError) => {
                            error!(?source, "Failed to connect to source");
                            Err(Error::ConnectionError)
                        }
                        Err(err) => Err(err),
                    }
                }
            })
            .collect();
        let mut keys = Vec::new();
        while let Some(output) = set.join_next().await {
            keys.extend(output.unwrap()?);
        }
        Ok(keys)
    }

    /// Get the allowed signers file entries corresponding to this signer.
    async fn get_entries(&self) -> Result<Vec<Entry>, Error> {
        let keys = self.get_keys().await?;

        Ok(keys
            .into_iter()
            .map(|key| Entry::new(self.principals.clone(), key))
            .collect())
    }
}

/// Get entries for multiple given signers concurrently.
async fn get_entries<S>(signers: S) -> Result<Vec<Entry>, Error>
where
    S: IntoIterator<Item = Signer>,
{
    let mut set: JoinSet<_> = signers
        .into_iter()
        .map(|signer| async move { signer.get_entries().await })
        .collect();
    let mut entries = Vec::new();
    while let Some(output) = set.join_next().await {
        entries.extend(output.unwrap()?);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write};

    /// When no entries are collected, the allowed signers file is not written to.
    #[tokio::test]
    async fn update_does_not_write_to_file_when_no_entries() {
        let content = "not@applicable.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS";
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        let path = file.into_temp_path();

        let outcome = update(&path, Vec::<Signer>::new()).await.unwrap();

        assert!(matches!(outcome, Outcome::SkippedNoEntries));
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }
}
