use thiserror::Error;

use crate::client::DownloadOutput;

#[derive(Error, Debug)]
pub enum ClientDownloaderError {
    #[error("An unexpected error has ocurred.")]
    UnknownError,

    #[error("No such version")]
    NoSuchVersion,

    #[error("No such library")]
    NoSuchLibrary,

    #[error("{0}")]
    Request(#[from] reqwest::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Download(#[from] DownloadError),
}

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("The game directory doesn't exist.")]
    GameDirNotExist,

    #[error("The java bin doesn't exist.")]
    JavaBinNotExist,

    #[error("An unexpected error has ocurred.")]
    UnknownError,

    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum DownloadError {
    /// The Setup is incomplete or bogus.
    #[error("Setup error: {0}")]
    Setup(String),
    /// A Definition of a `Download` is incomplete
    #[error("Download definition: {0}")]
    DownloadDefinition(String),
    /// Writing into a file failed during download.
    #[error("File creation failed: {0}")]
    File(DownloadOutput),
    /// A download failed
    #[error("Download failed for {0}")]
    Download(DownloadOutput),
    /// Download file verification failed.
    #[error("Verification failed for {0}")]
    Verification(DownloadOutput),
}
