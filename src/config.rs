use crate::{allowed_signers::Signer, cli::RuntimeConfiguration, Github, Gitlab, Source};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{debug, info, trace};

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Configuration {
    signers: Vec<SignerConfiguration>,
    sources: Option<Vec<SourceConfiguration>>,
    /// The path of the allowed signers file.
    file: PathBuf,
}

/// A `HashMap` containing sources by name.
/// Since signers need to contain references to sources and can move between threads,
/// an Arc is used for sources.
type NamedSources = HashMap<String, Arc<Box<dyn Source>>>;

impl Configuration {
    /// Returns the default GitHub and GitLab sources.
    fn default_sources() -> NamedSources {
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
        .map(|config| (config.name.clone(), Arc::new(config.build_source())))
        .collect()
    }

    /// Returns sources generated from their configuration extended by the default sources.
    #[must_use]
    pub fn sources(&self) -> NamedSources {
        let mut sources = Self::default_sources();
        if let Some(configs) = &self.sources {
            sources.extend(
                configs
                    .iter()
                    .map(|c| (c.name.clone(), Arc::new(c.build_source()))),
            );
        }
        sources
    }

    /// Returns signers generated from their configuration.
    ///
    /// # Panics
    ///
    /// Will panic if the given sources are missing a source configured within a signer.
    #[must_use]
    pub fn signers(&self, sources: &NamedSources) -> Vec<Signer> {
        let configs = &self.signers;
        configs
            .iter()
            .map(|c| {
                Signer {
                    name: c.name.clone(),
                    principals: c.principals.clone(),
                    sources: c
                        .source_names
                        .iter()
                        .map(|name| {
                            sources
                                .get(name)
                                .expect("signer references source that does not exist, config not validated correctly")
                                .clone()
                        })
                        .collect(),
                }
            })
            .collect()
    }

    /// The configured allowed signers file path.
    #[must_use]
    pub fn allowed_signers_file(&self) -> &Path {
        self.file.as_ref()
    }

    /// Load the configuration from a TOML file optionally merged with runtime configuration.
    #[tracing::instrument(skip(runtime_config))]
    pub fn load(path: &Path, runtime_config: Option<RuntimeConfiguration>) -> Result<Self, Error> {
        info!("Loading configuration file");
        let figment = {
            let toml = Figment::from(Toml::file_exact(path));
            if let Some(runtime_config) = runtime_config {
                debug!(
                    ?runtime_config,
                    "Merging configuration file with runtime configuration"
                );
                toml.merge(Serialized::defaults(runtime_config))
            } else {
                toml
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
            self.signers.iter().flat_map(|c| &c.source_names).collect();
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
        trace!(?self, "Validated configuration");
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

fn default_user_source() -> Vec<String> {
    vec!["github".to_string()]
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SignerConfiguration {
    pub name: String,
    pub principals: Vec<String>,
    #[serde(rename = "sources", default = "default_user_source")]
    pub source_names: Vec<String>,
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
            signers: vec![SignerConfiguration {
                name: "torvalds".to_string(),
                principals: vec!["torvalds@linux-foundation.org".to_string()],
                source_names: vec!["github".to_string()],
            }],
            sources: None,
            file: "~/allowed_signers".into(),
        };

        let sources = config.sources();

        assert!(sources.contains_key("github"));
        assert!(sources.contains_key("gitlab"));
    }

    /// Loading configuration missing sources returns an appropriate error.
    #[rstest]
    #[case(
        indoc!{r#"
            signers = [
                { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
                { name = "rdavis", principals = ["rdavis@lumon.industries"], sources = ["lumon-industries"] }
            ]
            file = "~/allowed_signers"

            [[sources]]
            name = "acme-corp"
            provider = "gitlab"
            url = "https://git.acme.corp"
        "#},
        vec!["lumon-industries".to_string()]
    )]
    #[case(
        indoc!{r#"
            signers = [
                { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
                { name = "rdavis", principals = ["rdavis@lumon.industries"], sources = ["lumon-industries"] }
            ]
            file = "~/allowed_signers"
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

    /// Runtime options override those specified in the configuration.
    #[rstest]
    fn runtime_option_overrides_config(config_path: PathBuf) {
        let config = indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"], sources = ["github"] },
            ]
            file = "/value/in/config"
        "#};
        let runtime_allowed_signers = PathBuf::from("/value/at/runtime");
        let runtime_config = RuntimeConfiguration {
            file: Some(runtime_allowed_signers.clone()),
            verbose: 0,
        };

        figment::Jail::expect_with(|jail| {
            jail.create_file(&config_path, config)?;

            let config = Configuration::load(&config_path, Some(runtime_config)).unwrap();

            assert_eq!(config.file, runtime_allowed_signers);
            Ok(())
        });
    }

    /// Users have a default GitHub source if no sources were configured explicitly.
    #[rstest]
    fn users_have_default_github_source(config_path: PathBuf) {
        let config = indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
            ]
            file = "~/allowed_signers"
        "#};

        figment::Jail::expect_with(|jail| {
            jail.create_file(&config_path, config)?;

            let mut config = Configuration::load(&config_path, None).unwrap();
            let signer_sources = config.signers.pop().unwrap().source_names;

            assert_eq!(signer_sources, vec!["github"]);
            Ok(())
        });
    }
}
