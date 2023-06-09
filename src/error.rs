use thiserror::Error;

/// A custom error type for this application.
#[derive(Debug, Error)]
pub enum Error {
    /// An error from the downloader.
    #[error("Error Downlaoding Audio: {0}")]
    DownloadError(String),

    /// An error from the transcriber service.
    #[error("Error Trascribing Audio: {0}")]
    TranscribeError(String),
}
