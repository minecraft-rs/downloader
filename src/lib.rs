pub mod launcher_manifest;
pub mod manifest;

use std::path::Path;

use launcher_manifest::{LauncherManifest, LauncherManifestVersion};
use manifest::Manifest;
use reqwest::blocking::Client;
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
        Ok(data)
    }

    pub fn get_list_versions(&self) -> Vec<LauncherManifestVersion> {
        self.main_manifest.versions.clone()
    }

    pub fn get_version(&self, id: String) -> Option<&LauncherManifestVersion> {
        self.main_manifest
            .versions
            .iter()
            .find(|v| v.id.eq_ignore_ascii_case(&id))
    }

    pub fn download_file(&self, path: &str, url: &str) {
        println!("\nSaved file {path}\nFrom {url}");
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

        self.download_file(&jar_file, &manifest.downloads.client.url);

        for lib in manifest.libraries {
            let artifact = lib.downloads.artifact;
            if let Some(download) = artifact {
                let path = download.path.unwrap();
                let final_path = main_dir
                    .join("libraries")
                    .join(path)
                    .to_str()
                    .unwrap()
                    .to_string();
                self.download_file(&final_path, &download.url);
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
