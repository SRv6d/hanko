use crate::{AllowedSignersEntry, Source, SourceError};

#[derive(Debug)]
pub struct User<'a> {
    pub name: String,
    pub principals: Vec<String>,
    pub sources: Vec<&'a dyn Source>,
}

impl User<'_> {
    /// Get the users allowed signer entries.
    pub async fn get_allowed_signers(
        &self,
        client: &reqwest::Client,
    ) -> Result<Vec<AllowedSignersEntry>, SourceError> {
        let mut keys = Vec::new();
        for source in &self.sources {
            keys.extend(source.get_keys_by_username(&self.name, client).await?);
        }

        Ok(keys
            .into_iter()
            .map(|key| AllowedSignersEntry {
                principals: self.principals.clone(),
                valid_after: None,
                valid_before: None,
                key,
            })
            .collect())
    }
}
