use clap::{App, Arg, SubCommand};
use songrec::{SongRec, Config, OutputFormat, RecognitionOutput};
use std::process;

fn main() {
    let matches = App::new("SongRec CLI")
        .version("0.4.3")
        .about("An open-source Shazam client library and CLI")
        .subcommand(
            SubCommand::with_name("recognize")
                .about("Recognize a song from an audio file")
                .arg(
                    Arg::with_name("input")
                        .required(true)
                        .help("Input audio file path")
                        .index(1)
                )
                .arg(
                    Arg::with_name("format")
                        .short("f")
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format: simple, json, csv")
                        .takes_value(true)
                        .default_value("simple")
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .help("Suppress verbose debug output (default)")
                )
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        .help("Enable verbose debug output")
                )
        )
        .subcommand(
            SubCommand::with_name("listen")
                .about("Listen continuously for songs")
                .arg(
                    Arg::with_name("device")
                        .short("d")
                        .long("device")
                        .value_name("DEVICE")
                        .help("Audio input device name")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("format")
                        .short("f")
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format: simple, json, csv")
                        .takes_value(true)
                        .default_value("simple")
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .help("Suppress verbose debug output (default)")
                )
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        .help("Enable verbose debug output")
                )
                .arg(
                    Arg::with_name("no-dedupe")
                        .long("no-dedupe")
                        .help("Disable request deduplication")
                )
        )
        .subcommand(
            SubCommand::with_name("devices")
                .about("List available audio input devices")
        )
        .get_matches();

    match matches.subcommand() {
        ("recognize", Some(sub_matches)) => {
            let input_file = sub_matches.value_of("input").unwrap();
            let format_str = sub_matches.value_of("format").unwrap();
            let verbose = sub_matches.is_present("verbose");
            
            let format = match format_str {
                "json" => OutputFormat::Json,
                "csv" => OutputFormat::Csv,
                _ => OutputFormat::Simple,
            };

            let config = Config::default()
                .with_quiet_mode(!verbose); // Invert: verbose mode disables quiet
            let songrec = SongRec::new(config);

            match songrec.recognize_from_file(input_file) {
                Ok(result) => {
                    let output = RecognitionOutput::format_result(&result, format);
                    println!("{}", output);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        ("listen", Some(sub_matches)) => {
            let device = sub_matches.value_of("device").map(|s| s.to_string());
            let format_str = sub_matches.value_of("format").unwrap();
            let verbose = sub_matches.is_present("verbose");
            let no_dedupe = sub_matches.is_present("no-dedupe");
            
            let format = match format_str {
                "json" => OutputFormat::Json,
                "csv" => OutputFormat::Csv,
                _ => OutputFormat::Simple,
            };

            let config = Config::default()
                .with_quiet_mode(!verbose) // Invert: verbose mode disables quiet
                .with_deduplication(!no_dedupe);
            let songrec = SongRec::new(config);

            if verbose {
                println!("Starting continuous recognition...");
            }
            if format == OutputFormat::Csv {
                println!("{}", RecognitionOutput::csv_header());
            }

            match songrec.start_continuous_recognition_with_device(device) {
                Ok(stream) => {
                    for result in stream {
                        match result {
                            Ok(recognition) => {
                                let output = RecognitionOutput::format_result(&recognition, format);
                                println!("{}", output);
                            }
                            Err(e) => {
                                if verbose {
                                    eprintln!("Recognition error: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("Error starting recognition: {}", e);
                    }
                    process::exit(1);
                }
            }
        }
        ("devices", Some(_)) => {
            match songrec::audio::AudioRecorder::list_input_devices() {
                Ok(devices) => {
                    println!("Available audio input devices:");
                    for (i, device) in devices.iter().enumerate() {
                        println!("  {}: {}", i, device);
                    }
                }
                Err(e) => {
                    eprintln!("Error listing devices: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            // No output in quiet mode for unknown subcommands
        }
    }
}
