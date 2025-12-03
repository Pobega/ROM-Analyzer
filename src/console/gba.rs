/// https://problemkaputt.de/gbatek-gba-cartridge-header.htm
use std::error::Error;

// Assuming check_region_mismatch! and print_separator are defined elsewhere and accessible.
// For this refactoring, we'll assume they are handled by the caller of analyze_gba_data.
// use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

/// Struct to hold the analysis results for a GBA ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct GbaAnalysis {
    /// The game title extracted from the ROM header.
    pub game_title: String,
    /// The game code extracted from the ROM header.
    pub game_code: String,
    /// The maker code extracted from the ROM header.
    pub maker_code: String,
    /// The identified region name (e.g., "Japan").
    pub region: String,
    /// The name of the source file.
    pub source_name: String,
}

impl GbaAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        print_separator();
        println!("Source:       {}", self.source_name);
        println!("System:       Game Boy Advance (GBA)");
        println!("Game Title:   {}", self.game_title);
        println!("Game Code:    {}", self.game_code);
        println!("Maker Code:   {}", self.maker_code);
        println!("Region:       {}", self.region);

        // The check_region_mismatch macro is called here, assuming it's available in scope.
        // It's important that the caller ensures this macro is accessible.
        // For example: `if analysis.region != "Unknown" { check_region_mismatch!(analysis.source_name, &analysis.region); }`
        print_separator();
    }
}

/// Analyzes GBA ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
pub fn analyze_gba_data(data: &[u8], source_name: &str) -> Result<GbaAnalysis, Box<dyn Error>> {
    // GBA header is at offset 0x0. Relevant info: Game Title (0xA0-0xAC), Game Code (0xAC-0xB0), Maker Code (0xB0-0xB2), Region (0xB4).
    // The header is typically 192 bytes (0xC0), but we'll use a slightly larger safety margin.
    const HEADER_SIZE: usize = 0xC0;
    if data.len() < HEADER_SIZE {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a GBA header (size: {} bytes, requires at least {} bytes).",
            data.len(),
            HEADER_SIZE
        ))));
    }

    // Extract Game Title (12 bytes, null-terminated)
    let game_title = String::from_utf8_lossy(&data[0xA0..0xAC])
        .trim_matches(char::from(0)) // Remove null bytes
        .to_string();

    // Extract Game Code (4 bytes, ASCII)
    let game_code = String::from_utf8_lossy(&data[0xAC..0xB0])
        .trim_matches(char::from(0)) // Remove null bytes, though usually not null-terminated here
        .to_string();

    // Extract Maker Code (2 bytes, ASCII)
    let maker_code = String::from_utf8_lossy(&data[0xB0..0xB2])
        .trim_matches(char::from(0)) // Remove null bytes
        .to_string();

    // Extract Region Code (1 byte at 0xB4)
    let region_code_byte = data[0xB4];

    // Determine region name based on the byte value.
    // Common codes are numeric (0=JP, 1=US, 2=EU) or ASCII characters.
    let region_name = match region_code_byte {
        0x00 => "Japan",
        0x01 => "USA",
        0x02 => "Europe",
        // ASCII representations are also common
        b'J' => "Japan",
        b'U' => "USA",
        b'E' => "Europe",
        b'P' => "Europe", // PAL
        _ => "Unknown",
    }
    .to_string();

    Ok(GbaAnalysis {
        game_title,
        game_code,
        maker_code,
        region: region_name,
        source_name: source_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Helper function to generate a minimal GBA header for testing.
    fn generate_gba_header(
        game_code: &str,
        maker_code: &str,
        region_byte: u8,
        title: &str,
    ) -> Vec<u8> {
        let mut data = vec![0; 0xC0]; // Ensure enough space for header

        // Game Title (max 10 chars + null, but we use 0xA0..0xAC which is 12 bytes for safety)
        let mut title_bytes = title.as_bytes().to_vec();
        title_bytes.resize(12, 0);
        data[0xA0..0xAC].copy_from_slice(&title_bytes);

        // Game Code (4 bytes, ASCII)
        let mut game_code_bytes = game_code.as_bytes().to_vec();
        game_code_bytes.resize(4, 0);
        data[0xAC..0xB0].copy_from_slice(&game_code_bytes);

        // Maker Code (2 bytes, ASCII)
        let mut maker_code_bytes = maker_code.as_bytes().to_vec();
        maker_code_bytes.resize(2, 0);
        data[0xB0..0xB2].copy_from_slice(&maker_code_bytes);

        // Region Code (1 byte at 0xB4)
        data[0xB4] = region_byte;

        data
    }

    #[test]
    fn test_analyze_gba_data_japan_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("ABCD", "XX", 0x00, "GBA JP GAME"); // Japan region code 0x00
        let analysis = analyze_gba_data(&data, "test_rom_jp.gba")?;

        assert_eq!(analysis.source_name, "test_rom_jp.gba");
        assert_eq!(analysis.game_title, "GBA JP GAME");
        assert_eq!(analysis.game_code, "ABCD");
        assert_eq!(analysis.maker_code, "XX");
        assert_eq!(analysis.region, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_usa_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("EFGH", "YY", 0x01, "GBA US GAME"); // USA region code 0x01
        let analysis = analyze_gba_data(&data, "test_rom_us.gba")?;

        assert_eq!(analysis.source_name, "test_rom_us.gba");
        assert_eq!(analysis.game_title, "GBA US GAME");
        assert_eq!(analysis.game_code, "EFGH");
        assert_eq!(analysis.maker_code, "YY");
        assert_eq!(analysis.region, "USA");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_europe_char() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("IJKL", "ZZ", b'E', "GBA EUR GAME"); // Europe region char 'E'
        let analysis = analyze_gba_data(&data, "test_rom_eur.gba")?;

        assert_eq!(analysis.source_name, "test_rom_eur.gba");
        assert_eq!(analysis.game_title, "GBA EUR GAME");
        assert_eq!(analysis.game_code, "IJKL");
        assert_eq!(analysis.maker_code, "ZZ");
        assert_eq!(analysis.region, "Europe");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_japan_char() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("MNOP", "AA", b'J', "GBA JP CHAR"); // Japan region char 'J'
        let analysis = analyze_gba_data(&data, "test_rom_jp_char.gba")?;

        assert_eq!(analysis.source_name, "test_rom_jp_char.gba");
        assert_eq!(analysis.game_title, "GBA JP CHAR");
        assert_eq!(analysis.game_code, "MNOP");
        assert_eq!(analysis.maker_code, "AA");
        assert_eq!(analysis.region, "Japan");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_unknown_code() -> Result<(), Box<dyn Error>> {
        let data = generate_gba_header("QRST", "BB", 0xFF, "GBA UNKNOWN"); // Unknown region code
        let analysis = analyze_gba_data(&data, "test_rom_unknown.gba")?;

        assert_eq!(analysis.source_name, "test_rom_unknown.gba");
        assert_eq!(analysis.region, "Unknown");
        Ok(())
    }

    #[test]
    fn test_analyze_gba_data_too_small() {
        // Test with data smaller than the minimum required size for analysis.
        let data = vec![0; 50]; // Smaller than 0xC0
        let result = analyze_gba_data(&data, "too_small.gba");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }
}
