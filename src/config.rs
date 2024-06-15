use crate::{user::User, Github, Gitlab, Source, SourceMap};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashSet,
    env, fmt,
    ops::Deref,
    path::{Path, PathBuf},
};
use tracing::{debug, info};

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Configuration {
    users: Option<Vec<UserConfiguration>>,
    sources: Vec<SourceConfiguration>,
    allowed_signers: Option<PathBuf>,
}

impl Default for Configuration {
    /// The default configuration containing common sources as well as the location of the allowed
    /// signers file if it is configured within Git.
    fn default() -> Self {
        Self::new(
            None,
            vec![
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
            ],
            git_allowed_signers(),
        )
    }
}

impl Configuration {
    fn new(
        users: Option<Vec<UserConfiguration>>,
        sources: Vec<SourceConfiguration>,
        allowed_signers: Option<PathBuf>,
    ) -> Self {
        Self {
            users,
            sources,
            allowed_signers,
        }
    }

    /// The configured path to write the allowed signers file to.
    #[must_use]
    pub fn allowed_signers(&self) -> Option<&Path> {
        self.allowed_signers.as_deref()
    }

    /// The configured sources.
    #[must_use]
    pub fn sources(&self) -> SourceMap {
        self.sources
            .iter()
            .map(|config| (config.name.clone(), config.build_source()))
            .collect()
    }

    /// The configured users.
    #[must_use]
    pub fn users<'b>(&self, sources: &'b SourceMap) -> Option<Vec<User<'b>>> {
        let configs = self.users.as_ref()?;
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

    /// Load the configuration from a TOML file, using defaults for values that were not provided.
    #[tracing::instrument]
    pub fn load(path: &Path, defaults: bool) -> Result<Self, Error> {
        let figment = {
            if defaults {
                Figment::from(Serialized::defaults(Configuration::default()))
            } else {
                Figment::new()
            }
        };
        info!("Loading configuration file");
        let config: Self = figment.admerge(Toml::file(path)).extract()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<(), Error> {
        if let Some(users) = &self.users {
            let used_sources: HashSet<&String> = users.iter().flat_map(|u| &u.sources).collect();
            let configured_sources: HashSet<&String> =
                self.sources.iter().map(|s| &s.name).collect();

            let missing_sources: Vec<String> = used_sources
                .difference(&configured_sources)
                .map(ToString::to_string)
                .collect();
            if !missing_sources.is_empty() {
                return Err(Error::MissingSources(MissingSourcesError(missing_sources)));
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

/// The path to the allowed signers file as configured within Git. If an error occurs while
/// retrieving the path, `None` is returned.
#[tracing::instrument(level = "debug")]
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

    debug!(
        path = %path.to_string_lossy(),
        "Found allowed signers file configured in Git."
    );
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

    /// The default configuration contains the default GitHub and GitLab sources.
    #[test]
    fn default_configuration_contains_default_sources() {
        let default_sources = Configuration::default().sources;
        assert!(default_sources.contains(&SourceConfiguration {
            name: "github".to_string(),
            provider: SourceType::Github,
            url: "https://api.github.com".parse().unwrap(),
        }));
        assert!(default_sources.contains(&SourceConfiguration {
            name: "gitlab".to_string(),
            provider: SourceType::Gitlab,
            url: "https://gitlab.com".parse().unwrap(),
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

            if let Error::MissingSources(missing_sources) = error {
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
