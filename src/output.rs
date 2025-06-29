use serde::{Deserialize, Serialize};
use crate::songrec::RecognitionResult;

/// Output format for recognition results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Simple song name format: "Artist - Song"
    Simple,
    /// Full JSON with all metadata
    Json,
    /// CSV format for logging
    Csv,
    /// Custom format with placeholders
    Custom(&'static str),
}

/// Formatted recognition output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionOutput {
    pub format: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RecognitionOutput {
    /// Format a recognition result according to the specified format
    pub fn format_result(result: &RecognitionResult, format: OutputFormat) -> Self {
        let content = match format {
            OutputFormat::Simple => {
                format!("{} - {}", result.artist_name, result.song_name)
            },
            OutputFormat::Json => {
                serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()) // Avoid verbose error messages
            },
            OutputFormat::Csv => {
                format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                    result.song_name,
                    result.artist_name,
                    result.album_name.as_deref().unwrap_or(""),
                    result.release_year.as_deref().unwrap_or(""),
                    result.genre.as_deref().unwrap_or(""),
                    result.recognition_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                )
            },
            OutputFormat::Custom(template) => {
                Self::format_custom(result, template)
            },
        };

        RecognitionOutput {
            format: format.to_string(),
            content,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Format using a custom template with placeholders
    fn format_custom(result: &RecognitionResult, template: &str) -> String {
        template
            .replace("{song}", &result.song_name)
            .replace("{artist}", &result.artist_name)
            .replace("{album}", result.album_name.as_deref().unwrap_or("Unknown"))
            .replace("{year}", result.release_year.as_deref().unwrap_or("Unknown"))
            .replace("{genre}", result.genre.as_deref().unwrap_or("Unknown"))
            .replace("{timestamp}", &result.recognition_timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string())
    }

    /// Get CSV header
    pub fn csv_header() -> &'static str {
        "\"Song\",\"Artist\",\"Album\",\"Year\",\"Genre\",\"Timestamp\""
    }
}

impl std::fmt::Display for RecognitionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Simple => write!(f, "Simple"),
            OutputFormat::Json => write!(f, "Json"),
            OutputFormat::Csv => write!(f, "Csv"),
            OutputFormat::Custom(template) => write!(f, "Custom({})", template),
        }
    }
}
