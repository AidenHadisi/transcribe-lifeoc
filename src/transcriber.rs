use std::time::Duration;

use crate::{error::Error, Result};
use log::debug;
use reqwest::header;
use serde::Deserialize;

/// A trait for transcribing audio.
pub trait Transcriber {
    /// Transcribes the audio.
    async fn transcribe(&self, audio_url: &str) -> super::Result<String>;
}

/// A Transcriber for latest youtube video.
pub struct AssemblyAiTranscriber {
    /// Api key for assembly ai.
    assemblyai_api_key: String,

    /// The client to use for making requests.
    client: reqwest::Client,
}

impl AssemblyAiTranscriber {
    /// The url to the assembly ai api for transcribing the audio.
    const ASSEMBLYAI_API_URL: &'static str = "https://api.assemblyai.com/v2/transcript";

    /// Creates a new [`YoutubeTranscriber`].
    pub fn new(assemblyai_api_key: impl ToString) -> Self {
        Self {
            assemblyai_api_key: assemblyai_api_key.to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    /// Starts the transcription process.
    async fn start_transcription(&self, audio_url: &str) -> Result<TranscribeResponse> {
        let response = self
            .client
            .post(Self::ASSEMBLYAI_API_URL)
            .header(header::AUTHORIZATION, &self.assemblyai_api_key)
            .json(&serde_json::json!({ "audio_url": audio_url }))
            .send()
            .await
            .map_err(|e| Error::TranscribeError(e.to_string()))?;

        match response.status().is_success() {
            true => response
                .json::<TranscribeResponse>()
                .await
                .map_err(|e| Error::TranscribeError(e.to_string())),
            false => Err(Error::DownloadError(
                response
                    .text()
                    .await
                    .map_err(|e| Error::TranscribeError(e.to_string()))?,
            )),
        }
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
}

impl Transcriber for AssemblyAiTranscriber {
    async fn transcribe(&self, audio_url: &str) -> super::Result<String> {
        const MAX_RETRY_COUNT: u32 = 30;
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        let id = self.start_transcription(audio_url).await?.id;

        for _ in 0..MAX_RETRY_COUNT {
            interval.tick().await;

            match self.get_transcription_result(&id).await? {
                //if the text field is some return it
                TranscribeResponse {
                    text: Some(text), ..
                } => return Ok(text),

                //log any other result
                result => debug!("Got result: {:?}", result),
            }
        }

        Err(Error::TranscribeError(
            "Transcription failed. Maximum retries reached.".into(),
        ))
    }
}

/// Response from assembly ai.
#[derive(Debug, Deserialize)]
struct TranscribeResponse {
    id: String,
    text: Option<String>,
}
