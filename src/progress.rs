type Callback = fn();

pub struct DownloadProgress {
    files_count: i16,
    files_downloaded: i16,
    current_file: Option<String>,
    progress: i16,
    finished: bool,

    // Callbacks
    on_download_start: Option<Callback>,
    on_download_finished: Option<Callback>,
    on_download_status: Option<Callback>,
}

impl DownloadProgress {
    pub fn new() -> Self {
        Self {
            current_file: None,
            files_count: 0,
            files_downloaded: 0,
            finished: false,
            progress: 0,

            on_download_start: None,
            on_download_finished: None,
            on_download_status: None,
        }
    }
}
