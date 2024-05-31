use crate::{Github, Gitlab, Source, SourceMap};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
};

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Configuration {
    pub allowed_signers: Option<PathBuf>,
    pub users: Option<Vec<UserConfiguration>>,
    local: Option<Vec<String>>,
    sources: Vec<SourceConfiguration>,
}

impl Default for Configuration {
    /// The default configuration containing common sources as well as the location of the allowed
    /// signers file if it is configured within Git.
    fn default() -> Self {
        Configuration {
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

impl Configuration {
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
    pub fn load(path: &Path, defaults: bool) -> Result<Self, ConfigError> {
        let figment = {
            if defaults {
                Figment::from(Serialized::defaults(Configuration::default()))
            } else {
                Figment::new()
            }
        };
        let config: Self = figment.admerge(Toml::file(path)).extract()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<(), ConfigError> {
        if let Some(users) = &self.users {
            let used_sources: HashSet<&String> = users.iter().flat_map(|u| &u.sources).collect();
            let configured_sources: HashSet<&String> =
                self.sources.iter().map(|s| &s.name).collect();

            let missing_sources: Vec<String> = used_sources
                .difference(&configured_sources)
                .map(ToString::to_string)
                .collect();
            if !missing_sources.is_empty() {
                return Err(ConfigError::MissingSources(missing_sources));
            }
        }
        Ok(())
    }

    /// Save the configuration.
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Save the configuration while preserving formatting.");
    }
}

/// An error that can occur when interacting with a [`Configuration`].
#[derive(Debug, PartialEq)]
pub enum ConfigError {
    SyntaxError(figment::Error),
    MissingSources(Vec<String>),
}

impl From<figment::Error> for ConfigError {
    fn from(error: figment::Error) -> Self {
        ConfigError::SyntaxError(error)
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
pub struct UserConfiguration {
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
    use rstest::*;

    #[fixture]
    fn config_path() -> PathBuf {
        PathBuf::from("config.toml")
    }

    /// The default configuration contains the default GitHub and GitLab sources.
    #[test]
    fn default_configuration_contains_default_sources() {
        let default_sources = Configuration::default().sources;
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

    /// Loading an empty configuration file with defaults enabled returns the default configuration.
    #[rstest]
    fn load_empty_file_with_default_returns_exact_default(config_path: PathBuf) {
        figment::Jail::expect_with(|jail| {
            jail.create_file(&config_path, "")?;
            let config = Configuration::load(&config_path, true).unwrap();
            assert_eq!(config, Configuration::default());
            Ok(())
        });
    }

    /// Loading an empty configuration file without defaults enabled returns an error because
    /// there are missing fields.
    #[rstest]
    fn load_empty_file_without_default_returns_error(config_path: PathBuf) {
        figment::Jail::expect_with(|jail| {
            jail.create_file(&config_path, "")?;
            Configuration::load(&config_path, false).unwrap_err();
            Ok(())
        });
    }

    /// Loading a configuration with a missing source returns an error.
    #[rstest]
    #[allow(clippy::panic)]
    fn loading_configuration_with_missing_source_returns_error(config_path: PathBuf) {
        let toml = indoc! {r#"
        users = [
            { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
            { name = "rdavis", principals = ["rdavis@lumon.industries"], sources = ["lumon-industries"] }
        ]
        "#};
        figment::Jail::expect_with(|jail| {
            jail.create_file(&config_path, toml)?;
            let error = Configuration::load(&config_path, true).unwrap_err();

            if let ConfigError::MissingSources(missing_sources) = error {
                assert!(["acme-corp".to_string(), "lumon-industries".to_string()]
                    .iter()
                    .all(|s| missing_sources.contains(s)));
            } else {
                panic!("Unexpected error returned: {error:?}");
            }
            Ok(())
        });
    }
}
