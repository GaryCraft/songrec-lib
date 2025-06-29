//! # SongRec Library
//! 
//! A Rust library for audio fingerprinting and song recognition using Shazam's algorithm.
//! This library provides both a simple API for one-shot recognition and streaming recognition
//! for continuous monitoring.
//! 
//! ## Features
//! 
//! - Audio fingerprinting using Shazam's algorithm
//! - Song recognition via Shazam's API
//! - Cross-platform audio recording
//! - Multiple output formats (JSON, CSV, text)
//! - Both library and CLI interfaces
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use songrec::{SongRec, Config};
//! 
//! let config = Config::default();
//! let songrec = SongRec::new(config);
//! 
//! // Recognize a song from an audio file
//! match songrec.recognize_from_file("song.mp3") {
//!     Ok(result) => println!("Recognized: {}", result.song_name),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

pub mod config;
pub mod recognition;
pub mod audio;
pub mod output;

// Re-export fingerprinting modules
pub mod fingerprinting {
    pub mod algorithm;
    pub mod signature_format;
    pub mod communication;
    pub mod user_agent;
    pub mod hanning;
}

// Core API
mod songrec;
pub use songrec::{SongRec, RecognitionResult, RecognitionStream};
pub use config::Config;
pub use output::{OutputFormat, RecognitionOutput};

// Re-export key types for convenience
pub use fingerprinting::signature_format::DecodedSignature;
pub use fingerprinting::algorithm::SignatureGenerator;

/// Current version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error types for the library
#[derive(Debug)]
pub enum SongRecError {
    AudioError(String),
    NetworkError(String),
    FingerprintingError(String),
    InvalidInput(String),
    ConfigError(String),
}

impl std::fmt::Display for SongRecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SongRecError::AudioError(msg) => write!(f, "Audio error: {}", msg),
            SongRecError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SongRecError::FingerprintingError(msg) => write!(f, "Fingerprinting error: {}", msg),
            SongRecError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SongRecError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for SongRecError {}

/// Result type for the library
pub type Result<T> = std::result::Result<T, SongRecError>;
