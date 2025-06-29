# SongRec Library

A clean, library-focused Rust crate for audio recognition using Shazam's algorithm, with comprehensive device management.

**Based on the original [SongRec](https://github.com/marin-m/SongRec) by marin-m.**

**Please note that this library was heavily modified with the help of an LLM, which means some stuff needs work, it is functional but not fully battle-tested. nor optimized**

## Features

- 🎵 **Audio Recognition**: Recognize songs from files or live audio
- 📊 **Multiple Formats**: JSON, CSV, and simple text output
- 🌐 **Cross-Platform**: Windows, and Linux support (macOS not tested)

## Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
songrec-lib = "0.5.0"
```

Basic usage:
```rust
use songrec::{SongRec, Config};

let config = Config::default().with_quiet_mode(true);
let songrec = SongRec::new(config);

// Recognize from file
let result = songrec.recognize_from_file("audio.wav")?;
println!("{} - {}", result.artist_name, result.song_name);
```

## Device Management

```rust
use songrec::audio::AudioRecorder;

// List available audio devices
let devices = AudioRecorder::list_input_devices()?;
for (i, device) in devices.iter().enumerate() {
    println!("Device {}: {}", i, device);
}

// Live recognition with specific device
let stream = songrec.start_continuous_recognition_with_device(
    Some("Microphone (USB Audio)".to_string())
)?;

for result in stream {
    match result {
        Ok(recognition) => println!("🎵 {} - {}", 
            recognition.artist_name, recognition.song_name),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Output Formats

```rust
use songrec::{OutputFormat, RecognitionOutput};

// JSON (for APIs)
let json = RecognitionOutput::format_result(&result, OutputFormat::Json);

// CSV (for logging)  
let csv = RecognitionOutput::format_result(&result, OutputFormat::Csv);

// Simple text
let simple = RecognitionOutput::format_result(&result, OutputFormat::Simple);
```

## Examples

```bash
# Basic library usage
cargo run --example library_usage

# Device management demonstration
cargo run --example device_usage
```

## CLI Tool

```bash
# List audio devices
cargo run --bin songrec-lib-cli devices

# Recognize from file
cargo run --bin songrec-lib-cli recognize audio.wav
```

## API Reference

### Core Types
- **`SongRec`**: Main recognition interface
- **`Config`**: Configuration builder
- **`RecognitionResult`**: Song metadata structure
- **`AudioRecorder`**: Device management

### Configuration
```rust
let config = Config::default()
    .with_sensitivity(0.7)          // Recognition sensitivity (0.0-1.0)
    .with_network_timeout(15)       // API timeout in seconds
    .with_quiet_mode(true);         // Suppress debug output
```

### Result Structure
```rust
pub struct RecognitionResult {
    pub song_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub track_key: String,
    pub release_year: Option<String>,
    pub genre: Option<String>,
    pub recognition_timestamp: DateTime<Utc>,
    pub raw_response: serde_json::Value,  // Full Shazam API response
}
```

## Requirements

- Rust 1.70+
- Network connection (for Shazam API)
- Audio system access (for device operations)

## License

GPL-3.0 (same as original SongRec)

## Credits

This library is based on the original [SongRec](https://github.com/marin-m/SongRec) project by marin-m. This version focuses on providing a clean library interface with enhanced device management capabilities while maintaining the core audio fingerprinting functionality from the original project.