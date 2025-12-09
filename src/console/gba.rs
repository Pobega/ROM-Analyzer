//! Provides header analysis functionality for Game Boy Advance (GBA) ROMs.
//!
//! This module can parse GBA ROM headers to extract game title, game code,
//! maker code, and region information.
//!
//! GBA header documentation referenced here:
//! <https://problemkaputt.de/gbatek-gba-cartridge-header.htm>

use std::error::Error;

use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

/// Struct to hold the analysis results for a GBA ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct GbaAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "Japan").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The game title extracted from the ROM header.
    pub game_title: String,
    /// The game code extracted from the ROM header.
    pub game_code: String,
    /// The maker code extracted from the ROM header.
    pub maker_code: String,
}

impl GbaAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        format!(
            "{}\n\
             System:       Game Boy Advance (GBA)\n\
             Game Title:   {}\n\
             Game Code:    {}\n\
             Maker Code:   {}\n\
             Region:       {}",
            self.source_name, self.game_title, self.game_code, self.maker_code, self.region
        )
    }
}

/// Determines the Game Boy Advance game region name based on a given region byte.
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
/// - A `&'static str` representing the region as written in the ROM header (e.g., "USA", "Japan",
///   "Europe") or "Unknown" if the region code is not recognized.
/// - A `Region` bitmask representing the region(s) associated with the code.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::console::gba::map_region;
/// use rom_analyzer::region::Region;
///
/// let (region_str, region_mask) = map_region(0x00);
/// assert_eq!(region_str, "Japan");
/// assert_eq!(region_mask, Region::JAPAN);
///
/// let (region_str, region_mask) = map_region(0x01);
/// assert_eq!(region_str, "USA");
/// assert_eq!(region_mask, Region::USA);
///
/// let (region_str, region_mask) = map_region(0x02);
/// assert_eq!(region_str, "Europe");
/// assert_eq!(region_mask, Region::EUROPE);
/// ```
pub fn map_region(region_byte: u8) -> (&'static str, Region) {
    match region_byte {
        0x00 => ("Japan", Region::JAPAN),
        0x01 => ("USA", Region::USA),
        0x02 => ("Europe", Region::EUROPE),
        // ASCII representations are also common
        b'J' => ("Japan", Region::JAPAN),
        b'U' => ("USA", Region::USA),
        b'E' => ("Europe", Region::EUROPE),
        b'P' => ("Europe", Region::EUROPE), // PAL
        _ => ("Unknown", Region::UNKNOWN),
    }
}

/// Analyzes Game Boy Advance (GBA) ROM data.
///
/// This function reads the GBA ROM header to extract the game title, game code,
/// maker code, and region information. It then normalizes the region and performs
/// a region mismatch check against the `source_name`.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) containing the raw ROM data.
/// * `source_name` - The name of the ROM file, used for region mismatch checks.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(GbaAnalysis)` containing the detailed analysis results.
/// - `Err(Box<dyn Error>)` if the ROM data is too small to contain a valid GBA header.
pub fn analyze_gba_data(data: &[u8], source_name: &str) -> Result<GbaAnalysis, Box<dyn Error>> {
    // GBA header is at offset 0x0. Relevant info: Game Title (0xA0-0xAC), Game Code (0xAC-0xB0), Maker Code (0xB0-0xB2), Region (0xB4).
    // The header is typically 192 bytes (0xC0), but we'll use a slightly larger safety margin.
    const HEADER_SIZE: usize = 0xC0;
    if data.len() < HEADER_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a GBA header (size: {} bytes, requires at least {} bytes).",
            data.len(),
            HEADER_SIZE
        ))));
    }

    // Extract Game Title (12 bytes, null-terminated)
    let game_title = String::from_utf8_lossy(&data[0xA0..0xAC])
        .trim_matches(char::from(0)) // Remove null bytes
        .to_string();

    // Extract Game Code (4 bytes, ASCII)
    let game_code = String::from_utf8_lossy(&data[0xAC..0xB0])
        .trim_matches(char::from(0)) // Remove null bytes, though usually not null-terminated here
        .to_string();

    // Extract Maker Code (2 bytes, ASCII)
    let maker_code = String::from_utf8_lossy(&data[0xB0..0xB2])
        .trim_matches(char::from(0)) // Remove null bytes
        .to_string();

    // Extract Region Code (1 byte at 0xB4)
    let region_code_byte = data[0xB4];

    // Determine region name based on the byte value.
    let (region_name, region) = map_region(region_code_byte);

    let region_mismatch = check_region_mismatch(source_name, region);

    Ok(GbaAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        game_title,
        game_code,
        maker_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Helper function to generate a minimal GBA header for testing.
    fn generate_gba_header(
        game_code: &str,
        maker_code: &str,
        region_byte: u8,
        title: &str,
    ) -> Vec<u8> {
        let mut data = vec![0; 0xC0]; // Ensure enough space for header

        // Game Title (max 10 chars + null, but we use 0xA0..0xAC which is 12 bytes for safety)
        let mut title_bytes = title.as_bytes().to_vec();
        title_bytes.resize(12, 0);
        data[0xA0..0xAC].copy_from_slice(&title_bytes);

        // Game Code (4 bytes, ASCII)
        let mut game_code_bytes = game_code.as_bytes().to_vec();
        game_code_bytes.resize(4, 0);
        data[0xAC..0xB0].copy_from_slice(&game_code_bytes);

        // Maker Code (2 bytes, ASCII)
        let mut maker_code_bytes = maker_code.as_bytes().to_vec();
        maker_code_bytes.resize(2, 0);
        data[0xB0..0xB2].copy_from_slice(&maker_code_bytes);

        // Region Code (1 byte at 0xB4)
        data[0xB4] = region_byte;

        data
    }

    #[test]
    fn test_analyze_gba_data_japan_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("ABCD", "XX", 0x00, "GBA JP GAME"); // Japan region code 0x00
        let analysis = analyze_gba_data(&data, "test_rom_jp.gba")?;

        assert_eq!(analysis.source_name, "test_rom_jp.gba");
        assert_eq!(analysis.game_title, "GBA JP GAME");
        assert_eq!(analysis.game_code, "ABCD");
        assert_eq!(analysis.maker_code, "XX");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_usa_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("EFGH", "YY", 0x01, "GBA US GAME"); // USA region code 0x01
        let analysis = analyze_gba_data(&data, "test_rom_us.gba")?;

        assert_eq!(analysis.source_name, "test_rom_us.gba");
        assert_eq!(analysis.game_title, "GBA US GAME");
        assert_eq!(analysis.game_code, "EFGH");
        assert_eq!(analysis.maker_code, "YY");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_europe_char() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("IJKL", "ZZ", b'E', "GBA EUR GAME"); // Europe region char 'E'
        let analysis = analyze_gba_data(&data, "test_rom_eur.gba")?;

        assert_eq!(analysis.source_name, "test_rom_eur.gba");
        assert_eq!(analysis.game_title, "GBA EUR GAME");
        assert_eq!(analysis.game_code, "IJKL");
        assert_eq!(analysis.maker_code, "ZZ");
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_japan_char() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("MNOP", "AA", b'J', "GBA JP CHAR"); // Japan region char 'J'
        let analysis = analyze_gba_data(&data, "test_rom_jp_char.gba")?;

        assert_eq!(analysis.source_name, "test_rom_jp_char.gba");
        assert_eq!(analysis.game_title, "GBA JP CHAR");
        assert_eq!(analysis.game_code, "MNOP");
        assert_eq!(analysis.maker_code, "AA");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_unknown_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("QRST", "BB", 0xFF, "GBA UNKNOWN"); // Unknown region code
        let analysis = analyze_gba_data(&data, "test_rom_unknown.gba")?;

        assert_eq!(analysis.source_name, "test_rom_unknown.gba");
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 50]; // Smaller than 0xC0
        let result = analyze_gba_data(&data, "too_small.gba");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
