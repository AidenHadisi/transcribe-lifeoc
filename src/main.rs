#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(let_chains)]

mod downloader;
mod error;
mod transcriber;

use dotenv::dotenv;
use downloader::YoutubeAudioDownloader;
use log::info;
use std::env;
use transcriber::{AssemblyAiTranscriber, Transcriber};

/// Custom result type for this crate.
pub type Result<T> = std::result::Result<T, error::Error>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    dotenv().ok();

    let google_api_key = env::var("GOOGLE_CLOUD_KEY").expect("GOOGLE_CLOUD_KEY must be set");
    let rapid_api_key = env::var("RAPID_API_KEY").expect("RAPID_API_KEY must be set");
    let channel = env::var("YOUTUBE_CHANNEL_ID").expect("YOUTUBE_CHANNEL must be set");
    let assembly_ai_key = env::var("ASSEMBLY_AI_KEY").expect("ASSEMBLY_AI_KEY must be set");

    let d = YoutubeAudioDownloader::new(google_api_key, rapid_api_key, channel);

    let video = d.get_latest_video().await?;

    let link = video.create_download_link().await?;

    info!("Downloading audio from {} ...", link);

    let transcriber = AssemblyAiTranscriber::new(assembly_ai_key);

    let text = transcriber.transcribe(&link).await?;

    info!("Got text: {}", text);

    Ok(())
}
