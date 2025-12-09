//! Provides header analysis functionality for Sega Genesis (also known as Mega Drive) ROMs.
//!
//! This module can parse Genesis ROM headers to extract system type, game titles
//! (domestic and international), and region information.
//!
//! Genesis header documentation referenced here:
//! <https://plutiedev.com/rom-header#system>

use std::error::Error;

use log::error;
use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

const SYSTEM_TYPE_START: usize = 0x100;
const SYSTEM_TYPE_END: usize = 0x110;
const DOMESTIC_TITLE_START: usize = 0x120;
const DOMESTIC_TITLE_END: usize = 0x150;
const INTL_TITLE_START: usize = 0x150;
const INTL_TITLE_END: usize = 0x180;
const REGION_CODE_BYTE: usize = 0x1F0;

/// Struct to hold the analysis results for a Sega cartridge (Genesis/Mega Drive) ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct GenesisAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "USA (NTSC-U)").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The raw region code byte.
    pub region_code_byte: u8,
    /// The detected console name (e.g., "SEGA MEGA DRIVE", "SEGA GENESIS").
    pub console_name: String,
    /// The domestic game title extracted from the ROM header.
    pub game_title_domestic: String,
    /// The international game title extracted from the ROM header.
    pub game_title_international: String,
}

impl GenesisAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        format!(
            "{}\n\
             System:       {}\n\
             Game Title (Domestic): {}\n\
             Game Title (Int.):   {}\n\
             Region Code:  0x{:02X} ('{}')\n\
             Region:       {}",
            self.source_name,
            self.console_name,
            self.game_title_domestic,
            self.game_title_international,
            self.region_code_byte,
            self.region_code_byte as char,
            self.region
        )
    }
}

/// Analyzes Sega Genesis/Mega Drive ROM data.
///
/// This function reads the ROM header to extract the console name (e.g., "SEGA MEGA DRIVE", "SEGA
/// GENESIS"), domestic and international game titles, and the region code byte. It then maps the
/// region code to a human-readable region name and performs a region mismatch check against the
/// `source_name`.  A warning is logged if an unexpected Sega header signature is found.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) containing the raw ROM data.
/// * `source_name` - The name of the ROM file, used for logging and region mismatch checks.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(GenesisAnalysis)` containing the detailed analysis results.
/// - `Err(Box<dyn Error>)` if the ROM data is too small to contain a valid Sega header.
pub fn analyze_genesis_data(
    data: &[u8],
    source_name: &str,
) -> Result<GenesisAnalysis, Box<dyn Error>> {
    // Sega Genesis/Mega Drive header is at offset 0x100. It's 256 bytes long.
    // The region byte is at offset 0x1F0 (relative to ROM start).
    const HEADER_SIZE: usize = 0x200; // Minimum size to contain the header and region byte.
    if data.len() < HEADER_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a Sega header (size: {} bytes, requires at least {} bytes).",
            data.len(),
            HEADER_SIZE
        ))));
    }

    // Verify Sega header signature "SEGA MEGA DRIVE " or "SEGA GENESIS"
    // This is not strictly necessary for region analysis but good for validation.
    let console_name_bytes = &data[SYSTEM_TYPE_START..SYSTEM_TYPE_END];
    let console_name = String::from_utf8_lossy(console_name_bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_string();

    // If the signature doesn't match, it might still be a valid ROM but with a different header convention.
    // We'll proceed with analysis but log a warning if the console name is unexpected.
    let is_valid_signature = console_name == "SEGA MEGA DRIVE" || console_name == "SEGA GENESIS";
    if !is_valid_signature {
        error!(
            "[!] Warning: Unexpected Sega header signature for {} at 0x{:x}. Found: '{}'",
            source_name, SYSTEM_TYPE_START, console_name
        );
    }

    // Game Title - Domestic (48 bytes, null-terminated)
    let game_title_domestic =
        String::from_utf8_lossy(&data[DOMESTIC_TITLE_START..DOMESTIC_TITLE_END])
            .trim_matches(char::from(0))
            .trim()
            .to_string();
    // Game Title - International (48 bytes, null-terminated)
    let game_title_international = String::from_utf8_lossy(&data[INTL_TITLE_START..INTL_TITLE_END])
        .trim_matches(char::from(0))
        .trim()
        .to_string();

    // Region Code byte is at offset 0x1F0 (which is 0xF0 relative to header_start)
    let region_code_byte = data[REGION_CODE_BYTE];

    let (region_name, region) = match region_code_byte {
        b'J' => ("Japan (NTSC-J)", Region::JAPAN),
        b'U' => ("USA (NTSC-U)", Region::USA),
        b'E' => ("Europe (PAL)", Region::EUROPE),
        b'A' => ("Asia (NTSC)", Region::ASIA),
        b'B' => ("Brazil (PAL-M)", Region::EUROPE),
        b'C' => ("China (NTSC)", Region::CHINA),
        b'F' => ("France (PAL)", Region::EUROPE),
        b'K' => ("Korea (NTSC)", Region::KOREA),
        b'L' => ("UK (PAL)", Region::EUROPE),
        b'S' => ("Scandinavia (PAL)", Region::EUROPE),
        b'T' => ("Taiwan (NTSC)", Region::ASIA),
        0x34 => ("USA/Europe (NTSC/PAL)", Region::USA | Region::EUROPE),
        _ => ("Unknown Code", Region::UNKNOWN),
    };

    let region_mismatch = check_region_mismatch(source_name, &region_name);

    Ok(GenesisAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        region_code_byte,
        console_name,
        game_title_domestic,
        game_title_international,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Helper function to generate a minimal Sega cartridge header for testing.
    fn generate_genesis_header(
        console_sig: &[u8],
        region_byte: u8,
        domestic_title: &str,
        international_title: &str,
    ) -> Vec<u8> {
        let mut data = vec![0; 0x200]; // Ensure enough space for header and region byte.

        // Console Name/Signature (16 bytes at 0x100)
        data[SYSTEM_TYPE_START..SYSTEM_TYPE_END].copy_from_slice(console_sig);

        // Game Title - Domestic (32 bytes, null-terminated)
        let mut domestic_title_bytes = domestic_title.as_bytes().to_vec();
        domestic_title_bytes.resize(48, 0);
        data[DOMESTIC_TITLE_START..DOMESTIC_TITLE_END].copy_from_slice(&domestic_title_bytes);

        // Game Title - International (32 bytes, null-terminated)
        let mut international_title_bytes = international_title.as_bytes().to_vec();
        international_title_bytes.resize(48, 0);
        data[INTL_TITLE_START..INTL_TITLE_END].copy_from_slice(&international_title_bytes);

        // Region Code byte at 0x1F0
        data[REGION_CODE_BYTE] = region_byte;

        data
    }

    #[test]
    fn test_analyze_genesis_data_usa() -> Result<(), Box<dyn Error>> {
        let data =
            generate_genesis_header(b"SEGA MEGA DRIVE ", b'U', "DOMESTIC US", "INTERNATIONAL US");
        let analysis = analyze_genesis_data(&data, "test_rom_us.md")?;

        assert_eq!(analysis.source_name, "test_rom_us.md");
        assert_eq!(analysis.console_name, "SEGA MEGA DRIVE");
        assert_eq!(analysis.game_title_domestic, "DOMESTIC US");
        assert_eq!(analysis.game_title_international, "INTERNATIONAL US");
        assert_eq!(analysis.region_code_byte, b'U');
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA (NTSC-U)");
        Ok(())
    }

    #[test]
    fn test_analyze_genesis_data_japan() -> Result<(), Box<dyn Error>> {
        let data =
            generate_genesis_header(b"SEGA MEGA DRIVE ", b'J', "DOMESTIC JP", "INTERNATIONAL JP");
        let analysis = analyze_genesis_data(&data, "test_rom_jp.md")?;

        assert_eq!(analysis.source_name, "test_rom_jp.md");
        assert_eq!(analysis.console_name, "SEGA MEGA DRIVE");
        assert_eq!(analysis.game_title_domestic, "DOMESTIC JP");
        assert_eq!(analysis.game_title_international, "INTERNATIONAL JP");
        assert_eq!(analysis.region_code_byte, b'J');
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC-J)");
        Ok(())
    }

    #[test]
    fn test_analyze_genesis_data_europe() -> Result<(), Box<dyn Error>> {
        let data = generate_genesis_header(
            b"SEGA MEGA DRIVE ",
            b'E',
            "DOMESTIC EUR",
            "INTERNATIONAL EUR",
        );
        let analysis = analyze_genesis_data(&data, "test_rom_eur.md")?;

        assert_eq!(analysis.source_name, "test_rom_eur.md");
        assert_eq!(analysis.console_name, "SEGA MEGA DRIVE");
        assert_eq!(analysis.game_title_domestic, "DOMESTIC EUR");
        assert_eq!(analysis.game_title_international, "INTERNATIONAL EUR");
        assert_eq!(analysis.region_code_byte, b'E');
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe (PAL)");
        Ok(())
    }

    #[test]
    fn test_analyze_genesis_data_genesis_signature() -> Result<(), Box<dyn Error>> {
        let data = generate_genesis_header(b"SEGA GENESIS    ", b'U', "GENESIS DOM", "GENESIS INT");
        let analysis = analyze_genesis_data(&data, "test_rom_genesis.gen")?;

        assert_eq!(analysis.source_name, "test_rom_genesis.gen");
        assert_eq!(analysis.console_name, "SEGA GENESIS");
        assert_eq!(analysis.region_code_byte, b'U');
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA (NTSC-U)");
        Ok(())
    }

    #[test]
    fn test_analyze_genesis_data_unknown_region() -> Result<(), Box<dyn Error>> {
        let data = generate_genesis_header(
            b"SEGA MEGA DRIVE ",
            b'Z',
            "DOMESTIC UNK",
            "INTERNATIONAL UNK",
        );
        let analysis = analyze_genesis_data(&data, "test_rom_unknown.md")?;

        assert_eq!(analysis.source_name, "test_rom_unknown.md");
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown Code");
        assert_eq!(analysis.region_code_byte, b'Z');
        Ok(())
    }

    #[test]
    fn test_analyze_genesis_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 100]; // Smaller than 0x200
        let result = analyze_genesis_data(&data, "too_small.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
