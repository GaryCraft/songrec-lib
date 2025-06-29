use songrec::{SongRec, Config, OutputFormat, RecognitionOutput};

#[test] 
fn test_config_creation() {
    let config = Config::default();
    assert_eq!(config.sample_rate, 16000);
    assert_eq!(config.sensitivity, 0.5);
    
    let custom_config = Config::new()
        .with_sensitivity(0.8)
        .with_sample_rate(44100)
        .with_network_timeout(30);
    
    assert_eq!(custom_config.sensitivity, 0.8);
    assert_eq!(custom_config.sample_rate, 44100);
    assert_eq!(custom_config.network_timeout, 30);
}

#[test]
fn test_songrec_creation() {
    let config = Config::default();
    let _songrec = SongRec::new(config);
    // SongRec should be created successfully
}

#[test]
fn test_output_format() {
    // Since we can't test actual recognition without audio files,
    // let's test the output formatting with a mock result
    use songrec::RecognitionResult;
    
    let mock_result = RecognitionResult {
        song_name: "Test Song".to_string(),
        artist_name: "Test Artist".to_string(),
        album_name: Some("Test Album".to_string()),
        track_key: "test_key".to_string(),
        release_year: Some("2023".to_string()),
        genre: Some("Pop".to_string()),
        recognition_timestamp: chrono::Utc::now(),
        raw_response: serde_json::json!({}),
    };
    
    let simple_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Simple);
    assert_eq!(simple_output.content, "Test Artist - Test Song");
    
    let json_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Json);
    assert!(json_output.content.contains("Test Song"));
    assert!(json_output.content.contains("Test Artist"));
    
    let csv_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Csv);
    assert!(csv_output.content.contains("Test Song"));
    assert!(csv_output.content.contains("Test Artist"));
    assert!(csv_output.content.contains("Test Album"));
}

#[test]
fn test_audio_device_listing() {
    // Test that we can list audio devices without panicking
    let result = songrec::audio::AudioRecorder::list_input_devices();
    // Should return Ok or Err, but not panic
    match result {
        Ok(devices) => {
            println!("Found {} audio devices", devices.len());
        }
        Err(e) => {
            println!("Error listing devices: {}", e);
        }
    }
}
