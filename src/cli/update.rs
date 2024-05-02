//! The update subcommand used to get the latest allowed signers and write them to the output file.
use crate::{
    config::UserConfiguration, AllowedSignersEntry, AllowedSignersFile, Configuration, SourceMap,
    SshPublicKey,
};
use tokio::runtime::Runtime;

pub(super) fn update(config: Configuration) {
    let rt = Runtime::new().unwrap();
    let sources = config.get_sources();

    let mut file = AllowedSignersFile::new(&config.allowed_signers.unwrap()).unwrap();
    if let Some(users) = config.users {
        for user in users {
            let public_keys = rt.block_on(get_public_keys(&user, &sources));
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

    file.write().unwrap();
}

async fn get_public_keys(user: &UserConfiguration, sources: &SourceMap) -> Vec<SshPublicKey> {
    let client = reqwest::Client::new();
    let mut keys: Vec<SshPublicKey> = Vec::new();
    for source_name in &user.sources {
        let source = sources.get(source_name).unwrap();
        keys.extend(
            source
                .get_keys_by_username(&user.name, &client)
                .await
                .unwrap(),
        );
    }
    keys
}
