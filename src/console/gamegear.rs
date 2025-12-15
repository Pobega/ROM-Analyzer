//! Provides header analysis functionality for Sega Game Gear ROMs.
//!
//! This module can parse Game Gear ROM headers to extract region information
//! and compare it with region inferences made from the filename.
//!
//! Game Gear header documentation referenced here:
//! <https://www.smspower.org/Development/ROMHeader>

use std::error::Error;

use log::debug;
use serde::Serialize;

use crate::region::{Region, check_region_mismatch, infer_region_from_filename};

const POSSIBLE_HEADER_STARTS: &[usize] = &[0x7ff0, 0x3ff0, 0x1ff0];
const REGION_CODE_OFFSET: usize = 0xf;
const SEGA_HEADER_SIGNATURE: &[u8] = b"TMR SEGA";

/// Struct to hold the analysis results for a Game Gear ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct GameGearAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "GameGear Japan").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// If the region is found in the header, or inferred from the filename.
    pub region_found: bool,
}

impl GameGearAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        let region_not_in_rom_header = if !self.region_found {
            "\nNote:         Region information not in ROM header, inferred from filename."
        } else {
            ""
        };
        format!(
            "{}\n\
             System:       Sega Game Gear\n\
             Region:       {}\
             {}",
            self.source_name, self.region, region_not_in_rom_header
        )
    }
}

/// Determines the Game Gear game region name based on a given region byte.
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
/// - A `&'static str` representing the region as written in the ROM header (e.g., "SMS Japan",
///   "GameGear International") or "Unknown" if the region code is not recognized.
/// - A [`Region`] bitmask representing the region(s) associated with the code.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::console::gamegear::map_region;
/// use rom_analyzer::region::Region;
///
/// let (region_str, region_mask) = map_region(0x30);
/// assert_eq!(region_str, "SMS Japan");
/// assert_eq!(region_mask, Region::JAPAN);
///
/// let (region_str, region_mask) = map_region(0x60);
/// assert_eq!(region_str, "GameGear Export");
/// assert_eq!(region_mask, Region::USA | Region::EUROPE);
///
/// let (region_str, region_mask) = map_region(0x20);
/// assert_eq!(region_str, "Unknown");
/// assert_eq!(region_mask, Region::UNKNOWN);
/// ```
pub fn map_region(region_byte: u8) -> (&'static str, Region) {
    let region_code_value: u8 = region_byte >> 4;
    match region_code_value {
        0x3 => ("SMS Japan", Region::JAPAN),
        0x4 => ("SMS Export", Region::USA | Region::EUROPE),
        0x5 => ("GameGear Japan", Region::JAPAN),
        0x6 => ("GameGear Export", Region::USA | Region::EUROPE),
        0x7 => ("GameGear International", Region::USA | Region::EUROPE),
        _ => ("Unknown", Region::UNKNOWN),
    }
}

/// Analyzes a Game Gear ROM and returns a struct containing the analysis results.
///
/// This function attempts to locate the 'TMR SEGA' header signature within the ROM data at
/// predefined offsets. If found, it extracts the region byte and determines the ROM's region.  If
/// the region cannot be determined from the header or if no header is found, it attempts to infer
/// the region from the `source_name`.
///
/// If a region is found in the header it also checks for mismatches between the inferred and
/// header-derived regions.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) containing the raw ROM data.
/// * `source_name` - The name of the ROM file, used for region inference if needed.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok`([`GameGearAnalysis`]) containing the detailed analysis results.
/// - `Err(Box<dyn Error>)` if any critical error occurs during analysis.
pub fn analyze_gamegear_data(
    data: &[u8],
    source_name: &str,
) -> Result<GameGearAnalysis, Box<dyn Error>> {
    // All headered Sega 8-bit ROMs should begin with 'TMR SEGA'
    // This can exist at one of three locations; 0x1ff0, 0x3ff0 or 0x7ff0
    let header_start_opt = POSSIBLE_HEADER_STARTS.iter().copied().find(|&offset| {
        data.get(offset..offset + SEGA_HEADER_SIGNATURE.len()) == Some(SEGA_HEADER_SIGNATURE)
    });

    let mut region = Region::UNKNOWN;
    let mut region_name = "Unknown".to_string();
    let mut region_found = false;

    if let Some(header_start) = header_start_opt {
        debug!("Found signature at 0x{:x}", header_start);
        if let Some(&region_byte) = data.get(header_start + REGION_CODE_OFFSET) {
            let (name, region_val) = map_region(region_byte);
            region_name = name.to_string();
            region = region_val;
            if region != Region::UNKNOWN {
                region_found = true;
            }
        } else {
            debug!(
                "ROM too small to read region code from header at 0x{:x}",
                header_start
            );
        }
    }

    if !region_found {
        region = infer_region_from_filename(source_name);
        region_name = region.to_string();
    }

    let region_mismatch = check_region_mismatch(source_name, region);

    Ok(GameGearAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        region_found,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // Helper function to create dummy ROM data with a Game Gear header
    fn create_rom_data_with_header(header_start: usize, region_code: u8) -> Vec<u8> {
        let mut data = vec![0; 0x8000]; // Sufficiently large dummy data
        if data.len() > header_start + REGION_CODE_OFFSET {
            // Write SEGA header signature
            data[header_start..header_start + SEGA_HEADER_SIGNATURE.len()]
                .copy_from_slice(SEGA_HEADER_SIGNATURE);
            // Write region code
            data[header_start + REGION_CODE_OFFSET] = region_code;
        }
        data
    }

    #[test]
    fn test_analyze_gamegear_data_header_signature_present_region_byte_missing()
    -> Result<(), Box<dyn Error>> {
        let header_start = 0x7ff0;
        let signature_len = SEGA_HEADER_SIGNATURE.len();
        // Create a ROM that has the signature but is too short for the region byte
        let mut data = vec![0; header_start + signature_len];
        data[header_start..].copy_from_slice(SEGA_HEADER_SIGNATURE);

        let analysis = analyze_gamegear_data(&data, "my_game_usa.gg")?;
        assert_eq!(analysis.source_name, "my_game_usa.gg");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA");
        assert!(!analysis.region_found); // Region should be inferred, not found in header
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_header_japan_0x7ff0() -> Result<(), Box<dyn Error>> {
        // 0x50 >> 4 = 0x5 (GameGear Japan)
        let data = create_rom_data_with_header(0x7ff0, 0x50);
        let analysis = analyze_gamegear_data(&data, "test_rom.gg")?;
        assert_eq!(analysis.source_name, "test_rom.gg");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "GameGear Japan");
        assert!(analysis.region_found);
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_header_export_0x3ff0() -> Result<(), Box<dyn Error>> {
        // 0x60 >> 4 = 0x6 (GameGear Export)
        let data = create_rom_data_with_header(0x3ff0, 0x60);
        let analysis = analyze_gamegear_data(&data, "test_rom.gg")?;
        assert_eq!(analysis.source_name, "test_rom.gg");
        assert_eq!(analysis.region, Region::USA | Region::EUROPE);
        assert_eq!(analysis.region_string, "GameGear Export");
        assert!(analysis.region_found);
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_header_international_0x1ff0() -> Result<(), Box<dyn Error>> {
        // 0x70 >> 4 = 0x7 (GameGear International)
        let data = create_rom_data_with_header(0x1ff0, 0x70);
        let analysis = analyze_gamegear_data(&data, "test_rom.gg")?;
        assert_eq!(analysis.source_name, "test_rom.gg");
        assert_eq!(analysis.region, Region::USA | Region::EUROPE);
        assert_eq!(analysis.region_string, "GameGear International");
        assert!(analysis.region_found);
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_no_header_infer_from_filename() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x8000]; // No header
        let analysis = analyze_gamegear_data(&data, "my_game_usa.gg")?;
        assert_eq!(analysis.source_name, "my_game_usa.gg");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA");
        assert!(!analysis.region_found);
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_header_unknown_region_infer_from_filename()
    -> Result<(), Box<dyn Error>> {
        // Header exists, but region code (0xF0 >> 4 = 0xF) is unknown, so it should infer from filename.
        let data = create_rom_data_with_header(0x7ff0, 0xF0);
        let analysis = analyze_gamegear_data(&data, "my_game_japan.gg")?;
        assert_eq!(analysis.source_name, "my_game_japan.gg");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan");
        assert!(!analysis.region_found); // Still false because the header didn't provide a known region
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_get_region_name() {
        assert_eq!(map_region(0x30), ("SMS Japan", Region::JAPAN));
        assert_eq!(
            map_region(0x40),
            ("SMS Export", Region::USA | Region::EUROPE)
        );
        assert_eq!(map_region(0x50), ("GameGear Japan", Region::JAPAN));
        assert_eq!(
            map_region(0x60),
            ("GameGear Export", Region::USA | Region::EUROPE)
        );
        assert_eq!(
            map_region(0x70),
            ("GameGear International", Region::USA | Region::EUROPE)
        );
        assert_eq!(map_region(0x00), ("Unknown", Region::UNKNOWN));
        assert_eq!(map_region(0xF0), ("Unknown", Region::UNKNOWN));
    }

    #[test]
    fn test_analyze_gamegear_data_usa() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom_usa.gg")?;
        assert_eq!(analysis.source_name, "test_rom_usa.gg");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA");
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_japan() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom_jp.gg")?;
        assert_eq!(analysis.source_name, "test_rom_jp.gg");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_europe() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom_eur.gg")?;
        assert_eq!(analysis.source_name, "test_rom_eur.gg");
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe");
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_unknown() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom.gg")?;
        assert_eq!(analysis.source_name, "test_rom.gg");
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown");
        Ok(())
    }
}
