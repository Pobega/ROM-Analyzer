/// NES header documentation referenced here:
/// https://www.nesdev.org/wiki/INES
/// https://www.nesdev.org/wiki/NES_2.0
use std::error::Error;

use crate::error::RomAnalyzerError;
use log::info;

const INES_REGION_BYTE: usize = 9;
const INES_REGION_MASK: u8 = 0x01;

const NES2_REGION_BYTE: usize = 12;
const NES2_REGION_MASK: u8 = 0x03;
const NES2_FORMAT_BYTE: usize = 7;
const NES2_FORMAT_MASK: u8 = 0x0C;
const NES2_FORMAT_EXPECTED_VALUE: u8 = 0x08;

/// Struct to hold the analysis results for a NES ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct NesAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region name (e.g., "NTSC (USA/Japan)").
    pub region: String,
    /// The raw byte value used for region determination (from iNES flag 9 or NES2 flag 12).
    pub region_byte_value: u8,
    /// Whether the ROM header is in NES 2.0 format.
    pub is_nes2_format: bool,
}

impl NesAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        let nes_flag_display = if self.is_nes2_format {
            format!("\n    NES2.0 Flag 12: 0x{:02X}", self.region_byte_value)
        } else {
            format!("\n    iNES Flag 9:  0x{:02X}", self.region_byte_value)
        };

        info!(
            "{}\n\
             System:       Nintendo Entertainment System (NES)\n\
             Region:       {}\
             {}",
            self.source_name, self.region, nes_flag_display
        );
    }
}

pub fn get_nes_region_name(region_byte: u8, nes2_format: bool) -> &'static str {
    if nes2_format {
        // NES 2.0 headers store region data in the CPU/PPU timing bit
        // in byte 12.
        match region_byte & NES2_REGION_MASK {
            0 => "NTSC (USA/Japan)",
            1 => "PAL (Europe/Oceania)",
            2 => "Multi-region",
            3 => "Dendy (Russia)",
            _ => "Unknown",
        }
    } else {
        // iNES headers store region data in byte 9.
        // It is only the lowest-order bit for NTSC vs PAL.
        // NTSC covers USA and Japan.
        match region_byte & INES_REGION_MASK {
            0 => "NTSC (USA/Japan)",
            1 => "PAL (Europe/Oceania)",
            _ => "Unknown",
        }
    }
}

/// Analyzes NES ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
pub fn analyze_nes_data(data: &[u8], source_name: &str) -> Result<NesAnalysis, Box<dyn Error>> {
    if data.len() < 16 {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain an iNES header (size: {} bytes).",
            data.len()
        ))));
    }

    // All headered NES ROMs should begin with 'NES<EOF>'
    let signature = &data[0..4];
    if signature != b"NES\x1a" {
        return Err(Box::new(RomAnalyzerError::new(
            "Invalid iNES header signature. Not a valid NES ROM.",
        )));
    }

    let mut region_byte_val = data[INES_REGION_BYTE];
    let is_nes2_format = (data[NES2_FORMAT_BYTE] & NES2_FORMAT_MASK) == NES2_FORMAT_EXPECTED_VALUE;

    if is_nes2_format {
        region_byte_val = data[NES2_REGION_BYTE];
    }

    let region_name = get_nes_region_name(region_byte_val, is_nes2_format);

    Ok(NesAnalysis {
        region: region_name.to_string(),
        is_nes2_format,
        region_byte_value: region_byte_val,
        source_name: source_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // Helper enum to specify header type for generation.
    enum NesHeaderType {
        Ines,
        Nes2,
    }

    /// Generates a 16-byte NES ROM header for testing.
    /// configures the header to be either iNES or NES 2.0 format,
    /// and sets the specified region value.
    fn generate_nes_header(header_type: NesHeaderType, region_value: u8) -> Vec<u8> {
        let mut data = vec![0; 16];
        data[0..4].copy_from_slice(b"NES\x1a"); // Signature

        match header_type {
            NesHeaderType::Ines => {
                // iNES format: region is in byte 9. Only the LSB (INES_REGION_MASK) matters.
                // We set the byte and let get_nes_region_name handle the masking.
                data[INES_REGION_BYTE] = region_value;
                // Ensure NES 2.0 flags are NOT set in byte 7.
                data[NES2_FORMAT_BYTE] &= !NES2_FORMAT_MASK;
            }
            NesHeaderType::Nes2 => {
                // NES 2.0 format: set NES 2.0 identification bits in byte 7.
                data[NES2_FORMAT_BYTE] |= NES2_FORMAT_EXPECTED_VALUE;
                // Region is in byte 12, masked by NES2_REGION_MASK.
                // We set the byte and let get_nes_region_name handle the masking.
                data[NES2_REGION_BYTE] = region_value;
            }
        }
        data
    }

    #[test]
    fn test_analyze_ines_data_ntsc() -> Result<(), Box<dyn Error>> {
        // iNES format, NTSC region (LSB is 0)
        let data = generate_nes_header(NesHeaderType::Ines, 0x00);
        let analysis = analyze_nes_data(&data, "test_rom_ntsc.nes")?;

        assert_eq!(analysis.source_name, "test_rom_ntsc.nes");
        assert_eq!(analysis.region, "NTSC (USA/Japan)");
        assert!(!analysis.is_nes2_format);
        assert_eq!(analysis.region_byte_value, 0x00);
        Ok(())
    }

    #[test]
    fn test_analyze_ines_data_pal() -> Result<(), Box<dyn Error>> {
        // iNES format, PAL region (LSB is 1)
        let data = generate_nes_header(NesHeaderType::Ines, 0x01);
        let analysis = analyze_nes_data(&data, "test_rom_pal.nes")?;

        assert_eq!(analysis.source_name, "test_rom_pal.nes");
        assert_eq!(analysis.region, "PAL (Europe/Oceania)");
        assert!(!analysis.is_nes2_format);
        assert_eq!(analysis.region_byte_value, 0x01);
        Ok(())
    }

    #[test]
    fn test_analyze_nes2_data_ntsc() -> Result<(), Box<dyn Error>> {
        // NES 2.0 format, NTSC region (value 0)
        let data = generate_nes_header(NesHeaderType::Nes2, 0x00);
        let analysis = analyze_nes_data(&data, "test_rom_nes2_ntsc.nes")?;

        assert_eq!(analysis.source_name, "test_rom_nes2_ntsc.nes");
        assert_eq!(analysis.region, "NTSC (USA/Japan)");
        assert!(analysis.is_nes2_format);
        assert_eq!(analysis.region_byte_value, 0x00);
        Ok(())
    }

    #[test]
    fn test_analyze_nes2_data_pal() -> Result<(), Box<dyn Error>> {
        // NES 2.0 format, PAL region (value 1)
        let data = generate_nes_header(NesHeaderType::Nes2, 0x01);
        let analysis = analyze_nes_data(&data, "test_rom_nes2_pal.nes")?;

        assert_eq!(analysis.source_name, "test_rom_nes2_pal.nes");
        assert_eq!(analysis.region, "PAL (Europe/Oceania)");
        assert!(analysis.is_nes2_format);
        assert_eq!(analysis.region_byte_value, 0x01);
        Ok(())
    }

    #[test]
    fn test_analyze_nes2_data_world() -> Result<(), Box<dyn Error>> {
        // NES 2.0 format, Multi-region (value 2)
        let data = generate_nes_header(NesHeaderType::Nes2, 0x02);
        let analysis = analyze_nes_data(&data, "test_rom_nes2_world.nes")?;

        assert_eq!(analysis.source_name, "test_rom_nes2_world.nes");
        assert_eq!(analysis.region, "Multi-region");
        assert!(analysis.is_nes2_format);
        assert_eq!(analysis.region_byte_value, 0x02);
        Ok(())
    }

    #[test]
    fn test_analyze_nes2_data_dendy() -> Result<(), Box<dyn Error>> {
        // NES 2.0 format, Dendy (Russia) (value 3)
        let data = generate_nes_header(NesHeaderType::Nes2, 0x03);
        let analysis = analyze_nes_data(&data, "test_rom_nes2_dendy.nes")?;

        assert_eq!(analysis.source_name, "test_rom_nes2_dendy.nes");
        assert_eq!(analysis.region, "Dendy (Russia)");
        assert!(analysis.is_nes2_format);
        assert_eq!(analysis.region_byte_value, 0x03);
        Ok(())
    }

    #[test]
    fn test_analyze_nes_data_too_small() {
        // Test with data smaller than the header size
        let data = vec![0; 10];
        let result = analyze_nes_data(&data, "too_small.nes");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_analyze_nes_invalid_signature() {
        // Test with incorrect signature
        let mut data = vec![0; 16];
        data[0..4].copy_from_slice(b"XXXX"); // Invalid signature
        let result = analyze_nes_data(&data, "invalid_sig.nes");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid iNES header signature")
        );
    }
}
