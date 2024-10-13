use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};
use hanko::Configuration;
use indoc::indoc;
use std::{io::Write, path::Path};

pub fn criterion_benchmark(c: &mut Criterion) {
    let toml = indoc! {r#"
        users = [
            { name = "torvalds", principals = ["torvalds@linux-foundation.org"], sources = ["github"] },
            { name = "gvanrossum", principals = ["guido@python.org"], sources = ["github", "gitlab"] },
            { name = "graydon", principals = ["graydon@pobox.com"], sources = ["github"] },
            { name = "cwoods", principals = ["cwoods@acme.corp"], sources = ["acme-corp"] },
            { name = "rdavis", principals = ["rdavis@acme.corp"], sources = ["acme-corp"] },
            { name = "pbrock", principals = ["pbrock@acme.corp"], sources = ["acme-corp"] }
        ]
        file = "~/allowed_signers"
        
        [[sources]]
        name = "acme-corp"
        provider = "gitlab"
        url = "https://git.acme.corp"
        "#};
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let path: &Path = &file.into_temp_path();

    c.bench_function("load the example configuration", |b| {
        b.iter(|| Configuration::load(path, None).unwrap());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
