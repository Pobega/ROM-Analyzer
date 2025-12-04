use log::info;

use crate::error::RomAnalyzerError;
use std::error::Error;

/// Struct to hold the analysis results for a PSX ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct PsxAnalysis {
    /// The name of the source file.
    pub source_name: String,
    /// The identified region name (e.g., "North America (NTSC-U)").
    pub region: String,
    /// The identified region code (e.g., "SLUS").
    pub code: String,
}

impl PsxAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        let executable_prefix_not_found = if self.code == "N/A" {
            "\nNote: Executable prefix (SLUS/SLES/SLPS) not found in header area. Requires main data track (.bin or .iso)."
        } else {
            ""
        };
        info!(
            "{}\n\
             System:       Sony PlayStation (PSX)\n\
             Region:       {}\n\
             Code:         {}\
             {}",
            self.source_name, self.region, self.code, executable_prefix_not_found
        );
    }
}

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
        ("SLUS".as_bytes(), "North America (NTSC-U)"),
        ("SLES".as_bytes(), "Europe (PAL)"),
        ("SLPS".as_bytes(), "Japan (NTSC-J)"),
    ];

    let mut found_code = "N/A".to_string();
    let mut region_name = "Unknown";

    for (prefix, region) in region_map.iter() {
        // Use windows to check for the prefix anywhere in the sample.
        if data_sample
            .windows(prefix.len())
            .any(|window| window.eq_ignore_ascii_case(prefix))
        {
            found_code = String::from_utf8_lossy(prefix).to_string();
            region_name = region;
            break;
        }
    }

    Ok(PsxAnalysis {
        region: region_name.to_string(),
        code: found_code,
        source_name: source_name.to_string(),
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
        assert_eq!(analysis.region, "North America (NTSC-U)");
        assert_eq!(analysis.code, "SLUS");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_sles() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x2000];
        data[0x100..0x104].copy_from_slice(b"SLES"); // Europe
        let analysis = analyze_psx_data(&data, "test_rom_eur.iso")?;

        assert_eq!(analysis.source_name, "test_rom_eur.iso");
        assert_eq!(analysis.region, "Europe (PAL)");
        assert_eq!(analysis.code, "SLES");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_slps() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x2000];
        data[0x100..0x104].copy_from_slice(b"SLPS"); // Japan
        let analysis = analyze_psx_data(&data, "test_rom_jp.iso")?;

        assert_eq!(analysis.source_name, "test_rom_jp.iso");
        assert_eq!(analysis.region, "Japan (NTSC-J)");
        assert_eq!(analysis.code, "SLPS");
        Ok(())
    }

    #[test]
    fn test_analyze_psx_data_unknown() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x2000];
        // No known prefix
        let analysis = analyze_psx_data(&data, "test_rom.iso")?;

        assert_eq!(analysis.source_name, "test_rom.iso");
        assert_eq!(analysis.region, "Unknown");
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
        assert_eq!(analysis.region, "North America (NTSC-U)");
        assert_eq!(analysis.code, "SLUS");
        Ok(())
    }
}
