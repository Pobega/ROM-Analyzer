//! Provides header analysis functionality for Super Nintendo Entertainment System (SNES) ROMs.
//!
//! This module can detect SNES ROM mapping types (LoROM, HiROM),
//! validate checksums, and extract game title and region information.
//!
//! Super Nintendo header documentation referenced here:
//! <https://snes.nesdev.org/wiki/ROM_header>

use std::error::Error;

use log::error;
use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

// Map Mode byte offset relative to the header start (0x7FC0 for LoROM, 0xFFC0 for HiROM)
const MAP_MODE_OFFSET: usize = 0x15;

// Expected Map Mode byte values for LoROM and HiROM
const LOROM_MAP_MODES: &[u8] = &[0x20, 0x30, 0x25, 0x35];
const HIROM_MAP_MODES: &[u8] = &[0x21, 0x31, 0x22, 0x32];

/// Struct to hold the analysis results for a SNES ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct SnesAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "Japan (NTSC)").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The raw region code byte.
    pub region_code: u8,
    /// The game title extracted from the ROM header.
    pub game_title: String,
    /// The detected mapping type (e.g., "LoROM", "HiROM").
    pub mapping_type: String,
}

impl SnesAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        format!(
            "{}\n\
             System:       Super Nintendo (SNES)\n\
             Game Title:   {}\n\
             Mapping:      {}\n\
             Region Code:  0x{:02X}\n\
             Region:       {}",
            self.source_name, self.game_title, self.mapping_type, self.region_code, self.region
        )
    }
}

/// Determines the SNES game region name based on a given region byte.
///
/// The region byte typically comes from the ROM header. This function extracts the relevant bits
/// from the byte and maps it to a human-readable region string and a Region bitmask.
///
/// # Arguments
///
/// * `region_byte` - The byte containing the region code, usually found in the ROM header.
///
/// # Returns
///
/// A tuple containing:
/// - A `&'static str` representing the region as written in the ROM header (e.g., "Japan (NTSC)",
///   "USA / Canada (NTSC)", etc.) or "Unknown" if the region code is not recognized.
/// - A [`Region`] bitmask representing the region(s) associated with the code.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::console::snes::map_region;
/// use rom_analyzer::region::Region;
///
/// let (region_str, region_mask) = map_region(0x00);
/// assert_eq!(region_str, "Japan (NTSC)");
/// assert_eq!(region_mask, Region::JAPAN);
///
/// let (region_str, region_mask) = map_region(0x01);
/// assert_eq!(region_str, "USA / Canada (NTSC)");
/// assert_eq!(region_mask, Region::USA);
///
/// let (region_str, region_mask) = map_region(0x02);
/// assert_eq!(region_str, "Europe / Oceania / Asia (PAL)");
/// assert_eq!(region_mask, Region::EUROPE | Region::ASIA);
/// ```
pub fn map_region(code: u8) -> (&'static str, Region) {
    match code {
        0x00 => ("Japan (NTSC)", Region::JAPAN),
        0x01 => ("USA / Canada (NTSC)", Region::USA),
        0x02 => (
            "Europe / Oceania / Asia (PAL)",
            Region::EUROPE | Region::ASIA,
        ),
        0x03 => ("Sweden / Scandinavia (PAL)", Region::EUROPE),
        0x04 => ("Finland (PAL)", Region::EUROPE),
        0x05 => ("Denmark (PAL)", Region::EUROPE),
        0x06 => ("France (PAL)", Region::EUROPE),
        0x07 => ("Netherlands (PAL)", Region::EUROPE),
        0x08 => ("Spain (PAL)", Region::EUROPE),
        0x09 => ("Germany (PAL)", Region::EUROPE),
        0x0A => ("Italy (PAL)", Region::EUROPE),
        0x0B => ("China (PAL)", Region::CHINA),
        0x0C => ("Indonesia (PAL)", Region::EUROPE | Region::ASIA),
        0x0D => ("South Korea (NTSC)", Region::KOREA),
        0x0E => (
            "Common / International",
            Region::USA | Region::EUROPE | Region::JAPAN | Region::ASIA,
        ),
        0x0F => ("Canada (NTSC)", Region::USA),
        0x10 => ("Brazil (NTSC)", Region::USA),
        0x11 => ("Australia (PAL)", Region::EUROPE),
        0x12 => ("Other (Variation 1)", Region::UNKNOWN),
        0x13 => ("Other (Variation 2)", Region::UNKNOWN),
        0x14 => ("Other (Variation 3)", Region::UNKNOWN),
        _ => ("Unknown", Region::UNKNOWN),
    }
}

/// Helper function to validate the SNES ROM checksum.
///
/// This function checks if the 16-bit checksum and its complement, located
/// within the SNES header, sum up to `0xFFFF`. This is a common method
/// for validating the integrity of SNES ROM headers.
///
/// # Arguments
///
/// * `rom_data` - A byte slice (`&[u8]`) containing the raw ROM data.
/// * `header_offset` - The starting offset of the SNES header within `rom_data`.
///
/// # Returns
///
/// `true` if the checksum and its complement are valid (sum to 0xFFFF),
/// `false` otherwise, or if the `header_offset` is out of bounds.
pub fn validate_snes_checksum(rom_data: &[u8], header_offset: usize) -> bool {
    // Ensure we have enough data for checksum and complement bytes.
    if header_offset + 0x20 > rom_data.len() {
        return false;
    }

    // Checksum is at 0x1E (relative to header start), complement at 0x1C.
    // Both are 16-bit values, little-endian.
    let complement_bytes: [u8; 2] =
        match rom_data[header_offset + 0x1C..header_offset + 0x1E].try_into() {
            Ok(b) => b,
            Err(_) => return false, // Should not happen if header_offset + 0x20 is within bounds.
        };
    let checksum_bytes: [u8; 2] =
        match rom_data[header_offset + 0x1E..header_offset + 0x20].try_into() {
            Ok(b) => b,
            Err(_) => return false, // Should not happen if header_offset + 0x20 is within bounds.
        };

    let complement = u16::from_le_bytes(complement_bytes);
    let checksum = u16::from_le_bytes(checksum_bytes);

    // The checksum algorithm: (checksum + complement) should equal 0xFFFF.
    (checksum as u32 + complement as u32) == 0xFFFF
}

/// Analyzes SNES ROM data.
///
/// This function first attempts to detect a copier header. It then tries to determine
/// the ROM's mapping type (LoROM or HiROM) by validating checksums and examining
/// the Map Mode byte at expected header locations. If both checksum and Map Mode
/// are consistent, that mapping is chosen. If only the checksum is valid, it uses
/// that mapping with an "Map Mode Unverified" tag. If neither is fully consistent,
/// it falls back to LoROM (Unverified). Once the header location is determined,
/// it extracts the game title and region code, maps the region code to a human-readable
/// name, and performs a region mismatch check against the `source_name`.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) containing the raw ROM data.
/// * `source_name` - The name of the ROM file, used for logging and region mismatch checks.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok`([`SnesAnalysis`]) containing the detailed analysis results.
/// - `Err(Box<dyn Error>)` if the ROM data is too small or the header is deemed invalid
///   such that critical information cannot be read.
pub fn analyze_snes_data(data: &[u8], source_name: &str) -> Result<SnesAnalysis, Box<dyn Error>> {
    let file_size = data.len();
    let mut header_offset = 0;

    // Detect copier header (often 512 bytes, common for some older dumps/tools)
    if file_size >= 512 && (file_size % 1024 == 512) {
        // Heuristic: If file size ends in 512 and is divisible by 1024
        header_offset = 512;
        // Note: This copier header detection is a simple heuristic and might not be foolproof.
        // More advanced detection could involve checking for specific patterns.
    }

    // Determine ROM mapping type (LoROM vs HiROM) by checking checksums and Map Mode byte.
    // The relevant header information is usually found at 0x7FC0 for LoROM and 0xFFC0 for HiROM
    // (relative to the start of the ROM, accounting for the header_offset).
    let lorom_header_start = 0x7FC0 + header_offset; // Header block starts here
    let hirom_header_start = 0xFFC0 + header_offset; // Header block starts here

    let mapping_type: String;
    let valid_header_offset: usize;

    let lorom_checksum_valid = validate_snes_checksum(data, lorom_header_start);
    let hirom_checksum_valid = validate_snes_checksum(data, hirom_header_start);

    // Get Map Mode bytes if headers are within bounds
    let lorom_map_mode_byte = if lorom_header_start + MAP_MODE_OFFSET < file_size {
        Some(data[lorom_header_start + MAP_MODE_OFFSET])
    } else {
        None
    };
    let hirom_map_mode_byte = if hirom_header_start + MAP_MODE_OFFSET < file_size {
        Some(data[hirom_header_start + MAP_MODE_OFFSET])
    } else {
        None
    };

    let is_lorom_map_mode = lorom_map_mode_byte.map_or(false, |b| LOROM_MAP_MODES.contains(&b));
    let is_hirom_map_mode = hirom_map_mode_byte.map_or(false, |b| HIROM_MAP_MODES.contains(&b));

    // Decision logic: Prioritize HiROM if both checksum and map mode are consistent.
    // Then check LoROM similarly. If only one checksum is valid, use that.
    // If neither is fully consistent, fallback to LoROM (unverified) with a warning.
    if hirom_checksum_valid && is_hirom_map_mode {
        mapping_type = "HiROM".to_string();
        valid_header_offset = hirom_header_start;
    } else if lorom_checksum_valid && is_lorom_map_mode {
        mapping_type = "LoROM".to_string();
        valid_header_offset = lorom_header_start;
    } else if hirom_checksum_valid {
        mapping_type = "HiROM (Map Mode Unverified)".to_string();
        valid_header_offset = hirom_header_start;
        error!(
            "[!] HiROM checksum valid for {}, but Map Mode byte (0x{:02X?}) is not a typical HiROM value. Falling back to HiROM.",
            source_name, hirom_map_mode_byte
        );
    } else if lorom_checksum_valid {
        mapping_type = "LoROM (Map Mode Unverified)".to_string();
        valid_header_offset = lorom_header_start;
        error!(
            "[!] LoROM checksum valid for {}, but Map Mode byte (0x{:02X?}) is not a typical LoROM value. Falling back to LoROM.",
            source_name, lorom_map_mode_byte
        );
    } else {
        // If neither checksum is valid, log a warning and try LoROM as a fallback, as it's more common.
        error!(
            "[!] Checksum validation failed for {}. Attempting to read header from LoROM location ({:X}) as fallback.",
            source_name, lorom_header_start
        );
        mapping_type = "LoROM (Unverified)".to_string();
        valid_header_offset = lorom_header_start; // Fallback to LoROM offset
    }

    // Ensure the determined header offset plus the header size needed for analysis is within the file bounds.
    // We need at least up to the region code (offset 0x19 relative to header start) and game title (offset 0x0 to 0x14).
    // Thus, we check if `valid_header_offset + 0x20` is within bounds, as this covers the checksum bytes.
    if valid_header_offset + 0x20 > file_size {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small or header is invalid. File size: {} bytes. Checked header at offset: {}. Required minimum size for header region: {}.",
            file_size,
            valid_header_offset,
            valid_header_offset + 0x20
        ))));
    }

    // Extract region code and game title from the identified header.
    let region_byte_offset = valid_header_offset + 0x19; // Offset for region code within the header
    let region_code = data[region_byte_offset];
    let (region_name, region) = map_region(region_code);

    // Game title is located at the beginning of the header (offset 0x0 relative to valid_header_offset) for 21 bytes.
    // It is null-terminated, so we trim null bytes and leading/trailing whitespace.
    let game_title = String::from_utf8_lossy(&data[valid_header_offset..valid_header_offset + 21])
        .trim_matches(char::from(0)) // Remove null bytes
        .trim()
        .to_string();

    let region_mismatch = check_region_mismatch(source_name, region);

    Ok(SnesAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        region_code,
        game_title,
        mapping_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Helper to create a dummy SNES ROM with a valid checksum.
    /// It allows specifying ROM size, copier header offset, region code, mapping type.
    fn generate_snes_header(
        rom_size: usize,
        copier_header_offset: usize,
        region_code: u8,
        is_hirom: bool,
        title: &str,
        map_mode_byte: Option<u8>,
    ) -> Vec<u8> {
        let mut data = vec![0; rom_size];

        // Calculate the actual start of the SNES header based on mapping type and copier offset.
        let header_start = (if is_hirom { 0xFFC0 } else { 0x7FC0 }) + copier_header_offset;

        // Ensure the data is large enough
        if header_start + 0x20 > rom_size {
            panic!(
                "Provided ROM size {} is too small for SNES header at offset {} (needs at least {}).",
                rom_size,
                header_start,
                header_start + 0x20
            );
        }

        // 1. Set Title (21 bytes starting at header_start + 0x00)
        let mut title_bytes: Vec<u8> = title.as_bytes().to_vec();
        // Truncate if longer than 21 bytes, then pad with spaces if shorter.
        title_bytes.truncate(21);
        title_bytes.resize(21, b' '); // Pad with spaces, standard SNES header practice

        data[header_start..header_start + 21].copy_from_slice(&title_bytes);

        // 2. Set Region Code (at header_start + 0x19)
        data[header_start + 0x19] = region_code;

        // 3. Set Map Mode byte if provided (at header_start + MAP_MODE_OFFSET)
        if let Some(map_mode) = map_mode_byte {
            data[header_start + MAP_MODE_OFFSET] = map_mode;
        }

        // 4. Set a valid checksum and its complement.
        // The checksum algorithm is (checksum + complement) == 0xFFFF. We use a simple pair.
        let complement: u16 = 0x5555;
        let checksum: u16 = 0xFFFF - complement; // 0xAAAA

        // Set Checksum Complement (0x1C relative to header start)
        data[header_start + 0x1C..header_start + 0x1E].copy_from_slice(&complement.to_le_bytes());
        // Set Checksum (0x1E relative to header start)
        data[header_start + 0x1E..header_start + 0x20].copy_from_slice(&checksum.to_le_bytes());

        data
    }

    #[test]
    fn test_analyze_snes_data_lorom_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_snes_header(0x80000, 0, 0x00, false, "TEST GAME TITLE", None); // 512KB ROM, LoROM, Japan
        let analysis = analyze_snes_data(&data, "test_lorom_jp.sfc")?;

        assert_eq!(analysis.source_name, "test_lorom_jp.sfc");
        assert_eq!(analysis.game_title, "TEST GAME TITLE");
        assert_eq!(analysis.mapping_type, "LoROM (Map Mode Unverified)");
        assert_eq!(analysis.region_code, 0x00);
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_hirom_usa() -> Result<(), Box<dyn Error>> {
        let data = generate_snes_header(0x100000, 0, 0x01, true, "TEST GAME TITLE", None); // 1MB ROM, HiROM, USA
        let analysis = analyze_snes_data(&data, "test_hirom_us.sfc")?;

        assert_eq!(analysis.source_name, "test_hirom_us.sfc");
        assert_eq!(analysis.game_title, "TEST GAME TITLE");
        assert_eq!(analysis.mapping_type, "HiROM (Map Mode Unverified)");
        assert_eq!(analysis.region_code, 0x01);
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA / Canada (NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_lorom_europe_copier_header() -> Result<(), Box<dyn Error>> {
        // Rom size ends with 512 bytes, e.g., 800KB + 512 bytes = 800512 bytes.
        let data = generate_snes_header(0x80000 + 512, 512, 0x02, false, "TEST GAME TITLE", None); // LoROM, Europe, with 512-byte copier header
        let analysis = analyze_snes_data(&data, "test_lorom_eur_copier.sfc")?;

        assert_eq!(analysis.source_name, "test_lorom_eur_copier.sfc");
        assert_eq!(analysis.game_title, "TEST GAME TITLE");
        assert_eq!(analysis.mapping_type, "LoROM (Map Mode Unverified)"); // Should detect copier header but still identify LoROM
        assert_eq!(analysis.region_code, 0x02);
        assert_eq!(analysis.region, Region::EUROPE | Region::ASIA);
        assert_eq!(analysis.region_string, "Europe / Oceania / Asia (PAL)");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_hirom_canada_copier_header() -> Result<(), Box<dyn Error>> {
        // Data size: 1MB + 512 bytes for copier header
        let data = generate_snes_header(
            0x100200,
            512,  // Copier Header offset
            0x0F, // Region: Canada (0x0F)
            true, // HiROM
            "TEST GAME TITLE",
            None,
        );
        let analysis = analyze_snes_data(&data, "test_hirom_can_copier.sfc")?;

        assert_eq!(analysis.source_name, "test_hirom_can_copier.sfc");
        assert_eq!(analysis.game_title, "TEST GAME TITLE");
        assert_eq!(analysis.mapping_type, "HiROM (Map Mode Unverified)");
        assert_eq!(analysis.region_code, 0x0F);
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "Canada (NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_unknown_region() -> Result<(), Box<dyn Error>> {
        let data = generate_snes_header(0x80000, 0, 0xFF, false, "TEST GAME TITLE", None); // LoROM, Unknown region
        let analysis = analyze_snes_data(&data, "test_lorom_unknown.sfc")?;

        assert_eq!(analysis.source_name, "test_lorom_unknown.sfc");
        assert_eq!(analysis.game_title, "TEST GAME TITLE");
        assert_eq!(analysis.mapping_type, "LoROM (Map Mode Unverified)");
        assert_eq!(analysis.region_code, 0xFF);
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_invalid_checksum() -> Result<(), Box<dyn Error>> {
        // FIX: Use the robust helper to generate a correctly formatted header first.
        let mut data = generate_snes_header(
            0x8000, // 32KB is enough for LoROM
            0,
            0x01,               // USA region code
            false,              // LoROM base
            "INVALID CHECKSUM", // Title to assert on
            None,
        );

        // Manually invalidate the checksum/complement pair.
        // LoROM header start is 0x7FC0. Checksum area starts at 0x7FC0 + 0x1C.
        let checksum_start = 0x7FC0 + 0x1C;

        // Overwrite the 4 bytes of checksum/complement with invalid data (e.g., all zeros)
        data[checksum_start..checksum_start + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let analysis = analyze_snes_data(&data, "test_invalid_checksum.sfc")?;

        assert_eq!(analysis.source_name, "test_invalid_checksum.sfc");
        assert_eq!(analysis.game_title, "INVALID CHECKSUM");
        assert_eq!(analysis.mapping_type, "LoROM (Unverified)"); // Expecting fallback
        assert_eq!(analysis.region_code, 0x01);
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA / Canada (NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_too_small() {
        // Test with data smaller than the minimum required size for header analysis.
        // The minimal size depends on mapping type and copier header. For LoROM without copier header,
        // it's header_start + 0x20 = 0x7FFC + 0x20 = 0x801C bytes.
        let data = vec![0; 0x1000]; // Significantly smaller than required.
        let result = analyze_snes_data(&data, "too_small.sfc");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("too small or header is invalid")
        );
    }

    #[test]
    fn test_analyze_snes_data_hirom_checksum_map_mode_consistent() -> Result<(), Box<dyn Error>> {
        let data =
            generate_snes_header(0x100000, 0, 0x01, true, "TEST HIROM CONSISTENT", Some(0x21)); // HiROM, USA, HiROM Map Mode
        let analysis = analyze_snes_data(&data, "test_hirom_consistent.sfc")?;

        assert_eq!(analysis.mapping_type, "HiROM");
        assert_eq!(analysis.game_title, "TEST HIROM CONSISTENT");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_lorom_checksum_map_mode_consistent() -> Result<(), Box<dyn Error>> {
        let data =
            generate_snes_header(0x80000, 0, 0x00, false, "TEST LOROM CONSISTENT", Some(0x20)); // LoROM, Japan, LoROM Map Mode
        let analysis = analyze_snes_data(&data, "test_lorom_consistent.sfc")?;

        assert_eq!(analysis.mapping_type, "LoROM");
        assert_eq!(analysis.game_title, "TEST LOROM CONSISTENT");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_hirom_checksum_map_mode_inconsistent() -> Result<(), Box<dyn Error>> {
        let data = generate_snes_header(
            0x100000,
            0,
            0x01,
            true,
            "TEST HIROM INCONSISTENT",
            Some(0x20),
        ); // HiROM, USA, LoROM Map Mode
        let analysis = analyze_snes_data(&data, "test_hirom_inconsistent.sfc")?;

        assert_eq!(analysis.mapping_type, "HiROM (Map Mode Unverified)");
        assert_eq!(analysis.game_title, "TEST HIROM INCONSISTE");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_lorom_checksum_map_mode_inconsistent() -> Result<(), Box<dyn Error>> {
        let data = generate_snes_header(
            0x80000,
            0,
            0x00,
            false,
            "TEST LOROM INCONSISTENT",
            Some(0x21),
        ); // LoROM, Japan, HiROM Map Mode
        let analysis = analyze_snes_data(&data, "test_lorom_inconsistent.sfc")?;

        assert_eq!(analysis.mapping_type, "LoROM (Map Mode Unverified)");
        assert_eq!(analysis.game_title, "TEST LOROM INCONSISTE");
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_no_valid_checksum_map_mode_consistent_hirom_only()
    -> Result<(), Box<dyn Error>> {
        let mut data = generate_snes_header(
            0x100000,
            0,
            0x01,
            true,
            "TEST NO CHECKSUM HIROM MAP",
            Some(0x21),
        ); // HiROM, USA, HiROM Map Mode
        // Invalidate both checksums
        let lorom_checksum_start = 0x7FC0 + 0x1C;
        data[lorom_checksum_start..lorom_checksum_start + 4]
            .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        let hirom_checksum_start = 0xFFC0 + 0x1C;
        data[hirom_checksum_start..hirom_checksum_start + 4]
            .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let analysis = analyze_snes_data(&data, "test_no_checksum_hirom_map.sfc")?;

        assert_eq!(analysis.mapping_type, "LoROM (Unverified)"); // Expect fallback
        Ok(())
    }
    #[test]
    fn test_analyze_snes_data_no_valid_checksum_map_mode_consistent_lorom_only()
    -> Result<(), Box<dyn Error>> {
        let mut data = generate_snes_header(
            0x80000,
            0,
            0x00,
            false,
            "TEST NO CHECKSUM LOROM MAP",
            Some(0x20),
        ); // LoROM, Japan, LoROM Map Mode
        // Invalidate both checksums
        let lorom_checksum_start = 0x7FC0 + 0x1C;
        data[lorom_checksum_start..lorom_checksum_start + 4]
            .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        let hirom_checksum_start = 0xFFC0 + 0x1C;
        data[hirom_checksum_start..hirom_checksum_start + 4]
            .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let analysis = analyze_snes_data(&data, "test_no_checksum_lorom_map.sfc")?;

        assert_eq!(analysis.mapping_type, "LoROM (Unverified)"); // Expect fallback
        Ok(())
    }
}
