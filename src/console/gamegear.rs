/// https://www.smspower.org/Development/ROMHeader
use crate::print_separator;
use crate::region::infer_region_from_filename;
use std::error::Error;

/// Struct to hold the analysis results for a Game Gear ROM.
#[derive(Debug, PartialEq, Clone)]
pub struct GameGearAnalysis {
    /// The identified region name (e.g., "USA").
    pub region: String,
    /// The name of the source file.
    pub source_name: String,
}

impl GameGearAnalysis {
    /// Prints the analysis results to the console.
    pub fn print(&self) {
        print_separator();
        println!("Source:       {}", self.source_name);
        println!("System:       Sega Game Gear");
        println!("Region:       {}", self.region);
        println!("Note:         Detailed region information often not available in header.");
        print_separator();
    }
}

/// Analyzes Game Gear ROM data and returns a struct containing the analysis results.
/// This function is now pure and does not perform console output.
pub fn analyze_gamegear_data(
    _data: &[u8],
    source_name: &str,
) -> Result<GameGearAnalysis, Box<dyn Error>> {
    // Sega Game Gear ROMs, like Master System, often lack a standardized region code in the header.
    // Region is typically inferred from filename.

    let region = infer_region_from_filename(source_name)
        .map(|s| s.to_string())
        .unwrap_or("Unknown".to_string());

    Ok(GameGearAnalysis {
        region,
        source_name: source_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_analyze_gamegear_data_usa() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom_usa.gg")?;
        assert_eq!(analysis.source_name, "test_rom_usa.gg");
        assert_eq!(analysis.region, "USA");
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_japan() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom_jp.gg")?;
        assert_eq!(analysis.source_name, "test_rom_jp.gg");
        assert_eq!(analysis.region, "JAPAN");
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_europe() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom_eur.gg")?;
        assert_eq!(analysis.source_name, "test_rom_eur.gg");
        assert_eq!(analysis.region, "EUROPE");
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_unknown() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        let analysis = analyze_gamegear_data(&data, "test_rom.gg")?;
        assert_eq!(analysis.source_name, "test_rom.gg");
        assert_eq!(analysis.region, "Unknown");
        Ok(())
    }
}
