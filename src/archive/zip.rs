use std::error::Error;
use std::io::Read;
use std::fs::File;
use zip::ZipArchive;

use crate::error::RomAnalyzerError;

use crate::SUPPORTED_ROM_EXTENSIONS;

pub fn process_zip_file(file: File, original_filename: &str, process_rom_data_fn: &dyn Fn(Vec<u8>, &str) -> Result<(), Box<dyn Error>>) -> Result<(), Box<dyn Error>> {
    let mut archive = ZipArchive::new(file)?;
    let mut found_rom = false;

    println!("[+] Analyzing zip archive: {}", original_filename);

    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)?;
        let entry_name = file_in_zip.name().to_string();
        let lower_entry_name = entry_name.to_lowercase();

        if file_in_zip.is_dir() {
            continue;
        }

        let mut is_supported_rom = false;
        for ext in SUPPORTED_ROM_EXTENSIONS {
            if lower_entry_name.ends_with(ext) {
                is_supported_rom = true;
                break;
            }
        }

        if is_supported_rom {
            println!("[+] Found supported ROM in zip: {}", entry_name);
            found_rom = true;
            let mut data = Vec::new();
            file_in_zip.read_to_end(&mut data)?;
            process_rom_data_fn(data, &entry_name)?;
        }
    }

    if !found_rom {
        return Err(Box::new(RomAnalyzerError::new(
            &format!("No supported ROM files found within the zip archive: {}", original_filename)
        )));
    }
    Ok(())
}
