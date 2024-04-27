# Example Configuration

```toml
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
```
