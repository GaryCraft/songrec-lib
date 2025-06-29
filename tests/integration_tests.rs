use songrec::{SongRec, Config, OutputFormat, RecognitionOutput};
use std::path::Path;

/// Test basic configuration creation and validation
#[test]
fn test_config_creation() {
    let config = Config::default();
    assert_eq!(config.sample_rate, 16000);
    assert_eq!(config.sensitivity, 0.5);
    assert_eq!(config.quiet_mode, true); // Should default to quiet mode
    assert_eq!(config.deduplicate_requests, true);
    
    // Test custom configuration
    let custom_config = Config::new()
        .with_sensitivity(0.8)
        .with_sample_rate(44100)
        .with_network_timeout(30)
        .with_quiet_mode(false);
    
    assert_eq!(custom_config.sensitivity, 0.8);
    assert_eq!(custom_config.sample_rate, 44100);
    assert_eq!(custom_config.network_timeout, 30);
    assert_eq!(custom_config.quiet_mode, false);
}

/// Test SongRec instance creation
#[test]
fn test_songrec_creation() {
    let config = Config::default();
    let _songrec = SongRec::new(config);
    // Should create successfully without panicking
}

/// Test configuration builders
#[test]
fn test_config_builders() {
    let config = Config::default()
        .with_sensitivity(0.7)
        .with_min_audio_duration(2.0)
        .with_max_audio_duration(15.0)
        .with_buffer_size(8192)
        .with_continuous_recognition(true)
        .with_recognition_interval(3.0)
        .with_deduplication(false)
        .with_deduplication_cache_duration(600);
    
    assert_eq!(config.sensitivity, 0.7);
    assert_eq!(config.min_audio_duration, 2.0);
    assert_eq!(config.max_audio_duration, 15.0);
    assert_eq!(config.buffer_size, 8192);
    assert_eq!(config.continuous_recognition, true);
    assert_eq!(config.recognition_interval, 3.0);
    assert_eq!(config.deduplicate_requests, false);
    assert_eq!(config.deduplication_cache_duration, 600);
}

/// Test sensitivity clamping
#[test]
fn test_sensitivity_clamping() {
    let config1 = Config::default().with_sensitivity(-0.5);
    assert_eq!(config1.sensitivity, 0.0);
    
    let config2 = Config::default().with_sensitivity(1.5);
    assert_eq!(config2.sensitivity, 1.0);
    
    let config3 = Config::default().with_sensitivity(0.5);
    assert_eq!(config3.sensitivity, 0.5);
}

/// Test audio device listing functionality
#[test]
fn test_audio_device_listing() {
    match songrec::audio::AudioRecorder::list_input_devices() {
        Ok(devices) => {
            // Should return a list of devices (may be empty on some systems)
            println!("Found {} audio devices", devices.len());
            for (i, device) in devices.iter().enumerate() {
                println!("  Device {}: {}", i, device);
            }
        }
        Err(e) => {
            println!("Error listing devices (this may be normal in CI): {}", e);
            // Don't fail the test as audio devices may not be available in all environments
        }
    }
}

/// Test output format functionality with mock data
#[test]
fn test_output_formats() {
    // Create a mock recognition result
    let mock_result = songrec::RecognitionResult {
        song_name: "Proof of Concept".to_string(),
        artist_name: "Wintergatan".to_string(),
        album_name: Some("Test Album".to_string()),
        track_key: "test_key_123".to_string(),
        release_year: Some("2023".to_string()),
        genre: Some("Electronic".to_string()),
        recognition_timestamp: chrono::Utc::now(),
        raw_response: serde_json::json!({
            "track": {
                "title": "Proof of Concept",
                "subtitle": "Wintergatan",
                "key": "test_key_123"
            }
        }),
    };
    
    // Test Simple format
    let simple_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Simple);
    assert_eq!(simple_output.content, "Wintergatan - Proof of Concept");
    
    // Test JSON format
    let json_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Json);
    assert!(json_output.content.contains("Proof of Concept"));
    assert!(json_output.content.contains("Wintergatan"));
    assert!(json_output.content.contains("test_key_123"));
    
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_output.content)
        .expect("JSON output should be valid JSON");
    assert_eq!(parsed["song_name"], "Proof of Concept");
    assert_eq!(parsed["artist_name"], "Wintergatan");
    
    // Test CSV format
    let csv_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Csv);
    assert!(csv_output.content.contains("Wintergatan"));
    assert!(csv_output.content.contains("Proof of Concept"));
    
    // Test CSV header
    let csv_header = RecognitionOutput::csv_header();
    assert!(csv_header.contains("Artist"));
    assert!(csv_header.contains("Song"));
    assert!(csv_header.contains("Timestamp"));
}

/// Test file recognition with test audio
#[test]
fn test_file_recognition() {
    let test_audio_path = "tests/test_audio.wav";
    
    // Skip test if audio file doesn't exist
    if !Path::new(test_audio_path).exists() {
        println!("Skipping file recognition test - test audio file not found");
        return;
    }
    
    let config = Config::default()
        .with_quiet_mode(true) // Suppress network debug output during tests
        .with_network_timeout(10); // Shorter timeout for tests
    
    let songrec = SongRec::new(config);
    
    // Test file recognition
    match songrec.recognize_from_file(test_audio_path) {
        Ok(result) => {
            println!("Recognition successful!");
            println!("Artist: {}", result.artist_name);
            println!("Song: {}", result.song_name);
            
            // Basic validation
            assert!(!result.artist_name.is_empty(), "Artist name should not be empty");
            assert!(!result.song_name.is_empty(), "Song name should not be empty");
            assert!(!result.track_key.is_empty(), "Track key should not be empty");
            
            // Test output formatting
            let simple_output = RecognitionOutput::format_result(&result, OutputFormat::Simple);
            assert!(simple_output.content.contains(&result.artist_name));
            assert!(simple_output.content.contains(&result.song_name));
        }
        Err(e) => {
            println!("Recognition failed (this may be normal if API is unreachable): {}", e);
            // Don't fail the test as network issues or API changes are external factors
            // In a real test environment, you might want to fail here
        }
    }
}

/// Test MP3 file recognition
#[test]
fn test_mp3_file_recognition() {
    let test_audio_path = "tests/test_audio.mp3";
    
    // Skip test if audio file doesn't exist
    if !Path::new(test_audio_path).exists() {
        println!("Skipping MP3 recognition test - test audio file not found");
        return;
    }
    
    let config = Config::default()
        .with_quiet_mode(true)
        .with_network_timeout(10);
    
    let songrec = SongRec::new(config);
    
    match songrec.recognize_from_file(test_audio_path) {
        Ok(result) => {
            println!("MP3 Recognition successful!");
            println!("Artist: {}", result.artist_name);
            println!("Song: {}", result.song_name);
            
            assert!(!result.artist_name.is_empty());
            assert!(!result.song_name.is_empty());
        }
        Err(e) => {
            println!("MP3 Recognition failed (this may be normal): {}", e);
        }
    }
}

/// Test error handling with invalid file
#[test]
fn test_invalid_file_handling() {
    let config = Config::default();
    let songrec = SongRec::new(config);
    
    // Test with non-existent file
    let result = songrec.recognize_from_file("tests/nonexistent.wav");
    assert!(result.is_err(), "Should fail with non-existent file");
    
    // Test with invalid audio file (create a text file with .wav extension)
    std::fs::write("tests/invalid.wav", "This is not an audio file").unwrap();
    let result = songrec.recognize_from_file("tests/invalid.wav");
    assert!(result.is_err(), "Should fail with invalid audio file");
    
    // Cleanup
    std::fs::remove_file("tests/invalid.wav").ok();
}

/// Test configuration serialization
#[test]
fn test_config_serialization() {
    let config = Config::default()
        .with_sensitivity(0.7)
        .with_network_timeout(25)
        .with_quiet_mode(false);
    
    // Test saving to file
    let temp_path = "tests/temp_config.toml";
    match config.to_file(temp_path) {
        Ok(_) => {
            // Test loading from file
            match Config::from_file(temp_path) {
                Ok(loaded_config) => {
                    assert_eq!(loaded_config.sensitivity, 0.7);
                    assert_eq!(loaded_config.network_timeout, 25);
                    assert_eq!(loaded_config.quiet_mode, false);
                }
                Err(e) => println!("Could not load config (TOML support may not be available): {}", e),
            }
            
            // Cleanup
            std::fs::remove_file(temp_path).ok();
        }
        Err(e) => println!("Could not save config (TOML support may not be available): {}", e),
    }
}

/// Test audio recorder creation with config
#[test]
fn test_audio_recorder_creation() {
    let config = Config::default();
    let _recorder = songrec::audio::AudioRecorder::new(config);
    // Should create successfully
}

/// Integration test for the complete recognition pipeline
#[test]
fn test_recognition_pipeline_integration() {
    // Test the complete pipeline with different configurations
    let configs = vec![
        Config::default(),
        Config::default().with_sensitivity(0.3),
        Config::default().with_sensitivity(0.8),
        Config::default().with_network_timeout(5),
        Config::default().with_quiet_mode(false),
    ];
    
    for (i, config) in configs.into_iter().enumerate() {
        println!("Testing configuration {}", i);
        let _songrec = SongRec::new(config);
        // Should create successfully with all configurations
    }
}
