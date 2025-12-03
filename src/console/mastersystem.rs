/// https://www.smspower.org/Development/ROMHeader
use crate::error::RomAnalyzerError;
use crate::print_separator;
use std::error::Error;

/// Struct to hold the analysis results for a Master System ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct MasterSystemAnalysis {
    /// The raw region byte value.
    pub region_byte: u8,
    /// The identified region name (e.g., "Japan (NTSC)").
    pub region: String,
    /// The name of the source file.
    pub source_name: String,
}

impl MasterSystemAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        print_separator();
        println!("Source:       {}", self.source_name);
        println!("System:       Sega Master System");
        println!("Region Code:  0x{:02X}", self.region_byte);
        println!("Region:       {}", self.region);

        print_separator();
    }
}

/// Analyzes Master System ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
pub fn analyze_mastersystem_data(
    data: &[u8],
    source_name: &str,
) -> Result<MasterSystemAnalysis, Box<dyn Error>> {
    // SMS Region/Language byte is at offset 0x7FFC.
    // The header size for SMS is not strictly defined in a way that guarantees a fixed length for all ROMs,
    // but 0x7FFD is a common size for the data containing this byte.
    const REQUIRED_SIZE: usize = 0x7FFD;
    if data.len() < REQUIRED_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain Master System region byte (size: {} bytes, requires at least {} bytes).",
            data.len(),
            REQUIRED_SIZE
        ))));
    }

    let sms_region_byte = data[0x7FFC];
    let region_name = match sms_region_byte {
        0x30 => "Japan (NTSC)",
        0x4C => "Europe / Overseas (PAL/NTSC)",
        _ => "Unknown Code",
    }
    .to_string();

    Ok(MasterSystemAnalysis {
        region_byte: sms_region_byte,
        region: region_name,
        source_name: source_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_analyze_mastersystem_data_japan() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x7FFD];
        data[0x7FFC] = 0x30; // Japan region
        let analysis = analyze_mastersystem_data(&data, "test_rom_jp.sms")?;

        assert_eq!(analysis.source_name, "test_rom_jp.sms");
        assert_eq!(analysis.region_byte, 0x30);
        assert_eq!(analysis.region, "Japan (NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_mastersystem_data_europe() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x7FFD];
        data[0x7FFC] = 0x4C; // Europe / Overseas region
        let analysis = analyze_mastersystem_data(&data, "test_rom_eur.sms")?;

        assert_eq!(analysis.source_name, "test_rom_eur.sms");
        assert_eq!(analysis.region_byte, 0x4C);
        assert_eq!(analysis.region, "Europe / Overseas (PAL/NTSC)");
        Ok(())
    }

    #[test]
    fn test_analyze_mastersystem_data_unknown() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x7FFD];
        data[0x7FFC] = 0x00; // Unknown region
        let analysis = analyze_mastersystem_data(&data, "test_rom.sms")?;

        assert_eq!(analysis.source_name, "test_rom.sms");
        assert_eq!(analysis.region_byte, 0x00);
        assert_eq!(analysis.region, "Unknown Code");
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
