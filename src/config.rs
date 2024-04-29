use crate::GitProvider;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    allowed_signers: Option<PathBuf>,
    users: Option<Vec<User>>,
    local: Option<Vec<String>>,
    sources: Option<Vec<Source>>,
}

impl Default for Config {
    /// The default configuration containing common sources as well as the location of the allowed
    /// signers file if it is configured within Git.
    fn default() -> Self {
        Config {
            allowed_signers: git_allowed_signers(),
            users: None,
            local: None,
            sources: Some(vec![
                Source {
                    name: "github".to_string(),
                    provider: GitProviderType::Github,
                    url: "https://api.github.com".to_string(),
                },
                Source {
                    name: "gitlab".to_string(),
                    provider: GitProviderType::Gitlab,
                    url: "https://gitlab.com".to_string(),
                },
            ]),
        }
    }
}

impl Config {
    /// Load the configuration from a TOML file, using defaults for values that were not provided.
    pub fn load(path: &Path) -> figment::Result<Self> {
        Figment::from(Serialized::defaults(Config::default()))
            .admerge(Toml::file(path))
            .extract()
    }

    /// Load the configuration from a figment provider without using any defaults.
    #[cfg(test)]
    fn _load_from_provider(provider: impl figment::Provider) -> figment::Result<Self> {
        Figment::from(provider).extract()
    }

    /// Save the configuration.
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Save the configuration while preserving formatting.");
    }
}

/// The path to the allowed signers file as configured within Git. If an error occurs while
/// retrieving the path, `None` is returned.
fn git_allowed_signers() -> Option<PathBuf> {
    let file = gix_config::File::from_globals().ok()?;
    let path = file
        .path("gpg", Some("ssh".into()), "allowedsignersfile")?
        .interpolate(gix_config::path::interpolate::Context {
            home_dir: env::var("HOME")
                .ok()
                .map(std::convert::Into::<PathBuf>::into)
                .as_deref(),
            ..Default::default()
        })
        .ok()?;

    Some(path.into())
}

/// The type of Git provider.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum GitProviderType {
    /// A Git provider that implements the GitHub API.
    Github,
    /// A Git provider that implements the GitLab API.
    Gitlab,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct User {
    name: String,
    sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Source {
    name: String,
    provider: GitProviderType,
    url: String,
}

impl From<Source> for GitProvider {
    fn from(source: Source) -> Self {
        match source.provider {
            GitProviderType::Github => GitProvider::github(source.url.parse().unwrap()),
            GitProviderType::Gitlab => GitProvider::gitlab(source.url.parse().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    /// The example configuration is rendered correctly.
    #[test]
    fn example_config() {
        let toml = indoc! {r#"
        users = [
            { name = "torvalds", sources = ["github"] },
            { name = "gvanrossum", sources = ["github", "gitlab"] },
            { name = "graydon", sources = ["github"] },
            { name = "cwoods", sources = ["acme-corp"] },
            { name = "rdavis", sources = ["acme-corp"] },
            { name = "pbrock", sources = ["acme-corp"] }
        ]
        local = [
            "jdoe@example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
        ]
        
        [[sources]]
        name = "acme-corp"
        provider = "gitlab"
        url = "https://git.acme.corp"
        "#};
        let expected = Config {
            allowed_signers: None,
            users: Some(vec![
                User {
                    name: "torvalds".to_string(),
                    sources: vec!["github".to_string()],
                },
                User {
                    name: "gvanrossum".to_string(),
                    sources: vec!["github".to_string(), "gitlab".to_string()],
                },
                User {
                    name: "graydon".to_string(),
                    sources: vec!["github".to_string()],
                },
                User {
                    name: "cwoods".to_string(),
                    sources: vec!["acme-corp".to_string()],
                },
                User {
                    name: "rdavis".to_string(),
                    sources: vec!["acme-corp".to_string()],
                },
                User {
                    name: "pbrock".to_string(),
                    sources: vec!["acme-corp".to_string()],
                },
            ]),
            local: Some(vec!["jdoe@example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw".parse().unwrap()]),
            sources: Some(vec![
                Source {
                    name: "acme-corp".to_string(),
                    provider: GitProviderType::Gitlab,
                    url: "https://git.acme.corp".to_string(),
                }
            ])
        };

        let config = Config::_load_from_provider(Toml::string(toml)).unwrap();
        assert_eq!(config, expected);
    }

    /// The default configuration contains the default GitHub and GitLab sources.
    #[test]
    fn default_configuration_contains_default_sources() {
        let default_sources = Config::default().sources.unwrap();
        assert!(default_sources.contains(&Source {
            name: "github".to_string(),
            provider: GitProviderType::Github,
            url: "https://api.github.com".to_string(),
        }));
        assert!(default_sources.contains(&Source {
            name: "gitlab".to_string(),
            provider: GitProviderType::Gitlab,
            url: "https://gitlab.com".to_string(),
        }));
    }
}
