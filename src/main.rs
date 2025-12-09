use std::path::Path;

use clap::{ArgAction, Parser};
use env_logger;
use log::{LevelFilter, error, info, warn};

use rom_analyzer::region::infer_region_from_filename;
use rom_analyzer::{RomAnalysisResult, analyze_rom_data};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Full path(s) to a ROM file(s)
    #[clap(value_parser, num_args = 1..)]
    file_paths: Vec<String>,

    /// Verbosity level (-vv for most verbose)
    #[clap(short, action = ArgAction::Count)]
    verbose: u8,

    /// Silence all output except errors
    #[clap(short, long, action = ArgAction::SetTrue)]
    quiet: bool,

    /// Format output as JSON (suppresses everything except STDERR)
    #[clap(short, long, action = ArgAction::SetTrue)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let default_log_level = if cli.quiet {
        LevelFilter::Error // Only show errors if --quiet is passed.
    } else {
        match cli.verbose {
            0 => LevelFilter::Info,  // (no -v): Show Info messages
            1 => LevelFilter::Debug, // -v: Show Debug messages
            _ => LevelFilter::Trace, // -vv or more: Show everything (Trace)
        }
    };

    env_logger::Builder::new()
        .filter_level(default_log_level)
        .format_timestamp(None)
        .format_module_path(false)
        .format_level(false)
        .format_target(false)
        .init();

    let mut had_error = false;

    let mut json_results: Vec<RomAnalysisResult> = Vec::new();

    for file_path in &cli.file_paths {
        let path = Path::new(file_path);

        if !path.exists() {
            error!("File not found: {}", file_path);
            had_error = true;
            continue;
        }

        let result = analyze_rom_data(file_path);
        match result {
            Ok(analysis) => {
                if cli.json {
                    json_results.push(analysis);
                } else {
                    info!("{}", analysis.print());
                    if analysis.region_mismatch() {
                        let inferred_region = infer_region_from_filename(analysis.source_name());
                        warn!(
                            "POSSIBLE REGION MISMATCH\n\
                             Source file:          {}\n\
                             Filename suggests:    {}\n\
                             ROM Header claims:    {}\n\
                             The ROM may be mislabeled or have been patched.",
                            analysis.source_name(),
                            inferred_region,
                            analysis.region(),
                        );
                    }
                }
            }
            Err(e) => {
                error!("Error processing file {}: {}", file_path, e);
                had_error = true;
            }
        }
    }

    if cli.json {
        match serde_json::to_string_pretty(&json_results) {
            Ok(json_output) => {
                println!("{}", json_output);
            }
            Err(e) => {
                eprintln!("Error serializing combined JSON output: {}", e);
                had_error = true;
            }
        }
    }

    if had_error {
        std::process::exit(1);
    }
}
