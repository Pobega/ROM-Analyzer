//! Provides header analysis functionality for Nintendo 64 (N64) ROMs.
//!
//! This module can parse N64 ROM headers to extract country code and infer
//! the geographical region.
//!
//! N64 header documentation referenced here:
//! <https://en64.shoutwiki.com/wiki/ROM>

use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

/// Struct to hold the analysis results for an N64 ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct N64Analysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "USA (NTSC)").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The country code extracted from the ROM header (e.g., "E", "J").
    pub country_code: String,
}

impl N64Analysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        format!(
            "{}\n\
             System:       Nintendo 64 (N64)\n\
             Region:       {}\n\
             Code:         {}",
            self.source_name, self.region, self.country_code
        )
    }
}

/// Determines the N64 game region based on a given country code.
///
/// The country code typically comes from the ROM header. This function maps it to a
/// human-readable region string and a Region bitmask.
///
/// # Arguments
///
/// * `country_code` - The country code string, usually found in the ROM header.
///
/// # Returns
///
/// A tuple containing:
/// - A `&'static str` representing the region (e.g., "USA (NTSC)", "Japan (NTSC)", etc)
///   or "Unknown" if the country code is not recognized.
/// - A `Region` bitmask representing the region(s) associated with the code.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::console::n64::map_region;
/// use rom_analyzer::region::Region;
///
/// let (region_str, region_mask) = map_region("E");
/// assert_eq!(region_str, "USA (NTSC)");
/// assert_eq!(region_mask, Region::USA);
///
/// let (region_str, region_mask) = map_region("J");
/// assert_eq!(region_str, "Japan (NTSC)");
/// assert_eq!(region_mask, Region::JAPAN);
///
/// let (region_str, region_mask) = map_region("P");
/// assert_eq!(region_str, "Europe (PAL)");
/// assert_eq!(region_mask, Region::EUROPE);
///
/// let (region_str, region_mask) = map_region("X");
/// assert_eq!(region_str, "Unknown");
/// assert_eq!(region_mask, Region::UNKNOWN);
/// ```
pub fn map_region(country_code: &str) -> (&'static str, Region) {
    match country_code {
        "E" => ("USA (NTSC)", Region::USA),
        "J" => ("Japan (NTSC)", Region::JAPAN),
        "P" => ("Europe (PAL)", Region::EUROPE),
        "D" => ("Germany (PAL)", Region::EUROPE),
        "F" => ("France (PAL)", Region::EUROPE),
        "U" => ("USA (Legacy)", Region::USA),
        _ => ("Unknown", Region::UNKNOWN),
    }
}

/// Analyzes N64 ROM data.
///
/// This function reads the N64 ROM header to extract the country code.
/// It then maps the country code to a human-readable region name and performs
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
/// - `Ok`([`N64Analysis`]) containing the detailed analysis results.
/// - `Err`([`RomAnalyzerError`]) if the ROM data is too small to contain a valid N64 header.
pub fn analyze_n64_data(data: &[u8], source_name: &str) -> Result<N64Analysis, RomAnalyzerError> {
    // N64 header is at offset 0x0. Country code is at offset 0x3E (2 bytes).
    const HEADER_SIZE: usize = 0x40;
    if data.len() < HEADER_SIZE {
        return Err(RomAnalyzerError::DataTooSmall {
            file_size: data.len(),
            required_size: HEADER_SIZE,
            details: "N64 header".to_string(),
        });
    }

    // Extract Country Code (2 bytes, ASCII)
    // The second byte is often a null terminator, or part of a two-character code.
    let country_code = String::from_utf8_lossy(&data[0x3E..0x40])
        .trim_matches(char::from(0))
        .to_string();

    // Determine region name based on the country code.
    let (region_name, region) = map_region(&country_code);

    let region_mismatch = check_region_mismatch(source_name, region);

    Ok(N64Analysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        country_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to generate a minimal N64 header for testing.
    fn generate_n64_header(country_code: &str) -> Vec<u8> {
        let mut data = vec![0; 0x40]; // Ensure enough space for header

        // Country Code (2 bytes at 0x3E)
        let mut cc_bytes = country_code.as_bytes().to_vec();
        cc_bytes.resize(2, 0);
        data[0x3E..0x40].copy_from_slice(&cc_bytes);

        data
    }

    #[test]
    fn test_analyze_n64_data_usa() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("E"); // USA region
        let analysis = analyze_n64_data(&data, "test_rom_us.n64")?;

        assert_eq!(analysis.source_name, "test_rom_us.n64");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA (NTSC)");
        assert_eq!(analysis.country_code, "E");
        assert_eq!(
            analysis.print(),
            "test_rom_us.n64\n\
             System:       Nintendo 64 (N64)\n\
             Region:       USA\n\
             Code:         E"
        );
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_japan() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("J"); // Japan region
        let analysis = analyze_n64_data(&data, "test_rom_jp.n64")?;

        assert_eq!(analysis.source_name, "test_rom_jp.n64");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC)");
        assert_eq!(analysis.country_code, "J");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_europe() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("P"); // Europe region
        let analysis = analyze_n64_data(&data, "test_rom_eur.n64")?;

        assert_eq!(analysis.source_name, "test_rom_eur.n64");
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe (PAL)");
        assert_eq!(analysis.country_code, "P");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_germany() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("D"); // Germany region
        let analysis = analyze_n64_data(&data, "test_rom_deu.n64")?;

        assert_eq!(analysis.source_name, "test_rom_deu.n64");
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Germany (PAL)");
        assert_eq!(analysis.country_code, "D");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_france() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("F"); // France region
        let analysis = analyze_n64_data(&data, "test_rom_fra.n64")?;

        assert_eq!(analysis.source_name, "test_rom_fra.n64");
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "France (PAL)");
        assert_eq!(analysis.country_code, "F");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_legacy_usa() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("U"); // Legacy USA region
        let analysis = analyze_n64_data(&data, "test_rom_usa_legacy.n64")?;

        assert_eq!(analysis.source_name, "test_rom_usa_legacy.n64");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA (Legacy)");
        assert_eq!(analysis.country_code, "U");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_unknown() -> Result<(), RomAnalyzerError> {
        let data = generate_n64_header("X"); // Unknown region
        let analysis = analyze_n64_data(&data, "test_rom.n64")?;

        assert_eq!(analysis.source_name, "test_rom.n64");
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown");
        assert_eq!(analysis.country_code, "X");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 30]; // Smaller than 0x40
        let result = analyze_n64_data(&data, "too_small.n64");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
