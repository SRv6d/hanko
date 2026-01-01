//! General top level CLI tests.
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn version_contains_version() {
    let version = format!("hanko {}", env!("CARGO_PKG_VERSION"));

    let mut cmd = cargo_bin_cmd!();
    cmd.arg("--version");

    cmd.assert().success();
    cmd.assert().stdout(predicate::str::starts_with(version));
}
