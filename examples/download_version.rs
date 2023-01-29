use std::env;

use mc_downloader::ClientDownloader;

fn main() {
    let dir = env::current_dir().unwrap();
    let target = format!("{}{}", dir.to_str().unwrap().to_string(), "\\.minecraft");
    match ClientDownloader::new() {
        Ok(downloader) => {
            downloader
                .download_version("1.19.3".to_string(), target)
                .unwrap();
        }
        Err(e) => println!("{e:?}"),
    }
}
