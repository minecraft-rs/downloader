pub mod launcher_manifest;
pub mod manifest;
pub mod progress;

use std::fs::File;
use std::io;
use std::path::Path;

use launcher_manifest::{LauncherManifest, LauncherManifestVersion};
use manifest::Manifest;
use reqwest::blocking::{get, Client};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientDownloaderError {
    #[error("An unexpected error has ocurred.")]
    UnknownError,

    #[error("No such version")]
    NoSuchVersion,

    #[error("{0}")]
    Request(#[from] reqwest::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub struct ClientDownloader {
    main_manifest: LauncherManifest,
}

impl ClientDownloader {
    pub fn new() -> Self {
        Self {
            main_manifest: Self::init().unwrap(),
        }
    }

    pub fn init() -> Result<LauncherManifest, ClientDownloaderError> {
        let client = Client::new();
        let response = client
            .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
            .send()?;

        let data: LauncherManifest = serde_json::from_reader(response)?;
        return Ok(data);
    }

    pub fn get_version(&self, id: String) -> Option<&LauncherManifestVersion> {
        for version in &self.main_manifest.versions {
            if version.id == id {
                return Some(&version);
            }
        }

        return None;
    }

    fn download_file(&self, path: &String, url: String) {
        let resp = get(url).expect("request failed");
        let body = resp.text().expect("body invalid");
        let mut out = File::create(path).expect("failed to create file");
        io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");
        println!("Downloaded {}", path);
    }

    pub fn download_by_manifest(
        &self,
        manifest: Manifest,
        dir: String,
    ) -> Result<(), ClientDownloaderError> {
        let main_dir = Path::new(&dir);
        let id = manifest.id;
        let jar_file_name = format!("{}.jar", id);
        let jar_file = main_dir
            .join("versions")
            .join(id)
            .join(jar_file_name)
            .to_str()
            .unwrap()
            .to_string();

        self.download_file(&jar_file, manifest.downloads.client.url);

        for lib in manifest.libraries {
            let artifact = lib.downloads.artifact;
            if artifact.is_some() {
                let download = artifact.unwrap();
                let path = download.path.unwrap();
                let final_path = main_dir
                    .join("libraries")
                    .join(path)
                    .to_str()
                    .unwrap()
                    .to_string()
                    .replace("/", "\\");
                self.download_file(&final_path, download.url);
            }
        }

        Ok(())
    }

    pub fn download_version(
        &self,
        version_id: String,
        dir: String,
    ) -> Result<(), ClientDownloaderError> {
        let client = Client::new();
        let version_option = self.get_version(version_id);

        if version_option.is_none() {
            return Err(ClientDownloaderError::NoSuchVersion);
        }

        let version = version_option.unwrap();
        let response = client.get(&version.url).send()?;
        let data: Manifest = serde_json::from_reader(response)?;
        self.download_by_manifest(data, dir)?;
        Ok(())
    }
}
