use clap::Parser;
use std::fs::{self, File};
use std::path::Path;

use rom_analyzer::archive::zip::process_zip_file;
use rom_analyzer::RomAnalysisResult;
use rom_analyzer::dispatcher::process_rom_data;
use rom_analyzer::print_separator;

use rom_analyzer::archive::chd::ChdAnalysis;
use rom_analyzer::archive::chd::analyze_chd_file;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The path to the ROM file or zip archive
    #[clap(value_parser, num_args = 1..)]
    file_paths: Vec<String>,
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

    print_separator();
    println!("ROM Analyzer CLI");
    print_separator();

    let mut had_error = false;

    for file_path in &cli.file_paths {
        let path = Path::new(file_path);

        if !path.exists() {
            eprintln!("File not found: {}", file_path);
            had_error = true;
            continue;
        }

        let file_name = if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            name
        } else {
            eprintln!("Could not get a valid UTF-8 filename for: {}", file_path);
            had_error = true;
            continue;
        };

        let result = process_single_file(file_path, path, file_name);
        match result {
            Ok(analysis) => analysis.print(),
            Err(e) => {
                eprintln!("Error processing file {}: {}", file_path, e);
                had_error = true;
            }
        }
    }

    if had_error {
        std::process::exit(1);
    }
}
