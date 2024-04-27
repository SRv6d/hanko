use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};
use hanko::Config;
use indoc::indoc;
use std::{io::Write, path::Path};

pub fn criterion_benchmark(c: &mut Criterion) {
    let toml = indoc! {r#"
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
        "#};
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let path: &Path = &file.into_temp_path();

    c.bench_function("load the example configuration", |b| {
        b.iter(|| Config::load(path).unwrap());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
