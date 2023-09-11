# MC Downloader

Download minecraft client and libraries from rust.

## Usage

Download the client and libraries:

```rust
let path = "./.minecraft".to_string();
let version = "1.19.4".to_string();

match ClientDownloader::new() {
    Ok(downloader) => {
        println!("Start Download Minecraft {version} version in {path}");
        downloader
            .download_version(
                &version,
                &PathBuf::from(path),
                None,
                None,
                None,
            )
            .unwrap();
    }
    Err(e) => println!("{e:?}"),
}

```

## Contribution

Feel free to contribute to the development of the library.
