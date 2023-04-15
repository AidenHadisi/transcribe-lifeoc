use thiserror::Error;

/// A custom error type for this application.
#[derive(Debug, Error)]
pub enum Error {
    /// An error from the downloader.
    #[error("Error: {0}")]
    DownloadError(String),
}
