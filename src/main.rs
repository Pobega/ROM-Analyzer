use clap::Parser;
use std::fs::{self, File};
use std::path::Path;

use rom_analyzer::archive::zip::process_zip_file;
use rom_analyzer::dispatcher::process_rom_data;
use rom_analyzer::print_separator;

use rom_analyzer::archive::chd::analyze_chd_file;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The path to the ROM file or zip archive
    #[clap(value_parser, num_args = 1..)]
    file_paths: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    print_separator();
    println!("ROM Analyzer CLI");
    print_separator();

    for file_path in &cli.file_paths {
        let path = Path::new(file_path);

        if !path.exists() {
            eprintln!("File not found: {}", file_path);
            continue;
        }

        let file_name = if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            name
        } else {
            eprintln!("Could not get a valid UTF-8 filename for: {}", file_path);
            continue;
        };

        let result = (|| {
            if file_path.to_lowercase().ends_with(".zip") {
                let file = File::open(path)?;
                process_zip_file(file, file_name, &process_rom_data)
            } else if file_path.to_lowercase().ends_with(".chd") {
                analyze_chd_file(path, file_name)
            } else {
                let data = fs::read(path)?;
                process_rom_data(data, file_name)
            }
        })();
        if let Err(e) = result {
            eprintln!("Error processing file {}: {}", file_path, e);
        }
    }

    Ok(())
}
