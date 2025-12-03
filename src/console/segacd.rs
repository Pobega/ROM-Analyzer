//use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;
use std::error::Error;
use log::error;

/// Struct to hold the analysis results for a Sega CD ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct SegaCdAnalysis {
    /// The detected signature from the boot file (e.g., "SEGA CD", "SEGA MEGA").
    pub signature: String,
    /// The identified region name (e.g., "Japan (NTSC-J)").
    pub region: String,
    /// The raw region code byte.
    pub region_code: u8,
    /// The name of the source file.
    pub source_name: String,
}

impl SegaCdAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        print_separator();
        println!("Source:       {}", self.source_name);
        println!("System:       Sega CD / Mega CD");
        println!("Signature:    {}", self.signature);
        println!("Region Code:  0x{:02X}", self.region_code);
        println!("Region:       {}", self.region);

        // The check_region_mismatch macro is called here, assuming it's available in scope.
        // It's important that the caller ensures this macro is accessible.
        //check_region_mismatch!(self.source_name, &self.region);
        print_separator();
    }
}

/// Analyzes Sega CD ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
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

    let region_name = match region_code {
        0x40 => "Japan (NTSC-J)",
        0x80 => "Europe (PAL)",
        0xC0 => "USA (NTSC-U)",
        0x00 => "Unrestricted/BIOS region", // May indicate region-free or BIOS-dependent.
        _ => "Unknown Code",
    }
    .to_string();

    // If the signature is not recognized, we might still proceed if the region byte is present,
    // but a warning could be logged or returned.
    if signature != "SEGA CD" && signature != "SEGA MEGA" {
        error!(
            "[!] Warning: File does not appear to be a standard Sega CD boot file (no SEGA CD or SEGA MEGA signature at 0x100) for {}. Found: '{}'",
            source_name, signature
        );
    }

    Ok(SegaCdAnalysis {
        signature,
        region: region_name,
        region_code,
        source_name: source_name.to_string(),
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
        assert_eq!(analysis.region, "Japan (NTSC-J)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_europe() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0x80); // Europe region
        let analysis = analyze_segacd_data(&data, "test_rom_eur.iso")?;

        assert_eq!(analysis.source_name, "test_rom_eur.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0x80);
        assert_eq!(analysis.region, "Europe (PAL)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_usa() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0xC0); // USA region
        let analysis = analyze_segacd_data(&data, "test_rom_us.iso")?;

        assert_eq!(analysis.source_name, "test_rom_us.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0xC0);
        assert_eq!(analysis.region, "USA (NTSC-U)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_unrestricted() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0x00); // Unrestricted region
        let analysis = analyze_segacd_data(&data, "test_rom_unrestricted.iso")?;

        assert_eq!(analysis.source_name, "test_rom_unrestricted.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0x00);
        assert_eq!(analysis.region, "Unrestricted/BIOS region");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_mega_signature() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA MEGA", 0x40); // Japan region
        let analysis = analyze_segacd_data(&data, "test_rom_mega_jp.iso")?;

        assert_eq!(analysis.source_name, "test_rom_mega_jp.iso");
        assert_eq!(analysis.signature, "SEGA MEGA");
        assert_eq!(analysis.region_code, 0x40);
        assert_eq!(analysis.region, "Japan (NTSC-J)");
        Ok(())
    }

    #[test]
    fn test_analyze_segacd_data_unknown_code() -> Result<(), Box<dyn Error>> {
        let data = generate_segacd_header("SEGA CD", 0xFF); // Unknown region code
        let analysis = analyze_segacd_data(&data, "test_rom_unknown.iso")?;

        assert_eq!(analysis.source_name, "test_rom_unknown.iso");
        assert_eq!(analysis.signature, "SEGA CD");
        assert_eq!(analysis.region_code, 0xFF);
        assert_eq!(analysis.region, "Unknown Code");
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
