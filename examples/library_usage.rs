use songrec::{SongRec, Config, OutputFormat, RecognitionOutput};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SongRec Library Usage Examples");
    println!("===============================\n");

    // Example 1: Basic Configuration
    println!("Example 1: Configuration");
    println!("------------------------");
    
    let config = Config::default()
        .with_quiet_mode(true)
        .with_network_timeout(15)
        .with_sensitivity(0.7);
    
    let songrec = SongRec::new(config);
    println!("Created SongRec instance with custom settings\n");

    // Example 2: File Recognition and Data Access
    println!("Example 2: File Recognition");
    println!("---------------------------");
    
    let audio_file = "tests/test_audio.wav";
    
    if Path::new(audio_file).exists() {
        match songrec.recognize_from_file(audio_file) {
            Ok(result) => {
                // Direct access to recognition data
                println!("Recognition data:");
                println!("  Artist: {}", result.artist_name);
                println!("  Song: {}", result.song_name);
                println!("  Album: {}", result.album_name.as_deref().unwrap_or("Unknown"));
                println!("  Year: {}", result.release_year.as_deref().unwrap_or("Unknown"));
                println!("  Genre: {}", result.genre.as_deref().unwrap_or("Unknown"));
                println!("  Track Key: {}", result.track_key);
                println!("  Timestamp: {}", result.recognition_timestamp);
                
                // Example 3: Output Formats
                println!("\nExample 3: Output Formats");
                println!("-------------------------");
                
                // Simple format
                let simple = RecognitionOutput::format_result(&result, OutputFormat::Simple);
                println!("Simple: {}", simple.content);
                
                // JSON format
                let json = RecognitionOutput::format_result(&result, OutputFormat::Json);
                println!("JSON length: {} characters", json.content.len());
                
                // Parse JSON to access fields
                let parsed: serde_json::Value = serde_json::from_str(&json.content)?;
                println!("Parsed JSON - Song: {}", parsed["song_name"]);
                
                // CSV format
                let csv = RecognitionOutput::format_result(&result, OutputFormat::Csv);
                println!("CSV: {}", csv.content);
                
                // Example 4: Raw API Response
                println!("\nExample 4: Raw API Data");
                println!("-----------------------");
                
                // Access raw Shazam API response
                if let Some(track) = result.raw_response.get("track") {
                    if let Some(images) = track.get("images") {
                        println!("Album art available: {}", images.is_object());
                    }
                    if let Some(hub) = track.get("hub") {
                        println!("Streaming links available: {}", hub.is_object());
                    }
                }
                
            }
            Err(e) => {
                println!("Recognition failed: {}", e);
            }
        }
    } else {
        println!("Audio file not found, skipping recognition");
    }

    // Example 5: Function for Integration
    println!("\nExample 5: Integration Function");
    println!("-------------------------------");
    
    fn recognize_audio(file_path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let config = Config::default().with_quiet_mode(true);
        let songrec = SongRec::new(config);
        
        let result = songrec.recognize_from_file(file_path)?;
        let json_output = RecognitionOutput::format_result(&result, OutputFormat::Json);
        let parsed = serde_json::from_str(&json_output.content)?;
        
        Ok(parsed)
    }
    
    if Path::new(audio_file).exists() {
        match recognize_audio(audio_file) {
            Ok(data) => {
                println!("Function returned JSON with {} fields", 
                    data.as_object().map(|o| o.len()).unwrap_or(0));
            }
            Err(e) => println!("Function error: {}", e)
        }
    }

    // Example 6: Configuration Options
    println!("\nExample 6: Configuration Options");
    println!("--------------------------------");
    
    let configs = [
        ("High sensitivity", Config::default().with_sensitivity(0.9)),
        ("Low sensitivity", Config::default().with_sensitivity(0.3)),
        ("Fast timeout", Config::default().with_network_timeout(5)),
        ("Long timeout", Config::default().with_network_timeout(30)),
        ("Verbose mode", Config::default().with_quiet_mode(false)),
    ];
    
    for (name, config) in configs {
        let _instance = SongRec::new(config);
        println!("Created: {}", name);
    }

    // Example 7: Error Handling
    println!("\nExample 7: Error Handling");
    println!("-------------------------");
    
    fn safe_recognize(file_path: &str) -> Result<(String, String), String> {
        if !Path::new(file_path).exists() {
            return Err(format!("File not found: {}", file_path));
        }
        
        let config = Config::default().with_quiet_mode(true);
        let songrec = SongRec::new(config);
        
        match songrec.recognize_from_file(file_path) {
            Ok(result) => Ok((result.artist_name, result.song_name)),
            Err(e) => Err(format!("Recognition error: {}", e)),
        }
    }
    
    // Test with invalid file
    match safe_recognize("nonexistent.wav") {
        Ok((artist, song)) => println!("Result: {} - {}", artist, song),
        Err(e) => println!("Expected error: {}", e),
    }

    // Example 8: Data Structure Access
    println!("\nExample 8: Data Structure Access");
    println!("--------------------------------");
    
    if Path::new(audio_file).exists() {
        if let Ok(result) = songrec.recognize_from_file(audio_file) {
            // Access all available fields
            println!("Available data fields:");
            println!("  song_name: {}", result.song_name);
            println!("  artist_name: {}", result.artist_name);
            println!("  album_name: {:?}", result.album_name);
            println!("  track_key: {}", result.track_key);
            println!("  release_year: {:?}", result.release_year);
            println!("  genre: {:?}", result.genre);
            println!("  recognition_timestamp: {}", result.recognition_timestamp);
            println!("  raw_response keys: {:?}", 
                result.raw_response.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        }
    }

    // Example 9: Available Audio Devices
    println!("\nExample 9: Available Audio Devices");
    println!("----------------------------------");
    
    match songrec::audio::AudioRecorder::list_input_devices() {
        Ok(devices) => {
            println!("Found {} audio input devices:", devices.len());
            for (index, device_name) in devices.iter().enumerate() {
                println!("  Device {}: {}", index, device_name);
            }
            
            // Show how to use a specific device for live recognition
            if !devices.is_empty() {
                println!("\nExample: Using device for live recognition:");
                println!("let device_name = \"{}\";", devices[0]);
                println!("let stream = songrec.start_continuous_recognition_with_device(Some(device_name.to_string()))?;");
                println!("for result in stream {{");
                println!("    match result {{");
                println!("        Ok(recognition) => println!(\"Recognized: {{}} - {{}}\", recognition.artist_name, recognition.song_name),");
                println!("        Err(e) => eprintln!(\"Recognition error: {{}}\", e),");
                println!("    }}");
                println!("}}");
            }
        }
        Err(e) => {
            println!("Error listing audio devices: {}", e);
            println!("This may occur if no audio devices are available or there are permission issues.");
        }
    }

    // Example 10: Device Selection Function
    println!("\nExample 10: Device Selection Function");
    println!("------------------------------------");
    
    fn select_audio_device(device_name_pattern: &str) -> Result<String, String> {
        let devices = songrec::audio::AudioRecorder::list_input_devices()
            .map_err(|e| format!("Failed to list devices: {}", e))?;
        
        // Find device by name pattern
        for device in devices {
            if device.to_lowercase().contains(&device_name_pattern.to_lowercase()) {
                return Ok(device);
            }
        }
        
        Err(format!("No device found matching pattern: {}", device_name_pattern))
    }
    
    // Test device selection
    match select_audio_device("microphone") {
        Ok(device) => println!("Found microphone device: {}", device),
        Err(e) => println!("Device selection result: {}", e),
    }
    
    match select_audio_device("voicemeeter") {
        Ok(device) => println!("Found Voicemeeter device: {}", device),
        Err(e) => println!("Voicemeeter search result: {}", e),
    }

    // Example 11: Complete Device Integration Example
    println!("\nExample 11: Complete Device Integration");
    println!("--------------------------------------");
    
    fn get_best_audio_device() -> Result<String, String> {
        let devices = songrec::audio::AudioRecorder::list_input_devices()
            .map_err(|e| format!("Cannot access audio devices: {}", e))?;
        
        if devices.is_empty() {
            return Err("No audio devices available".to_string());
        }
        
        // Priority order for device selection
        let preferred_patterns = ["voicemeeter", "stereo mix", "microphone", "mic"];
        
        for pattern in preferred_patterns {
            for device in &devices {
                if device.to_lowercase().contains(pattern) {
                    return Ok(device.clone());
                }
            }
        }
        
        // Fallback to first available device
        Ok(devices[0].clone())
    }
    
    match get_best_audio_device() {
        Ok(device) => {
            println!("Selected audio device: {}", device);
            println!("This device can be used for live recognition:");
            println!("  songrec.start_continuous_recognition_with_device(Some(\"{}\".to_string()))", device);
        }
        Err(e) => println!("Device selection failed: {}", e),
    }

    println!("\nLibrary demonstration complete.");
    
    Ok(())
}
