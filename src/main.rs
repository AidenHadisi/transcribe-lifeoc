#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use std::time::Duration;

use downloader::Downloader;
use transcriber::Trasncriber;

mod error;
mod downloader;
mod transcriber;

pub type Result<T> = std::result::Result<T, error::Error>;

#[tokio::main]
async fn main() -> Result<()> {
    // let downloader = downloader::YoutubeAudioDownloader::new(
    //     "UCeY0bbntWzzVIaj2z3QigXg".to_string(),
    //     "".to_string(),
    // );
    // downloader.download_latest_service_audio().await?;


    let transcriber = transcriber::WhisperTranscriber::new(
        "./Qj3g3VZ567I.wav".to_string(),
        Duration::from_secs(60),
    );

    transcriber.transcribe().await?;
    Ok(())
}