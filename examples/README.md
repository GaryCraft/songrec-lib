# Examples

## Available Examples

### Basic Library Usage
```bash
cargo run --example library_usage
```
Demonstrates file recognition, output formats, configuration options, device listing, and integration patterns.

### Device Management  
```bash
cargo run --example device_usage
```
Shows audio device discovery, selection strategies, live recognition setup, and error handling patterns.

## Key APIs Demonstrated

- **File Recognition**: `songrec.recognize_from_file()`
- **Device Listing**: `AudioRecorder::list_input_devices()`
- **Live Recognition**: `songrec.start_continuous_recognition_with_device()`
- **Output Formats**: JSON, CSV, Simple text
- **Configuration**: Sensitivity, timeouts, quiet mode
