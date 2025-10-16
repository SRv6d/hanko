use std::sync::Arc;

use tokio::task::JoinSet;
use tracing::{debug, error, warn};

use super::{file::{Entry, PublicKey}};
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
    pub(super) async fn get_entries(&self) -> Result<Vec<Entry>, Error> {
        let keys = self.get_keys().await?;

        Ok(keys
            .into_iter()
            .map(|key| Entry::new(self.principals.clone(), key))
            .collect())
    }
}

/// Get entries for multiple given signers concurrently.
pub(super) async fn get_entries<S>(signers: S) -> Result<Vec<Entry>, Error>
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
