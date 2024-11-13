//! Ensure correct behavior of the signer management subcommand.
use assert_cmd::Command;
use indoc::indoc;
use rstest::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// When adding a signer, the configuration file is updated accordingly.
#[rstest]
#[case(
    indoc!{r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
            ]
    "#},
    vec!["octocat", "octocat@github.com"],
    indoc!{r#"
            signers = [
                { name = "torvalds", principals = ["torvalds@linux-foundation.org"] },
                { name = "octocat", principals = ["octocat@github.com"] },
            ]
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
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--config")
        .arg(config.path())
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
