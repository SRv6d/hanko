//! Types and functions to interact with the OpenSSH `allowed_signers` file.
//!
//! [File Format Documentation](https://man.openbsd.org/ssh-keygen.1#ALLOWED_SIGNERS)
use crate::SshPublicKey;
use chrono::{DateTime, Local};
use std::{
    fmt,
    fs::File,
    io::{self, Write},
    path::Path,
};

/// A single entry in the allowed signers file.
#[derive(Debug)]
pub struct AllowedSigner {
    pub principal: String,
    pub valid_after: Option<DateTime<Local>>,
    pub valid_before: Option<DateTime<Local>>,
    pub key: SshPublicKey,
}

impl AllowedSigner {
    /// The format string for timestamps.
    const TIMESTAMP_FMT: &'static str = "%Y%m%d%H%M%S";
}

impl fmt::Display for AllowedSigner {
    /// Display the allowed signer in the format expected by the `allowed_signers` file.
    ///
    /// # Examples
    /// ```
    /// # use hanko::AllowedSigner;
    /// # use chrono::{TimeZone, Local};
    /// let signer = AllowedSigner {
    ///     principal: "cwoods@universal.exports".to_string(),
    ///     valid_after: None,
    ///     valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
    ///     key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
    ///         .parse()
    ///         .unwrap(),
    /// };
    /// assert_eq!(signer.to_string(), "cwoods@universal.exports valid-before=20300101000000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.principal)?;

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
    pub signers: Vec<AllowedSigner>,
}

impl AllowedSignersFile {
    pub fn new(path: &Path, signers: Vec<AllowedSigner>) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self { file, signers })
    }

    /// Write the allowed signers file.
    pub fn write(&mut self) -> io::Result<()> {
        let mut file_buf = io::BufWriter::new(&mut self.file);
        for signer in &self.signers {
            writeln!(file_buf, "{}", signer)?;
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
    fn signer_jsnow() -> AllowedSigner {
        AllowedSigner {
            principal: "j.snow@wall.com".to_string(),
            valid_after: None,
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn signer_imalcom() -> AllowedSigner {
        AllowedSigner {
            principal: "ian.malcom@acme.corp".to_string(),
            valid_after: Some(Local.with_ymd_and_hms(2024, 4, 11, 22, 00, 00).unwrap()),
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn signer_cwoods() -> AllowedSigner {
        AllowedSigner {
            principal: "cwoods@universal.exports".to_string(),
            valid_after: None,
            valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
                .parse()
                .unwrap(),
        }
    }

    #[fixture]
    fn example_signers() -> Vec<AllowedSigner> {
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
    fn display_allowed_signer(#[case] signer: AllowedSigner, #[case] expected_display: &str) {
        assert_eq!(signer.to_string(), expected_display);
    }

    #[rstest]
    fn write_allowed_signers_file(example_signers: Vec<AllowedSigner>) {
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let mut expected_content = String::new();
        for signer in &example_signers {
            expected_content.push_str(&format!("{}\n", signer));
        }
        expected_content.push('\n');

        {
            let mut file = AllowedSignersFile::new(&path, example_signers).unwrap();
            file.write().unwrap();
        }

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, expected_content);
    }

    #[rstest]
    fn writing_overrides_existing_content(example_signers: Vec<AllowedSigner>) {
        let existing_content = "gathered dust";
        let mut existing_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(existing_file, "{}", existing_content).unwrap();
        let path = existing_file.into_temp_path();

        {
            let mut file = AllowedSignersFile::new(&path, example_signers).unwrap();
            file.write().unwrap();
        }

        let content = fs::read_to_string(path).unwrap();
        assert!(!content.contains(existing_content));
    }
}
