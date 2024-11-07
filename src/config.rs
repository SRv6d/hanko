use crate::{allowed_signers::Signer, Github, Gitlab, Source};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{HashMap, HashSet},
    fmt, fs, io,
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};
use toml_edit::{
    de::{Deserializer as TomlDeserializer, Error as TomlDeserializationError},
    DocumentMut, TomlError,
};
use tracing::{debug, info, trace};

/// A mutable and format preserving representation of a TOML file.
#[derive(Debug, Default)]
struct TomlFile {
    path: PathBuf,
    document: DocumentMut,
}

impl TomlFile {
    /// Load from a TOML file.
    fn load(path: PathBuf) -> Result<Self, Error> {
        info!("Loading TOML configuration file {}", path.display());
        let content = fs::read_to_string(&path)?;
        let document = content.parse()?;
        Ok(Self { path, document })
    }

    /// Save back to TOML file.
    fn save(&self) -> Result<(), io::Error> {
        info!("Saving TOML configuration file {}", self.path.display());
        fs::write(&self.path, self.document.to_string())
    }
}

/// The main configuration.
#[derive(Debug, Deserialize)]
pub struct Configuration {
    signers: Vec<SignerConfiguration>,
    #[serde(default)]
    sources: Vec<SourceConfiguration>,
    #[serde(skip)]
    file: TomlFile,
}

impl TryFrom<TomlFile> for Configuration {
    type Error = Error;

    /// Create a configuration from a TOML file without performing any semantic validation.
    fn try_from(file: TomlFile) -> Result<Self, Self::Error> {
        let deserializer = TomlDeserializer::from(file.document.clone());
        let mut s = Self::deserialize(deserializer)?;
        s.file = file;
        Ok(s)
    }
}

/// A `HashMap` containing sources by name.
/// Since signers need to contain references to sources and can move between threads,
/// an Arc is used for sources.
type NamedSources = HashMap<String, Arc<Box<dyn Source>>>;

impl Configuration {
    /// Returns configuration for the default GitHub and GitLab sources.
    fn default_sources() -> Vec<SourceConfiguration> {
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
        ]
    }

    /// Extend the configuration by the default sources.
    fn add_default_sources(&mut self) {
        let default_sources = Self::default_sources();
        debug!(
            ?default_sources,
            "Extending configuration with default sources."
        );
        self.sources.extend(default_sources);
    }

    /// Returns sources generated from their configuration.
    #[must_use]
    pub fn sources(&self) -> NamedSources {
        self.sources
            .iter()
            .map(|c| (c.name.clone(), Arc::new(c.build_source())))
            .collect()
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

    /// Load the configuration from a TOML file.
    /// Extends the configuration by default sources and performs semantic validation before returning.
    pub fn load(path: PathBuf) -> Result<Self, Error> {
        let file = TomlFile::load(path)?;

        let mut c = Self::try_from(file)?;
        c.add_default_sources();
        c.validate_semantics()?;

        Ok(c)
    }

    /// Save the configuration back to file.
    pub fn save(&self) -> Result<(), io::Error> {
        self.file.save()
    }

    /// Perform semantic validation of the configuration.
    fn validate_semantics(&self) -> Result<(), Error> {
        trace!(?self, "Validating configuration semantics");

        let used_sources: HashSet<&str> = self
            .signers
            .iter()
            .flat_map(|c| c.source_names.iter())
            .map(String::as_str)
            .collect();
        let existing_sources: HashSet<&str> =
            self.sources.iter().map(|c| c.name.as_str()).collect();
        let missing_sources: Vec<String> = used_sources
            .difference(&existing_sources)
            .map(ToString::to_string)
            .collect();
        if !missing_sources.is_empty() {
            return Err(MissingSourcesError::new(missing_sources).into());
        }

        Ok(())
    }
}

/// An error that can occur when interacting with a [`Configuration`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Toml(#[from] TomlError),
    #[error("{0}")]
    Syntax(#[from] TomlDeserializationError),
    #[error("missing sources {0}")]
    MissingSources(#[from] MissingSourcesError),
}

/// An error that occurs when sources are used that are not present in the configuration.
/// Contains names of the missing sources and displays them as a comma separated string.
#[derive(Debug, PartialEq, thiserror::Error)]
pub struct MissingSourcesError(Vec<String>);
impl fmt::Display for MissingSourcesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join(", "))
    }
}

impl MissingSourcesError {
    fn new(names: impl IntoIterator<Item = String>) -> Self {
        let mut v: Vec<_> = names.into_iter().collect();
        v.sort();
        Self(v)
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[fixture]
    fn tmp_config_toml() -> NamedTempFile {
        tempfile::Builder::new()
            .prefix("config")
            .suffix(".toml")
            .tempfile()
            .unwrap()
    }

    /// When loading a configuration, the returned instance always contains the default sources.
    #[rstest]
    #[case(
        indoc!{r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"], sources = ["github"] },
            ]
            file = "~/allowed_signers"
        "#}
    )]
    fn loaded_configuration_has_default_sources(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
    ) {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let config = Configuration::load(tmp_config_toml.path().to_path_buf()).unwrap();
        for default_source in Configuration::default_sources() {
            assert!(config.sources.contains(&default_source));
        }
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
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
        #[case] mut expected_missing: Vec<String>,
    ) -> Result<(), String> {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let err = Configuration::load(tmp_config_toml.path().to_path_buf()).unwrap_err();
        if let Error::MissingSources(err_missing) = err {
            expected_missing.sort();
            let err_missing = {
                let mut m = err_missing.clone();
                m.sort();
                m
            };
            assert_eq!(expected_missing, *err_missing);
            return Ok(());
        };

        Err("Did not return expected error".to_string())
    }

    /// Signers have a default GitHub source if no sources were configured explicitly.
    #[rstest]
    #[case(
        indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
            ]
            file = "~/allowed_signers"
        "#}
    )]
    fn signers_have_default_github_source(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
    ) {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let mut config = Configuration::load(tmp_config_toml.path().to_path_buf()).unwrap();
        let signer_sources = config.signers.pop().unwrap().source_names;

        assert_eq!(signer_sources, vec!["github"]);
    }

    /// When saving a configuration back to file, the TOML formatting matches that of the original file.
    #[rstest]
    #[case(
        indoc! {r#"
            [[signers]]
            name = "octocat"
            principals = ["octocat@github.com"]
        "#}
    )]
    #[case(
        indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
            ]
            file = "~/allowed_signers"
        "#}
    )]
    fn saving_configuration_preserves_formatting(
        mut tmp_config_toml: NamedTempFile,
        #[case] content: &str,
    ) {
        writeln!(tmp_config_toml, "{content}").unwrap();
        let config = Configuration::load(tmp_config_toml.path().to_path_buf()).unwrap();
        tmp_config_toml.as_file().set_len(0).unwrap();

        config.save().unwrap();
        let result = fs::read_to_string(tmp_config_toml.path()).unwrap();

        assert_eq!(result, content);
    }
}
