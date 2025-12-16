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

/// Max ROM size to extract from the zip (128kb).
/// This avoids us  extracting larger files to memory which is a concern for memory constrained
/// systems that may be utilizing this functionality.
const MAX_ROM_SIZE: u64 = 128 * 1024;

/// Processes a ZIP archive to find and extract the first supported ROM file.
///
/// This function opens the provided ZIP file, iterates through its entries,
/// and checks if any entry has a file extension listed in [`SUPPORTED_ROM_EXTENSIONS`].
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
        let file_in_zip = archive.by_index(i)?;
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
            // Read the file up to MAX_ROM_SIZE.
            let mut limited_reader = file_in_zip.take(MAX_ROM_SIZE);
            let mut data = Vec::new();
            limited_reader.read_to_end(&mut data)?;

            return Ok((data, entry_name));
        }
    }

    Err(Box::new(RomAnalyzerError::new(&format!(
        "No supported ROM files found within the zip archive: {}",
        original_filename
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::{FileOptions, ZipWriter};

    /// This struct will hold both the path and the temporary directory handle
    /// to ensure the file is not deleted until this struct is dropped.
    struct TestZip {
        path: String,
        // The TempDir MUST be kept to prevent the zip file from being deleted.
        _dir: tempfile::TempDir,
    }

    /// Test helper function to create a temporary Zip file for testing.
    fn create_zip_file(filename: &str, file_contents: &[u8]) -> Result<TestZip, Box<dyn Error>> {
        let dir = tempdir()?;
        let zip_path = dir.path().join("test.zip");
        let zip_file = File::create(&zip_path)?;

        let mut zip = ZipWriter::new(zip_file);
        zip.start_file(filename, FileOptions::default())?;
        zip.write_all(file_contents)?;
        zip.finish()?;

        let zip_path_string: String = zip_path
            .to_str()
            .ok_or_else(|| RomAnalyzerError::new("Path contained invalid UTF-8"))?
            .to_string();

        Ok(TestZip {
            path: zip_path_string,
            _dir: dir,
        })
    }

    #[test]
    fn test_process_zip_file_no_supported_roms() {
        let expected_filename = "unsupported.txt";
        let expected_data = b"This is not a ROM.";

        let zip_path = create_zip_file(expected_filename, expected_data)
            .expect("Failed to create test zip file");
        let zip_file = File::open(&zip_path.path).expect("Failed to open zip for reading");

        let result = process_zip_file(zip_file, &zip_path.path);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let rom_analyzer_err = error
            .downcast_ref::<RomAnalyzerError>()
            .expect("Error should be a RomAnalyzerError");
        assert!(
            rom_analyzer_err
                .to_string()
                .starts_with("No supported ROM files found within the zip archive")
        );
    }

    #[test]
    fn test_process_zip_file_with_supported_rom() {
        let expected_filename = "game.nes";
        // Create data larger than 1152 bytes but smaller than 128KB to test size limits
        let mut expected_data = vec![0u8; 2000];
        expected_data[0..12].copy_from_slice(b"NES ROM DATA");

        let zip_path = create_zip_file(expected_filename, &expected_data)
            .expect("Failed to create test zip file");
        let zip_file = File::open(&zip_path.path).expect("Failed to open zip for reading");

        let result = process_zip_file(zip_file, &zip_path.path);

        assert!(result.is_ok());
        let (extracted_data, extracted_filename) = result.unwrap();
        assert_eq!(extracted_data, expected_data);
        assert_eq!(extracted_filename, expected_filename);
    }
}
