use std::sync::Arc;

use tokio::task::JoinSet;

use super::file::Entry;
use crate::source::Source;

/// An allowed signer.
#[derive(Debug)]
pub struct Signer {
    pub name: String,
    pub principals: Vec<String>,
    pub sources: Vec<Arc<dyn Source>>,
}

impl Signer {
    /// Get the allowed signers file entries corresponding to this signer.
    pub(super) async fn get_entries(&self) -> Result<Vec<Entry>, ()> {
        todo!()
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
