use crate::print_separator;
use crate::region::infer_region_from_filename;
use std::error::Error;

pub fn analyze_gamegear_data(_data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    // Sega Game Gear ROMs, like Master System, often lack a standardized region code in the header.
    // Region is typically inferred from filename.

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sega Game Gear");
    println!("Note:         Detailed region information often not available in header.");

    // Attempt to infer from filename if possible
    if let Some(inferred) = infer_region_from_filename(source_name) {
        println!("Region (inferred from filename): {}", inferred);
    } else {
        println!("Region:       Unknown (Filename inference failed).");
    }
    print_separator();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_analyze_gamegear_data_usa() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        analyze_gamegear_data(&data, "test_rom_usa.gg")?;
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_japan() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        analyze_gamegear_data(&data, "test_rom_jp.gg")?;
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_europe() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        analyze_gamegear_data(&data, "test_rom_eur.gg")?;
        Ok(())
    }

    #[test]
    fn test_analyze_gamegear_data_unknown() -> Result<(), Box<dyn Error>> {
        let data = vec![0; 0x100]; // Dummy data
        analyze_gamegear_data(&data, "test_rom.gg")?;
        Ok(())
    }
}
