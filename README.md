<h1 align="center"><code>hanko</code></h1>
<div align="center">
  <a href="https://github.com/srv6d/hanko/actions">
    <img src="https://github.com/srv6d/hanko/workflows/CI/badge.svg" alt="CI status" />
  </a>
  <a href="https://codecov.io/github/SRv6d/hanko">
    <img src="https://codecov.io/github/SRv6d/hanko/graph/badge.svg?token=PIRC5DZL9C" />
  </a>
  <a href="https://codspeed.io/SRv6d/hanko">
    <img src="https://img.shields.io/endpoint?url=https://codspeed.io/badge.json" alt="CodSpeed Badge" />
  </a>
  <a href="https://crates.io/crates/hanko">
    <img src="https://img.shields.io/crates/v/hanko.svg?logo=rust" alt="Cargo version" />
  </a>
</div>
<div align="center">
  <a href="https://scorecard.dev/viewer/?uri=github.com/SRv6d/hanko">
    <img src="https://api.scorecard.dev/projects/github.com/SRv6d/hanko/badge" />
  </a>
  <a href="https://www.bestpractices.dev/projects/9526">
    <img src="https://www.bestpractices.dev/projects/9526/badge" />
  </a>
</div>
<br />

`hanko` keeps your allowed signers file up to date with signing keys configured on platforms like GitHub and GitLab.

# Usage

```
Keeps your allowed signers file up to date with signing keys configured on platforms like GitHub and GitLab.

Usage: hanko [OPTIONS] <COMMAND>

Commands:
  update  Update the allowed signers file
  signer  Manage allowed signers
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <PATH>  The configuration file [env: HANKO_CONFIG=]
      --file <PATH>    The allowed signers file [env: HANKO_ALLOWED_SIGNERS=]
  -v, --verbose...     Use verbose output
  -h, --help           Print help
  -V, --version        Print version
```

## Configuring Signers

To use `hanko`, a set of signers to track need to be configured first. As an example,
we'll create a configuration file in the default location `~/.config/hanko/config.toml`
containing a single signer using the default GitHub source.

```toml
[[signers]]
name = "octocat"
principals = ["octocat@github.com"]
```

## Updating the allowed signers file

Once we've configured our signers, we can run the `update` command.

If an allowed signers file is configured in Git, `hanko` will write to that file.
Should no allowed signers file be configured within Git, or you want to specify a different path, the `--file` runtime option may be used.

```sh
$ hanko -v update
2024-10-25T14:01:49.140028Z  INFO load_and_validate: hanko::config: Loading configuration file path="/home/vscode/.config/hanko/config.toml"
2024-10-25T14:01:49.243660Z  INFO hanko::cli::main: Updated allowed signers file /home/vscode/.config/git/allowed_signers in 105.315473ms
```

# Example Configuration

```toml
signers = [
    { name = "torvalds", principals = ["torvalds@linux-foundation.org"], sources = ["github"] },
    { name = "gvanrossum", principals = ["guido@python.org"], sources = ["github", "gitlab"] },
    { name = "graydon", principals = ["graydon@pobox.com"], sources = ["github"] },
    { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
    { name = "rdavis", principals = ["rdavis@acme.corp"], sources = ["acme-corp"] },
    { name = "pbrock", principals = ["pbrock@acme.corp"], sources = ["acme-corp"] }
]

[[sources]]
name = "acme-corp"
provider = "gitlab"
url = "https://git.acme.corp"
```

## Optional Features

The following cargo features can be used to enable additional functionality.

- **detect-allowed-signers** _(enabled by default)_: Enables use of the [gix-config] crate to detect the location of the allowed signers file from Git configuration.

## Contributing

Contributions of all sizes that improve `hanko` in any way, be it DX/UX, documentation, performance or other are highly appreciated.
To get started, please read the [contribution guidelines](.github/CONTRIBUTING.md). Before starting work on a new feature you would like to contribute that may impact simplicity, reliability or performance, please open an issue first.

## License

The source code of this project is licensed under the MIT License. For more information, see [LICENSE](LICENSE).

[gix-config]: https://crates.io/crates/gix-config
