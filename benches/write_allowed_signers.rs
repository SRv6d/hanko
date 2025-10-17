//! Benchmark writing allowed signers to file.
#![allow(clippy::missing_panics_doc)]
use chrono::{Local, TimeZone};
use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use hanko::allowed_signers::{Entry, File, PublicKey};

pub fn criterion_benchmark(c: &mut Criterion) {
    let file = File::from_entries(
        tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .to_path_buf(),
        vec![
            Entry::new(
                vec!["j.snow@wall.com".to_string()],
                PublicKey{
                    blob: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS".to_string(),
                    valid_after: None,
                    valid_before: None
                }
            ),
            Entry::new(
                vec!["ian.malcom@acme.corp".to_string()],
                PublicKey{
                    blob: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9".to_string(),
                    valid_after: Some(Local.with_ymd_and_hms(2024, 4, 11, 22, 00, 00).unwrap().into()),
                    valid_before: None
                }
            ),
            Entry::new(
                vec!["cwoods@universal.exports".to_string()],
                PublicKey{
                    blob: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw".to_string(),
                    valid_after: None,
                    valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap().into()),
                }
            ),
        ],
    );

    c.bench_function("write the signers file", |b| {
        b.iter(|| file.write().unwrap());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
