//! Provides functionality for analyzing CHD (Compressed Hunks of Data) files.
//!
//! This module focuses on decompressing and extracting relevant header data from CHD files.
//! It exposes a function to decompress a portion of a CHD file for header analysis.

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use chd::Chd;
use log::debug;

// We only need the first few KB for header analysis for PSX and SegaCD.
const MAX_HEADER_SIZE: usize = 0x20000; // 128KB

/// Analyzes a CHD (Compressed Hunks of Data) file, decompressing a portion of it.
///
/// This function opens a CHD file, reads its header to determine hunk size and count,
/// and then decompresses a maximum of `MAX_HEADER_SIZE` bytes from the beginning
/// of the CHD data. This decompressed data is typically sufficient for extracting
/// console-specific headers without decompressing the entire (potentially very large)
/// CHD file.
///
/// # Arguments
///
/// * `filepath` - The path to the CHD file.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(Vec<u8>)` containing the decompressed initial bytes of the CHD file.
/// - `Err(Box<dyn Error>)` if any error occurs when processing the CHD.
///
/// # Errors
///
/// This function can return an error if:
/// - The file cannot be opened.
/// - The CHD format is invalid or corrupted.
/// - There are issues during hunk decompression.
pub fn analyze_chd_file(filepath: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut chd = Chd::open(&mut reader, None)?;

    let hunk_count = chd.header().hunk_count();
    let hunk_size = chd.header().hunk_size();

    debug!(
        "[+] Analyzing CHD file: {}",
        filepath
            .file_name()
            .unwrap_or_else(|| filepath.as_ref())
            .to_string_lossy()
    );

    let mut decompressed_data = Vec::new();
    decompressed_data.reserve_exact(
        ((hunk_count as u64) * (hunk_size as u64)).min(MAX_HEADER_SIZE as u64) as usize,
    );

    let mut out_buf = chd.get_hunksized_buffer();
    let mut temp_buf = Vec::new();

    for hunk_num in 0..hunk_count {
        if decompressed_data.len() >= MAX_HEADER_SIZE {
            break;
        }

        let mut hunk = chd.hunk(hunk_num)?;
        hunk.read_hunk_in(&mut temp_buf, &mut out_buf)?;

        let remaining_capacity = MAX_HEADER_SIZE - decompressed_data.len();
        let data_to_add = out_buf.len().min(remaining_capacity);
        decompressed_data.extend_from_slice(&out_buf[..data_to_add]);
    }

    debug!(
        "[+] Decompressed first {} bytes for header analysis.",
        decompressed_data.len()
    );

    Ok(decompressed_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_analyze_chd_file_non_existent() {
        let non_existent_path = Path::new("non_existent_file.chd");
        let result = analyze_chd_file(non_existent_path);

        assert!(result.is_err());
        let error = result.unwrap_err();
        // Check if the error is due to the file not found
        assert_eq!(
            error.downcast_ref::<std::io::Error>().map(|e| e.kind()),
            Some(ErrorKind::NotFound)
        );
    }
}
