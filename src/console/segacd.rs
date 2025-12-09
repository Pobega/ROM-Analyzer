//! Provides header analysis functionality for Sega CD (also known as Mega CD) ROMs.
//!
//! This module can parse Sega CD boot file headers to extract signature and region information.
//!
//! SegaCD header documentation referenced here:
//! <https://segaretro.org/ROM_header>

use std::error::Error;

use log::error;
use serde::Serialize;

use crate::error::RomAnalyzerError;
use crate::region::{Region, check_region_mismatch};

/// Struct to hold the analysis results for a Sega CD ROM.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct SegaCdAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region(s) as a region::Region bitmask.
    pub region: Region,
    /// The identified region name (e.g., "Japan (NTSC-J)").
    pub region_string: String,
    /// If the region in the ROM header doesn't match the region in the filename.
    pub region_mismatch: bool,
    /// The raw region code byte.
    pub region_code: u8,
    /// The detected signature from the boot file (e.g., "SEGA CD", "SEGA MEGA").
    pub signature: String,
}

impl SegaCdAnalysis {
    /// Returns a printable String of the analysis results.
    pub fn print(&self) -> String {
        format!(
            "{}\n\
             System:       Sega CD / Mega CD\n\
             Signature:    {}\n\
             Region Code:  0x{:02X}\n\
             Region:       {}",
            self.source_name, self.signature, self.region_code, self.region
        )
    }
}

/// Analyzes Sega CD ROM data.
///
/// This function reads the Sega CD boot program header to extract its signature
/// (e.g., "SEGA CD", "SEGA MEGA") and the region code byte. It then maps the region
/// code to a human-readable region name and performs a region mismatch check against
/// the `source_name`. A warning is logged if an unexpected signature is found.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) containing the raw ROM data (e.g., from a `.bin` or `.iso` file).
/// * `source_name` - The name of the ROM file, used for logging and region mismatch checks.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(SegaCdAnalysis)` containing the detailed analysis results.
/// - `Err(Box<dyn Error>)` if the ROM data is too small to contain a valid Sega CD header.
pub fn analyze_segacd_data(
    data: &[u8],
    source_name: &str,
) -> Result<SegaCdAnalysis, Box<dyn Error>> {
    // The Sega CD boot program header information is typically found early in the file.
    // A common minimum size to check for the signature and region byte is 0x200 bytes.
    const REQUIRED_SIZE: usize = 0x200;
    if data.len() < REQUIRED_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "Sega CD boot file too small (size: {} bytes, requires at least {} bytes).",
            data.len(),
            REQUIRED_SIZE
        ))));
    }

    // Extract the signature from the boot program (typically at offset 0x100).
    // It's often "SEGA CD" or "SEGA MEGA".
    let signature_bytes = &data[0x100..0x109];
    let signature = String::from_utf8_lossy(signature_bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_string();

    // Region byte is at offset 0x10B in the boot program.
    let region_code = data[0x10B];

    let (region_name, region) = match region_code {
        0x40 => ("Japan (NTSC-J)", Region::JAPAN),
        0x80 => ("Europe (PAL)", Region::EUROPE),
        0xC0 => ("USA (NTSC-U)", Region::USA),
        0x00 => (
            // May indicate region-free or BIOS-dependent.
            "Unrestricted/BIOS region",
            Region::USA | Region::EUROPE | Region::JAPAN,
        ),
        _ => ("Unknown Code", Region::UNKNOWN),
    };

    // If the signature is not recognized, we might still proceed if the region byte is present,
    // but a warning could be logged or returned.
    if signature != "SEGA CD" && signature != "SEGA MEGA" {
        error!(
            "[!] Warning: File does not appear to be a standard Sega CD boot file (no SEGA CD or SEGA MEGA signature at 0x100) for {}. Found: '{}'",
            source_name, signature
        );
    }

    let region_mismatch = check_region_mismatch(source_name, region);

    Ok(SegaCdAnalysis {
        source_name: source_name.to_string(),
        region,
        region_string: region_name.to_string(),
        region_mismatch,
        region_code,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Helper function to generate a minimal Sega CD boot file header for testing.
    fn generate_segacd_header(signature_str: &str, region_byte: u8) -> Vec<u8> {
        let mut data = vec![0; 0x200]; // Ensure enough space for signature and region byte.

        const SIG_MAX_LEN: usize = 9;
        let mut signature_bytes = signature_str.as_bytes().to_vec();
        if signature_bytes.len() > SIG_MAX_LEN {
            panic!("Signature must be <= 9 bytes");
        }
        signature_bytes.resize(SIG_MAX_LEN, 0);

        data[0x100..0x109].copy_from_slice(&signature_bytes);

        // Region Code byte at 0x10B
        data[0x10B] = region_byte;

        data
    }

    #[test]
    fn test_analyze_segacd_data_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0x40); // Japan region
        let analysis = analyze_segacd_data(&data, "test_rom_jp.iso")?;

        assert_eq!(analysis.source_name, "test_rom_jp.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0x40);
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC-J)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_europe() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0x80); // Europe region
        let analysis = analyze_segacd_data(&data, "test_rom_eur.iso")?;

        assert_eq!(analysis.source_name, "test_rom_eur.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0x80);
        assert_eq!(analysis.region, Region::EUROPE);
        assert_eq!(analysis.region_string, "Europe (PAL)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_usa() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0xC0); // USA region
        let analysis = analyze_segacd_data(&data, "test_rom_us.iso")?;

        assert_eq!(analysis.source_name, "test_rom_us.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0xC0);
        assert_eq!(analysis.region, Region::USA);
        assert_eq!(analysis.region_string, "USA (NTSC-U)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_unrestricted() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0x00); // Unrestricted region
        let analysis = analyze_segacd_data(&data, "test_rom_unrestricted.iso")?;

        assert_eq!(analysis.source_name, "test_rom_unrestricted.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0x00);
        assert_eq!(
            analysis.region,
            Region::USA | Region::EUROPE | Region::JAPAN
        );
        assert_eq!(analysis.region_string, "Unrestricted/BIOS region");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_mega_signature() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA MEGA", 0x40); // Japan region
        let analysis = analyze_segacd_data(&data, "test_rom_mega_jp.iso")?;

        assert_eq!(analysis.source_name, "test_rom_mega_jp.iso");
        assert_eq!(analysis.signature, "SEGA MEGA");
        assert_eq!(analysis.region_code, 0x40);
        assert_eq!(analysis.region, Region::JAPAN);
        assert_eq!(analysis.region_string, "Japan (NTSC-J)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_unknown_code() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0xFF); // Unknown region code
        let analysis = analyze_segacd_data(&data, "test_rom_unknown.iso")?;

        assert_eq!(analysis.source_name, "test_rom_unknown.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0xFF);
        assert_eq!(analysis.region, Region::UNKNOWN);
        assert_eq!(analysis.region_string, "Unknown Code");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 100]; // Smaller than 0x200
        let result = analyze_segacd_data(&data, "too_small.iso");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
