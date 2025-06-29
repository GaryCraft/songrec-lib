use songrec::{SongRec, Config, OutputFormat, RecognitionOutput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SongRec Library Example");
    println!("=======================");

    // Create a custom configuration
    let config = Config::new()
        .with_sensitivity(0.8)
        .with_network_timeout(30)
        .with_sample_rate(16000);

    println!("Configuration: {:?}", config);

    // Create SongRec instance
    let _songrec = SongRec::new(config);
    println!("✅ SongRec instance created successfully");

    // Demonstrate different output formats with mock data
    println!("\n📄 Output Format Examples:");
    println!("=========================");

    // Create a mock recognition result for demonstration
    let mock_result = songrec::RecognitionResult {
        song_name: "Bohemian Rhapsody".to_string(),
        artist_name: "Queen".to_string(),
        album_name: Some("A Night at the Opera".to_string()),
        track_key: "123456789".to_string(),
        release_year: Some("1975".to_string()),
        genre: Some("Rock".to_string()),
        recognition_timestamp: chrono::Utc::now(),
        raw_response: serde_json::json!({
            "track": {
                "title": "Bohemian Rhapsody",
                "subtitle": "Queen"
            }
        }),
    };

    // Demonstrate different output formats
    println!("\n🎵 Simple Format:");
    let simple_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Simple);
    println!("{}", simple_output);

    println!("\n📋 CSV Format:");
    println!("{}", RecognitionOutput::csv_header());
    let csv_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Csv);
    println!("{}", csv_output);

    println!("\n📄 JSON Format:");
    let json_output = RecognitionOutput::format_result(&mock_result, OutputFormat::Json);
    println!("{}", json_output);

    // List available audio devices
    println!("\n🎤 Available Audio Devices:");
    println!("===========================");
    match songrec::audio::AudioRecorder::list_input_devices() {
        Ok(devices) => {
            for (i, device) in devices.iter().enumerate() {
                println!("  {}: {}", i, device);
            }
        }
        Err(e) => {
            println!("❌ Error listing devices: {}", e);
        }
    }

    println!("\n🚀 Library initialization completed successfully!");
    println!("\nTo use this library for actual recognition:");
    println!("1. For file recognition: songrec.recognize_from_file(\"path/to/song.mp3\")");
    println!("2. For continuous recognition: songrec.start_continuous_recognition()");
    println!("3. Or use the CLI: ./target/debug/songrec-cli --help");

    Ok(())
}
