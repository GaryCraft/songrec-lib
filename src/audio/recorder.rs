use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};

use crate::config::Config;

/// Cross-platform audio recorder using CPAL
pub struct AudioRecorder {
    config: Config,
}

/// Audio recording error
#[derive(Debug)]
pub enum AudioError {
    DeviceError(String),
    StreamError(String),
    ConfigError(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::DeviceError(msg) => write!(f, "Audio device error: {}", msg),
            AudioError::StreamError(msg) => write!(f, "Audio stream error: {}", msg),
            AudioError::ConfigError(msg) => write!(f, "Audio config error: {}", msg),
        }
    }
}

impl std::error::Error for AudioError {}

impl AudioRecorder {
    /// Create a new audio recorder with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Start recording audio and return a receiver for audio samples
    pub fn start_recording(
        &mut self,
        device_name: Option<String>,
        _control_rx: mpsc::Receiver<()>,
    ) -> Result<mpsc::Receiver<Vec<i16>>, AudioError> {
        let host = cpal::default_host();

        // Get the audio device
        let device = if let Some(name) = device_name {
            self.find_device_by_name(&host, &name)?
        } else {
            host.default_input_device().ok_or_else(|| {
                AudioError::DeviceError("No default input device found".to_string())
            })?
        };

        // Get the default input config
        let config = device.default_input_config().map_err(|e| {
            AudioError::ConfigError(format!("Failed to get default input config: {}", e))
        })?;

        // Create a channel for sending audio samples
        let (sample_tx, sample_rx) = mpsc::channel();

        // Start the audio stream
        let stream = self.create_input_stream(&device, config, sample_tx)?;

        // Start the stream
        stream
            .play()
            .map_err(|e| AudioError::StreamError(format!("Failed to start stream: {}", e)))?;

        // We need to keep the stream alive somehow, but we can't move it to another thread on Windows
        // For now, let's leak it to keep it alive (not ideal but works for testing)
        std::mem::forget(stream);

        Ok(sample_rx)
    }

    /// Find a device by name
    fn find_device_by_name(&self, host: &cpal::Host, name: &str) -> Result<Device, AudioError> {
        let devices = host.input_devices().map_err(|e| {
            AudioError::DeviceError(format!("Failed to enumerate input devices: {}", e))
        })?;

        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Ok(device);
                }
            }
        }

        let devices = host.output_devices().map_err(|e| {
            AudioError::DeviceError(format!("Failed to enumerate output devices: {}", e))
        })?;

        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Ok(device);
                }
            }
        }

        Err(AudioError::DeviceError(format!(
            "Device '{}' not found",
            name
        )))
    }

    /// Create an input stream for the given device
    fn create_input_stream(
        &self,
        device: &Device,
        config: cpal::SupportedStreamConfig,
        sample_tx: mpsc::Sender<Vec<i16>>,
    ) -> Result<Stream, AudioError> {
        // Create a buffer for collecting samples
        let buffer_size = self.config.buffer_size;
        let mut sample_buffer = Vec::with_capacity(buffer_size);

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        // Capture config values for use in closures
        let quiet_mode = self.config.quiet_mode;

        let stream: Result<Stream, cpal::BuildStreamError> = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let channels = config.channels() as usize;
                let sample_rate = config.sample_rate().0;

                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Process audio properly for fingerprinting
                        let processed_samples =
                            Self::process_audio_data_f32(data, channels, sample_rate);

                        for sample in processed_samples {
                            sample_buffer.push(sample);

                            if sample_buffer.len() >= buffer_size {
                                if sample_tx.send(sample_buffer.clone()).is_err() {
                                    return; // Receiver dropped, stop recording
                                }
                                sample_buffer.clear();
                            }
                        }
                    },
                    move |err| {
                        if !quiet_mode {
                            eprintln!("An error occurred on the input audio stream: {}", err);
                        }
                    },
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let channels = config.channels() as usize;
                let sample_rate = config.sample_rate().0;

                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Process audio properly for fingerprinting
                        let processed_samples =
                            Self::process_audio_data_i16(data, channels, sample_rate);

                        for sample in processed_samples {
                            sample_buffer.push(sample);

                            if sample_buffer.len() >= buffer_size {
                                if sample_tx.send(sample_buffer.clone()).is_err() {
                                    return; // Receiver dropped, stop recording
                                }
                                sample_buffer.clear();
                            }
                        }
                    },
                    move |err| {
                        if !quiet_mode {
                            eprintln!("An error occurred on the input audio stream: {}", err);
                        }
                    },
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert u16 samples to i16
                        for &sample in data.iter() {
                            let sample_i16 = (sample as i32 - 32768) as i16;
                            sample_buffer.push(sample_i16);

                            if sample_buffer.len() >= buffer_size {
                                if sample_tx.send(sample_buffer.clone()).is_err() {
                                    return; // Receiver dropped, stop recording
                                }
                                sample_buffer.clear();
                            }
                        }
                    },
                    move |err| {
                        if !quiet_mode {
                            eprintln!("An error occurred on the input audio stream: {}", err);
                        }
                    },
                    None,
                )
            }
            _ => {
                return Err(AudioError::ConfigError(format!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                )));
            }
        };

        stream.map_err(|e| AudioError::StreamError(format!("Failed to create input stream: {}", e)))
    }

    /// List available input devices
    pub fn list_input_devices() -> Result<Vec<String>, AudioError> {
        let host = cpal::default_host();
        let devices = host.input_devices().map_err(|e| {
            AudioError::DeviceError(format!("Failed to enumerate input devices: {}", e))
        })?;
        let o_devices = host.output_devices().map_err(|e| {
            AudioError::DeviceError(format!("Failed to enumerate output devices: {}", e))
        })?;

        let mut device_names = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                device_names.push(name);
            }
        }
        for device in o_devices {
            if let Ok(name) = device.name() {
                device_names.push(name);
            }
        }

        Ok(device_names)
    }

    /// Process F32 audio data - convert to mono, resample if needed, and convert to i16
    fn process_audio_data_f32(data: &[f32], channels: usize, sample_rate: u32) -> Vec<i16> {
        // Convert to mono if stereo
        let mono_data: Vec<f32> = if channels == 2 {
            // Convert stereo to mono by averaging left and right channels
            data.chunks_exact(2)
                .map(|stereo_pair| (stereo_pair[0] + stereo_pair[1]) / 2.0)
                .collect()
        } else {
            // Already mono or handle other channel configurations
            data.iter().step_by(channels).cloned().collect()
        };

        // Simple downsampling if needed (note: this is basic, could be improved with proper filtering)
        let target_sample_rate = 16000u32;
        let downsampled_data: Vec<f32> = if sample_rate > target_sample_rate {
            let downsample_factor = sample_rate / target_sample_rate;
            mono_data
                .iter()
                .step_by(downsample_factor as usize)
                .cloned()
                .collect()
        } else {
            mono_data
        };

        // Convert to i16
        downsampled_data
            .iter()
            .map(|&sample| (sample * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect()
    }

    /// Process I16 audio data - convert to mono, resample if needed
    fn process_audio_data_i16(data: &[i16], channels: usize, sample_rate: u32) -> Vec<i16> {
        // Convert to mono if stereo
        let mono_data: Vec<i16> = if channels == 2 {
            // Convert stereo to mono by averaging left and right channels
            data.chunks_exact(2)
                .map(|stereo_pair| ((stereo_pair[0] as i32 + stereo_pair[1] as i32) / 2) as i16)
                .collect()
        } else {
            // Already mono or handle other channel configurations
            data.iter().step_by(channels).cloned().collect()
        };

        // Simple downsampling if needed
        let target_sample_rate = 16000u32;
        let downsampled_data: Vec<i16> = if sample_rate > target_sample_rate {
            let downsample_factor = sample_rate / target_sample_rate;
            mono_data
                .iter()
                .step_by(downsample_factor as usize)
                .cloned()
                .collect()
        } else {
            mono_data
        };

        downsampled_data
    }
}
