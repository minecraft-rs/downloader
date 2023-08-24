use std::path::PathBuf;

use chksum::chksum;
use chksum::hash::SHA1;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub enum VerifyStatus {
    /// The file has not been verified
    #[default]
    NotVerified,
    /// The file failed the verification process.
    Failed,
    /// The file passed the verification process.
    Ok,
}

impl std::fmt::Display for VerifyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::NotVerified => "Not Verified",
                Self::Failed => "FAILED",
                Self::Ok => "Ok",
            }
        )
    }
}

pub fn verify_file(hash: &str, path: PathBuf) -> VerifyStatus {
    if let Ok(file) = std::fs::OpenOptions::new().read(true).open(&path) {
        return match chksum::<SHA1, _>(file) {
            Ok(digest) => {
                if digest.to_hex_lowercase() == hash.to_lowercase() {
                    VerifyStatus::Ok
                } else {
                    VerifyStatus::Failed
                }
            },
            Err(_) => VerifyStatus::Failed,
        };
    }

    VerifyStatus::Failed
}
