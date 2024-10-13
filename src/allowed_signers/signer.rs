use std::sync::Arc;

use tokio::task::JoinSet;
use tracing::debug;

use super::{file::Entry, ssh::PublicKey};
use crate::{source::Source, SourceError};

/// An allowed signer.
#[derive(Debug)]
pub struct Signer {
    pub name: String,
    pub principals: Vec<String>,
    pub sources: Vec<Arc<dyn Source>>,
}

impl Signer {
    /// Get the signers public keys from all of it's sources.
    #[tracing::instrument(skip_all, fields(username=self.name), level = "debug")]
    async fn get_keys(&self) -> Result<Vec<PublicKey>, SourceError> {
        debug!("Getting keys from users configured sources");
        // TODO: Join futures
        let mut keys = Vec::new();
        for source in &self.sources {
            // TODO: Handle some error cases gracefully, e.g if the user has no keys, while returning others.
            keys.extend(source.get_keys_by_username(&self.name).await?);
        }
        Ok(keys)
    }

    /// Get the allowed signers file entries corresponding to this signer.
    pub(super) async fn get_entries(&self) -> Result<Vec<Entry>, SourceError> {
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
    let mut set = JoinSet::new();
    for signer in signers {
        set.spawn(async move { signer.get_entries().await });
    }
    let results = set.join_all().await;

    results.into_iter().flat_map(|r| r.unwrap()).collect()
}
