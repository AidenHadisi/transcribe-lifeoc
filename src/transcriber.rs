use std::time::Duration;

use crate::{error::Error, Result};
use log::{debug, info};
use reqwest::{header, Url};
use serde::Deserialize;

/// A trait for transcribing audio.
pub trait Trasncriber {
    /// Transcribes the audio.
    async fn transcribe(&self) -> super::Result<String>;
}

/// A trascriber for latest youtube video.
pub struct YoutubeTranscriber {
    /// The path to the audio file.
    google_api_key: String,

    /// Api key for assembly ai.
    assemblyai_api_key: String,

    /// The youtube channel to transcribe the latest video from.
    channel: String,

    /// The client to use for making requests.
    client: reqwest::Client,
}

impl YoutubeTranscriber {
    /// The url to the google api for getting the latest video id.
    const GOOGLE_API_URL: &'static str = "https://www.googleapis.com/youtube/v3/search";

    /// The url to the assembly ai api for transcribing the audio.
    const ASSEMBLYAI_API_URL: &'static str = "https://api.assemblyai.com/v2/transcript";

    /// Creates a new [`YoutubeTranscriber`].
    pub fn new(google_api_key: String, assemblyai_api_key: String, channel: String) -> Self {
        Self {
            google_api_key,
            assemblyai_api_key,
            channel,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    /// Starts the transcription process.
    async fn start_transcription(&self) -> Result<TranscribeResponse> {
        let audio_url = self.get_latest_video_url().await?;

        let request_body = serde_json::json!({ "audio_url": audio_url });

        self.client
            .post(Self::ASSEMBLYAI_API_URL)
            .header(header::AUTHORIZATION, &self.assemblyai_api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::TranscribeError(e.to_string()))?
            .json()
            .await
            .map_err(|_| {
                Error::TranscribeError("Failed to parse response from assembly ai".to_string())
            })
    }

    /// Gets the transcription result.
    async fn get_transcription_result(&self, id: &str) -> Result<TranscribeResponse> {
        let url = format!("{}/{}", Self::ASSEMBLYAI_API_URL, id);
        self.client
            .get(&url)
            .header(header::AUTHORIZATION, &self.assemblyai_api_key)
            .send()
            .await
            .map_err(|e| Error::TranscribeError(e.to_string()))?
            .json()
            .await
            .map_err(|e| Error::TranscribeError(e.to_string()))
    }

    /// Gets the latest video url.
    async fn get_latest_video_url(&self) -> Result<String> {
        let query_params = [
            ("key", self.google_api_key.as_ref()),
            ("channelId", self.channel.as_ref()),
            ("part", "snippet,id"),
            ("order", "date"),
            ("maxResults", "1"),
        ];

        let url = Url::parse_with_params(Self::GOOGLE_API_URL, &query_params)
            .map_err(|e| Error::DownloadError(e.to_string()))?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .text()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?;

        let video_id = serde_json::from_str::<serde_json::Value>(&response)
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .get("items")
            .and_then(|items| items.get(0))
            .and_then(|item| item.get("id"))
            .and_then(|id| id.get("videoId"))
            .and_then(|video_id| video_id.as_str())
            .ok_or(Error::DownloadError("No video id found".to_string()))?
            .to_owned();

        Ok(format!("https://www.youtube.com/watch?v={}", video_id))
    }
}

impl Trasncriber for YoutubeTranscriber {
    async fn transcribe(&self) -> super::Result<String> {
        /// The max number of times to retry getting the transcription result.
        const MAX_RETRY_COUNT: u32 = 100;
        const RETRY_INTERVAL: u64 = 30;

        let mut result = self.start_transcription().await?;
        let id = result.id.clone();

        let mut max_tries = (0..MAX_RETRY_COUNT).into_iter();

        while result.status != "completed" && max_tries.next().is_some() {
            result = self.get_transcription_result(id.as_str()).await?;
            debug!(
                "Transcription not completed. Retrying in {} seconds",
                RETRY_INTERVAL
            );
            debug!("Current result: {:?}", result);
            tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL)).await;
        }

        info!("Final Result: {:?}", result);

        match (result.status.as_ref(), result.text) {
            ("completed", _) => Err(Error::TranscribeError(format!(
                "Transcription failed after {} tries. Got error: {}",
                MAX_RETRY_COUNT,
                result.error.unwrap_or("No error".to_string())
            ))),
            (_, Some(text)) => Ok(text),
            (_, None) => Err(Error::TranscribeError(
                "No text found in transcription result".to_string(),
            )),
        }
    }
}

/// Response from assembly ai.
#[derive(Debug, Deserialize)]
struct TranscribeResponse {
    id: String,
    status: String,
    error: Option<String>,
    text: Option<String>,
}
