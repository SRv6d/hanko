use crate::SshPublicKey;
use async_trait::async_trait;

/// A Git provider.
#[async_trait]
pub trait GitProvider {
    type Err;

    /// Get the public keys of a user by their username.
    async fn get_keys_by_username(&self, username: &str) -> Result<Vec<SshPublicKey>, Self::Err>;
}
