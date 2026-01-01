//! General top level CLI tests.
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_contains_version() {
    let version = format!("hanko {}", env!("CARGO_PKG_VERSION"));

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--version");

    cmd.assert().success();
    cmd.assert().stdout(predicate::str::starts_with(version));
}
