mod client_downloader;
mod downloader;
mod verify;

use std::path::PathBuf;

pub use client_downloader::*;
pub use downloader::*;
pub use verify::*;

use crate::{
    error::{ClientDownloaderError, DownloadError},
    manifest::Manifest,
};

pub type DownloadResult = Result<DownloadOutput, DownloadError>;

#[derive(Default, Clone)]
pub struct DownloadOutput {
    pub status: u16,
    pub file_name: String,
    pub file_path: PathBuf,
    pub verified: VerifyStatus,
}

/// A Progress reporter to use for the `Download`
pub type Progress = std::sync::Arc<dyn Reporter>;

/// An interface for `ProgressReporter`s
pub trait Reporter: Send + Sync {
    fn setup(&self, max_progress: u64);
    /// Report progress
    fn progress(&self, current: u64);
    /// Finish up after progress reporting is done
    fn done(&self);
}

pub trait DownloadVersion {
    fn download_version(
        &self,
        _version_id: &str,
        _dir: &str,
        _progress: Progress,
    ) -> Result<Vec<DownloadResult>, ClientDownloaderError>;

    fn download_by_manifest(
        &self,
        _manifest: Manifest,
        _dir: &str,
        _progress: Progress,
    ) -> Result<Vec<DownloadResult>, ClientDownloaderError>;
}

pub trait DownloadJava {
    fn check_version(&self, _root_path: &str, _expected_version: &str) -> bool;
    fn download_java(&self, _root_path: &str, _version: &str, _progress: Progress);
}

fn download_result_to_fmt(
    f: &mut std::fmt::Formatter<'_>,
    summary: &DownloadOutput,
) -> std::fmt::Result {
    writeln!(
        f,
        "{}: (verification: {}) Status: {}",
        summary.file_name,
        match summary.verified {
            VerifyStatus::NotVerified => "unverified",
            VerifyStatus::Failed => "FAILED",
            VerifyStatus::Ok => "Ok",
        },
        summary.status,
    )?;
    Ok(())
}

impl std::fmt::Display for DownloadOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        download_result_to_fmt(f, self)
    }
}

impl std::fmt::Debug for DownloadOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        download_result_to_fmt(f, self)
    }
}
