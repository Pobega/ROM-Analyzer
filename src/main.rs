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

    /// Number of threads to use for parallel processing (0 or omitted uses all available threads)
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
/// Each result is an analysis on success, or a RomAnalyzerError on failure.
/// Results are returned in the same order as the input file paths.
fn process_files_parallel(
    file_paths: &[String],
) -> Vec<Result<RomAnalysisResult, RomAnalyzerError>> {
    file_paths
        .par_iter()
        .map(|file_path| match analyze_rom_data(file_path) {
            Ok(analysis) => Ok(analysis),
            Err(e) => {
                // Convert NotFound IO errors to FileNotFound (no wrapping needed, path is included)
                // Wrap other errors with WithPath for context
                let err = match e {
                    RomAnalyzerError::IoError(io_err)
                        if io_err.kind() == std::io::ErrorKind::NotFound =>
                    {
                        RomAnalyzerError::FileNotFound(file_path.clone())
                    }
                    other => RomAnalyzerError::WithPath(file_path.clone(), Box::new(other)),
                };
                Err(err)
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
        match &results[0] {
            Err(RomAnalyzerError::FileNotFound(path)) => {
                assert_eq!(path, "non_existent_file.nes");
            }
            _ => panic!("Expected FileNotFound error, but got {:?}", results[0]),
        }
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
        let file_paths = vec![file_path_str.clone()];
        let results = process_files_parallel(&file_paths);
        assert_eq!(results.len(), 1);
        match &results[0] {
            Ok(analysis) => {
                assert_eq!(analysis.source_name(), &file_path_str);
                assert_eq!(analysis.source_name(), &file_path_str);
            }
            Err(e) => panic!("Expected Ok, but got error: {:?}", e),
        }
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
        let file_paths = vec![
            valid_file.to_str().unwrap().to_string(),
            "invalid.nes".to_string(),
        ];
        let results = process_files_parallel(&file_paths);
        let ok_count = results.iter().filter(|r| r.is_ok()).count();
        let err_count = results.iter().filter(|r| r.is_err()).count();
        assert_eq!(results.len(), 2);
        assert_eq!(ok_count, 1);
        assert_eq!(err_count, 1);
    }

    #[test]
    fn test_process_files_parallel_empty_input() {
        let results = process_files_parallel(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_process_files_parallel_order_preserved() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("a.nes");
        let file2 = dir.path().join("b.nes");
        let file3 = dir.path().join("c.nes");

        let nes_header = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        fs::write(&file1, nes_header).unwrap();
        fs::write(&file2, nes_header).unwrap();
        fs::write(&file3, nes_header).unwrap();

        let file_paths = vec![
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
        ];
        let results = process_files_parallel(&file_paths);

        assert_eq!(results.len(), 3);
        for (i, result) in results.iter().enumerate() {
            match result {
                Ok(analysis) => assert_eq!(analysis.source_name(), &file_paths[i]),
                Err(e) => panic!("Expected Ok, but got error: {:?}", e),
            }
        }
    }

    #[test]
    fn test_process_files_parallel_other_errors_wrapped() {
        // Test that non-NotFound errors get wrapped with WithPath
        let dir = tempdir().unwrap();
        let invalid_file = dir.path().join("invalid.nes");
        fs::write(&invalid_file, b"not a valid NES file").unwrap();

        let file_paths = vec![invalid_file.to_str().unwrap().to_string()];
        let results = process_files_parallel(&file_paths);

        assert_eq!(results.len(), 1);
        match &results[0] {
            Err(RomAnalyzerError::WithPath(path, _)) => {
                assert_eq!(path, invalid_file.to_str().unwrap());
            }
            other => panic!("Expected WithPath error, but got {:?}", other),
        }
    }
}
