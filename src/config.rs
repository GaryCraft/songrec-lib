use serde::{Deserialize, Serialize};

/// Configuration for SongRec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Recognition sensitivity (0.0 to 1.0)
    pub sensitivity: f32,
    
    /// Timeout for network requests in seconds
    pub network_timeout: u64,
    
    /// Minimum duration of audio to analyze (in seconds)
    pub min_audio_duration: f32,
    
    /// Maximum duration of audio to analyze (in seconds)  
    pub max_audio_duration: f32,
    
    /// Sample rate for audio processing
    pub sample_rate: u32,
    
    /// Buffer size for audio processing
    pub buffer_size: usize,
    
    /// Whether to enable continuous recognition
    pub continuous_recognition: bool,
    
    /// Interval between recognition attempts in continuous mode (seconds)
    pub recognition_interval: f32,
    
    /// Whether to suppress verbose debug output
    pub quiet_mode: bool,
    
    /// Whether to deduplicate requests (prevent sending same signature multiple times)
    pub deduplicate_requests: bool,
    
    /// Time in seconds to remember signatures for deduplication
    pub deduplication_cache_duration: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sensitivity: 0.5,
            network_timeout: 20,
            min_audio_duration: 3.0,
            max_audio_duration: 12.0,
            sample_rate: 16000,
            buffer_size: 4096,
            continuous_recognition: false,
            recognition_interval: 5.0,
            quiet_mode: true, // Default to quiet mode for clean output
            deduplicate_requests: true,
            deduplication_cache_duration: 300, // 5 minutes
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the sensitivity level
    pub fn with_sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity.clamp(0.0, 1.0);
        self
    }
    
    /// Set the network timeout
    pub fn with_network_timeout(mut self, timeout: u64) -> Self {
        self.network_timeout = timeout;
        self
    }
    
    /// Set the minimum audio duration
    pub fn with_min_audio_duration(mut self, duration: f32) -> Self {
        self.min_audio_duration = duration;
        self
    }
    
    /// Set the maximum audio duration
    pub fn with_max_audio_duration(mut self, duration: f32) -> Self {
        self.max_audio_duration = duration;
        self
    }
    
    /// Set the sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }
    
    /// Set the buffer size
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
    
    /// Enable or disable continuous recognition
    pub fn with_continuous_recognition(mut self, enabled: bool) -> Self {
        self.continuous_recognition = enabled;
        self
    }
    
    /// Set the recognition interval for continuous mode
    pub fn with_recognition_interval(mut self, interval: f32) -> Self {
        self.recognition_interval = interval;
        self
    }
    
    /// Enable or disable quiet mode (suppress verbose output)
    pub fn with_quiet_mode(mut self, quiet: bool) -> Self {
        self.quiet_mode = quiet;
        self
    }
    
    /// Enable or disable request deduplication
    pub fn with_deduplication(mut self, enabled: bool) -> Self {
        self.deduplicate_requests = enabled;
        self
    }
    
    /// Set the deduplication cache duration
    pub fn with_deduplication_cache_duration(mut self, duration: u64) -> Self {
        self.deduplication_cache_duration = duration;
        self
    }
    
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a TOML file
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
