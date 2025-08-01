//! Types representing the OpenSSH `allowed_signers` file.
//!
//! [File Format Documentation](https://man.openbsd.org/ssh-keygen.1#ALLOWED_SIGNERS)
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Local};
use tracing::trace;

use super::{
    signer::{get_entries, Signer},
    ssh::PublicKey,
};

/// The allowed signers file.
#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub entries: Vec<Entry>, // TODO: Use HashSet
}

impl File {
    /// Write the file to disk.
    #[tracing::instrument(skip(self), fields(path = %self.path.display()), level = "trace")]
    pub fn write(&self) -> io::Result<()> {
        trace!("Opening allowed signers file for writing");
        let file = fs::File::create(&self.path)?;
        let mut file_buf = io::BufWriter::new(file);

        let sorted_entries = {
            let mut entries = self.entries.iter().collect::<Vec<_>>();
            entries.sort();
            entries
        };
        trace!("Writing to allowed signers file");
        for entry in sorted_entries {
            writeln!(file_buf, "{entry}")?;
        }
        writeln!(file_buf)?;
        Ok(())
    }

    /// Create an instance from a collection of entries.
    pub fn from_entries<E>(path: PathBuf, entries: E) -> Self
    where
        E: IntoIterator<Item = Entry>,
    {
        Self {
            path,
            entries: entries.into_iter().collect(),
        }
    }
}

/// An entry in the allowed signers file.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entry {
    pub principals: Vec<String>,
    pub valid_after: Option<DateTime<Local>>,
    pub valid_before: Option<DateTime<Local>>,
    pub key: PublicKey,
}

impl Entry {
    #[must_use]
    /// Create a new signer entry.
    ///
    /// # Panics
    /// If the provided principals are empty.
    pub fn new(
        principals: Vec<String>,
        valid_after: Option<DateTime<Local>>,
        valid_before: Option<DateTime<Local>>,
        key: PublicKey,
    ) -> Self {
        assert!(
            !principals.is_empty(),
            "signer entry requires at least one principal"
        );
        Entry {
            principals,
            valid_after,
            valid_before,
            key,
        }
    }
}

impl fmt::Display for Entry {
    /// Display the entry in the format expected by the allowed signers file.
    ///
    /// # Examples
    /// ```
    /// # use hanko::allowed_signers::Entry;
    /// # use chrono::{TimeZone, Local};
    /// let signer = Entry {
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
        const TIMESTAMP_FMT: &str = "%Y%m%d%H%M%S";

        write!(f, "{}", self.principals.join(","))?;

        if let Some(valid_after) = self.valid_after {
            write!(f, " valid-after={}", valid_after.format(TIMESTAMP_FMT))?;
        }
        if let Some(valid_before) = self.valid_before {
            write!(f, " valid-before={}", valid_before.format(TIMESTAMP_FMT))?;
        }

        write!(f, " {}", self.key)
    }
}

/// Update the allowed signers file.
pub async fn update<S>(path: &Path, signers: S) -> anyhow::Result<()>
where
    S: IntoIterator<Item = Signer>,
{
    let entries = get_entries(signers).await?;

    let file = File::from_entries(path.to_path_buf(), entries);
    file.write().context(format!(
        "Failed to write allowed signers file to {}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone as _;
    use rstest::*;
    use std::fs;

    #[fixture]
    fn entry_jsnow() -> Entry {
        Entry {
            principals: vec!["j.snow@wall.com".to_string()],
            valid_after: None,
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn entry_imalcom() -> Entry {
        Entry {
            principals: vec!["ian.malcom@acme.corp".to_string()],
            valid_after: Some(Local.with_ymd_and_hms(2024, 4, 11, 22, 00, 00).unwrap()),
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn entry_cwoods() -> Entry {
        Entry {
            principals: vec!["cwoods@universal.exports".to_string()],
            valid_after: None,
            valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn entry_ebert() -> Entry {
        Entry {
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
    fn example_allowed_signers() -> (File, tempfile::TempPath) {
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        (
            File::from_entries(
                path.to_path_buf(),
                [
                    entry_jsnow(),
                    entry_imalcom(),
                    entry_cwoods(),
                    entry_ebert(),
                ],
            ),
            path,
        )
    }

    #[test]
    #[should_panic(expected = "signer entry requires at least one principal")]
    fn new_entry_without_principal_panics() {
        let _ = Entry::new(vec![], None, None, entry_jsnow().key);
    }

    #[rstest]
    #[case(
        entry_jsnow(),
        "j.snow@wall.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
    )]
    #[case(
        entry_imalcom(),
        "ian.malcom@acme.corp valid-after=20240411220000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
    )]
    #[case(
        entry_cwoods(),
        "cwoods@universal.exports valid-before=20300101000000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
    )]
    #[case(
        entry_ebert(),
        "ernie@muppets.com,bert@muppets.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDw32w3ciofX3/gFoyCtPWxSsWYmylwdKZ9Q/BmoBR/g"
    )]
    fn display_allowed_signer(#[case] signer: Entry, #[case] expected_display: &str) {
        assert_eq!(signer.to_string(), expected_display);
    }

    /// Writing the allowed signers file creates a file that contains all entries.
    #[rstest]
    fn written_signers_file_contains_all_entries(
        example_allowed_signers: (File, tempfile::TempPath),
    ) {
        let (file, path) = example_allowed_signers;

        file.write().unwrap();

        let content = fs::read_to_string(path).unwrap();
        for entry in &file.entries {
            assert!(content.contains(&entry.to_string()));
        }
    }

    /// Writing the allowed signers file creates a file that is newline terminated.
    #[rstest]
    fn written_signers_file_is_newline_terminated(
        example_allowed_signers: (File, tempfile::TempPath),
    ) {
        let (file, path) = example_allowed_signers;

        file.write().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.ends_with("\n\n")); // Two newlines since the last entry already ends with one.
    }

    #[rstest]
    fn writing_overrides_existing_content(example_allowed_signers: (File, tempfile::TempPath)) {
        let (file, path) = example_allowed_signers;
        let existing_content = "gathered dust";
        fs::write(&path, existing_content).unwrap();

        file.write().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(!content.contains(existing_content));
    }
}
