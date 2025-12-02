use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

pub fn analyze_gb_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x14B {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a Game Boy header (size: {} bytes, requires at least 0x14B).",
            data.len()
        ))));
    }

    let system_type = if data[0x143] == 0x80 || data[0x143] == 0xC0 {
        "Game Boy Color (GBC)"
    } else {
        "Game Boy (GB)"
    };

    let game_title = String::from_utf8_lossy(&data[0x134..0x143])
        .trim_matches(char::from(0))
        .to_string();

    let destination_code = data[0x14A];
    let region_name = match destination_code {
        0x00 => "Japan",
        0x01 => "Non-Japan (International)",
        _ => "Unknown Code",
    }
    .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       {}", system_type);
    println!("Game Title:   {}", game_title);
    println!("Region Code:  0x{:02X}", destination_code);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_analyze_gb_data_japan() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x14B];
        data[0x14A] = 0x00; // Japan region
        data[0x143] = 0x00; // Original Game Boy
        data[0x134..0x143].copy_from_slice(b"GAMETITLE      ");
        analyze_gb_data(&data, "test_rom_jp.gb")?;
        Ok(())
    }

    #[test]
    fn test_analyze_gb_data_non_japan() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x14B];
        data[0x14A] = 0x01; // Non-Japan region
        data[0x143] = 0x00; // Original Game Boy
        data[0x134..0x143].copy_from_slice(b"GAMETITLE      ");
        analyze_gb_data(&data, "test_rom_us.gb")?;
        Ok(())
    }

    #[test]
    fn test_analyze_gbc_data_japan() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x14B];
        data[0x14A] = 0x00; // Japan region
        data[0x143] = 0x80; // Game Boy Color
        data[0x134..0x143].copy_from_slice(b"GBC TITLE      ");
        analyze_gb_data(&data, "test_rom_jp.gbc")?;
        Ok(())
    }
}
