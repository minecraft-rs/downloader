use futures::stream::{self, StreamExt};
use reqwest::Client;
use sha1::Sha1;
use std::fs::create_dir_all;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinError;

use crate::error::DownloadError;
use crate::manifest::ManifestFile;

use super::{verify, DownloadOutput, DownloadResult, Progress, VerifyStatus};

#[derive(Clone, Debug)]
pub struct DownloadData {
    pub(crate) url: String,
    pub(crate) file_name: String,
    pub(crate) output_path: String,
    pub(crate) sha1: String,
    pub(crate) total_size: u64,
}

#[derive(Clone)]
pub struct DownloaderService {
    client: Client,
    downloads: Vec<DownloadData>,
    parallel_requests: u16,
    retries: u16,
    download_folder: PathBuf,
}

fn file_name_from_url(url: &str) -> std::path::PathBuf {
    if url.is_empty() {
        return std::path::PathBuf::new();
    }
    let Ok(url) = reqwest::Url::parse(url) else { return std::path::PathBuf::new() };

    url.path_segments()
        .map_or_else(std::path::PathBuf::new, |f| {
            std::path::PathBuf::from(f.last().unwrap_or(""))
        })
}

async fn download_url(
    client: reqwest::Client,
    url: String,
    writer: &mut std::io::BufWriter<std::fs::File>,
    progress_opt: Option<Progress>,
) -> u16 {
    let Some(progress) = progress_opt else { return reqwest::StatusCode::NOT_IMPLEMENTED.as_u16() };
    if let Ok(mut response) = client.get(&url).send().await {
        let mut current: u64 = 0;
        writer.seek(SeekFrom::Start(current)).unwrap_or(0);

        while let Some(bytes) = response.chunk().await.unwrap_or(None) {
            if writer.write_all(&bytes).is_err() {}

            current += bytes.len() as u64;
            progress.lock().unwrap().progress(bytes.len() as u64);
        }

        response.status().as_u16()
    } else {
        reqwest::StatusCode::BAD_REQUEST.as_u16()
    }
}

async fn download(
    client: Client,
    download: DownloadData,
    retries: u16,
    download_folder: PathBuf,
    progress: Option<Progress>,
) -> Result<DownloadOutput, DownloadError> {
    let mut result = DownloadOutput {
        status: reqwest::StatusCode::OK.as_u16(),
        // @TODO
        file_name: download.file_name.clone(),
        file_path: PathBuf::from(download.output_path.clone()),
        verified: VerifyStatus::NotVerified,
    };

    let mut download_successful = false;
    let mut output_path = download_folder.clone();
    output_path.push(download.output_path);

    create_dir_all(output_path.parent().unwrap())
        .map_err(|e| DownloadError::Setup(e.to_string()))?;

    if let Ok(file) = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output_path)
    {
        let mut writer = std::io::BufWriter::new(file);

        let url = download.url;
        for _ in 1..=retries {
            let s = reqwest::StatusCode::from_u16(
                download_url(client.clone(), url.clone(), &mut writer, progress.clone()).await,
            )
            .unwrap_or(reqwest::StatusCode::BAD_REQUEST);

            result.status = s.as_u16();

            if s.is_server_error() {
                break;
            }

            if s.is_success() {
                download_successful = true;
                break;
            }
        }
    }

    if !download_successful {
        return Err(DownloadError::Download(result));
    }

    result.verified = if !download.sha1.is_empty() {
        // @FIX: verification system
        verify::verify_file::<Sha1>(download.sha1.into_bytes(), result.file_path.clone())
    } else {
        VerifyStatus::Ok
    };

    // Ignoring verification
    // if result.verified == VerifyStatus::Failed {
    //     return Err(DownloadError::Verification(result));
    // }

    Ok(result)
}

impl DownloadData {
    pub fn new(url: &str, path: &str) -> Self {
        Self {
            url: url.to_string(),
            file_name: file_name_from_url(url)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            output_path: path.to_string(),
            sha1: String::new(),
            total_size: 0,
        }
    }
}

impl From<ManifestFile> for DownloadData {
    fn from(manifest: ManifestFile) -> Self {
        Self {
            url: manifest.url.clone(),
            file_name: file_name_from_url(&manifest.url)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            output_path: manifest.path.unwrap_or_default(),
            sha1: manifest.sha1,
            total_size: manifest.size,
        }
    }
}

impl Default for DownloaderService {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent(format!(
                    "{}/{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ))
                .connect_timeout(Duration::from_secs(30))
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            downloads: Vec::new(),
            parallel_requests: 32,
            retries: 3,
            download_folder: Default::default(),
        }
    }
}

impl DownloaderService {
    pub fn new(download_folder: &str) -> Self {
        Self {
            download_folder: PathBuf::from(download_folder),
            ..Default::default()
        }
    }

    pub fn with_client(&mut self, client: Client) -> &mut Self {
        self.client = client;
        self
    }

    pub fn with_downloads(&mut self, downloads: Vec<DownloadData>) -> &mut Self {
        self.downloads = downloads;
        self
    }

    pub fn with_parallel_requests(&mut self, parallel_requests: u16) -> &mut Self {
        self.parallel_requests = parallel_requests;
        self
    }

    pub fn with_retries(&mut self, retries: u16) -> &mut Self {
        self.retries = retries;
        self
    }

    pub fn with_download_folder(&mut self, download_folder: PathBuf) -> &mut Self {
        self.download_folder = download_folder;
        self
    }

    pub fn run(&self, progress: Option<Progress>) -> Result<Vec<DownloadResult>, JoinError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cl = self.client.clone();
        let max = self
            .downloads
            .clone()
            .iter()
            .map(|d| d.total_size)
            .reduce(|accum, i| accum + i)
            .unwrap_or_default();

        let download_folder = self.download_folder.clone();
        let downloads = self.downloads.clone();
        let retries = self.retries;
        let parallel_requests = self.parallel_requests;
        let progress = progress.clone();

        if progress.is_some() {
            progress.as_ref().unwrap().lock().unwrap().setup(max);
        }

        let result = rt.spawn(async move {
            let progress = progress.clone();
            let res = {
                stream::iter(downloads)
                    .map(|d| {
                        download(
                            cl.clone(),
                            d,
                            retries,
                            download_folder.clone(),
                            progress.clone(),
                        )
                    })
                    .buffered(parallel_requests as usize)
                    .collect::<Vec<DownloadResult>>()
                    .await
            };

            if progress.is_some() {
                progress.unwrap().lock().unwrap().done();
            }
            res
        });

        futures::executor::block_on(result)
    }
}
