use std::{
    env::args,
    io::Stdout,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, Arc,
    },
};

use mc_downloader::prelude::{ClientDownloader, DownloadVersion, Reporter};
use pbr::ProgressBar;

struct ProgressTrack {
    curr: AtomicU64,
    total: AtomicU64,
    pb: Mutex<ProgressBar<Stdout>>,
}

fn main() {
    println!("Start Minecraft Downloader");
    // We received the path where the minecraft versions will be downloaded
    // from the command line as first parameter
    let args = args().collect::<Vec<String>>();
    let default_path = "./.minecraft".to_string();
    let path = args.get(1).unwrap_or(&default_path);
    match ClientDownloader::new() {
        Ok(downloader) => {
            println!("Start Download Minecraft 1.19.3 version in {path}");
            downloader
                .download_version("1.19.3", path, Some(Arc::new(Mutex::new(ProgressTrack::default()))))
                .unwrap();
        }
        Err(e) => println!("{e:?}"),
    }
}

impl Default for ProgressTrack {
    fn default() -> Self {
        Self {
            curr: AtomicU64::new(0),
            total: AtomicU64::new(0),
            pb: Mutex::new(ProgressBar::new(0)),
        }
    }
}

impl Reporter for ProgressTrack {
    fn setup(&mut self, max_progress: u64) {
        self.total.store(max_progress, Ordering::SeqCst);
        let mut pb = self.pb.lock().unwrap();
        *pb = ProgressBar::new(max_progress);
        pb.set_units(pbr::Units::Bytes);
        pb.format("[=> ]");
        println!(
            "Start tracker process; version weight: {}",
            convert_bytes(max_progress as f64)
        );
    }

    fn progress(&mut self, current: u64) {
        let mut curr = self.curr.load(Ordering::SeqCst);
        curr += current;
        self.curr.store(curr, Ordering::SeqCst);
        // Show progress bar
        let mut pb = self.pb.lock().unwrap();
        pb.set(curr);
    }

    fn done(&mut self) {
        let mut pb = self.pb.lock().unwrap();
        pb.finish();
        println!("\nDone!! Download Finish");
    }
}

//
// IGNORE
//
fn convert_bytes(num: f64) -> String {
    let negative = if num.is_sign_positive() { "" } else { "-" };
    let num = num.abs();
    let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if num < 1_f64 {
        return format!("{}{} {}", negative, num, "B");
    }
    let delimiter = 1000_f64;
    let exponent = std::cmp::min(
        (num.ln() / delimiter.ln()).floor() as i32,
        (units.len() - 1) as i32,
    );
    let pretty_bytes = format!("{:.2}", num / delimiter.powi(exponent))
        .parse::<f64>()
        .unwrap()
        * 1_f64;
    let unit = units[exponent as usize];
    format!("{}{} {}", negative, pretty_bytes, unit)
}
