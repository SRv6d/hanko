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
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <PATH>           The configuration file [env: HANKO_CONFIG=]
      --allowed-signers <PATH>  The allowed signers file [env: HANKO_ALLOWED_SIGNERS=]
  -v, --verbose...              Increase verbosity
  -h, --help                    Print help
  -V, --version                 Print version
```

## Configuring Users

To use `hanko`, a set of users to track need to be configured first. As an example,
we'll create a configuration file in the default location `~/.config/hanko/config.toml`.

```toml
[[users]]
name = "hynek"
principals = ["hs@example.com"]

[[users]]
name = "fasterthanlime"
principals = ["amos@example.com"]

[[users]]
name = "adriangb"
principals = ["adriangb@example.com"]
```

## Updating the allowed signers file

Once we've configured our users, we can run the `update` command.
If it exists, `hanko` will write it's output to the path of the allowed signers file configured within git. If no allowed signers file is configured within git,
or you want `hanko` to write to a different path, the `--allowed-signers` runtime option may be used.

```sh
$ hanko -v update
...
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

## Contributing

Contributions of all sizes that improve `hanko` in any way, be it DX/UX, documentation, performance or other are highly appreciated.
To get started, please read the [contribution guidelines](.github/CONTRIBUTING.md). Before starting work on a new feature you would like to contribute that may impact simplicity, reliability or performance, please open an issue first.

## License

The source code of this project is licensed under the MIT License. For more information, see [LICENSE](LICENSE).
