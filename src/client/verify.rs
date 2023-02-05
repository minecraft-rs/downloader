use std::{
    io::{BufReader, Read},
    path::PathBuf,
};

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

pub fn verify_file<D: digest::Digest>(hash: Vec<u8>, path: PathBuf) -> VerifyStatus {
    if let Ok(file) = std::fs::OpenOptions::new().read(true).open(&path) {
        let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
        return verify_raw::<D>(hash, &mut reader);
    }

    VerifyStatus::Failed
}

pub fn verify_raw<D: digest::Digest>(
    hash: Vec<u8>,
    reader: &mut BufReader<impl Read>,
) -> VerifyStatus {
    let mut hasher = D::new();

    let mut buffer = [0_u8; 1024 * 1024];
    while let Ok(n) = reader.read(&mut buffer[..]) {
        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();

    if result.len() != hash.len() {
        return VerifyStatus::Failed;
    }
    for i in 0..result.len() {
        if result[i] != hash[i] {
            return VerifyStatus::Failed;
        }
    }
    VerifyStatus::Ok
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use sha1::Sha1;
    use std::io::BufReader;
    use stringreader::StringReader;

    use super::{verify_raw, VerifyStatus};

    #[test]
    fn check_verify() {
        let hash = hex!("661295c9cbf9d6b2f6428414504a8deed3020641").to_vec();
        let mut reader = BufReader::new(StringReader::new("test string"));

        let result = verify_raw::<Sha1>(hash, &mut reader);

        assert_eq!(result, VerifyStatus::Ok);
    }

    #[test]
    fn failed_verify() {
        let hash = hex!("661295c9cbf9d6b2f6428414504a8deed3020641").to_vec();
        let mut reader = BufReader::new(StringReader::new("test strin"));

        let result = verify_raw::<Sha1>(hash, &mut reader);

        assert_eq!(result, VerifyStatus::Failed);
    }
}
