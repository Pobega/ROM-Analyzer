use std::path::Path;

use clap::{ArgAction, Parser};
use log::{LevelFilter, error, info, warn};
use rayon::prelude::*;
use walkdir::WalkDir;

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

    /// Recursively process directories for ROM files
    #[clap(short, long, action = ArgAction::SetTrue)]
    recursive: bool,
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

/// Recursively expands directory paths into a list of file paths.
/// If recursive is false, directories are skipped with a warning.
/// Uses walkdir to handle edge cases like circular symbolic links gracefully.
fn expand_paths(paths: &[String], recursive: bool) -> Vec<String> {
    let mut found_files = std::collections::BTreeSet::new();
    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_dir() {
            if recursive {
                for node_result in WalkDir::new(path) {
                    match node_result {
                        Ok(entry) => {
                            if entry.file_type().is_file()
                                && let Some(entry_path_str) = entry.path().to_str()
                            {
                                found_files.insert(entry_path_str.to_string());
                            }
                        }
                        Err(e) => warn!("Error walking directory: {}", e),
                    }
                }
            } else {
                warn!(
                    "Skipping directory {} (use -r for recursion)",
                    path.display()
                );
            }
        } else {
            found_files.insert(path_str.clone());
        }
    }
    found_files.into_iter().collect()
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
                // Convert NotFound IO errors to FileNotFound (no wrapping needed, path is included,)
                // Wrap other errors with WithPath for context.
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

    let expanded_file_paths = expand_paths(&cli.file_paths, cli.recursive);
    let results = process_files_parallel(&expanded_file_paths);

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

    const TEST_NES_HEADER: &[u8] =
        b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

    #[test]
    fn test_get_log_level_quiet() {
        // Tests that quiet mode sets log level to Error regardless of verbosity.
        assert_eq!(get_log_level(true, 0), LevelFilter::Error);
        assert_eq!(get_log_level(true, 1), LevelFilter::Error);
    }

    #[test]
    fn test_get_log_level_verbose_levels() {
        // Tests that verbosity levels set appropriate log levels when not quiet.
        assert_eq!(get_log_level(false, 0), LevelFilter::Info);
        assert_eq!(get_log_level(false, 1), LevelFilter::Debug);
        assert_eq!(get_log_level(false, 2), LevelFilter::Trace);
        assert_eq!(get_log_level(false, 10), LevelFilter::Trace);
    }

    #[test]
    fn test_process_files_parallel_non_existent_file() {
        // Tests processing a non-existent file returns a FileNotFound error.
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
        // Tests processing a valid NES file succeeds and returns correct source name.

        // Create a temporary directory and file.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.nes");
        fs::write(&file_path, TEST_NES_HEADER).unwrap(); // Minimal NES header
        let file_path_str = file_path.to_str().unwrap().to_string();
        let file_paths = vec![file_path_str.clone()];

        let results = process_files_parallel(&file_paths);
        assert_eq!(results.len(), 1);
        match &results[0] {
            Ok(analysis) => assert_eq!(analysis.source_name(), &file_path_str),
            Err(e) => panic!("Expected Ok, but got error: {:?}", e),
        }
    }

    #[test]
    fn test_process_files_parallel_mixed_files() {
        // Tests processing a mix of valid and invalid files returns appropriate results.

        // Create a temporary directory with a valid NES ROM file.
        let dir = tempdir().unwrap();
        let valid_file = dir.path().join("valid.nes");
        fs::write(&valid_file, TEST_NES_HEADER).unwrap();
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
        // Tests processing an empty list of files returns an empty results list.
        let results = process_files_parallel(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_process_files_parallel_order_preserved() {
        // Tests that processing multiple files preserves the order of results.

        // Create three temporary NES files.
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("a.nes");
        let file2 = dir.path().join("b.nes");
        let file3 = dir.path().join("c.nes");

        // Write minimal NES headers to each.
        fs::write(&file1, TEST_NES_HEADER).unwrap();
        fs::write(&file2, TEST_NES_HEADER).unwrap();
        fs::write(&file3, TEST_NES_HEADER).unwrap();

        // Collect their paths into a vector.
        let file_paths = vec![
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
        ];
        // Process the files in parallel.
        let results = process_files_parallel(&file_paths);

        // Assert the results are in the correct order.
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
        // Tests that non-NotFound errors are wrapped with WithPath for context.

        // Create a temporary directory and invalid file.
        let dir = tempdir().unwrap();
        let invalid_file = dir.path().join("invalid.nes");
        fs::write(&invalid_file, b"not a valid NES file").unwrap();

        let file_paths = vec![invalid_file.to_str().unwrap().to_string()];

        // Process the file, expecting a RomAnalyzerError::WithPath.
        let results = process_files_parallel(&file_paths);

        assert_eq!(results.len(), 1);
        match &results[0] {
            Err(RomAnalyzerError::WithPath(path, _)) => {
                assert_eq!(path, invalid_file.to_str().unwrap());
            }
            other => panic!("Expected WithPath error, but got {:?}", other),
        }
    }

    #[test]
    fn test_expand_paths_non_recursive_skips_dirs() {
        // Tests that non-recursive mode skips directories without expanding them.

        // Create a temporary directory with a file inside it.
        let dir = tempdir().unwrap();
        let file_in_dir = dir.path().join("file.nes");
        fs::write(&file_in_dir, TEST_NES_HEADER).unwrap();
        let paths = vec![dir.path().to_str().unwrap().to_string()];

        // Expand paths non-recursively.
        let expanded = expand_paths(&paths, false);
        assert!(expanded.is_empty()); // Directory skipped
    }

    #[test]
    fn test_expand_paths_recursive_expands_dirs() {
        // Tests that recursive mode expands directories to include files within.

        // Create a temporary directory with a file inside it.
        let dir = tempdir().unwrap();
        let file_in_dir = dir.path().join("file.nes");
        fs::write(&file_in_dir, TEST_NES_HEADER).unwrap();
        let paths = vec![dir.path().to_str().unwrap().to_string()];

        // Expand paths recursively.
        let expanded = expand_paths(&paths, true);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], file_in_dir.to_str().unwrap());
    }

    #[test]
    fn test_expand_paths_nested_dirs() {
        // Tests that nested directories are handled recursively.

        // Create a temporary root directory and subdirectory.
        let root_dir = tempdir().unwrap();
        let sub_dir = root_dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();

        // Create a file in the nested subdirectory.
        let file_in_subdir = sub_dir.join("nested.nes");
        fs::write(&file_in_subdir, TEST_NES_HEADER).unwrap();

        // Expand paths recursively.
        let paths = vec![root_dir.path().to_str().unwrap().to_string()];
        let expanded = expand_paths(&paths, true);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], file_in_subdir.to_str().unwrap());
    }

    #[test]
    fn test_expand_paths_mixed_files_and_dirs() {
        // Tests handling a mix of files and directories in input paths.

        // Create temporary directories and some files.
        let dir = tempdir().unwrap();
        let file_in_dir = dir.path().join("dir_file.nes");
        fs::write(&file_in_dir, TEST_NES_HEADER).unwrap();
        let other_dir = tempdir().unwrap();
        let standalone_file = other_dir.path().join("standalone.nes");
        fs::write(&standalone_file, TEST_NES_HEADER).unwrap();
        let paths = vec![
            dir.path().to_str().unwrap().to_string(),
            standalone_file.to_str().unwrap().to_string(),
        ];

        // Expand paths recursively.
        let expanded = expand_paths(&paths, true);
        assert_eq!(expanded.len(), 2);
        assert!(expanded.contains(&file_in_dir.to_str().unwrap().to_string()));
        assert!(expanded.contains(&standalone_file.to_str().unwrap().to_string()));
    }

    #[test]
    fn test_expand_paths_empty_dir() {
        // Tests that empty directories are handled without including any files.
        let dir = tempdir().unwrap();
        let paths = vec![dir.path().to_str().unwrap().to_string()];
        let expanded = expand_paths(&paths, true);
        assert!(expanded.is_empty());
    }

    #[test]
    fn test_expand_paths_deduplicates() {
        // Tests that duplicate file paths are deduplicated in the output.

        // Create temporary directory and some files.
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.nes");
        let file2 = dir.path().join("file2.nes");
        fs::write(&file1, TEST_NES_HEADER).unwrap();
        fs::write(&file2, TEST_NES_HEADER).unwrap();
        let file1_str = file1.to_str().unwrap().to_string();
        let file2_str = file2.to_str().unwrap().to_string();
        let paths = vec![file1_str.clone(), file2_str.clone(), file1_str.clone()];

        // Expand paths non-recursively.
        let expanded = expand_paths(&paths, false);
        assert_eq!(expanded.len(), 2);
        assert!(expanded.contains(&file1_str));
        assert!(expanded.contains(&file2_str));
    }

    #[test]
    fn test_expand_paths_empty_input() {
        // Tests that empty input paths result in empty output.
        let expanded = expand_paths(&[], true);
        assert!(expanded.is_empty());
        let expanded_non_recursive = expand_paths(&[], false);
        assert!(expanded_non_recursive.is_empty());
    }

    #[test]
    fn test_expand_paths_deeply_nested() {
        // Tests handling deeply nested directory structures.

        // Create deeply nested directory structure.
        let root = tempdir().unwrap();
        let level1 = root.path().join("a");
        let level2 = level1.join("b");
        let level3 = level2.join("c");
        fs::create_dir_all(&level3).unwrap();
        let deep_file = level3.join("deep.nes");
        fs::write(&deep_file, TEST_NES_HEADER).unwrap();
        let paths = vec![root.path().to_str().unwrap().to_string()];

        // Expand paths recursively.
        let expanded = expand_paths(&paths, true);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], deep_file.to_str().unwrap());
    }

    #[test]
    fn test_expand_paths_nonexistent_file() {
        // Tests that non-existent file paths are passed through unchanged.
        let paths = vec!["nonexistent_file.nes".to_string()];
        let expanded = expand_paths(&paths, true);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], "nonexistent_file.nes");
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_paths_follows_symlinks() {
        // Tests that symlinks to files are followed and included.
        use std::os::unix::fs::symlink;

        // Create temporary directory and target file.
        let dir = tempdir().unwrap();
        let target_file = dir.path().join("target.nes");
        fs::write(&target_file, TEST_NES_HEADER).unwrap();
        let symlink_file = dir.path().join("link.nes");
        symlink(&target_file, &symlink_file).unwrap();
        let paths = vec![symlink_file.to_str().unwrap().to_string()];

        // Expand paths non-recursively and ensure that symlink is included.
        let expanded = expand_paths(&paths, false);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], symlink_file.to_str().unwrap());
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_paths_symlink_to_directory() {
        // Tests that symlinks to directories are followed recursively.
        use std::os::unix::fs::symlink;

        // Create temporary directory and target directory with file.
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        let file_in_target = target_dir.join("file.nes");
        fs::write(&file_in_target, TEST_NES_HEADER).unwrap();

        // Create symlink to the temporary directory.
        let symlink_dir = dir.path().join("link");
        symlink(&target_dir, &symlink_dir).unwrap();

        // Run expand_paths on the symlink pointing at our tempdir.
        let paths = vec![symlink_dir.to_str().unwrap().to_string()];
        let expanded = expand_paths(&paths, true);
        assert_eq!(expanded.len(), 1);

        // The expanded path should be through the symlink.
        assert!(expanded[0].contains("link"));
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_paths_unreadable_dir() {
        // Tests graceful handling of unreadable directories with warnings.
        use std::os::unix::fs::PermissionsExt;

        // Create temporary directory and unreadable subdirectory.
        let root = tempdir().unwrap();
        let unreadable_dir = root.path().join("unreadable");
        fs::create_dir(&unreadable_dir).unwrap();
        let file_in_unreadable = unreadable_dir.join("file.nes");
        fs::write(&file_in_unreadable, TEST_NES_HEADER).unwrap();

        // Remove read permissions.
        let mut perms = fs::metadata(&unreadable_dir).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&unreadable_dir, perms).unwrap();

        let paths = vec![root.path().to_str().unwrap().to_string()];
        // Expand paths recursively.
        let expanded = expand_paths(&paths, true);

        // Restore permissions for cleanup.
        let mut perms = fs::metadata(&unreadable_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&unreadable_dir, perms).unwrap();

        // Should not include files from unreadable directory.
        assert!(expanded.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_paths_circular_symlink() {
        // Tests handling of circular symbolic links without infinite loops.
        use std::os::unix::fs::symlink;
        let root = tempdir().unwrap();

        // Create a file in the root directory.
        let file_in_root = root.path().join("file.nes");
        fs::write(&file_in_root, TEST_NES_HEADER).unwrap();

        // Create a subdirectory.
        let subdir = root.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Create a symlink in the subdirectory that points back to the root.
        let circular_link = subdir.join("circular");
        symlink(root.path(), &circular_link).unwrap();

        let paths = vec![root.path().to_str().unwrap().to_string()];
        // This should complete without stack overflow or infinite loop.
        let expanded = expand_paths(&paths, true);

        // Verify that file.nes was found.
        assert!(!expanded.is_empty());
        assert!(expanded.iter().any(|p| p.ends_with("file.nes")));
    }
}
