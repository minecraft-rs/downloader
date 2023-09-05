use crate::error::{ClientDownloaderError, DownloadError};
use crate::launcher_manifest::{LauncherManifest, LauncherManifestVersion};
use crate::manifest::Manifest;
use reqwest::blocking::Client;
use serde_json::Value;

use std::path::PathBuf;

use super::{
    DownloadData, DownloadJava, DownloadResult, DownloadVersion, DownloaderService, Progress,
};

pub struct ClientDownloader {
    pub main_manifest: LauncherManifest,
}

impl ClientDownloader {
    pub fn new() -> Result<Self, ClientDownloaderError> {
        Ok(Self {
            main_manifest: Self::init()?,
        })
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

    pub fn get_version(&self, id: &str) -> Option<&LauncherManifestVersion> {
        self.main_manifest
            .versions
            .iter()
            .find(|v| v.id.eq_ignore_ascii_case(id))
    }
}

impl DownloadJava for ClientDownloader {
    fn check_version(&self, root_path: &str, expected_version: &str) -> bool {
        let mut path = PathBuf::from(root_path);
        path.push(expected_version);

        path.exists() && path.is_dir()
    }

    fn download_java(&self, root_path: &str, version: &str, progress: Option<Progress>) {
        if !self.check_version(root_path, version) {
            let os = std::env::consts::OS;
            let arch = std::env::consts::ARCH;
            let ext = match os {
                "macos" | "linux" => ".tar.gz",
                _ => ".zip",
            };
            let downloads = vec![DownloadData {
                url: format!(
          "https://download.oracle.com/java/{version}/archive/jdk-{version}_{os}-{arch}_bin{ext}"
        ),
                file_name: format!("jdk-{version}{ext}"),
                output_path: format!("jdk-{version}{ext}"),
                sha1: String::new(),
                total_size: 0,
            }];
            DownloaderService::new(PathBuf::from(root_path))
                .with_downloads(downloads)
                .run(progress)
                .unwrap();
        }
    }
}

impl DownloadVersion for ClientDownloader {
    fn download_version(
        &self,
        version_id: &str,
        game_path: &PathBuf,
        manifest_path: Option<&PathBuf>,
        version_path: Option<&PathBuf>,
        progress: Option<Progress>,
    ) -> Result<Vec<DownloadResult>, ClientDownloaderError> {
        let version = game_path.clone().join("versions").join(version_id);
        let manifest_path = manifest_path
            .unwrap_or(&version.join(format!("{}.json", version_id)))
            .clone();

        let client = Client::new();
        let version_option = self.get_version(version_id);

        if version_option.is_none() {
            return Err(ClientDownloaderError::NoSuchVersion);
        }

        let version = version_option.unwrap();
        let response = client.get(&version.url).send()?;
        let manifest: Manifest = response.json()?;
        {
            let response = client.get(&version.url).send()?;
            let response_str = response.text()?;
            std::fs::create_dir_all(&game_path)?;
            std::fs::create_dir_all(&manifest_path.parent().unwrap())?;
            std::fs::write(manifest_path, response_str)?;
        }

        self.download_by_manifest(&manifest, game_path, version_path, progress)
    }

    fn download_by_manifest(
        &self,
        manifest: &Manifest,
        game_path: &PathBuf,
        version_path: Option<&PathBuf>,
        progress: Option<Progress>,
    ) -> Result<Vec<DownloadResult>, ClientDownloaderError> {
        let version_path = version_path
            .unwrap_or(
                &game_path
                    .join("versions")
                    .join(manifest.clone().id)
                    .join(format!("{}.jar", manifest.id)),
            )
            .clone();
        std::fs::create_dir_all(&version_path.parent().unwrap())?;

        let client = Client::new();
        let mut downloads: Vec<DownloadData> = Vec::new();

        // Add client
        {
            downloads.push(DownloadData {
                url: manifest.clone().downloads.client.url,
                file_name: version_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                output_path: version_path.as_path().to_str().unwrap().to_string(),
                sha1: manifest.clone().downloads.client.sha1,
                total_size: manifest.downloads.client.size,
            });
        }

        // Add asset index
        {
            let mut path = game_path.clone();
            path.push("assets");
            path.push("indexes");
            path.push(format!("{}.json", manifest.asset_index.id));

            let path = path.to_str().unwrap();
            let size = manifest.asset_index.size as u64;

            downloads.push(DownloadData {
                url: manifest.asset_index.url.clone(),
                file_name: format!("{}.json", manifest.asset_index.id),
                output_path: path.to_string(),
                sha1: manifest.clone().asset_index.sha1,
                total_size: size,
            });
        }

        // Add assets
        {
            let mut path = game_path.clone();
            path.push("assets");

            let mut objects_path = path.clone();
            objects_path.push("objects");

            let response = client.get(manifest.clone().asset_index.url).send()?;

            let data: Value = serde_json::from_reader(response)?;
            let object = data.get("objects").unwrap().as_object().unwrap();
            downloads.extend(
                object
                    .iter()
                    .map(|(p, obj)| {
                        let hash = obj.get("hash").unwrap().as_str().unwrap();
                        let size = obj.get("size").unwrap().as_u64().unwrap();

                        let mut path = objects_path.clone();
                        path.push(hash[..2].to_string());
                        path.push(hash.to_string());

                        DownloadData {
                            url: format!(
                                "https://resources.download.minecraft.net/{}/{}",
                                hash[..2].to_string(),
                                hash
                            ),
                            file_name: p.clone(),
                            output_path: path.to_str().unwrap().to_string(),
                            sha1: hash.to_string(),
                            total_size: size,
                        }
                    })
                    .collect::<Vec<DownloadData>>(),
            );
        }

        // Add libraries to download
        {
            let mut path = game_path.to_path_buf();
            path.push("libraries");
            downloads.extend(
                manifest
                    .libraries
                    .iter()
                    .filter_map(|l| {
                        if let Some(artifact) = l.downloads.artifact.clone() {
                            let mut path = path.clone();
                            if let Some(p) = artifact.clone().path {
                                path.push(p);
                            }
                            let data = DownloadData {
                                output_path: path.to_str().unwrap().to_string(),
                                ..DownloadData::from(artifact)
                            };
                            return Some(data);
                        }
                        None
                    })
                    .collect::<Vec<DownloadData>>(),
            );
        }

        let results = DownloaderService::new(game_path.parent().unwrap().to_path_buf())
            .with_downloads(downloads)
            .run(progress)
            .unwrap();

        if results.is_empty() {
            return Err(ClientDownloaderError::Download(
                DownloadError::DownloadDefinition("No Downloaded files".to_string()),
            ));
        }

        Ok(results)
    }
}
