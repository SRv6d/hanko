use std::sync::Arc;

use tokio::task::JoinSet;
use tracing::debug;

use super::{file::Entry, ssh::PublicKey};
use crate::{source::Source, Error};

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
                    source.get_keys_by_username(&username).await
                }
            })
            .collect();
        let mut keys = Vec::new();
        while let Some(output) = set.join_next().await {
            // TODO: Handle some error cases gracefully, e.g if the user has no keys, while returning others.
            keys.extend(output.unwrap()?);
        }
        Ok(keys)
    }

    /// Get the allowed signers file entries corresponding to this signer.
    pub(super) async fn get_entries(&self) -> Result<Vec<Entry>, Error> {
        let keys = self.get_keys().await?;

        Ok(keys
            .into_iter()
            .map(|key| Entry {
                principals: self.principals.clone(),
                valid_after: None,
                valid_before: None,
                key,
            })
            .collect())
    }
}

/// Get entries for multiple given signers concurrently.
pub(super) async fn get_entries<S>(signers: S) -> Vec<Entry>
where
    S: IntoIterator<Item = Signer>,
{
    let set: JoinSet<_> = signers
        .into_iter()
        .map(|signer| async move { signer.get_entries().await })
        .collect();
    let results = set.join_all().await;

    results.into_iter().flat_map(|r| r.unwrap()).collect()
}
