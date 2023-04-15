use std::time::Duration;

use crate::error::Error;

pub trait Trasncriber {
    async fn transcribe(&self) -> super::Result<()>;
}

pub struct WhisperTranscriber {
    path: String,
    skip_duration: Duration,
}

impl WhisperTranscriber {
    pub fn new(path: String, skip_duration: Duration) -> Self {
        Self {
            path,
            skip_duration,
        }
    }
}

impl Trasncriber for WhisperTranscriber {
    async fn transcribe(&self) -> super::Result<()> {
        let mut reader =
            hound::WavReader::open(&self.path).map_err(|e| Error::TrascribeError(e.to_string()))?;

        let spec = reader.spec();

        // Skip the specified duration
        let num_samples_to_skip = (spec.sample_rate as u32) * self.skip_duration.as_secs() as u32;
        reader
            .seek(num_samples_to_skip)
            .map_err(|e| Error::TrascribeError(e.to_string()))?;

        let samples = reader.samples::<i16>();

        let mut writer = hound::WavWriter::create("good_morning_10.mp3", spec)
            .map_err(|e| Error::TrascribeError(e.to_string()))?;

        reader
            .samples::<i16>()
            .flatten()
            .try_for_each(|sample| writer.write_sample(sample))
            .map_err(|e| Error::TrascribeError(e.to_string()))?;

        writer.flush().map_err(|e| Error::TrascribeError(e.to_string()))?;
        writer.finalize().map_err(|e| Error::TrascribeError(e.to_string()))?;
        // let mut chunk = Vec::new();
        // let mut chunk_size = 0;
        // let mut chunk_count = 0;
        // let mut chunk_start = 0;
        // let mut chunk_end = 0;

        // for sample in reader.samples::<i16>() {
        //     let sample = sample.unwrap();
        //     chunk.push(sample);
        //     chunk_size += 1;
        //     if chunk_size == 30 * 60 * sample_rate {
        //         chunk_count += 1;
        //         chunk_start = chunk_end;
        //         chunk_end = chunk_start + chunk_size;
        //         //transcribe the chunk
        //         let mut transcriber = WhisperTranscriber::new();
        //         transcriber.transcribe_chunk(chunk, chunk_start, chunk_end, chunk_count);
        //         chunk_size = 0;
        //         chunk = Vec::new();
        //     }
        // }

        // //transcribe the last chunk
        // if chunk_size > 0 {
        //     chunk_count += 1;
        //     chunk_start = chunk_end;
        //     chunk_end = chunk_start + chunk_size;
        //     //transcribe the chunk
        //     let mut transcriber = WhisperTranscriber::new();
        //     transcriber.transcribe_chunk(chunk, chunk_start, chunk_end, chunk_count);
        // }

        Ok(())
    }
}
