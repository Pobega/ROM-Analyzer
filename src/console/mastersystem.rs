//! Provides header analysis functionality for Sega Master System ROMs.
//!
//! This module can parse Master System ROM headers to extract region information.
//!
//! Master System header documentation referenced here:
//! <https://www.smspower.org/Development/ROMHeader>

use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

/// Struct to hold the analysis results for a Master System ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct MasterSystemAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "Japan (NTSC)").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The raw region byte value.
    pub region_byte: u8,
}

impl MasterSystemAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        format!(
            "{}\n\
             System:       Sega Master System\n\
             Region Code:  0x{:02X}\n\
             Region:       {}",
            self.source_name, self.region_byte, self.region
        )
    }
}

/// Determines the Sega Master System game region name based on a given region byte.
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
/// - A `&'static str` representing the region as written in the ROM header (e.g., "Japan (NTSC-J)",
///   "Europe / Overseas (PAL/NTSC)") or "Unknown" if the region code is not recognized.
/// - A [`Region`] bitmask representing the region(s) associated with the code.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::console::mastersystem::map_region;
/// use rom_analyzer::region::Region;
///
/// let (region_str, region_mask) = map_region(0x30);
/// assert_eq!(region_str, "Japan (NTSC)");
/// assert_eq!(region_mask, Region::JAPAN);
///
/// let (region_str, region_mask) = map_region(0x4C);
/// assert_eq!(region_str, "Europe / Overseas (PAL/NTSC)");
/// assert_eq!(region_mask, Region::USA | Region::EUROPE);
///
/// let (region_str, region_mask) = map_region(0x99);
/// assert_eq!(region_str, "Unknown");
/// assert_eq!(region_mask, Region::UNKNOWN);
/// ```
pub fn map_region(region_byte: u8) -> (&'static str, Region) {
    match region_byte {
        0x30 => ("Japan (NTSC)", Region::JAPAN),
        0x4C => ("Europe / Overseas (PAL/NTSC)", Region::USA | Region::EUROPE),
        _ => ("Unknown", Region::UNKNOWN),
    }
}

/// Analyzes Master System ROM data.
///
/// This function reads the Master System ROM header to extract the region byte.
/// It then maps the region byte to a human-readable region name and performs
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
/// - `Ok`([`MasterSystemAnalysis`]) containing the detailed analysis results.
/// - `Err`([`RomAnalyzerError`]) if the ROM data is too small to contain the region byte.
pub fn analyze_mastersystem_data(
    data: &[u8],
    source_name: &str,
) -> Result<MasterSystemAnalysis, RomAnalyzerError> {
    // SMS Region/Language byte is at offset 0x7FFC.
    // The header size for SMS is not strictly defined in a way that guarantees a fixed length for all ROMs,
    // but 0x7FFD is a common size for the data containing this byte.
    const REQUIRED_SIZE: usize = 0x7FFD;
    if data.len() < REQUIRED_SIZE {
        return Err(RomAnalyzerError::DataTooSmall {
            file_size: data.len(),
            required_size: REQUIRED_SIZE,
            details: "Master System region byte".to_string(),
        });
    }

    let sms_region_byte = data[0x7FFC];
    let (region_name, region) = map_region(sms_region_byte);

    let region_mismatch = check_region_mismatch(source_name, region);

    Ok(MasterSystemAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        region_byte: sms_region_byte,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_mastersystem_data_japan() -> Result<(), RomAnalyzerError> {
        let mut data = vec![0; 0x7FFD];
        data[0x7FFC] = 0x30; // Japan region
        let analysis = analyze_mastersystem_data(&data, "test_rom_jp.sms")?;

        assert_eq!(analysis.source_name, "test_rom_jp.sms");
        assert_eq!(analysis.region_byte, 0x30);
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC)");
        assert_eq!(
            analysis.print(),
            "test_rom_jp.sms\n\
             System:       Sega Master System\n\
             Region Code:  0x30\n\
             Region:       Japan"
        );
        Ok(())
    }

    #[test]
    fn test_analyze_mastersystem_data_europe() -> Result<(), RomAnalyzerError> {
        let mut data = vec![0; 0x7FFD];
        data[0x7FFC] = 0x4C; // Europe / Overseas region
        let analysis = analyze_mastersystem_data(&data, "test_rom_eur.sms")?;

        assert_eq!(analysis.source_name, "test_rom_eur.sms");
        assert_eq!(analysis.region_byte, 0x4C);
        assert_eq!(analysis.region, Region::USA | Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe / Overseas (PAL/NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_mastersystem_data_unknown() -> Result<(), RomAnalyzerError> {
        let mut data = vec![0; 0x7FFD];
        data[0x7FFC] = 0x00; // Unknown region
        let analysis = analyze_mastersystem_data(&data, "test_rom.sms")?;

        assert_eq!(analysis.source_name, "test_rom.sms");
        assert_eq!(analysis.region_byte, 0x00);
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown");
        Ok(())
    }

    #[test]
    fn test_analyze_mastersystem_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 100]; // Smaller than 0x7FFD
        let result = analyze_mastersystem_data(&data, "too_small.sms");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
