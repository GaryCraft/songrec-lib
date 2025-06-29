use serde_json::{json, Value};
use reqwest::header::HeaderMap;
use std::time::SystemTime;
use std::error::Error;
use std::time::Duration;
use std::thread;
use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::fingerprinting::signature_format::DecodedSignature;
use crate::fingerprinting::user_agent::USER_AGENTS;
use crate::config::Config;

pub fn recognize_song_from_signature(signature: &DecodedSignature) -> Result<Value, Box<dyn Error>> {
    recognize_song_from_signature_with_config(signature, &Config::default())
}

pub fn recognize_song_from_signature_with_config(signature: &DecodedSignature, config: &Config) -> Result<Value, Box<dyn Error>> {
    let timestamp_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis();
    
    let post_data = json!({
        "geolocation": {
            "altitude": 300,
            "latitude": 45,
            "longitude": 2
        },
        "signature": {
            "samplems": (signature.number_samples as f32 / signature.sample_rate_hz as f32 * 1000.) as u32,
            "timestamp": timestamp_ms as u32,
            "uri": signature.encode_to_uri()?
        },
        "timestamp": timestamp_ms as u32,
        "timezone": "Europe/Paris"
    });

    let uuid_1 = Uuid::new_v4().to_hyphenated().to_string().to_uppercase();
    let uuid_2 = Uuid::new_v4().to_hyphenated().to_string();

    let url = format!("https://amp.shazam.com/discovery/v5/en/US/android/-/tag/{}/{}", uuid_1, uuid_2);

    // Only show debug info if not in quiet mode
    if !config.quiet_mode {
        eprintln!("Sending recognition request...");
    }

    // Try multiple attempts with different client configurations
    for attempt in 1..=3 {
        if !config.quiet_mode {
            eprintln!("Attempt {}/3...", attempt);
        }
        match try_shazam_request_with_config(&url, &post_data, attempt, config) {
            Ok(response) => {
                if !config.quiet_mode {
                    eprintln!("Successfully received response on attempt {}", attempt);
                }
                return Ok(response);
            },
            Err(e) => {
                if !config.quiet_mode {
                    eprintln!("Attempt {} failed: {}", attempt, e);
                }
                if attempt < 3 {
                    if !config.quiet_mode {
                        eprintln!("Waiting 2 seconds before retry...");
                    }
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }

    Err("All API requests failed".into())
}

fn try_shazam_request_with_config(url: &str, post_data: &Value, attempt: u32, config: &Config) -> Result<Value, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", USER_AGENTS.choose(&mut rand::thread_rng()).unwrap().parse()?);
    headers.insert("Content-Language", "en_US".parse()?);

    // Try different client configurations based on attempt
    let client = match attempt {
        1 => reqwest_client_native_tls()?,     // Native TLS for better compatibility
        2 => reqwest_client_basic()?,      // Basic client with minimal features
        _ => reqwest_client_legacy()?,     // Legacy fallback
    };
    
    let response = client.post(url)
        .timeout(Duration::from_secs(30)) // Longer timeout for Windows
        .query(&[
            ("sync", "true"),
            ("webv3", "true"),
            ("sampling", "true"),
            ("connected", ""),
            ("shazamapiversion", "v3"),
            ("sharehub", "true"),
            ("video", "v3")
        ])
        .headers(headers)
        .json(post_data)
        .send()?;
    
    // Check status code
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")).into());
    }
    
    // Get response as text first to see what we're receiving
    let response_text = response.text()?;
    
    // Only show debug info if not in quiet mode
    if !config.quiet_mode {
        eprintln!("Raw response (attempt {}): {}", attempt, response_text);
    }
    
    // Try to parse as JSON
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON response: {}. Raw response: '{}'", e, response_text))?;
    
    // Only show detailed analysis if not in quiet mode
    if config.quiet_mode {
        // Extract response info in quiet mode (minimal output)
        extract_simple_response_info(&response_json);
    } else {
        eprintln!("=== COMPLETE SHAZAM API RESPONSE ===");
        eprintln!("Raw JSON: {}", serde_json::to_string_pretty(&response_json)?);
        eprintln!("=====================================");
        
        // Extract ALL possible information from the response (verbose mode)
        extract_complete_response_info(&response_json)?;
    }
    
    Ok(response_json)
}

pub fn obtain_raw_cover_image(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {

    let mut headers = HeaderMap::new();
    
    headers.insert("User-Agent", USER_AGENTS.choose(&mut rand::thread_rng()).unwrap().parse()?);
    headers.insert("Content-Language", "en_US".parse()?);

    let client = reqwest_client_native_tls()?;
    let response = client.get(url)
        .timeout(Duration::from_secs(20))
        .headers(headers)
        .send()?;
    
    Ok(response.bytes()?.as_ref().to_vec())

}

fn extract_simple_response_info(_response: &Value) {
    // In quiet mode, only output parseable information
    // No console output here - let the main program handle result formatting
}

fn extract_complete_response_info(response: &Value) -> Result<(), Box<dyn Error>> {
    eprintln!("\n🔍 EXHAUSTIVE RESPONSE ANALYSIS 🔍");
    eprintln!("═══════════════════════════════════════");
    
    // Top-level response metadata
    eprintln!("\n📊 RESPONSE METADATA:");
    extract_value_info(response, "tagid", "Tag ID");
    extract_value_info(response, "timestamp", "Timestamp");
    extract_value_info(response, "timezone", "Timezone");
    extract_value_info(response, "retailer", "Retailer");
    extract_value_info(response, "server", "Server");
    extract_value_info(response, "uuid", "UUID");
    extract_value_info(response, "version", "API Version");
    extract_value_info(response, "track", "Track (top-level)");
    extract_value_info(response, "status", "Status");
    extract_value_info(response, "error", "Error");
    extract_value_info(response, "message", "Message");
    
    // Location information
    if let Some(location) = response.get("location") {
        eprintln!("\n📍 LOCATION DATA:");
        extract_value_info(location, "latitude", "Latitude");
        extract_value_info(location, "longitude", "Longitude");
        extract_value_info(location, "altitude", "Altitude");
        extract_value_info(location, "accuracy", "Accuracy");
        extract_value_info(location, "country", "Country");
        extract_value_info(location, "city", "City");
        extract_value_info(location, "region", "Region");
        extract_value_info(location, "timezone", "Location Timezone");
        extract_value_info(location, "ip", "IP Address");
        extract_value_info(location, "provider", "Location Provider");
        // Any additional location fields
        if let Some(obj) = location.as_object() {
            for (key, value) in obj {
                if !["latitude", "longitude", "altitude", "accuracy", "country", "city", "region", "timezone", "ip", "provider"].contains(&key.as_str()) {
                    eprintln!("   🏷️  Location {}: {}", key, value);
                }
            }
        }
    }
    
    // Matches array - complete analysis
    if let Some(matches) = response.get("matches").and_then(|m| m.as_array()) {
        eprintln!("\n🎵 MATCHES FOUND: {}", matches.len());
        
        if matches.is_empty() {
            eprintln!("   ❌ No songs recognized");
        } else {
            for (i, match_obj) in matches.iter().enumerate() {
                eprintln!("\n🎶 MATCH #{} - COMPLETE DETAILS:", i + 1);
                eprintln!("─────────────────────────────────────");
                
                // Match-level information
                extract_value_info(match_obj, "id", "Match ID");
                extract_value_info(match_obj, "offset", "Offset");
                extract_value_info(match_obj, "timeskew", "Time Skew");
                extract_value_info(match_obj, "frequencyskew", "Frequency Skew");
                
                // Track information - comprehensive extraction
                if let Some(track) = match_obj.get("track") {
                    eprintln!("\n🎼 TRACK INFORMATION:");
                    
                    // Basic track info
                    extract_value_info(track, "key", "Track Key");
                    extract_value_info(track, "title", "Title");
                    extract_value_info(track, "subtitle", "Artist/Subtitle");
                    extract_value_info(track, "layout", "Layout");
                    extract_value_info(track, "type", "Type");
                    extract_value_info(track, "isrc", "ISRC");
                    extract_value_info(track, "albumadamid", "Album Adam ID");
                    extract_value_info(track, "artistadamid", "Artist Adam ID");
                    extract_value_info(track, "trackadamid", "Track Adam ID");
                    extract_value_info(track, "myshazam", "MyShazam");
                    
                    // Images
                    if let Some(images) = track.get("images") {
                        eprintln!("\n🖼️  IMAGES:");
                        extract_images_info(images);
                    }
                    
                    // Share information
                    if let Some(share) = track.get("share") {
                        eprintln!("\n🔗 SHARE INFORMATION:");
                        extract_value_info(share, "subject", "Subject");
                        extract_value_info(share, "text", "Text");
                        extract_value_info(share, "href", "Share Link");
                        extract_value_info(share, "image", "Share Image");
                        extract_value_info(share, "twitter", "Twitter");
                        extract_value_info(share, "html", "HTML");
                        extract_value_info(share, "avatar", "Avatar");
                        extract_value_info(share, "snapchat", "Snapchat");
                        extract_value_info(share, "facebook", "Facebook");
                        extract_value_info(share, "whatsapp", "WhatsApp");
                        extract_value_info(share, "telegram", "Telegram");
                        extract_value_info(share, "instagram", "Instagram");
                        extract_value_info(share, "pinterest", "Pinterest");
                        extract_value_info(share, "linkedin", "LinkedIn");
                        extract_value_info(share, "reddit", "Reddit");
                        extract_value_info(share, "tumblr", "Tumblr");
                        extract_value_info(share, "discord", "Discord");
                        extract_value_info(share, "email", "Email");
                        extract_value_info(share, "sms", "SMS");
                        extract_value_info(share, "copy", "Copy Link");
                        
                        // Any unknown share fields
                        if let Some(share_obj) = share.as_object() {
                            let known_share_fields = [
                                "subject", "text", "href", "image", "twitter", "html", "avatar", "snapchat",
                                "facebook", "whatsapp", "telegram", "instagram", "pinterest", "linkedin",
                                "reddit", "tumblr", "discord", "email", "sms", "copy"
                            ];
                            for (key, value) in share_obj {
                                if !known_share_fields.contains(&key.as_str()) {
                                    eprintln!("   🆕 UNKNOWN SHARE FIELD {}: {}", key, value);
                                }
                            }
                        }
                    }
                    
                    // Hub information
                    if let Some(hub) = track.get("hub") {
                        eprintln!("\n🎧 HUB INFORMATION:");
                        extract_value_info(hub, "type", "Hub Type");
                        extract_value_info(hub, "image", "Hub Image");
                        extract_value_info(hub, "displayname", "Display Name");
                        extract_value_info(hub, "explicit", "Explicit");
                        extract_value_info(hub, "uri", "URI");
                        extract_value_info(hub, "name", "Hub Name");
                        
                        if let Some(actions) = hub.get("actions").and_then(|a| a.as_array()) {
                            eprintln!("\n🎯 HUB ACTIONS:");
                            for (j, action) in actions.iter().enumerate() {
                                eprintln!("   Action #{}: {}", j + 1, serde_json::to_string_pretty(action)?);
                            }
                        }
                        
                        if let Some(options) = hub.get("options").and_then(|o| o.as_array()) {
                            eprintln!("\n⚙️  HUB OPTIONS:");
                            for (j, option) in options.iter().enumerate() {
                                eprintln!("   Option #{}: {}", j + 1, serde_json::to_string_pretty(option)?);
                            }
                        }
                        
                        if let Some(providers) = hub.get("providers").and_then(|p| p.as_array()) {
                            eprintln!("\n🏢 PROVIDERS:");
                            for (j, provider) in providers.iter().enumerate() {
                                eprintln!("   Provider #{}: {}", j + 1, serde_json::to_string_pretty(provider)?);
                            }
                        }
                        
                        // Any unknown hub fields
                        eprintln!("\n🔍 ALL HUB FIELDS:");
                        if let Some(hub_obj) = hub.as_object() {
                            let known_hub_fields = [
                                "type", "image", "displayname", "explicit", "uri", "name",
                                "actions", "options", "providers"
                            ];
                            for (key, value) in hub_obj {
                                if !known_hub_fields.contains(&key.as_str()) {
                                    eprintln!("   🆕 UNKNOWN HUB FIELD {}: {}", key, value);
                                }
                            }
                        }
                    }
                    
                    // Sections - detailed analysis
                    if let Some(sections) = track.get("sections").and_then(|s| s.as_array()) {
                        eprintln!("\n📚 SECTIONS ({} found):", sections.len());
                        
                        for (j, section) in sections.iter().enumerate() {
                            eprintln!("\n   📄 SECTION #{}: ", j + 1);
                            extract_value_info(section, "type", "   Type");
                            extract_value_info(section, "metapages", "   Metapages");
                            extract_value_info(section, "tabname", "   Tab Name");
                            extract_value_info(section, "text", "   Text");
                            extract_value_info(section, "url", "   URL");
                            extract_value_info(section, "youtubeurl", "   YouTube URL");
                            extract_value_info(section, "actions", "   Actions");
                            extract_value_info(section, "options", "   Options");
                            extract_value_info(section, "footer", "   Footer");
                            extract_value_info(section, "header", "   Header");
                            extract_value_info(section, "subtitle", "   Subtitle");
                            extract_value_info(section, "title", "   Title");
                            
                            // Metadata within sections
                            if let Some(metadata) = section.get("metadata").and_then(|m| m.as_array()) {
                                eprintln!("      📋 METADATA ({} items):", metadata.len());
                                for (k, meta_item) in metadata.iter().enumerate() {
                                    eprintln!("         Metadata #{}: {}", k + 1, serde_json::to_string_pretty(meta_item)?);
                                }
                            }
                            
                            // Beacons
                            if let Some(beacons) = section.get("beacons").and_then(|b| b.as_array()) {
                                eprintln!("      🚨 BEACONS ({} items):", beacons.len());
                                for (k, beacon) in beacons.iter().enumerate() {
                                    eprintln!("         Beacon #{}: {}", k + 1, serde_json::to_string_pretty(beacon)?);
                                }
                            }
                            
                            // Unknown section fields
                            eprintln!("      🔍 ALL SECTION FIELDS:");
                            if let Some(section_obj) = section.as_object() {
                                let known_section_fields = [
                                    "type", "metapages", "tabname", "text", "url", "youtubeurl", 
                                    "actions", "options", "footer", "header", "subtitle", "title",
                                    "metadata", "beacons"
                                ];
                                for (key, value) in section_obj {
                                    if !known_section_fields.contains(&key.as_str()) {
                                        eprintln!("         🆕 UNKNOWN SECTION FIELD {}: {}", key, value);
                                    }
                                }
                            }
                        }
                    }
                    
                    // URL links
                    if let Some(url) = track.get("url") {
                        eprintln!("\n🌐 TRACK URL: {}", url);
                    }
                    
                    // Additional track fields - enhanced search
                    extract_value_info(track, "genres", "Genres");
                    extract_value_info(track, "label", "Label");
                    extract_value_info(track, "copyright", "Copyright");
                    extract_value_info(track, "releasedate", "Release Date");
                    extract_value_info(track, "duration", "Duration");
                    extract_value_info(track, "albumname", "Album Name");
                    extract_value_info(track, "artistname", "Artist Name");
                    extract_value_info(track, "trackname", "Track Name");
                    extract_value_info(track, "explicit", "Explicit Content");
                    extract_value_info(track, "preview", "Preview");
                    extract_value_info(track, "popularity", "Popularity");
                    extract_value_info(track, "rank", "Rank");
                    extract_value_info(track, "year", "Year");
                    extract_value_info(track, "bpm", "BPM");
                    extract_value_info(track, "mood", "Mood");
                    extract_value_info(track, "energy", "Energy");
                    extract_value_info(track, "danceability", "Danceability");
                    extract_value_info(track, "acousticness", "Acousticness");
                    extract_value_info(track, "instrumentalness", "Instrumentalness");
                    extract_value_info(track, "liveness", "Liveness");
                    extract_value_info(track, "loudness", "Loudness");
                    extract_value_info(track, "speechiness", "Speechiness");
                    extract_value_info(track, "valence", "Valence");
                    extract_value_info(track, "tempo", "Tempo");
                    extract_value_info(track, "time_signature", "Time Signature");
                    extract_value_info(track, "key_signature", "Key Signature");
                    extract_value_info(track, "mode", "Mode");
                    extract_value_info(track, "camelot", "Camelot Key");
                    extract_value_info(track, "open_key", "Open Key");
                    extract_value_info(track, "created_at", "Created At");
                    extract_value_info(track, "updated_at", "Updated At");
                    extract_value_info(track, "language", "Language");
                    extract_value_info(track, "lyrics", "Lyrics");
                    extract_value_info(track, "credits", "Credits");
                    extract_value_info(track, "composer", "Composer");
                    extract_value_info(track, "producer", "Producer");
                    extract_value_info(track, "writer", "Writer");
                    extract_value_info(track, "publisher", "Publisher");
                    extract_value_info(track, "recordingdate", "Recording Date");
                    extract_value_info(track, "studio", "Studio");
                    extract_value_info(track, "originalyear", "Original Year");
                    extract_value_info(track, "remix", "Remix");
                    extract_value_info(track, "version", "Version");
                    extract_value_info(track, "featuring", "Featuring");
                    extract_value_info(track, "collaborations", "Collaborations");
                    extract_value_info(track, "samples", "Samples");
                    extract_value_info(track, "covers", "Covers");
                    extract_value_info(track, "tags", "Tags");
                    extract_value_info(track, "similar", "Similar Tracks");
                    extract_value_info(track, "recommendations", "Recommendations");
                    extract_value_info(track, "playlists", "Playlists");
                    extract_value_info(track, "charts", "Charts");
                    
                    // Any other fields in track - expanded exclusion list
                    eprintln!("\n🔍 ALL TRACK FIELDS:");
                    if let Some(obj) = track.as_object() {
                        let known_fields = [
                            "key", "title", "subtitle", "layout", "type", "isrc", "images", "share", "hub", "sections", "url",
                            "genres", "label", "copyright", "releasedate", "duration", "albumname", "artistname", "trackname",
                            "albumadamid", "artistadamid", "trackadamid", "myshazam", "explicit", "preview", "popularity",
                            "rank", "year", "bpm", "mood", "energy", "danceability", "acousticness", "instrumentalness",
                            "liveness", "loudness", "speechiness", "valence", "tempo", "time_signature", "key_signature",
                            "mode", "camelot", "open_key", "created_at", "updated_at", "language", "lyrics", "credits",
                            "composer", "producer", "writer", "publisher", "recordingdate", "studio", "originalyear",
                            "remix", "version", "featuring", "collaborations", "samples", "covers", "tags", "similar",
                            "recommendations", "playlists", "charts"
                        ];
                        
                        for (key, value) in obj {
                            if !known_fields.contains(&key.as_str()) {
                                eprintln!("   � UNKNOWN TRACK FIELD {}: {}", key, value);
                            }
                        }
                    }
                }
                
                // Any other fields in the match - enhanced
                eprintln!("\n🔍 ALL MATCH FIELDS:");
                if let Some(obj) = match_obj.as_object() {
                    let known_match_fields = ["id", "offset", "timeskew", "frequencyskew", "track"];
                    for (key, value) in obj {
                        if !known_match_fields.contains(&key.as_str()) {
                            eprintln!("   � UNKNOWN MATCH FIELD {}: {}", key, value);
                        }
                    }
                }
            }
        }
    }
    
    // Check for top-level track information (alternative response format)
    if let Some(_track) = response.get("track") {
        eprintln!("\n🎼 TOP-LEVEL TRACK INFORMATION:");
        // extract_track_information(track)?; // Removed undefined function call
    }
    
    // Top-level fields we haven't covered - enhanced analysis
    eprintln!("\n🔍 ALL TOP-LEVEL FIELDS:");
    if let Some(obj) = response.as_object() {
        let known_top_level_fields = [
            "tagid", "timestamp", "timezone", "retailer", "server", "uuid", "location", 
            "matches", "version", "track", "status", "error", "message"
        ];
        
        for (key, value) in obj {
            if !known_top_level_fields.contains(&key.as_str()) {
                eprintln!("   � UNKNOWN TOP-LEVEL FIELD {}: {}", key, value);
            }
        }
        
        // Additional checks for arrays or objects we may have missed
        eprintln!("\n🔍 COMPREHENSIVE FIELD TYPE ANALYSIS:");
        for (key, value) in obj {
            match value {
                Value::Array(arr) if !arr.is_empty() => {
                    eprintln!("   📋 Array field '{}' with {} items - first item: {}", 
                        key, arr.len(), 
                        serde_json::to_string_pretty(&arr[0]).unwrap_or_else(|_| "unparseable".to_string()));
                },
                Value::Object(obj) if !obj.is_empty() => {
                    eprintln!("   📦 Object field '{}' with keys: {:?}", key, obj.keys().collect::<Vec<_>>());
                },
                _ => {} // Already handled in known fields above
            }
        }
    }
    
    eprintln!("\n═══════════════════════════════════════");
    eprintln!("🏁 END OF COMPLETE RESPONSE ANALYSIS");
    
    Ok(())
}

fn extract_value_info(obj: &Value, key: &str, label: &str) {
    if let Some(value) = obj.get(key) {
        match value {
            Value::String(s) => eprintln!("   {} {}: {}", "🏷️", label, s),
            Value::Number(n) => eprintln!("   {} {}: {}", "🔢", label, n),
            Value::Bool(b) => eprintln!("   {} {}: {}", "✅", label, b),
            Value::Array(arr) => {
                eprintln!("   {} {} (array, {} items):", "📋", label, arr.len());
                for (i, item) in arr.iter().enumerate() {
                    eprintln!("      [{}]: {}", i, item);
                }
            },
            Value::Object(_) => eprintln!("   {} {} (object): {}", "📦", label, serde_json::to_string_pretty(value).unwrap_or_else(|_| "Failed to serialize".to_string())),
            Value::Null => eprintln!("   {} {}: null", "❌", label),
        }
    }
}

fn extract_images_info(images: &Value) {
    if let Some(obj) = images.as_object() {
        for (key, value) in obj {
            eprintln!("      🖼️  {} Image: {}", key, value);
        }
    } else if let Some(arr) = images.as_array() {
        for (i, image) in arr.iter().enumerate() {
            eprintln!("      🖼️  Image #{}: {}", i + 1, image);
        }
    } else {
        eprintln!("      🖼️  Image: {}", images);
    }
}

fn reqwest_client_native_tls() -> Result<reqwest::blocking::Client, Box<dyn Error>> {
    //eprintln!("Creating Windows-compatible client...");
    let builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("SongRec/0.4.3")
        .danger_accept_invalid_certs(false)
        .tcp_keepalive(Duration::from_secs(60))
        .pool_idle_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10);

    Ok(builder.build()?)
}

fn reqwest_client_basic() -> Result<reqwest::blocking::Client, Box<dyn Error>> {
    eprintln!("Creating basic client...");
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("SongRec/0.4.3")
        .build()?)
}

fn reqwest_client_legacy() -> Result<reqwest::blocking::Client, Box<dyn Error>> {
    eprintln!("Creating simple client...");
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?)
}


