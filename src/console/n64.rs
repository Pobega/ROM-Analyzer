/// N64 header documentation referenced here:
/// https://en64.shoutwiki.com/wiki/ROM
use std::error::Error;

use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::check_region_mismatch;

/// Struct to hold the analysis results for an N64 ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct N64Analysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region name (e.g., "USA / NTSC").
    pub region: String,
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

/// Analyzes N64 ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
pub fn analyze_n64_data(data: &[u8], source_name: &str) -> Result<N64Analysis, Box<dyn Error>> {
    // N64 header is at offset 0x0. Country code is at offset 0x3E (2 bytes).
    const HEADER_SIZE: usize = 0x40;
    if data.len() < HEADER_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain an N64 header (size: {} bytes, requires at least {} bytes).",
            data.len(),
            HEADER_SIZE
        ))));
    }

    // Extract Country Code (2 bytes, ASCII)
    // The second byte is often a null terminator, or part of a two-character code.
    let country_code = String::from_utf8_lossy(&data[0x3E..0x40])
        .trim_matches(char::from(0))
        .to_string();

    // Determine region name based on the country code.
    let region_name = match country_code.as_ref() {
        "E" => "USA / NTSC",
        "J" => "Japan / NTSC",
        "P" => "Europe / PAL",
        "D" => "Germany / PAL",
        "F" => "France / PAL",
        "U" => "USA (Legacy)", // Sometimes used, though 'E' is more common for US
        _ => "Unknown Code",
    }
    .to_string();

    let region_mismatch = check_region_mismatch(source_name, &region_name);

    Ok(N64Analysis {
        source_name: source_name.to_string(),
        region: region_name,
        region_mismatch,
        country_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

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
    fn test_analyze_n64_data_usa() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("E"); // USA region
        let analysis = analyze_n64_data(&data, "test_rom_us.n64")?;

        assert_eq!(analysis.source_name, "test_rom_us.n64");
        assert_eq!(analysis.region, "USA / NTSC");
        assert_eq!(analysis.country_code, "E");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("J"); // Japan region
        let analysis = analyze_n64_data(&data, "test_rom_jp.n64")?;

        assert_eq!(analysis.source_name, "test_rom_jp.n64");
        assert_eq!(analysis.region, "Japan / NTSC");
        assert_eq!(analysis.country_code, "J");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_europe() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("P"); // Europe region
        let analysis = analyze_n64_data(&data, "test_rom_eur.n64")?;

        assert_eq!(analysis.source_name, "test_rom_eur.n64");
        assert_eq!(analysis.region, "Europe / PAL");
        assert_eq!(analysis.country_code, "P");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_germany() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("D"); // Germany region
        let analysis = analyze_n64_data(&data, "test_rom_deu.n64")?;

        assert_eq!(analysis.source_name, "test_rom_deu.n64");
        assert_eq!(analysis.region, "Germany / PAL");
        assert_eq!(analysis.country_code, "D");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_france() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("F"); // France region
        let analysis = analyze_n64_data(&data, "test_rom_fra.n64")?;

        assert_eq!(analysis.source_name, "test_rom_fra.n64");
        assert_eq!(analysis.region, "France / PAL");
        assert_eq!(analysis.country_code, "F");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_legacy_usa() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("U"); // Legacy USA region
        let analysis = analyze_n64_data(&data, "test_rom_usa_legacy.n64")?;

        assert_eq!(analysis.source_name, "test_rom_usa_legacy.n64");
        assert_eq!(analysis.region, "USA (Legacy)");
        assert_eq!(analysis.country_code, "U");
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_unknown() -> Result<(), Box<dyn Error>> {
        let data = generate_n64_header("X"); // Unknown region
        let analysis = analyze_n64_data(&data, "test_rom.n64")?;

        assert_eq!(analysis.source_name, "test_rom.n64");
        assert_eq!(analysis.region, "Unknown Code");
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
