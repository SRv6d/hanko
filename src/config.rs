//! Types used to configure hanko.
//!
//! Configuration is handled by two cooperating types: [`TomlFile`] and [`Configuration`].
//!
//! [`TomlFile`] holds the raw [`toml_edit::DocumentMut`] and is responsible for format-preserving
//! load, mutation, and atomic save operations. [`toml_edit`] is used to ensure that user formatting,
//! comments, and key ordering are not destroyed when hanko writes back to the config file.
//!
//! [`Configuration`] is the typed, validated domain object. It is derived from [`TomlFile`] and
//!  owns the parsed signers and sources. [`TomlFile`] is retained as a field on [`Configuration`]
//! so that mutations made through the public API can be written back to disk through the
//! original document, preserving formatting.
//!
//! Keeping the two types separate rather than collapsing them into one also means
//! [`Configuration`] can be constructed from an in-memory document without a real file path,
//! which keeps unit tests independent of the filesystem.
//!
//! Fallible functions return [`anyhow::Result`] since errors here are reported directly to the
//! user without further programmatic handling.

use crate::{Github, Gitlab, Source, allowed_signers::Signer, parent_dir};
use anyhow::{Context, Error, Result, bail};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::NamedTempFile;
use tracing::{debug, info, trace};

/// A mutable and format preserving representation of a TOML file.
#[derive(Debug, Default)]
struct TomlFile {
    path: PathBuf,
    document: toml_edit::DocumentMut,
}

impl TomlFile {
    /// Add an allowed signer to the file.
    fn add_signer(&mut self, name: &str, principals: Vec<&str>, source_names: Vec<&str>) {
        use toml_edit::{Array, ArrayOfTables, Item, Table, Value};

        let mut table = Table::new();
        table.insert("name", name.into());
        table.insert(
            "principals",
            principals.into_iter().collect::<Array>().into(),
        );
        if source_names != default_user_source() {
            table.insert(
                "sources",
                source_names.into_iter().collect::<Array>().into(),
            );
        }

        match self.document.get_mut("signers") {
            None => {
                let mut item = ArrayOfTables::new();
                item.push(table);
                self.document.insert("signers", Item::ArrayOfTables(item));
            }
            Some(Item::Value(Value::Array(a))) if a.iter().all(Value::is_inline_table) => {
                a.push(table.into_inline_table());
            }
            Some(Item::ArrayOfTables(a)) => a.push(table),
            _ => unreachable!("signers key has invalid format"),
        }
    }

    /// Load from a TOML file.
    fn load(path: PathBuf) -> Result<Self> {
        info!("Loading TOML configuration file");
        let content = fs::read_to_string(&path)?;
        let document = content.parse()?;
        Ok(Self { path, document })
    }

    /// Save back to TOML file.
    fn save(&self) -> Result<()> {
        info!("Saving TOML configuration file");
        let dir = parent_dir(&self.path)?;
        let mut file = NamedTempFile::new_in(dir)?;
        write!(file, "{}", self.document)?;
        file.persist(&self.path)?;
        Ok(())
    }
}

/// The main configuration.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Configuration {
    signers: Vec<SignerConfiguration>,
    sources: Vec<SourceConfiguration>,
    #[serde(skip)]
    file: TomlFile,
}

impl TryFrom<TomlFile> for Configuration {
    type Error = Error;

    /// Create a configuration from a TOML file without performing any semantic validation.
    fn try_from(file: TomlFile) -> Result<Self> {
        let deserializer = toml_edit::de::Deserializer::from(file.document.clone());
        let mut s = Self::deserialize(deserializer)?;
        s.file = file;
        Ok(s)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            signers: Vec::default(),
            sources: Self::default_sources(),
            file: TomlFile::default(),
        }
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
            "Extending configuration with default sources"
        );
        self.sources.extend(default_sources);
    }

    /// Add an allowed signer to the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the given sources don't exist.
    pub fn add_signer(
        &mut self,
        name: String,
        principals: Vec<String>,
        source_names: Vec<String>,
    ) -> Result<()> {
        self.check_sources_exist(source_names.iter().map(String::as_str))?;

        let signer = SignerConfiguration {
            name,
            principals,
            source_names,
        };
        self.file.add_signer(
            &signer.name,
            signer.principals.iter().map(AsRef::as_ref).collect(),
            signer.source_names.iter().map(AsRef::as_ref).collect(),
        );
        self.signers.push(signer);

        Ok(())
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
    ///
    /// # Errors
    ///
    /// When the file fails to load or it's content is invalid.
    #[tracing::instrument]
    pub fn load(path: &Path) -> Result<Self> {
        let file = TomlFile::load(path.to_path_buf())?;

        let mut c = Self::try_from(file)?;
        c.add_default_sources();
        c.validate_semantics()?;

        Ok(c)
    }

    /// Load the configuration from a TOML file, returning a default instance if it doesn't exist.
    ///
    /// # Errors
    ///
    /// When the file at the given path has invalid content.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        Self::load(path).or_else(|err| match err.downcast_ref::<io::Error>() {
            Some(io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                info!("Configuration file does not exist yet and will be created");
                let dir = parent_dir(path)?;
                fs::create_dir_all(dir).context(format!(
                    "Failed to create configuration directory {}",
                    dir.display()
                ))?;
                Ok(Configuration {
                    file: TomlFile {
                        path: path.to_path_buf(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            }
            _ => Err(err),
        })
    }

    /// Save the configuration back to file.
    ///
    /// # Errors
    ///
    /// When an IO error occurs while trying to write the underlying file to disk.
    pub fn save(&self) -> Result<()> {
        self.file.save()
    }

    /// Perform semantic validation of the configuration.
    fn validate_semantics(&self) -> Result<()> {
        trace!(?self, "Validating configuration semantics");

        self.check_sources_exist(
            self.signers
                .iter()
                .flat_map(|c| c.source_names.iter().map(String::as_str)),
        )?;
        self.check_signers_have_one_or_more_principals()?;

        Ok(())
    }

    /// Check if the given sources exist, returning an error if not.
    fn check_sources_exist<'a>(
        &self,
        source_names: impl IntoIterator<Item = &'a str>,
    ) -> Result<()> {
        let used_sources: HashSet<&str> = source_names.into_iter().collect();
        let existing_sources: HashSet<&str> =
            self.sources.iter().map(|c| c.name.as_str()).collect();
        let mut missing_sources: Vec<String> = used_sources
            .difference(&existing_sources)
            .map(ToString::to_string)
            .collect();
        if !missing_sources.is_empty() {
            missing_sources.sort();
            bail!("Missing sources: {}", missing_sources.join(", "))
        }
        Ok(())
    }

    /// Check that all signers have at least one principal configured.
    fn check_signers_have_one_or_more_principals(&self) -> Result<()> {
        for config in &self.signers {
            if config.principals.is_empty() {
                bail!("Signer {} missing principals", config.name)
            }
        }
        Ok(())
    }
}

/// The type of source.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Github,
    Gitlab,
}

#[must_use]
pub fn default_user_source() -> Vec<String> {
    vec!["github".to_string()]
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SignerConfiguration {
    pub name: String,
    pub principals: Vec<String>,
    #[serde(rename = "sources")]
    pub source_names: Vec<String>,
}

impl Default for SignerConfiguration {
    fn default() -> Self {
        Self {
            name: String::default(),
            principals: Vec::default(),
            source_names: default_user_source(),
        }
    }
}

/// The representation of a [`Source`] in configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
    use tempfile::{NamedTempFile, TempDir};

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
        "#}
    )]
    fn loaded_configuration_has_default_sources(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
    ) {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let config = Configuration::load(tmp_config_toml.path()).unwrap();
        for default_source in Configuration::default_sources() {
            assert!(config.sources.contains(&default_source));
        }
    }

    /// When loading configuration from a path that doesn't exist without using the
    /// explicit `load_or_default` constructor, an error is returned.
    #[rstest]
    fn loading_non_existent_configuration_returns_error() {
        let tmpdir = TempDir::new().unwrap();
        let path = tmpdir.path().join("config.toml");
        assert!(!path.exists());

        let err = Configuration::load(&path).unwrap_err();

        assert_eq!(
            err.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::NotFound
        );
    }

    /// Loading configuration missing sources returns an appropriate error.
    #[rstest]
    #[case(
        indoc!{r#"
            signers = [
                { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
                { name = "rdavis", principals = ["rdavis@lumon.industries"], sources = ["lumon-industries"] }
            ]

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
        "#},
        vec!["acme-corp".to_string(), "lumon-industries".to_string()]
    )]
    fn loading_configuration_with_missing_source_returns_error(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
        #[case] mut expected_missing: Vec<String>,
    ) {
        expected_missing.sort();
        writeln!(tmp_config_toml, "{config}").unwrap();

        let err = Configuration::load(tmp_config_toml.path()).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!("Missing sources: {}", expected_missing.join(", "))
        );
    }

    /// Loading configuration containing a signer without at least one principal returns an appropriate error.
    #[rstest]
    #[case(
        indoc!{r#"
            [[signers]]
            name = "octocat"
        "#},
    )]
    #[case(
        indoc!{r#"
            [[signers]]
            name = "octocat"
            principals = []
        "#},
    )]
    fn loading_configuration_with_signer_missing_principal_returns_error(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
    ) {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let err = Configuration::load(tmp_config_toml.path()).unwrap_err();

        assert_eq!(err.to_string(), "Signer octocat missing principals");
    }

    #[rstest]
    #[case(
        indoc!{r#"
            [[signers]]
            name = "cwoods"
            principals = ["cwoods@acme.corp"]
            nonsense = ["acme-corp"]

            [[sources]]
            name = "acme-corp"
            provider = "gitlab"
            url = "https://git.acme.corp"
        "#},
        "unknown field `nonsense`"
    )]
    fn loading_configuration_with_unknown_field_returns_error(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
        #[case] expected_msg: &str,
    ) {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let err = Configuration::load(tmp_config_toml.path()).unwrap_err();

        assert!(err.to_string().contains(expected_msg));
    }

    /// Signers have a default GitHub source if no sources were configured explicitly.
    #[rstest]
    #[case(
        indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
            ]
        "#}
    )]
    fn signers_have_default_github_source(
        mut tmp_config_toml: NamedTempFile,
        #[case] config: &str,
    ) {
        writeln!(tmp_config_toml, "{config}").unwrap();

        let mut config = Configuration::load(tmp_config_toml.path()).unwrap();
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
        "#}
    )]
    fn saving_configuration_preserves_formatting(
        mut tmp_config_toml: NamedTempFile,
        #[case] content: &str,
    ) {
        write!(tmp_config_toml, "{content}").unwrap();
        let config = Configuration::load(tmp_config_toml.path()).unwrap();
        tmp_config_toml.as_file().set_len(0).unwrap();

        config.save().unwrap();
        let result = fs::read_to_string(tmp_config_toml.path()).unwrap();

        assert_eq!(result, content);
    }

    /// When adding a signer to a configuration, it is added to the contained signers.
    #[rstest]
    #[case(
        SignerConfiguration {
            name: "octocat".to_string(),
            principals: vec!["octocat@github.com".to_string()],
            ..Default::default()
        }
    )]
    fn adding_signer_adds_to_signers(#[case] signer: SignerConfiguration) {
        let mut config = Configuration::default();

        config
            .add_signer(
                signer.name.clone().clone(),
                signer.principals.clone(),
                signer.source_names.clone(),
            )
            .unwrap();

        assert!(config.signers.contains(&signer));
    }

    /// When adding a signer to a configuration, it is added to the TOML configuration file contained within.
    #[rstest]
    #[case(
        "",
        SignerConfiguration {
            name: "octocat".to_string(),
            principals: vec!["octocat@github.com".to_string()],
            ..Default::default()
        },
        indoc! {r#"
            [[signers]]
            name = "octocat"
            principals = ["octocat@github.com"]
        "#},
    )]
    #[case(
        indoc! {r#"
            [[signers]]
            name = "torvalds"
            principals = ["torvalds@linux-foundation.org"]
        "#},
        SignerConfiguration {
            name: "octocat".to_string(),
            principals: vec!["octocat@github.com".to_string()],
            ..Default::default()
        },
        indoc! {r#"
            [[signers]]
            name = "torvalds"
            principals = ["torvalds@linux-foundation.org"]

            [[signers]]
            name = "octocat"
            principals = ["octocat@github.com"]
        "#},
    )]
    #[case(
        indoc! {r#"
            [[signers]]
            name = "torvalds"
            principals = ["torvalds@linux-foundation.org"]

            [[sources]]
            name = "acme-corp"
            provider = "gitlab"
            url = "https://git.acme.corp"
        "#},
        SignerConfiguration {
            name: "octocat".to_string(),
            principals: vec!["octocat@github.com".to_string()],
            source_names: vec!["acme-corp".to_string()],
        },
        indoc! {r#"
            [[signers]]
            name = "torvalds"
            principals = ["torvalds@linux-foundation.org"]

            [[signers]]
            name = "octocat"
            principals = ["octocat@github.com"]
            sources = ["acme-corp"]

            [[sources]]
            name = "acme-corp"
            provider = "gitlab"
            url = "https://git.acme.corp"
        "#},
    )]
    #[case(
        indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
                { name = "cwoods", principals = ["cwoods@acme.corp"] },
            ]
        "#},
        SignerConfiguration {
            name: "octocat".to_string(),
            principals: vec!["octocat@github.com".to_string()],
            ..Default::default()
        },
        indoc! {r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
                { name = "cwoods", principals = ["cwoods@acme.corp"] }, { name = "octocat", principals = ["octocat@github.com"] },
            ]
        "#},
    )]
    fn adding_signer_adds_to_file(
        #[case] toml: &str,
        #[case] signer: SignerConfiguration,
        #[case] expected: &str,
    ) {
        let mut config = Configuration::try_from(TomlFile {
            document: toml.parse().unwrap(),
            ..Default::default()
        })
        .unwrap();

        config
            .add_signer(signer.name, signer.principals, signer.source_names)
            .unwrap();

        assert_eq!(config.file.document.to_string(), expected);
    }

    #[rstest]
    #[case(
        Configuration::default(),
        SignerConfiguration {
            name: "cwoods".to_string(),
            principals: vec!["cwoods@acme.corp".to_string()],
            source_names: vec!["acme-corp".to_string()],
        },
        vec!["acme-corp".to_string()]
    )]
    fn adding_signer_with_missing_source_returns_error(
        #[case] mut config: Configuration,
        #[case] signer: SignerConfiguration,
        #[case] mut expected_missing: Vec<String>,
    ) {
        expected_missing.sort();

        let err = config
            .add_signer(signer.name, signer.principals, signer.source_names)
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            format!("Missing sources: {}", expected_missing.join(", "))
        );
    }
}
