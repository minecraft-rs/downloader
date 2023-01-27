use mc_downloader::ClientDownloader;

fn main() {
    match ClientDownloader::new() {
        Ok(downloader) => {
            downloader
                .download_version("1.19.3".to_string(), "%appdata%\\.minecraft".to_string())
                .unwrap();
        }
        Err(e) => println!("{e:?}"),
    }
}
