default: check-lockfile lint test

# Check if the lockfile is up to date
check-lockfile:
    cargo update -w --locked --offline

# Lint code and check formatting
lint: lint-justfile
    cargo clippy --all-targets --all-features
    cargo fmt --all --check

lint-justfile:
    just --check --fmt --unstable

# Run tests
test:
    cargo test --all-features

# Publish the crate
publish: _validate_version_tag
    cargo publish --no-verify

# Validate that the crate version matches that of the git tag
_validate_version_tag:
    #!/usr/bin/env bash
    set -euxo pipefail
    PROJECT_VERSION="$(grep -Po '(?<=^version = ").*(?=")' Cargo.toml)"
    GIT_TAG="$(git describe --exact-match --tags)"

    if [ ! $PROJECT_VERSION == ${GIT_TAG:1} ]; then
        echo Project version $PROJECT_VERSION does not match git tag $GIT_TAG
        exit 1
    fi
