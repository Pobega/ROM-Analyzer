//! Provides functionality for processing ZIP archives to extract ROM files.
//!
//! This module can open a ZIP file, iterate through its contents, and identify
//! supported ROM files based on their file extensions. It then extracts the
//! raw byte data of the first supported ROM found within the archive.

use std::error::Error;
use std::fs::File;
use std::io::Read;

use log::debug;
use zip::ZipArchive;

use crate::SUPPORTED_ROM_EXTENSIONS;
use crate::error::RomAnalyzerError;

/// Processes a ZIP archive to find and extract the first supported ROM file.
///
/// This function opens the provided ZIP file, iterates through its entries,
/// and checks if any entry has a file extension listed in `SUPPORTED_ROM_EXTENSIONS`.
/// If a supported ROM is found, its decompressed data and filename are returned.
/// Only the first supported ROM encountered is extracted.
///
/// # Arguments
///
/// * `file` - A `File` object representing the opened ZIP archive.
/// * `original_filename` - The name of the ZIP file, used for error reporting.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok((Vec<u8>, String))` containing the raw byte data of the extracted ROM
///   and its original filename within the archive.
/// - `Err(Box<dyn Error>)` if:
///   - The ZIP archive is invalid or corrupted.
///   - An I/O error occurs during reading.
///   - No supported ROM files are found within the archive.
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
