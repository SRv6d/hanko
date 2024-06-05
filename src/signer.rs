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
    path::PathBuf,
};
use tracing::trace;

/// A single entry in the allowed signers file.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub path: PathBuf,
    pub signers: HashSet<AllowedSignersEntry>,
}

impl AllowedSignersFile {
    /// Create a new allowed signers file.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            signers: HashSet::new(),
        }
    }

    /// Create a new allowed signers file with a set of signers.
    pub fn with_signers(
        path: PathBuf,
        signers: impl IntoIterator<Item = AllowedSignersEntry>,
    ) -> Self {
        Self {
            path,
            signers: HashSet::from_iter(signers),
        }
    }

    /// Add an entry to the file.
    pub fn add(&mut self, signer: AllowedSignersEntry) {
        self.signers.insert(signer);
    }

    /// Write the allowed signers file.
    #[tracing::instrument(skip(self), fields(path = %self.path.display()), level = "trace")]
    pub fn write(&self) -> io::Result<()> {
        trace!("Opening allowed signers file for writing");
        let file = File::create(&self.path)?;
        let mut file_buf = io::BufWriter::new(file);

        let sorted_signers = {
            let mut signers = self.signers.iter().collect::<Vec<_>>();
            signers.sort();
            signers
        };
        trace!("Writing to allowed signers file");
        for signer in sorted_signers {
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
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDw32w3ciofX3/gFoyCtPWxSsWYmylwdKZ9Q/BmoBR/g"
                .parse()
                .unwrap(),
        }
    }

    /// Returns an example allowed signers file containing a temporary `File` that will be
    /// cleaned up, along with the path to that temporary file.
    #[fixture]
    fn example_allowed_signers() -> (AllowedSignersFile, tempfile::TempPath) {
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        (
            AllowedSignersFile::with_signers(
                path.to_path_buf(),
                vec![
                    signer_jsnow(),
                    signer_imalcom(),
                    signer_cwoods(),
                    signer_ebert(),
                ],
            ),
            path,
        )
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
        "ernie@muppets.com,bert@muppets.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDw32w3ciofX3/gFoyCtPWxSsWYmylwdKZ9Q/BmoBR/g"
    )]
    fn display_allowed_signer(#[case] signer: AllowedSignersEntry, #[case] expected_display: &str) {
        assert_eq!(signer.to_string(), expected_display);
    }

    /// Writing the allowed signers file creates a file that contains all entries.
    #[rstest]
    fn written_signers_file_contains_all_entries(
        example_allowed_signers: (AllowedSignersFile, tempfile::TempPath),
    ) {
        let (mut file, path) = example_allowed_signers;

        file.write().unwrap();

        let content = fs::read_to_string(path).unwrap();
        for entry in &file.signers {
            assert!(content.contains(&entry.to_string()));
        }
    }

    /// Writing the allowed signers file creates a file that is newline terminated.
    #[rstest]
    fn written_signers_file_is_newline_terminated(
        example_allowed_signers: (AllowedSignersFile, tempfile::TempPath),
    ) {
        let (mut file, path) = example_allowed_signers;

        file.write().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.ends_with("\n\n")); // Two newlines since the last entry already ends with one.
    }

    #[rstest]
    fn writing_overrides_existing_content(
        example_allowed_signers: (AllowedSignersFile, tempfile::TempPath),
    ) {
        let (mut file, path) = example_allowed_signers;
        let existing_content = "gathered dust";
        fs::write(&path, existing_content).unwrap();

        file.write().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(!content.contains(existing_content));
    }
}
