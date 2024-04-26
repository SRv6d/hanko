use crate::GitProvider;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The main configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    users: Option<Vec<User>>,
    organizations: Option<Vec<Organization>>,
    local: Option<Vec<String>>,
    sources: Option<Vec<Source>>,
}

impl Default for Config {
    /// The default configuration containing common sources.
    fn default() -> Self {
        Config {
            users: None,
            organizations: None,
            local: None,
            sources: Some(vec![
                Source {
                    name: "github".to_string(),
                    provider: GitProvider::Github,
                    url: "https://api.github.com".to_string(),
                },
                Source {
                    name: "gitlab".to_string(),
                    provider: GitProvider::Gitlab,
                    url: "https://gitlab.com".to_string(),
                },
            ]),
        }
    }
}

impl Config {
    /// Load the configuration from a TOML file at the given path.
    pub fn load(path: &Path) -> figment::Result<Self> {
        Figment::from(Serialized::defaults(Config::default()))
            .admerge(Toml::file(path))
            .extract()
    }

    /// Create the configuration from a TOML string.
    fn from_toml(toml: &str) -> figment::Result<Self> {
        Figment::from(Serialized::defaults(Config::default()))
            .admerge(Toml::string(toml))
            .extract()
    }

    /// Save the configuration.
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Save the configuration while preserving formatting.");
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct User {
    name: String,
    sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Organization {
    name: String,
    sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Source {
    name: String,
    provider: GitProvider,
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

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
        organizations = [
            { name = "rust-lang", sources = ["github"] }
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
            organizations: Some(vec![
                Organization {
                    name: "rust-lang".to_string(),
                    sources: vec!["github".to_string()],
                }
            ]),
            local: Some(vec!["jdoe@example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw".parse().unwrap()]),
            sources: Some(vec![
                Source {
                    name: "github".to_string(),
                    provider: GitProvider::Github,
                    url: "https://api.github.com".to_string(),
                },
                Source {
                    name: "gitlab".to_string(),
                    provider: GitProvider::Gitlab,
                    url: "https://gitlab.com".to_string(),
                },
                Source {
                    name: "acme-corp".to_string(),
                    provider: GitProvider::Gitlab,
                    url: "https://git.acme.corp".to_string(),
                },
            ])
        };

        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config, expected);
    }
}
