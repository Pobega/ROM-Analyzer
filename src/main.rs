use clap::Parser;
use std::fs::{self, File};
use std::path::Path;

use rom_analyzer::archive::zip::process_zip_file;
use rom_analyzer::dispatcher::process_rom_data;
use rom_analyzer::error::RomAnalyzerError;
use rom_analyzer::print_separator;

use rom_analyzer::archive::chd::analyze_chd_file;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The path to the ROM file or zip archive
    #[clap(value_parser)]
    file_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let file_path = &cli.file_path;
    let path = Path::new(file_path);

    print_separator();
    println!("ROM Analyzer CLI");
    print_separator();

    if !path.exists() {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "File not found: {}",
            file_path
        ))));
    }

    let file_name = path.file_name().unwrap().to_str().unwrap();

    if file_path.to_lowercase().ends_with(".zip") {
        let file = File::open(path)?;
        process_zip_file(file, file_name, &process_rom_data)?;
    } else if file_path.to_lowercase().ends_with(".chd") {
        analyze_chd_file(path, file_name)?;
    } else {
        let data = fs::read(path)?;
        process_rom_data(data, file_name)?;
    }

    Ok(())
}
