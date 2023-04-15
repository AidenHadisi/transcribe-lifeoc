use reqwest::Url;
use rustube::{Id, Video};

use crate::error::Error;

/// A trait for downloading the latest service audio from a given source.
pub trait Downloader {
    /// Downloads the latest service audio.
    async fn download_latest_service_audio(&self) -> super::Result<()>;
}

/// A downloader for youtube audio.
pub struct YoutubeAudioDownloader {
    /// The channel id to download the latest video from.
    channel: String,

    /// The youtube api key to use.
    api_key: String,
}

/// A downloader for local audio.
impl Downloader for YoutubeAudioDownloader {
    /// Downloads the latest service audio from youtube.
    async fn download_latest_service_audio(&self) -> super::Result<()> {
        let video_id = self.get_latest_video_id().await?;
        let id = Id::from_string(video_id).unwrap();
        Video::from_id(id)
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .streams()
            .iter()
            .filter(|stream| stream.includes_audio_track && !stream.includes_video_track)
            .max_by_key(|stream| stream.quality_label)
            .ok_or(Error::DownloadError("no audio found".into()))?
            .download()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?;

        Ok(())
    }
}

impl YoutubeAudioDownloader {
    /// Creates a new youtube audio downloader.
    pub fn new(channel: String, api_key: String) -> Self {
        Self { channel, api_key }
    }

    async fn get_latest_video_id(&self) -> super::Result<String> {
        let url = Url::parse_with_params(
            "https://www.googleapis.com/youtube/v3/search",
            &[
                ("key", self.api_key.as_ref()),
                ("channelId", self.channel.as_ref()),
                ("part", "snippet,id"),
                ("order", "date"),
                ("maxResults", "1"),
            ],
        )
        .map_err(|e| Error::DownloadError(e.to_string()))?;

        let response = reqwest::get(url)
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?
            .text()
            .await
            .map_err(|e| Error::DownloadError(e.to_string()))?;
        let data: serde_json::Value =
            serde_json::from_str(&response).map_err(|e| Error::DownloadError(e.to_string()))?;
        let video_id = data["items"][0]["id"]["videoId"]
            .as_str()
            .ok_or(Error::DownloadError("No video id found".to_string()))?;
        Ok(video_id.to_string())
    }
}
