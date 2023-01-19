use mc_downloader::ClientDownloader;

fn main() {
    let downloader: ClientDownloader = ClientDownloader::new();
    downloader
        .download_version("1.19.3".to_string(), "%appdata%\\.minecraft".to_string())
        .unwrap();
}
