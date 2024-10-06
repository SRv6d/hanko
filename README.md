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
  <a href="https://www.bestpractices.dev/projects/9526"><img src="https://www.bestpractices.dev/projects/9526/badge" /></a>
</div>
<br />

`hanko` manages your local Git allowed signers file for you using signing keys
configured on platforms like GitHub and GitLab.

> [!WARNING]  
> This project is a work in progress and not (yet) ready for production usage.

# Example Configuration

```toml
users = [
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
