use std::path::Path;

use clap::{ArgAction, Parser};
use log::{LevelFilter, error, info, warn};
use rayon::prelude::*;

use rom_analyzer::error::RomAnalyzerError;
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

    /// Number of threads to use for parallel processing
    #[clap(long, value_name = "N")]
    threads: Option<usize>,
}

fn get_log_level(quiet: bool, verbose: u8) -> LevelFilter {
    if quiet {
        LevelFilter::Error // Only show errors if --quiet is passed.
    } else {
        match verbose {
            0 => LevelFilter::Info,  // (no -v): Show Info messages
            1 => LevelFilter::Debug, // -v: Show Debug messages
            _ => LevelFilter::Trace, // -vv or more: Show everything (Trace)
        }
    }
}

/// Processes a list of file paths in parallel, returning a vector of results.
/// Each result is an analysis on success (with the file path), or a RomAnalyzerError on failure.
fn process_files_parallel(
    file_paths: &[String],
) -> Vec<Result<(String, RomAnalysisResult), RomAnalyzerError>> {
    file_paths
        .par_iter()
        .map(|file_path| {
            let path = Path::new(file_path);

            if !path.exists() {
                return Err(RomAnalyzerError::WithPath(
                    file_path.clone(),
                    Box::new(RomAnalyzerError::FileNotFound(file_path.clone())),
                ));
            }

            match analyze_rom_data(file_path) {
                Ok(analysis) => Ok((file_path.clone(), analysis)),
                Err(e) => Err(RomAnalyzerError::WithPath(file_path.clone(), Box::new(e))),
            }
        })
        .collect()
}

fn main() {
    let cli = Cli::parse();

    if let Some(num_threads) = cli.threads
        && num_threads != 0
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap_or_else(|e| {
                eprintln!("Failed to set thread pool: {}", e);
                std::process::exit(1);
            });
    }

    let default_log_level = get_log_level(cli.quiet, cli.verbose);

    env_logger::Builder::new()
        .filter_level(default_log_level)
        .format_timestamp(None)
        .format_module_path(false)
        .format_level(false)
        .format_target(false)
        .init();

    let mut had_error = false;

    let mut json_results: Vec<RomAnalysisResult> = Vec::new();

    let results = process_files_parallel(&cli.file_paths);

    for result in results {
        match result {
            Ok((_file_path, analysis)) => {
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
                error!("{}", e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_log_level_quiet() {
        assert_eq!(get_log_level(true, 0), LevelFilter::Error);
        assert_eq!(get_log_level(true, 1), LevelFilter::Error);
    }

    #[test]
    fn test_get_log_level_verbose_levels() {
        assert_eq!(get_log_level(false, 0), LevelFilter::Info);
        assert_eq!(get_log_level(false, 1), LevelFilter::Debug);
        assert_eq!(get_log_level(false, 2), LevelFilter::Trace);
        assert_eq!(get_log_level(false, 10), LevelFilter::Trace);
    }

    #[test]
    fn test_process_files_parallel_non_existent_file() {
        let non_existent = ["non_existent_file.nes".to_string()];
        let results = process_files_parallel(&non_existent);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(results[0].as_ref().unwrap_err().to_string().contains(
            "Error processing file non_existent_file.nes: File not found: non_existent_file.nes"
        ));
    }

    #[test]
    fn test_process_files_parallel_valid_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.nes");
        fs::write(
            &file_path,
            b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        )
        .unwrap(); // Minimal NES header
        let file_path_str = file_path.to_str().unwrap().to_string();
        let results = process_files_parallel(std::slice::from_ref(&file_path_str));
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        let (path, analysis) = results[0].as_ref().unwrap();
        assert_eq!(path, &file_path_str);
        assert_eq!(analysis.source_name(), &file_path_str);
    }

    #[test]
    fn test_process_files_parallel_mixed_files() {
        let dir = tempdir().unwrap();
        let valid_file = dir.path().join("valid.nes");
        fs::write(
            &valid_file,
            b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        )
        .unwrap();
        let valid_path = valid_file.to_str().unwrap().to_string();
        let invalid_path = "invalid.nes".to_string();
        let file_paths = vec![valid_path.clone(), invalid_path];
        let results = process_files_parallel(&file_paths);
        let ok_count = results.iter().filter(|r| r.is_ok()).count();
        let err_count = results.iter().filter(|r| r.is_err()).count();
        assert_eq!(results.len(), 2);
        assert_eq!(ok_count, 1);
        assert_eq!(err_count, 1);
    }
}
