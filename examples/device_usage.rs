use songrec::{SongRec, Config, audio::AudioRecorder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SongRec Device Management and Live Recognition Examples");
    println!("========================================================\n");

    // Example 1: List Available Audio Devices
    println!("Example 1: List Available Audio Devices");
    println!("---------------------------------------");
    
    match AudioRecorder::list_input_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("No audio input devices found.");
                return Ok(());
            }
            
            println!("Found {} audio input devices:", devices.len());
            for (index, device_name) in devices.iter().enumerate() {
                println!("  {}: {}", index, device_name);
            }
            println!();
            
            // Example 2: Device Selection Strategies
            println!("Example 2: Device Selection Strategies");
            println!("--------------------------------------");
            
            // Strategy 1: Select by exact name
            let first_device = &devices[0];
            println!("Strategy 1 - Exact name selection:");
            println!("  Selected: {}", first_device);
            
            // Strategy 2: Select by pattern matching
            println!("\nStrategy 2 - Pattern matching:");
            for pattern in &["microphone", "mic", "voicemeeter", "stereo mix"] {
                if let Some(device) = find_device_by_pattern(&devices, pattern) {
                    println!("  Pattern '{}' found: {}", pattern, device);
                }
            }
            
            // Strategy 3: Priority-based selection
            println!("\nStrategy 3 - Priority-based selection:");
            let selected_device = select_best_device(&devices);
            println!("  Best device: {}", selected_device);
            
            // Example 3: Live Recognition with Device Selection
            println!("\nExample 3: Live Recognition Setup");
            println!("---------------------------------");
            
            // Show how to set up live recognition with different devices
            println!("Setting up live recognition with various devices:\n");
            
            for (i, device) in devices.iter().enumerate().take(3) {
                println!("Device {}: {}", i, device);
                
                // Configuration for this device
                let config = Config::default()
                    .with_quiet_mode(true)
                    .with_network_timeout(10)
                    .with_sensitivity(0.6);
                
                let _songrec = SongRec::new(config);
                
                println!("  -> Configuration: quiet mode, 10s timeout, 0.6 sensitivity");
                println!("  -> Usage: songrec.start_continuous_recognition_with_device(Some(\"{}\".to_string()))", device);
                println!("  -> Ready for live recognition\n");
            }
            
            // Example 4: Complete Live Recognition Example (Commented)
            println!("Example 4: Complete Live Recognition Code");
            println!("------------------------------------------");
            
            print_live_recognition_example(&selected_device);
            
            // Example 5: Device-Specific Configuration
            println!("\nExample 5: Device-Specific Configuration");
            println!("----------------------------------------");
            
            for device in devices.iter().take(2) {
                let config = create_device_specific_config(device);
                println!("Device: {}", device);
                println!("  Recommended config: quiet={}, timeout={}s, sensitivity={}", 
                    config.quiet_mode, config.network_timeout, config.sensitivity);
            }
            
            // Example 6: Error Handling for Device Operations
            println!("\nExample 6: Error Handling");
            println!("-------------------------");
            
            demonstrate_error_handling();
            
        }
        Err(e) => {
            println!("Error listing audio devices: {}", e);
            println!("Common causes:");
            println!("- No audio devices available");
            println!("- Permission issues");
            println!("- Audio system not initialized");
            println!("- Driver problems");
        }
    }
    
    Ok(())
}

/// Find a device by pattern matching (case-insensitive)
fn find_device_by_pattern(devices: &[String], pattern: &str) -> Option<String> {
    devices.iter()
        .find(|device| device.to_lowercase().contains(&pattern.to_lowercase()))
        .cloned()
}

/// Select the best device based on priority patterns
fn select_best_device(devices: &[String]) -> String {
    // Priority order for different device types
    let priority_patterns = [
        "voicemeeter",      // Virtual audio cable (great for system audio)
        "stereo mix",       // Windows stereo mix (system audio)
        "what u hear",      // Some sound cards' system audio
        "microphone",       // Physical microphone
        "mic",             // Short form microphone
        "line in",         // Line input
    ];
    
    // Try to find device by priority
    for pattern in &priority_patterns {
        if let Some(device) = find_device_by_pattern(devices, pattern) {
            return device;
        }
    }
    
    // Fallback to first device
    devices[0].clone()
}

/// Create device-specific configuration
fn create_device_specific_config(device_name: &str) -> Config {
    let device_lower = device_name.to_lowercase();
    
    if device_lower.contains("voicemeeter") {
        // Voicemeeter: System audio, higher sensitivity
        Config::default()
            .with_quiet_mode(true)
            .with_network_timeout(8)
            .with_sensitivity(0.8)
    } else if device_lower.contains("microphone") || device_lower.contains("mic") {
        // Microphone: Lower sensitivity, longer timeout
        Config::default()
            .with_quiet_mode(true)
            .with_network_timeout(15)
            .with_sensitivity(0.5)
    } else if device_lower.contains("stereo mix") {
        // Stereo Mix: System audio, balanced settings
        Config::default()
            .with_quiet_mode(true)
            .with_network_timeout(10)
            .with_sensitivity(0.7)
    } else {
        // Default configuration
        Config::default()
            .with_quiet_mode(true)
            .with_network_timeout(12)
            .with_sensitivity(0.6)
    }
}

/// Print a complete live recognition example
fn print_live_recognition_example(device_name: &str) {
    println!("Complete live recognition example with device selection:");
    println!("```rust");
    println!("use songrec::{{SongRec, Config}};");
    println!("use std::time::Duration;");
    println!();
    println!("fn start_live_recognition() -> Result<(), Box<dyn std::error::Error>> {{");
    println!("    // Create configuration");
    println!("    let config = Config::default()");
    println!("        .with_quiet_mode(true)");
    println!("        .with_network_timeout(10)");
    println!("        .with_sensitivity(0.6);");
    println!();
    println!("    let songrec = SongRec::new(config);");
    println!();
    println!("    // Start recognition with specific device");
    println!("    let device_name = \"{}\";", device_name);
    println!("    let stream = songrec.start_continuous_recognition_with_device(");
    println!("        Some(device_name.to_string())");
    println!("    )?;");
    println!();
    println!("    println!(\"Starting live recognition with device: {{}}\", device_name);");
    println!("    println!(\"Listening for audio...\");");
    println!();
    println!("    // Process recognition results");
    println!("    for result in stream {{");
    println!("        match result {{");
    println!("            Ok(recognition) => {{");
    println!("                println!(\"🎵 Recognized: {{}} - {{}}\", ");
    println!("                    recognition.artist_name, recognition.song_name);");
    println!("                ");
    println!("                // Access additional data");
    println!("                if let Some(album) = &recognition.album_name {{");
    println!("                    println!(\"   Album: {{}}\", album);");
    println!("                }}");
    println!("                if let Some(year) = &recognition.release_year {{");
    println!("                    println!(\"   Year: {{}}\", year);");
    println!("                }}");
    println!("            }}");
    println!("            Err(e) => {{");
    println!("                eprintln!(\"Recognition error: {{}}\", e);");
    println!("                // Decide whether to continue or break");
    println!("            }}");
    println!("        }}");
    println!("    }}");
    println!();
    println!("    Ok(())");
    println!("}}");
    println!("```");
}

/// Demonstrate error handling for device operations
fn demonstrate_error_handling() {
    println!("Error handling examples:");
    
    // 1. Device not found
    println!("1. Handling device not found:");
    println!("```rust");
    println!("match songrec.start_continuous_recognition_with_device(Some(\"NonExistentDevice\".to_string())) {{");
    println!("    Ok(stream) => {{ /* Handle stream */ }}");
    println!("    Err(e) => {{");
    println!("        eprintln!(\"Failed to start recognition: {{}}\", e);");
    println!("        // Fallback to default device");
    println!("        let stream = songrec.start_continuous_recognition()?;");
    println!("    }}");
    println!("}}");
    println!("```");
    
    // 2. No devices available
    println!("\n2. Handling no devices available:");
    println!("```rust");
    println!("match AudioRecorder::list_input_devices() {{");
    println!("    Ok(devices) if devices.is_empty() => {{");
    println!("        eprintln!(\"No audio devices available\");");
    println!("        return Err(\"No audio hardware found\".into());");
    println!("    }}");
    println!("    Ok(devices) => {{ /* Use devices */ }}");
    println!("    Err(e) => {{");
    println!("        eprintln!(\"Cannot access audio system: {{}}\", e);");
    println!("        return Err(e.into());");
    println!("    }}");
    println!("}}");
    println!("```");
    
    // 3. Robust device selection
    println!("\n3. Robust device selection:");
    println!("```rust");
    println!("fn select_device_safely() -> Result<String, Box<dyn std::error::Error>> {{");
    println!("    let devices = AudioRecorder::list_input_devices()?;");
    println!("    ");
    println!("    if devices.is_empty() {{");
    println!("        return Err(\"No audio devices available\".into());");
    println!("    }}");
    println!("    ");
    println!("    // Try preferred devices first");
    println!("    for pattern in [\"voicemeeter\", \"stereo mix\", \"microphone\"] {{");
    println!("        if let Some(device) = devices.iter()");
    println!("            .find(|d| d.to_lowercase().contains(pattern)) {{");
    println!("            return Ok(device.clone());");
    println!("        }}");
    println!("    }}");
    println!("    ");
    println!("    // Fallback to first device");
    println!("    Ok(devices[0].clone())");
    println!("}}");
    println!("```");
}
