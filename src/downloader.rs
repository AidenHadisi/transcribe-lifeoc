use std::time::Duration;

use log::debug;
use reqwest::Url;

use crate::{error::Error, Result};

/// A trait for downloading audio from a video.
pub trait AudioDownloader {
    /// Creates a download link for the audio
    fn create_download_link(&self, video_url: &str) -> Result<String>;
}

/// An audio downloader for YouTube videos.><
pub struct YoutubeAudioDownloader {
    /// The HTTP client used to make requests.
    client: reqwest::Client,

    /// The google api key to use for making requests.
    google_api_key: String,

    /// The rapid api key to use for making requests.
    rapid_api_key: String,

    /// The YouTube channel to download the latest video from.
    channel: String,
}

impl YoutubeAudioDownloader {
    /// The url to the google api for getting the latest video id.
    const GOOGLE_API_URL: &'static str = "https://www.googleapis.com/youtube/v3/search";

    /// Creates a new YouTube audio downloader.
    pub fn new(
        google_api_key: impl ToString,
        rapid_api_key: impl ToString,
        channel: impl ToString,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            google_api_key: google_api_key.to_string(),
            rapid_api_key: rapid_api_key.to_string(),
            channel: channel.to_string(),
        }
    }

    /// Gets the latest video url.
    pub async fn get_latest_video(&self) -> Result<YoutubeVideo> {
        let url = Url::parse_with_params(
            Self::GOOGLE_API_URL,
            &[
                ("key", self.google_api_key.as_ref()),
                ("channelId", self.channel.as_ref()),
                ("part", "snippet,id"),
                ("order", "date"),
                ("maxResults", "1"),
            ],
        )
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
            .and_then(|items| items.get(0)?.get("id")?.get("videoId")?.as_str())
            .ok_or(Error::DownloadError("No video id found".to_string()))?
            .to_string();

        Ok(YoutubeVideo::new(
            &video_id,
            &self.rapid_api_key,
            self.client.clone(),
        ))
    }
}

/// A YouTube video.
pub struct YoutubeVideo {
    /// The url of the video.
    url: String,

    /// The HTTP client used to make requests.
    client: reqwest::Client,

    /// The API key used to make requests.
    key: String,
}

impl YoutubeVideo {
    const CREATE_PROCESS_URL: &'static str =
        "https://t-one-youtube-converter.p.rapidapi.com/api/v1/createProcess";

    const GET_STATUS_URL: &'static str =
        "https://t-one-youtube-converter.p.rapidapi.com/api/v1/statusProcess";

    /// Creates a new YouTube video by converting the id to a url.
    pub fn new(id: &impl ToString, key: &impl ToString, client: reqwest::Client) -> Self {
        let url = format!("https://www.youtube.com/watch?v={}", id.to_string());
        Self {
            url,
            client,
            key: key.to_string(),
        }
    }

    /// Creates a downloadable mp3 link from the video.
    pub async fn create_download_link(&self) -> Result<String> {
        /// The maximum number of times to check the status of the download.
        const MAX_RETRIES: usize = 10;

        let guid = self.start_download().await?;
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        for _ in 0..MAX_RETRIES {
            interval.tick().await;
            if let Some(url) = self.check_status(guid.clone()).await? {
                return Ok(url);
            }
        }

        Err(Error::DownloadError("Download timed out".to_string()))
    }

    /// Creates a downloadable mp3 link from the video.
    async fn start_download(&self) -> Result<String> {
        let url = Url::parse_with_params(
            Self::CREATE_PROCESS_URL,
            &[
                ("url", self.url.as_ref()),
                ("format", "mp3"),
                ("lang", "en"),
                ("response", "json"),
            ],
        )
        .map_err(|e| Error::DownloadError(e.to_string()))?;

        let text = self
            .client
            .get(url)
            .header("x-rapidapi-key", self.key.as_str())
            .send()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .text()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?;

        debug!("Response: {}", text);

        let guid = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .get("guid")
            .and_then(|guid| guid.as_str())
            .ok_or(Error::DownloadError("No guid found".to_string()))?
            .to_owned();

        Ok(guid)
    }

    async fn check_status(&self, guid: String) -> Result<Option<String>> {
        let url = Url::parse_with_params(
            Self::GET_STATUS_URL,
            &[("guid", guid.as_ref()), ("response", "json")],
        )
        .map_err(|e| Error::DownloadError(e.to_string()))?;

        let text = self
            .client
            .get(url)
            .header("x-rapidapi-key", self.key.as_str())
            .send()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .text()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?;

        debug!("Response: {}", text);

        let status = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .get("file")
            .and_then(|file| file.as_str())
            .map(|file| file.to_owned());

        Ok(status)
    }
}
