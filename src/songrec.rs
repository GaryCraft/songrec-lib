use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::fingerprinting::algorithm::SignatureGenerator;
use crate::fingerprinting::communication::{recognize_song_from_signature_with_config, recognize_song_from_signature};
use crate::audio::recorder::AudioRecorder;
use crate::audio::processor::AudioProcessor;
use crate::{Result, SongRecError};

/// Main SongRec struct for audio recognition
pub struct SongRec {
    config: Config,
}

/// Result of a song recognition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecognitionResult {
    pub song_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub track_key: String,
    pub release_year: Option<String>,
    pub genre: Option<String>,
    pub recognition_timestamp: chrono::DateTime<chrono::Utc>,
    pub raw_response: serde_json::Value,
}

/// Stream of recognition results for continuous monitoring
pub struct RecognitionStream {
    receiver: mpsc::Receiver<Result<RecognitionResult>>,
    _handles: Vec<thread::JoinHandle<()>>, // Keep handles to prevent threads from being dropped
}

impl SongRec {
    /// Create a new SongRec instance with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Recognize a song from an audio file
    pub fn recognize_from_file(&self, file_path: &str) -> Result<RecognitionResult> {
        // Generate signature from file
        let signature = SignatureGenerator::make_signature_from_file(file_path)
            .map_err(|e| SongRecError::FingerprintingError(e.to_string()))?;

        // Recognize song from signature with config
        let response = recognize_song_from_signature_with_config(&signature, &self.config)
            .map_err(|e| SongRecError::NetworkError(e.to_string()))?;

        // Parse response into RecognitionResult
        self.parse_recognition_response(response)
    }

    /// Recognize a song from raw audio samples
    pub fn recognize_from_samples(&self, samples: &[i16], sample_rate: u32) -> Result<RecognitionResult> {
        // Create signature generator and process samples
        let mut generator = SignatureGenerator::new();
        
        // Process the samples to generate a signature
        for chunk in samples.chunks(128) {
            generator.do_fft(chunk, sample_rate);
        }

        let signature = generator.get_signature();

        // Recognize song from signature
        let response = recognize_song_from_signature(&signature)
            .map_err(|e| SongRecError::NetworkError(e.to_string()))?;

        // Parse response into RecognitionResult
        self.parse_recognition_response(response)
    }

    /// Start continuous recognition from the default audio device
    pub fn start_continuous_recognition(&self) -> Result<RecognitionStream> {
        self.start_continuous_recognition_with_device(None)
    }

    /// Start continuous recognition from a specific audio device
    pub fn start_continuous_recognition_with_device(&self, device_name: Option<String>) -> Result<RecognitionStream> {
        let (result_tx, result_rx) = mpsc::channel();
        let (_control_tx, control_rx) = mpsc::channel();
        
        let config = self.config.clone();
        
        // Start audio recording thread
        let recorder_handle = {
            let result_tx = result_tx.clone();
            let config_for_thread = config.clone();
            
            thread::spawn(move || {
                let mut recorder = AudioRecorder::new(config_for_thread.clone());
                
                match recorder.start_recording(device_name, control_rx) {
                    Ok(sample_rx) => {
                        // Process audio samples
                        let mut processor = AudioProcessor::with_config(config_for_thread.clone());
                        
                        for samples in sample_rx {
                            match processor.process_samples(&samples) {
                                Ok(Some(signature)) => {
                                    // Try to recognize the signature with config
                                    match recognize_song_from_signature_with_config(&signature, &config_for_thread) {
                                        Ok(response) => {
                                            // Parse and send result
                                            match SongRec::parse_recognition_response_static(response) {
                                                Ok(result) => {
                                                    if result_tx.send(Ok(result)).is_err() {
                                                        break; // Receiver dropped, stop processing
                                                    }
                                                },
                                                Err(e) => {
                                                    if result_tx.send(Err(e)).is_err() {
                                                        break;
                                                    }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            let error = SongRecError::NetworkError(e.to_string());
                                            if result_tx.send(Err(error)).is_err() {
                                                break;
                                            }
                                        }
                                    }
                                },
                                Ok(None) => {
                                    // Not enough samples yet, continue
                                },
                                Err(e) => {
                                    let error = SongRecError::FingerprintingError(e.to_string());
                                    if result_tx.send(Err(error)).is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        let error = SongRecError::AudioError(e.to_string());
                        let _ = result_tx.send(Err(error));
                    }
                }
            })
        };

        Ok(RecognitionStream {
            receiver: result_rx,
            _handles: vec![recorder_handle],
        })
    }

    /// Parse a recognition response from the API into a RecognitionResult
    fn parse_recognition_response(&self, response: serde_json::Value) -> Result<RecognitionResult> {
        Self::parse_recognition_response_static(response)
    }

    /// Static version of parse_recognition_response for use in threads
    fn parse_recognition_response_static(response: serde_json::Value) -> Result<RecognitionResult> {
        // First check if we have any matches
        let matches = response.get("matches")
            .and_then(|m| m.as_array())
            .ok_or_else(|| SongRecError::NetworkError("Invalid response format: no matches array".to_string()))?;
            
        if matches.is_empty() {
            return Err(SongRecError::NetworkError("No track found in response".to_string()));
        }
        
        // The track info is at the top level of the response, not inside the matches
        let track = response.get("track")
            .ok_or_else(|| SongRecError::NetworkError("No track found in response".to_string()))?;

        // Extract song details from the track
        let song_name = track
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let artist_name = track
            .get("subtitle")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let album_name = track
            .pointer("/sections/0/metadata/0/text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let track_key = track
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let release_year = track
            .pointer("/sections/0/metadata")
            .and_then(|metadata| {
                if let Some(metadata_array) = metadata.as_array() {
                    for item in metadata_array {
                        if let Some(title) = item.pointer("/title").and_then(|v| v.as_str()) {
                            if title == "Released" {
                                return item.pointer("/text").and_then(|v| v.as_str()).map(|s| s.to_string());
                            }
                        }
                    }
                }
                None
            });

        let genre = track
            .pointer("/genres/primary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(RecognitionResult {
            song_name,
            artist_name,
            album_name,
            track_key,
            release_year,
            genre,
            recognition_timestamp: chrono::Utc::now(),
            raw_response: response,
        })
    }
}

impl RecognitionStream {
    /// Get the next recognition result from the stream
    pub fn next(&self) -> Option<Result<RecognitionResult>> {
        self.receiver.recv().ok()
    }

    /// Try to get the next recognition result without blocking
    pub fn try_next(&self) -> Option<Result<RecognitionResult>> {
        self.receiver.try_recv().ok()
    }

    /// Wait for the next recognition result with a timeout
    pub fn next_timeout(&self, timeout: Duration) -> Option<Result<RecognitionResult>> {
        self.receiver.recv_timeout(timeout).ok()
    }
}

impl Iterator for RecognitionStream {
    type Item = Result<RecognitionResult>;

    fn next(&mut self) -> Option<Self::Item> {
        RecognitionStream::next(self)
    }
}
