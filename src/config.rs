use crate::{Github, Gitlab, Source, SourceMap};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    allowed_signers: Option<PathBuf>,
    pub users: Option<Vec<User>>,
    local: Option<Vec<String>>,
    sources: Vec<SourceConfiguration>,
}

impl Default for Config {
    /// The default configuration containing common sources as well as the location of the allowed
    /// signers file if it is configured within Git.
    fn default() -> Self {
        Config {
            allowed_signers: git_allowed_signers(),
            users: None,
            local: None,
            sources: vec![
                SourceConfiguration {
                    name: "github".to_string(),
                    provider: SourceType::Github,
                    url: "https://api.github.com".to_string(),
                },
                SourceConfiguration {
                    name: "gitlab".to_string(),
                    provider: SourceType::Gitlab,
                    url: "https://gitlab.com".to_string(),
                },
            ],
        }
    }
}

impl Config {
    /// Get the configured sources.
    #[must_use]
    pub fn get_sources(&self) -> SourceMap {
        self.sources
            .iter()
            .map(|source_config| {
                let name = source_config.name.clone();
                let source: Box<dyn Source> = match source_config.provider {
                    SourceType::Github => Box::new(Github::new(source_config.url.parse().unwrap())),
                    SourceType::Gitlab => Box::new(Gitlab::new(source_config.url.parse().unwrap())),
                };
                (name, source)
            })
            .collect()
    }

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

/// The type of source.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Github,
    Gitlab,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub name: String,
    pub principals: Vec<String>,
    pub sources: Vec<String>,
}

/// The representation of a [`Source`] in configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct SourceConfiguration {
    name: String,
    provider: SourceType,
    url: String,
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
            { name = "torvalds", principals = ["torvalds@linux-foundation.org"], sources = ["github"] },
            { name = "gvanrossum", principals = ["guido@python.org"], sources = ["github", "gitlab"] },
            { name = "graydon", principals = ["graydon@pobox.com"], sources = ["github"] },
            { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
            { name = "rdavis", principals = ["rdavis@acme.corp"], sources = ["acme-corp"] },
            { name = "pbrock", principals = ["pbrock@acme.corp"], sources = ["acme-corp"] }
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
                    principals: vec!["torvalds@linux-foundation.org".to_string()],
                    sources: vec!["github".to_string()],
                },
                User {
                    name: "gvanrossum".to_string(),
                    principals: vec!["guido@python.org".to_string()],
                    sources: vec!["github".to_string(), "gitlab".to_string()],
                },
                User {
                    name: "graydon".to_string(),
                    principals: vec!["graydon@pobox.com".to_string()],
                    sources: vec!["github".to_string()],
                },
                User {
                    name: "cwoods".to_string(),
                    principals: vec!["cwoods@acme.corp".to_string()],
                    sources: vec!["acme-corp".to_string()],
                },
                User {
                    name: "rdavis".to_string(),
                    principals: vec!["rdavis@acme.corp".to_string()],
                    sources: vec!["acme-corp".to_string()],
                },
                User {
                    name: "pbrock".to_string(),
                    principals: vec!["pbrock@acme.corp".to_string()],
                    sources: vec!["acme-corp".to_string()],
                },
            ]),
            local: Some(vec!["jdoe@example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw".parse().unwrap()]),
            sources: vec![
                SourceConfiguration {
                    name: "acme-corp".to_string(),
                    provider: SourceType::Gitlab,
                    url: "https://git.acme.corp".to_string(),
                }
            ]
        };

        let config = Config::_load_from_provider(Toml::string(toml)).unwrap();
        assert_eq!(config, expected);
    }

    /// The default configuration contains the default GitHub and GitLab sources.
    #[test]
    fn default_configuration_contains_default_sources() {
        let default_sources = Config::default().sources;
        assert!(default_sources.contains(&SourceConfiguration {
            name: "github".to_string(),
            provider: SourceType::Github,
            url: "https://api.github.com".to_string(),
        }));
        assert!(default_sources.contains(&SourceConfiguration {
            name: "gitlab".to_string(),
            provider: SourceType::Gitlab,
            url: "https://gitlab.com".to_string(),
        }));
    }
}
