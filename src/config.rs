use crate::{cli::RuntimeConfiguration, user::User, Github, Gitlab, Source, SourceMap};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashSet,
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};
use tracing::info;

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Configuration {
    users: Vec<UserConfiguration>,
    sources: Option<Vec<SourceConfiguration>>,
    allowed_signers: PathBuf,
}

impl Configuration {
    /// The default GitHub and GitLab sources.
    fn default_sources() -> SourceMap {
        [
            SourceConfiguration {
                name: "github".to_string(),
                provider: SourceType::Github,
                url: "https://api.github.com".parse().unwrap(),
            },
            SourceConfiguration {
                name: "gitlab".to_string(),
                provider: SourceType::Gitlab,
                url: "https://gitlab.com".parse().unwrap(),
            },
        ]
        .iter()
        .map(|config| (config.name.clone(), config.build_source()))
        .collect()
    }

    /// The configured path to write the allowed signers file to.
    #[must_use]
    pub fn allowed_signers(&self) -> &Path {
        self.allowed_signers.as_ref()
    }

    /// The configured and default sources.
    #[must_use]
    pub fn sources(&self) -> SourceMap {
        let mut sources = Self::default_sources();
        if let Some(configs) = &self.sources {
            sources.extend(
                configs
                    .iter()
                    .map(|config| (config.name.clone(), config.build_source())),
            );
        }
        sources
    }

    /// The configured users.
    #[must_use]
    pub fn users<'b>(&self, sources: &'b SourceMap) -> Option<Vec<User<'b>>> {
        let configs = &self.users;
        let users = configs
            .iter()
            .map(|config| {
                let sources = config
                    .sources
                    .iter()
                    .map(|name| sources.get(name).unwrap().as_ref())
                    .collect();
                User {
                    // TODO: Use references instead of cloning.
                    name: config.name.clone(),
                    principals: config.principals.clone(),
                    sources,
                }
            })
            .collect();
        Some(users)
    }

    /// Load the configuration from a TOML file optionally merged with runtime configuration.
    #[tracing::instrument]
    pub fn load(path: &Path, runtime_config: Option<RuntimeConfiguration>) -> Result<Self, Error> {
        info!("Loading configuration file");
        let figment = {
            if let Some(runtime_config) = runtime_config {
                Figment::from(Toml::file(path)).merge(Serialized::defaults(runtime_config))
            } else {
                Figment::from(Toml::file(path))
            }
        };
        let config: Self = figment.extract()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<(), Error> {
        let configured_sources = self.sources();
        let used_source_names: HashSet<&String> =
            self.users.iter().flat_map(|u| &u.sources).collect();
        let configured_source_names: HashSet<&String> = configured_sources.keys().collect();

        let missing_source_names: Vec<String> = used_source_names
            .difference(&configured_source_names)
            .map(ToString::to_string)
            .collect();
        if !missing_source_names.is_empty() {
            return Err(Error::MissingSources(MissingSourcesError(
                missing_source_names,
            )));
        }
        Ok(())
    }

    /// Save the configuration.
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Save the configuration while preserving formatting.");
    }
}

/// An error that can occur when interacting with a [`Configuration`].
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    SyntaxError(figment::Error),
    #[error("missing sources {0}")]
    MissingSources(MissingSourcesError),
}

impl From<figment::Error> for Error {
    fn from(error: figment::Error) -> Self {
        Error::SyntaxError(error)
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub struct MissingSourcesError(Vec<String>);
impl fmt::Display for MissingSourcesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join(", "))
    }
}
impl Deref for MissingSourcesError {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
    #[serde(serialize_with = "serialize_url", deserialize_with = "deserialize_url")]
    url: Url,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let url = reqwest::Url::parse(&s).map_err(serde::de::Error::custom)?;
    Ok(url)
}

fn serialize_url<U, S>(url: U, serializer: S) -> Result<S::Ok, S::Error>
where
    U: AsRef<str>,
    S: Serializer,
{
    serializer.serialize_str(url.as_ref())
}

impl SourceConfiguration {
    fn build_source(&self) -> Box<dyn Source> {
        let url = self.url.clone();
        match self.provider {
            SourceType::Github => Box::new(Github::new(url)),
            SourceType::Gitlab => Box::new(Gitlab::new(url)),
        }
    }
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

    /// A configuration without any explicitly configured sources contains the default sources.
    #[test]
    fn configuration_has_default_sources() {
        let config = Configuration {
            users: vec![UserConfiguration {
                name: "torvalds".to_string(),
                principals: vec!["torvalds@linux-foundation.org".to_string()],
                sources: vec!["github".to_string()],
            }],
            sources: None,
            allowed_signers: "~/allowed_signers".into(),
        };

        let sources = config.sources();

        assert!(sources.contains_key("github"));
        assert!(sources.contains_key("gitlab"));
    }

    /// Loading configuration missing sources returns an appropriate error.
    #[rstest]
    #[case(
        indoc!{r#"
            users = [
                { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
                { name = "rdavis", principals = ["rdavis@lumon.industries"], sources = ["lumon-industries"] }
            ]
            allowed_signers = "~/allowed_signers"

            [[sources]]
            name = "acme-corp"
            provider = "gitlab"
            url = "https://git.acme.corp"
        "#},
        vec!["lumon-industries".to_string()]
    )]
    #[case(
        indoc!{r#"
            users = [
                { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
                { name = "rdavis", principals = ["rdavis@lumon.industries"], sources = ["lumon-industries"] }
            ]
            allowed_signers = "~/allowed_signers"
        "#},
        vec!["acme-corp".to_string(), "lumon-industries".to_string()]
    )]
    #[allow(clippy::panic)]
    fn loading_configuration_with_missing_source_returns_error(
        config_path: PathBuf,
        #[case] config: &str,
        #[case] mut expected_missing: Vec<String>,
    ) {
        figment::Jail::expect_with(|jail| {
            jail.create_file(&config_path, config)?;

            let err = Configuration::load(&config_path, None).unwrap_err();
            if let Error::MissingSources(err_missing) = err {
                expected_missing.sort();
                let err_missing = {
                    let mut m = err_missing.clone();
                    m.sort();
                    m
                };
                assert_eq!(expected_missing, *err_missing);
                Ok(())
            } else {
                Err("Did not return expected error".into())
            }
        });
    }
}
