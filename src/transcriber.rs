use std::time::Duration;

pub trait Trasncriber {
    async fn transcribe(&self) -> super::Result<()>;
}

pub struct WhisperTranscriber {
    path: String,
}

impl WhisperTranscriber {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Trasncriber for WhisperTranscriber {
    async fn transcribe(&self) -> super::Result<()> {
        //use hound to read the audio file

        let mut reader = hound::WavReader::open(&self.path).expect("open failed");
        let spec = reader.spec();

        //read the audio file into 30 minute chunks
        let thirty_mins = Duration::from_secs(30 * 60);

        let num_samples = thirty_mins.as_secs() as usize * spec.sample_rate as usize;

        // Seek to the position that corresponds to 50 minutes
        let num_samples_to_skip = (spec.sample_rate as u32) * 60 * 1;
        reader.seek(num_samples_to_skip).unwrap();

        let samples = reader.samples::<i16>();

        let mut writer =
            hound::WavWriter::create("good_morning_10.mp3", spec).expect("writer failed");
        for sample in reader.samples::<i16>().filter_map(|s| s.ok()) {
            writer.write_sample(sample).unwrap();
        }

        writer.flush().unwrap();
        writer.finalize().expect("finalize failed");
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
