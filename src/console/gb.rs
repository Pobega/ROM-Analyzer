use std::error::Error;

//use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

const GB_TITLE_START: usize = 0x134;
const GB_TITLE_END: usize = 0x143;
const GB_DESTINATION: usize = 0x14A;

const GBC_SYSTEM_TYPE: usize = 0x143;
const GBC_TITLE_END: usize = 0x13F;

/// Struct to hold the analysis results for a Game Boy ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct GbAnalysis {
    /// The identified system type (e.g., "Game Boy (GB)" or "Game Boy Color (GBC)").
    pub system_type: String,
    /// The game title extracted from the ROM header.
    pub game_title: String,
    /// The raw destination code byte.
    pub destination_code: u8,
    /// The identified region name (e.g., "Japan").
    pub region: String,
    /// The name of the source file.
    pub source_name: String,
}

impl GbAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        print_separator();
        println!("Source:       {}", self.source_name);
        println!("System:       {}", self.system_type);
        println!("Game Title:   {}", self.game_title);
        println!("Region Code:  0x{:02X}", self.destination_code);
        println!("Region:       {}", self.region);

        // The check_region_mismatch macro is called here, assuming it's available in scope.
        // It's important that the caller ensures this macro is accessible.
        //check_region_mismatch!(self.source_name, &self.region);
        print_separator();
    }
}

/// Analyzes Game Boy ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
pub fn analyze_gb_data(data: &[u8], source_name: &str) -> Result<GbAnalysis, Box<dyn Error>> {
    // The Game Boy header is located at offset 0x100.
    // The relevant information for region and system type are within the first 0x150 bytes.
    const HEADER_SIZE: usize = 0x150;
    if data.len() < HEADER_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a Game Boy header (size: {} bytes, requires at least {} bytes).",
            data.len(),
            HEADER_SIZE
        ))));
    }

    // System type is determined by a specific byte in the header.
    // 0x80 or 0xC0 indicates GBC
    let system_type = if data[GBC_SYSTEM_TYPE] == 0x80 || data[GBC_SYSTEM_TYPE] == 0xC0 {
        "Game Boy Color (GBC)"
    } else {
        "Game Boy (GB)"
    };

    let title_end = if system_type == "Game Boy Color (GBC)" {
        GBC_TITLE_END
    } else {
        GB_TITLE_END
    };
    let game_title = String::from_utf8_lossy(&data[GB_TITLE_START..title_end])
        .trim_matches(char::from(0))
        .to_string();

    let destination_code = data[GB_DESTINATION];
    let region_name = match destination_code {
        0x00 => "Japan",
        0x01 => "Non-Japan (International)",
        _ => "Unknown Code",
    };

    Ok(GbAnalysis {
        system_type: system_type.to_string(),
        game_title,
        destination_code,
        region: region_name.to_string(),
        source_name: source_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Helper function to generate a minimal Game Boy header for testing.
    fn generate_gb_header(destination_code: u8, system_byte: u8, title: &str) -> Vec<u8> {
        let mut data = vec![0; 0x150]; // Ensure enough space for header

        // Signature (usually present, but not strictly required for region/system analysis)
        data[0x100..0x104].copy_from_slice(b"LOGO"); // Dummy signature

        // Game Title (11 chars for GBC, 15 for GB)
        let mut title_bytes = title.as_bytes().to_vec();
        let mut title_length = 11;
        // Check if GBC
        if system_byte & 0x80 == 0x00 {
            title_length = 15;
        }
        title_bytes.resize(title_length, 0);
        data[GB_TITLE_START..(GB_TITLE_START + title_length)].copy_from_slice(&title_bytes);

        data[GB_DESTINATION] = destination_code;

        // System Type Byte
        data[GBC_SYSTEM_TYPE] = system_byte;

        data
    }

    #[test]
    fn test_analyze_gb_data_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x00, 0x00, "GAMETITLE"); // Japan, GB
        let analysis = analyze_gb_data(&data, "test_rom_jp.gb")?;

        assert_eq!(analysis.source_name, "test_rom_jp.gb");
        assert_eq!(analysis.system_type, "Game Boy (GB)");
        assert_eq!(analysis.game_title, "GAMETITLE");
        assert_eq!(analysis.destination_code, 0x00);
        assert_eq!(analysis.region, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gb_data_non_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x01, 0x00, "GAMETITLE"); // Non-Japan, GB
        let analysis = analyze_gb_data(&data, "test_rom_us.gb")?;

        assert_eq!(analysis.source_name, "test_rom_us.gb");
        assert_eq!(analysis.system_type, "Game Boy (GB)");
        assert_eq!(analysis.game_title, "GAMETITLE");
        assert_eq!(analysis.destination_code, 0x01);
        assert_eq!(analysis.region, "Non-Japan (International)");
        Ok(())
    }

    #[test]
    fn test_analyze_gbc_data_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x00, 0x80, "GBC TITLE"); // Japan, GBC
        let analysis = analyze_gb_data(&data, "test_rom_jp.gbc")?;

        assert_eq!(analysis.source_name, "test_rom_jp.gbc");
        assert_eq!(analysis.system_type, "Game Boy Color (GBC)");
        assert_eq!(analysis.game_title, "GBC TITLE");
        assert_eq!(analysis.destination_code, 0x00);
        assert_eq!(analysis.region, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gbc_data_non_japan() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x01, 0xC0, "GBC TITLE"); // Non-Japan, GBC (using 0xC0 for system byte)
        let analysis = analyze_gb_data(&data, "test_rom_eur.gbc")?;

        assert_eq!(analysis.source_name, "test_rom_eur.gbc");
        assert_eq!(analysis.system_type, "Game Boy Color (GBC)");
        assert_eq!(analysis.game_title, "GBC TITLE");
        assert_eq!(analysis.destination_code, 0x01);
        assert_eq!(analysis.region, "Non-Japan (International)");
        Ok(())
    }

    // GB uses 15 bits for title name while GBC uses 11
    // Test that we properly read longer title names
    #[test]
    fn test_analyze_gb_long_title() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x00, 0x00, "LOOOOOONG TITLE"); // Japan, GB
        let analysis = analyze_gb_data(&data, "test_rom_jp.gbc")?;

        assert_eq!(analysis.source_name, "test_rom_jp.gbc");
        assert_eq!(analysis.system_type, "Game Boy (GB)");
        assert_eq!(analysis.game_title, "LOOOOOONG TITLE");
        assert_eq!(analysis.destination_code, 0x00);
        assert_eq!(analysis.region, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gbc_long_title() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x00, 0x80, "LOONG TITLE"); // Japan, GB
        let analysis = analyze_gb_data(&data, "test_rom_jp.gbc")?;

        assert_eq!(analysis.source_name, "test_rom_jp.gbc");
        assert_eq!(analysis.system_type, "Game Boy Color (GBC)");
        assert_eq!(analysis.game_title, "LOONG TITLE");
        assert_eq!(analysis.destination_code, 0x00);
        assert_eq!(analysis.region, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gb_unknown_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gb_header(0x02, 0x00, "UNKNOWN REG"); // Unknown region code
        let analysis = analyze_gb_data(&data, "test_rom_unknown.gb")?;

        assert_eq!(analysis.source_name, "test_rom_unknown.gb");
        assert_eq!(analysis.region, "Unknown Code");
        Ok(())
    }

    #[test]
    fn test_analyze_gb_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 100]; // Smaller than 0x150
        let result = analyze_gb_data(&data, "too_small.gb");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
