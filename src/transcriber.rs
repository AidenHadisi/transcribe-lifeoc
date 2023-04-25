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

    max_timeout: Duration,
}

impl AssemblyAiTranscriber {
    /// The url to the assembly ai api for transcribing the audio.
    const ASSEMBLYAI_API_URL: &'static str = "https://api.assemblyai.com/v2/transcript";

    /// Creates a new [`YoutubeTranscriber`].
    pub fn new(assemblyai_api_key: impl ToString, max_timeout: Duration) -> Self {
        Self {
            assemblyai_api_key: assemblyai_api_key.to_string(),
            max_timeout,
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
        let id = self.start_transcription(audio_url).await?.id;

        let poll_status = async {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;

                let response = self.get_transcription_result(&id).await?;
                debug!("Transcription response: {:?}", response);
                if let Some(text) = response.text {
                    return Ok(text);
                }
            }
        };

        tokio::time::timeout(self.max_timeout, poll_status)
            .await
            .map_err(|_| {
                Error::TranscribeError("Transcription failed. Maximum timeout reached.".into())
            })?
    }
}

/// Response from assembly ai.
#[derive(Debug, Deserialize)]
struct TranscribeResponse {
    id: String,
    text: Option<String>,
}
