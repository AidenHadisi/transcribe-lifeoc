#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(let_chains)]

use std::time::Duration;

use transcriber::{Trasncriber, YoutubeTranscriber};

mod error;
mod transcriber;

pub type Result<T> = std::result::Result<T, error::Error>;

#[tokio::main]
async fn main() -> Result<()> {
 
    // transcriber.transcribe().await?;
    pretty_env_logger::init();
    let transcriber = YoutubeTranscriber::new(
      
    );

    transcriber.transcribe().await?;
    Ok(())
}
