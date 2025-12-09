//! Provides header analysis functionality for Sony PlayStation (PSX) ROMs, typically in CD image formats.
//!
//! This module focuses on identifying the region of PSX games by searching for known
//! executable prefixes (e.g., "SLUS", "SLES", "SLPS") within the initial data tracks.

use std::error::Error;

use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

/// Struct to hold the analysis results for a PSX ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PsxAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "North America (NTSC-U)").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The identified region code (e.g., "SLUS").
    pub code: String,
}

impl PsxAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        let executable_prefix_not_found = if self.code == "N/A" {
            "\nNote: Executable prefix (SLUS/SLES/SLPS) not found in header area. Requires main data track (.bin or .iso)."
        } else {
            ""
        };
        format!(
            "{}\n\
             System:       Sony PlayStation (PSX)\n\
             Region:       {}\n\
             Code:         {}\
             {}",
            self.source_name, self.region, self.code, executable_prefix_not_found
        )
    }
}

/// Analyzes PlayStation (PSX) ROM data, typically from CD images.
///
/// This function scans a portion of the ROM data (up to `0x20000` bytes) for
/// common PSX executable prefixes like "SLUS", "SLES", or "SLPS". These prefixes
/// indicate the game's region. If a prefix is found, the corresponding region
/// and code are extracted. A region mismatch check is also performed against the `source_name`.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) containing the raw ROM data (e.g., from a `.bin` or `.iso` file).
/// * `source_name` - The name of the ROM file, used for region mismatch checks.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(PsxAnalysis)` containing the detailed analysis results.
/// - `Err(Box<dyn Error>)` if the ROM data is too small for reliable analysis.
pub fn analyze_psx_data(data: &[u8], source_name: &str) -> Result<PsxAnalysis, Box<dyn Error>> {
    // Check the first 128KB (0x20000 bytes)
    let check_size = std::cmp::min(data.len(), 0x20000);
    if check_size < 0x2000 {
        // Need enough data for Volume Descriptor/Boot file
        return Err(Box::new(RomAnalyzerError::new(
            "PSX boot file too small for reliable analysis.",
        )));
    }

    let data_sample = &data[..check_size];

    let region_map = [
        ("SLUS".as_bytes(), "North America (NTSC-U)", Region::USA),
        ("SLES".as_bytes(), "Europe (PAL)", Region::EUROPE),
        ("SLPS".as_bytes(), "Japan (NTSC-J)", Region::JAPAN),
    ];

    let mut found_code = "N/A".to_string();
    let mut region_name = "Unknown";
    let mut region = Region::UNKNOWN;

    for (prefix, region_string, region_struct) in region_map.iter() {
        // Use windows to check for the prefix anywhere in the sample.
        if data_sample
            .windows(prefix.len())
            .any(|window| window.eq_ignore_ascii_case(prefix))
        {
            found_code = String::from_utf8_lossy(prefix).to_string();
            region_name = region_string;
            region = region_struct.clone();
            break;
        }
    }

    let region_mismatch = check_region_mismatch(source_name, &region_name);

    Ok(PsxAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        code: found_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_analyze_psx_data_slus() -> Result<(), Box<dyn Error>> {
        // Ensure sufficient data for analysis, at least 0x2000 bytes.
        let mut data = vec![0; 0x2000];
        // Place the region code at an offset where it's expected.
        data[0x100..0x104].copy_from_slice(b"SLUS"); // North America
        let analysis = analyze_psx_data(&data, "test_rom_us.iso")?;

        assert_eq!(analysis.source_name, "test_rom_us.iso");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "North America (NTSC-U)");
        assert_eq!(analysis.code, "SLUS");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_sles() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x2000];
        data[0x100..0x104].copy_from_slice(b"SLES"); // Europe
        let analysis = analyze_psx_data(&data, "test_rom_eur.iso")?;

        assert_eq!(analysis.source_name, "test_rom_eur.iso");
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe (PAL)");
        assert_eq!(analysis.code, "SLES");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_slps() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x2000];
        data[0x100..0x104].copy_from_slice(b"SLPS"); // Japan
        let analysis = analyze_psx_data(&data, "test_rom_jp.iso")?;

        assert_eq!(analysis.source_name, "test_rom_jp.iso");
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC-J)");
        assert_eq!(analysis.code, "SLPS");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_unknown() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x2000];
        // No known prefix
        let analysis = analyze_psx_data(&data, "test_rom.iso")?;

        assert_eq!(analysis.source_name, "test_rom.iso");
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown");
        assert_eq!(analysis.code, "N/A");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 100]; // Smaller than 0x2000
        let result = analyze_psx_data(&data, "too_small.iso");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_analyze_psx_data_case_insensitivity() -> Result<(), Box<dyn Error>> {
        // Test that the matching is case-insensitive.
        let mut data = vec![0; 0x2000];
        data[0x100..0x104].copy_from_slice(b"sLuS"); // Mixed case
        let analysis = analyze_psx_data(&data, "test_rom_mixedcase.iso")?;

        assert_eq!(analysis.source_name, "test_rom_mixedcase.iso");
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "North America (NTSC-U)");
        assert_eq!(analysis.code, "SLUS");
        Ok(())
    }
}
