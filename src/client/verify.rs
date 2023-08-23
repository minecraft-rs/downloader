use std::path::PathBuf;

use chksum::{hash::sha1::Digest, prelude::HashDigest, Chksum};

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
    if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(&path) {
        return match file.chksum(chksum::prelude::HashAlgorithm::SHA1) {
            Ok(digest) => match Digest::try_from(hash) {
                Ok(expected) => {
                    if digest == HashDigest::SHA1(expected) {
                        VerifyStatus::Ok
                    } else {
                        VerifyStatus::Failed
                    }
                }
                Err(_) => VerifyStatus::Failed,
            },
            Err(_) => VerifyStatus::Failed,
        };
    }

    VerifyStatus::Failed
}
