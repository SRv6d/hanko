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

`hanko` keeps your Git allowed signers file up to date with signing keys configured on software development platforms like GitHub and GitLab.

# Usage

```
Keeps your Git allowed signers file up to date with signing keys configured on software development platforms like GitHub and GitLab.

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

## Adding an allowed signer

To use `hanko`, a set of signers to track need to be configured first.
For starters, we'll add the GitHub user `octocat` with a single principal `octocat@github.com`.

```sh
$ hanko signer add --no-update octocat octocat@github.com
Updated configuration file ~/.config/hanko/config.toml
```

Since we didn't have a configuration file yet, hanko went ahead and created one for us in the default location at `~/.config/hanko/config.toml`, containing our newly added signer.

```toml
[[signers]]
name = "octocat"
principals = ["octocat@github.com"]
```

Given that we told `hanko` not to touch the allowed signers file yet using the `--no-update` argument, it is left as-is. We'll update it in the next step.

> [!TIP]
> Should you prefer to create the configuration file by hand, head to [Configuration](#configuration).

## Updating the allowed signers file

Now that we've configured at least one signer, it's time to update the Git allowed signers file with their signing keys.

```sh
$ hanko update
Updated allowed signers file ~/.config/git/allowed_signers in 105.315473ms.
```

If an allowed signers file is configured in Git, `hanko` will write to that file.
Should no allowed signers file be configured within Git, or should you want to specify a different path, the `--file` runtime option may be used.

Our allowed signers file now contains all signing keys configured by `octocat` under the principal `octocat@github.com`.

```
octocat@github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIN3ZSWa2S+RI/GdKi6WXl4k+FZ8ecAo0H2dtfLRWuhIs
octocat@github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILSK47p5e3KlWAqe1yPkPZUSK3TJVJUzLqKdaPq/ClOa
```

Any commits made by octocat with the email `octocat@github.com` and signed by one of their signing keys will no be considered as valid by Git.

# Configuration

## Example

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
