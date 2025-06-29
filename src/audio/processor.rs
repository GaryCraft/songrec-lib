use crate::fingerprinting::algorithm::SignatureGenerator;
use crate::fingerprinting::signature_format::DecodedSignature;
use crate::config::Config;

/// Audio processor for generating fingerprints from audio samples
pub struct AudioProcessor {
    signature_generator: SignatureGenerator,
    sample_buffer: Vec<i16>,
    samples_processed: usize,
    target_sample_rate: u32,
    config: Config,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new() -> Self {
        Self {
            signature_generator: SignatureGenerator::new(),
            sample_buffer: Vec::new(),
            samples_processed: 0,
            target_sample_rate: 16000, // Standard sample rate for fingerprinting
            config: Config::default(),
        }
    }

    /// Create a new audio processor with config
    pub fn with_config(config: Config) -> Self {
        Self {
            signature_generator: SignatureGenerator::new(),
            sample_buffer: Vec::new(),
            samples_processed: 0,
            target_sample_rate: 16000, // Standard sample rate for fingerprinting
            config,
        }
    }

    /// Process a batch of audio samples
    /// Returns Some(signature) when enough samples have been processed
    pub fn process_samples(&mut self, samples: &[i16]) -> Result<Option<DecodedSignature>, Box<dyn std::error::Error>> {
        // Add samples to our buffer
        self.sample_buffer.extend_from_slice(samples);
        
        // Process samples in chunks of 128 (as per original algorithm)
        while self.sample_buffer.len() >= 128 {
            let chunk: Vec<i16> = self.sample_buffer.drain(0..128).collect();
            
            // Process the chunk
            self.signature_generator.do_fft(&chunk, self.target_sample_rate);
            self.samples_processed += 128;
            
            // Check if we have enough samples for a signature
            // Use 12 seconds for better recognition accuracy (Shazam's optimal window)
            let min_samples = (12.0 * self.target_sample_rate as f32) as usize;
            
            if self.samples_processed >= min_samples {
                if !self.config.quiet_mode {
                    eprintln!("Attempting recognition with {} samples", self.samples_processed);
                }
                // Get the signature
                let signature = self.signature_generator.get_signature();
                
                // Debug: Check if we have any frequency peaks
                let total_peaks: usize = signature.frequency_band_to_sound_peaks.values().map(|v| v.len()).sum();
                if !self.config.quiet_mode {
                    eprintln!("Generated signature with {} total frequency peaks across {} bands", 
                        total_peaks, signature.frequency_band_to_sound_peaks.len());
                
                    if total_peaks == 0 {
                        eprintln!("WARNING: No frequency peaks detected in audio - may be too quiet or not musical content");
                    }
                }
                
                // Removed delay to test rate-limiting impact
                
                // Reset for next recognition
                self.reset();
                
                return Ok(Some(signature));
            }
        }
        
        Ok(None)
    }

    /// Reset the processor for a new recognition session
    pub fn reset(&mut self) {
        self.signature_generator = SignatureGenerator::new();
        self.sample_buffer.clear();
        self.samples_processed = 0;
    }

    /// Get the current progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        let min_samples = (12.0 * self.target_sample_rate as f32) as usize;
        (self.samples_processed as f32 / min_samples as f32).min(1.0)
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}
