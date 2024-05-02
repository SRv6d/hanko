//! Types and functions to interact with the OpenSSH `allowed_signers` file.
//!
//! [File Format Documentation](https://man.openbsd.org/ssh-keygen.1#ALLOWED_SIGNERS)
use crate::SshPublicKey;
use chrono::{DateTime, Local};
use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{self, Write},
    path::Path,
};

/// A single entry in the allowed signers file.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AllowedSignersEntry {
    pub principals: Vec<String>,
    pub valid_after: Option<DateTime<Local>>,
    pub valid_before: Option<DateTime<Local>>,
    pub key: SshPublicKey,
}

impl AllowedSignersEntry {
    /// The format string for timestamps.
    const TIMESTAMP_FMT: &'static str = "%Y%m%d%H%M%S";
}

impl fmt::Display for AllowedSignersEntry {
    /// Display the allowed signer in the format expected by the `allowed_signers` file.
    ///
    /// # Examples
    /// ```
    /// # use hanko::AllowedSignersEntry;
    /// # use chrono::{TimeZone, Local};
    /// let signer = AllowedSignersEntry {
    ///     principals: vec!["cwoods@universal.exports".to_string()],
    ///     valid_after: None,
    ///     valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
    ///     key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
    ///         .parse()
    ///         .unwrap(),
    /// };
    /// assert_eq!(signer.to_string(), "cwoods@universal.exports valid-before=20300101000000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.principals.join(","))?;

        if let Some(valid_after) = self.valid_after {
            write!(
                f,
                " valid-after={}",
                valid_after.format(Self::TIMESTAMP_FMT)
            )?;
        };
        if let Some(valid_before) = self.valid_before {
            write!(
                f,
                " valid-before={}",
                valid_before.format(Self::TIMESTAMP_FMT)
            )?;
        };

        write!(f, " {}", self.key)
    }
}

/// The allowed signers file.
#[derive(Debug)]
pub struct AllowedSignersFile {
    pub file: File,
    pub signers: HashSet<AllowedSignersEntry>,
}

impl AllowedSignersFile {
    /// Create a new allowed signers file.
    pub fn new(path: &Path) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file,
            signers: HashSet::new(),
        })
    }

    /// Create a new allowed signers file with a set of signers.
    pub fn with_signers(
        path: &Path,
        signers: impl IntoIterator<Item = AllowedSignersEntry>,
    ) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file,
            signers: HashSet::from_iter(signers),
        })
    }

    /// Add an entry to the file.
    pub fn add(&mut self, signer: AllowedSignersEntry) {
        self.signers.insert(signer);
    }

    /// Write the allowed signers file.
    pub fn write(&mut self) -> io::Result<()> {
        let mut file_buf = io::BufWriter::new(&mut self.file);
        for signer in &self.signers {
            writeln!(file_buf, "{signer}")?;
        }
        writeln!(file_buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone as _;
    use rstest::*;
    use std::fs;

    #[fixture]
    fn signer_jsnow() -> AllowedSignersEntry {
        AllowedSignersEntry {
            principals: vec!["j.snow@wall.com".to_string()],
            valid_after: None,
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn signer_imalcom() -> AllowedSignersEntry {
        AllowedSignersEntry {
            principals: vec!["ian.malcom@acme.corp".to_string()],
            valid_after: Some(Local.with_ymd_and_hms(2024, 4, 11, 22, 00, 00).unwrap()),
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn signer_cwoods() -> AllowedSignersEntry {
        AllowedSignersEntry {
            principals: vec!["cwoods@universal.exports".to_string()],
            valid_after: None,
            valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn signer_ebert() -> AllowedSignersEntry {
        AllowedSignersEntry {
            principals: vec![
                "ernie@muppets.com".to_string(),
                "bert@muppets.com".to_string(),
            ],
            valid_after: None,
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE6h5pPnCWurUHIiHuVp4Hd4mQbEf0bE3EFpETQ2OJt4"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn example_signers() -> Vec<AllowedSignersEntry> {
        vec![signer_jsnow(), signer_imalcom(), signer_cwoods()]
    }

    #[rstest]
    #[case(
        signer_jsnow(),
        "j.snow@wall.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
    )]
    #[case(
        signer_imalcom(),
        "ian.malcom@acme.corp valid-after=20240411220000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
    )]
    #[case(
        signer_cwoods(),
        "cwoods@universal.exports valid-before=20300101000000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
    )]
    #[case(
        signer_ebert(),
        "ernie@muppets.com,bert@muppets.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE6h5pPnCWurUHIiHuVp4Hd4mQbEf0bE3EFpETQ2OJt4"
    )]
    fn display_allowed_signer(#[case] signer: AllowedSignersEntry, #[case] expected_display: &str) {
        assert_eq!(signer.to_string(), expected_display);
    }

    #[rstest]
    fn write_allowed_signers_file(example_signers: Vec<AllowedSignersEntry>) {
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let mut expected_content = String::new();
        for signer in &example_signers {
            expected_content.push_str(&format!("{signer}\n"));
        }
        expected_content.push('\n');

        {
            let mut file = AllowedSignersFile::with_signers(&path, example_signers).unwrap();
            file.write().unwrap();
        }

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, expected_content);
    }

    #[rstest]
    fn writing_overrides_existing_content(example_signers: Vec<AllowedSignersEntry>) {
        let existing_content = "gathered dust";
        let mut existing_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(existing_file, "{existing_content}").unwrap();
        let path = existing_file.into_temp_path();

        {
            let mut file = AllowedSignersFile::with_signers(&path, example_signers).unwrap();
            file.write().unwrap();
        }

        let content = fs::read_to_string(path).unwrap();
        assert!(!content.contains(existing_content));
    }
}
