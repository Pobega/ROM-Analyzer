use std::fs::{self, File};
use std::path::Path;

use clap::{ArgAction, Parser};
use env_logger;
use log::{LevelFilter, error, info, warn};

use rom_analyzer::RomAnalysisResult;
use rom_analyzer::archive::chd::{ChdAnalysis, analyze_chd_file};
use rom_analyzer::archive::zip::process_zip_file;
use rom_analyzer::dispatcher::process_rom_data;
use rom_analyzer::region::{check_region_mismatch, infer_region_from_filename};

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

fn get_file_extension_lowercase(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase()
}

fn process_single_file(
    file_path: &str,
    path: &Path,
    file_name: &str,
) -> Result<RomAnalysisResult, Box<dyn std::error::Error>> {
    match get_file_extension_lowercase(file_path).as_str() {
        "zip" => {
            let file = File::open(path)?;
            process_zip_file(file, file_name, &process_rom_data)
        }
        "chd" => {
            let analysis_result = analyze_chd_file(path, file_name)?;
            match analysis_result {
                ChdAnalysis::SegaCD(analysis) => Ok(RomAnalysisResult::SegaCD(analysis)),
                ChdAnalysis::PSX(analysis) => Ok(RomAnalysisResult::PSX(analysis)),
            }
        }
        _ => {
            let data = fs::read(path)?;
            process_rom_data(data, file_name)
        }
    }
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

    for file_path in &cli.file_paths {
        let path = Path::new(file_path);

        if !path.exists() {
            error!("File not found: {}", file_path);
            had_error = true;
            continue;
        }

        let file_name = if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            name
        } else {
            error!("Could not get a valid UTF-8 filename for: {}", file_path);
            had_error = true;
            continue;
        };

        let result = process_single_file(file_path, path, file_name);
        match result {
            Ok(analysis) => {
                let printable: String = if cli.json {
                    analysis.json()
                } else {
                    analysis.print()
                };
                info!("{}", printable);
                if check_region_mismatch(analysis.source_name(), analysis.region()) {
                    let inferred_region =
                        infer_region_from_filename(analysis.source_name()).unwrap_or("Unknown");
                    warn!(
                        "~~~ POSSIBLE REGION MISMATCH ~~~\n\
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
            Err(e) => {
                error!("Error processing file {}: {}", file_path, e);
                had_error = true;
            }
        }
    }

    if had_error {
        std::process::exit(1);
    }
}
