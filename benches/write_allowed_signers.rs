use chrono::{Local, TimeZone};
use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};
use hanko::{AllowedSignersEntry, AllowedSignersFile};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut file = AllowedSignersFile {
        file: tempfile::tempfile().unwrap(),
        signers: vec![
            AllowedSignersEntry {
                principals: vec!["j.snow@wall.com".to_string()],
                valid_after: None,
                valid_before: None,
                key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
                    .parse()
                    .unwrap(),
            },
            AllowedSignersEntry {
                principals: vec!["ian.malcom@acme.corp".to_string()],
                valid_after: Some(Local.with_ymd_and_hms(2024, 4, 11, 22, 00, 00).unwrap()),
                valid_before: None,
                key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
                    .parse()
                    .unwrap(),
            },
            AllowedSignersEntry {
                principals: vec!["cwoods@universal.exports".to_string()],
                valid_after: None,
                valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
                key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
                    .parse()
                    .unwrap(),
            },
        ],
    };
    c.bench_function("write the signers file", |b| {
        b.iter(|| file.write().unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
