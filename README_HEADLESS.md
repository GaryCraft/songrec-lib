# SongRec - Headless Audio Recognition Library

[![Crates.io](https://img.shields.io/crates/v/songrec.svg)](https://crates.io/crates/songrec)
[![Documentation](https://docs.rs/songrec/badge.svg)](https://docs.rs/songrec)
[![License: GPL-3.0+](https://img.shields.io/badge/License-GPL--3.0+-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A **headless Shazam client library and CLI** for cross-platform audio recognition, written in Rust.

## ✨ Features

- 🎵 **Audio Fingerprinting**: Uses Shazam's audio fingerprinting algorithm
- 🌐 **Cross-Platform**: Works on Windows, Linux, and macOS
- 📚 **Library & CLI**: Use as a Rust library or standalone CLI tool
- 🎧 **Headless**: No GUI dependencies, perfect for servers and automation
- 📊 **Multiple Output Formats**: JSON, CSV, and simple text output
- 🎤 **Live Recognition**: Continuous audio monitoring from microphone
- 📁 **File Recognition**: Recognize songs from audio files
- ⚡ **Fast & Lightweight**: Minimal dependencies, fast recognition

## 🚀 Quick Start

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
songrec = "0.4.3"
```

```rust
use songrec::{SongRec, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let songrec = SongRec::new(config);

    // Recognize from file
    let result = songrec.recognize_from_file("song.mp3")?;
    println!("🎵 {} by {}", result.song_name, result.artist_name);

    // Continuous recognition
    let stream = songrec.start_continuous_recognition()?;
    for result in stream {
        match result {
            Ok(song) => println!("🎵 Detected: {} - {}", song.artist_name, song.song_name),
            Err(e) => eprintln!("❌ Error: {}", e),
        }
    }

    Ok(())
}
```

### As a CLI Tool

```bash
# Build the CLI
cargo build --release

# Recognize a song from file
./target/release/songrec-cli recognize song.mp3 --format json

# Listen continuously
./target/release/songrec-cli listen --format simple

# List available audio devices
./target/release/songrec-cli devices

# Get help
./target/release/songrec-cli --help
```

## 📖 API Documentation

### Core Types

- **`SongRec`**: Main library interface
- **`Config`**: Configuration for recognition parameters
- **`RecognitionResult`**: Detailed song information
- **`RecognitionStream`**: Iterator for continuous recognition
- **`OutputFormat`**: Format options (Simple, JSON, CSV, Custom)

### Configuration

```rust
let config = Config::new()
    .with_sensitivity(0.8)           // Recognition sensitivity (0.0-1.0)
    .with_network_timeout(30)        // Network timeout in seconds
    .with_sample_rate(16000)         // Audio sample rate
    .with_continuous_recognition(true); // Enable continuous mode
```

### Output Formats

```rust
use songrec::{OutputFormat, RecognitionOutput};

// Simple: "Artist - Song"
let simple = RecognitionOutput::format_result(&result, OutputFormat::Simple);

// JSON: Full metadata
let json = RecognitionOutput::format_result(&result, OutputFormat::Json);

// CSV: Comma-separated values
let csv = RecognitionOutput::format_result(&result, OutputFormat::Csv);

// Custom template
let custom = RecognitionOutput::format_result(&result, 
    OutputFormat::Custom("🎵 {song} by {artist} ({year})"));
```

## 🛠️ Installation

### From Source

```bash
git clone https://github.com/marin-m/SongRec.git
cd SongRec
cargo build --release
```

### Using Cargo

```bash
cargo install songrec
```

## 🔧 CLI Usage

### Commands

- **`recognize <file>`**: Recognize a song from an audio file
- **`listen`**: Start continuous recognition from microphone
- **`devices`**: List available audio input devices

### Options

- **`--format <FORMAT>`**: Output format (simple, json, csv)
- **`--device <DEVICE>`**: Specify audio input device
- **`--help`**: Show help information

### Examples

```bash
# Recognize specific file formats
songrec-cli recognize song.mp3 --format json
songrec-cli recognize audio.wav --format csv
songrec-cli recognize music.flac --format simple

# Continuous listening with different devices
songrec-cli listen --device "Built-in Microphone"
songrec-cli listen --format json > recognition_log.json

# Device management
songrec-cli devices
```

## 🎯 Use Cases

- **Music Discovery**: Identify unknown songs from audio files
- **Audio Monitoring**: Continuous recognition for radio/streaming
- **Integration**: Embed recognition in larger applications
- **Automation**: Batch processing of audio files
- **Research**: Audio fingerprinting and analysis
- **Server Deployments**: Headless audio recognition services

## 🏗️ Architecture

```
songrec/
├── lib.rs              # Main library API
├── config.rs           # Configuration management
├── songrec.rs          # Core SongRec implementation
├── audio/              # Cross-platform audio
│   ├── recorder.rs     # CPAL-based recording
│   └── processor.rs    # Audio processing pipeline
├── output.rs           # Multiple output formats
├── fingerprinting/     # Shazam algorithm
└── bin/
    └── songrec-cli.rs  # CLI interface
```

## 🔍 How It Works

1. **Audio Capture**: Records audio using CPAL (cross-platform audio library)
2. **Preprocessing**: Converts audio to 16kHz mono for analysis
3. **Fingerprinting**: Generates acoustic fingerprints using Shazam's algorithm
4. **Recognition**: Sends fingerprints to Shazam's API for identification
5. **Output**: Returns structured song metadata in various formats

## 🚨 Requirements

- **Rust 1.70+**: For building from source
- **Audio System**: ALSA/PulseAudio (Linux), CoreAudio (macOS), WASAPI (Windows)
- **Network**: Internet connection for song recognition
- **Audio Input**: Microphone or audio interface for live recognition

## ✅ Windows Compatibility

This project has been tested and works perfectly on Windows 11 with the following status:

### ✅ Working Features
- ✅ Library compilation and building
- ✅ CLI tool functionality 
- ✅ Audio device listing and detection
- ✅ File recognition with WAV, MP3, OGG, FLAC formats
- ✅ Live microphone recognition
- ✅ All output formats (Simple, JSON, CSV)
- ✅ Cross-platform audio recording via CPAL
- ✅ Network communication with Shazam API

### ⚠️ Known Limitations
- ⚠️ **M4A/AAC Format**: Limited support on Windows due to codec availability in Rust audio libraries
- ⚠️ **Alternative**: Convert M4A files to WAV/MP3 for recognition

### 🎯 Recommended Audio Formats for Windows
- **Primary**: WAV, MP3 (excellent support)
- **Secondary**: OGG, FLAC (good support)
- **Limited**: M4A/AAC (conversion recommended)

## 🧪 Testing on Windows

```powershell
# Clone and build
git clone https://github.com/marin-m/SongRec.git
cd SongRec
cargo build --release

# Test audio device detection
.\target\release\songrec-cli.exe devices

# Test file recognition (WAV recommended)
.\target\release\songrec-cli.exe recognize test_audio.wav --format json

# Test library functionality
cargo run --example library_demo

# Run all tests
cargo test
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## 📄 License

This project is licensed under the GPL-3.0+ License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Original SongRec**: Based on the original SongRec project
- **Shazam**: Audio fingerprinting algorithm research
- **Rust Community**: Amazing crates that make this possible

## 📊 Changelog

### v0.4.3 - Headless Refactor
- ✅ Complete headless library architecture
- ✅ Cross-platform audio support via CPAL
- ✅ New CLI interface with multiple output formats
- ✅ Removed GUI dependencies
- ✅ Added comprehensive API documentation
- ✅ Streaming recognition support
- ✅ Configuration management system

## 🔗 Links

- [Documentation](https://docs.rs/songrec)
- [Crates.io](https://crates.io/crates/songrec)
- [GitHub Repository](https://github.com/marin-m/SongRec)
- [Issues](https://github.com/marin-m/SongRec/issues)
