use std::error::Error;
use std::fs::File;
use std::io::Read;

use log::debug;
use zip::ZipArchive;

use crate::SUPPORTED_ROM_EXTENSIONS;
use crate::error::RomAnalyzerError;

pub fn process_zip_file(
    file: File,
    original_filename: &str,
) -> Result<(Vec<u8>, String), Box<dyn Error>> {
    let mut archive = ZipArchive::new(file)?;

    debug!("[+] Analyzing ZIP archive: {}", original_filename);

    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)?;
        let entry_name = file_in_zip.name().to_string();
        let lower_entry_name = entry_name.to_lowercase();

        if file_in_zip.is_dir() {
            continue;
        }

        let is_supported_rom = SUPPORTED_ROM_EXTENSIONS
            .iter()
            .any(|ext| lower_entry_name.ends_with(ext));

        if is_supported_rom {
            debug!("[+] Found supported ROM in zip: {}", entry_name);
            let mut data = Vec::new();
            file_in_zip.read_to_end(&mut data)?;

            return Ok((data, entry_name));
        }
    }

    Err(Box::new(RomAnalyzerError::new(&format!(
        "No supported ROM files found within the zip archive: {}",
        original_filename
    ))))
}
