//! Ensure correct behavior of the signer management subcommand.
use assert_cmd::cargo::cargo_bin_cmd;
use indoc::indoc;
use predicates::prelude::*;
use rstest::*;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

/// When adding a signer, the configuration file is updated accordingly.
#[rstest]
#[case(
    indoc!{r#"
            [[signers]]
            name = "torvalds"
            principals = ["torvalds@linux-foundation.org"]
    "#},
    vec!["octocat", "octocat@github.com"],
    indoc!{r#"
            [[signers]]
            name = "torvalds"
            principals = ["torvalds@linux-foundation.org"]

            [[signers]]
            name = "octocat"
            principals = ["octocat@github.com"]
    "#}
)]
fn adding_signer_updates_configuration(
    #[case] config: &str,
    #[case] args: Vec<&str>,
    #[case] expected: &str,
) {
    let config = {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(config.as_bytes()).unwrap();
        file
    };
    let mut cmd = cargo_bin_cmd!();

    cmd.arg("--config")
        .arg(config.path())
        .arg("--file")
        .arg(NamedTempFile::new().unwrap().path())
        .arg("signer")
        .arg("add")
        .arg("--no-update");
    for arg in args {
        cmd.arg(arg);
    }

    cmd.assert().success();
    let result = std::fs::read_to_string(config.path()).unwrap();

    assert_eq!(result, expected);
}

/// Adding a signer creates a configuration file with the corresponding signer if no file exists yet.
#[rstest]
#[case(
    vec!["octocat", "octocat@github.com"],
    indoc!{r#"
            [[signers]]
            name = "octocat"
            principals = ["octocat@github.com"]
    "#}
)]
fn adding_signer_creates_configuration_if_none_exists(
    #[case] args: Vec<&str>,
    #[case] expected: &str,
) {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().join("config.toml");
    assert!(!path.exists());
    let mut cmd = cargo_bin_cmd!();
    cmd.arg("--config")
        .arg(&path)
        .arg("--file")
        .arg(NamedTempFile::new().unwrap().path())
        .arg("signer")
        .arg("add")
        .arg("--no-update");
    for arg in args {
        cmd.arg(arg);
    }

    cmd.assert().success();
    let result = std::fs::read_to_string(path).unwrap();

    assert_eq!(result, expected);
}

/// Adding a signer requires specifying at least one principal.
#[test]
fn adding_signer_requires_principal() {
    let mut cmd = cargo_bin_cmd!();
    cmd.arg("--file")
        .arg(NamedTempFile::new().unwrap().path())
        .arg("signer")
        .arg("add")
        .arg("--no-update")
        .arg("octocat");

    cmd.assert().failure();
    cmd.assert().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
    cmd.assert().stderr(predicate::str::contains("PRINCIPALS"));
}
