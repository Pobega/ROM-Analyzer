use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

pub fn analyze_n64_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x40 {
        return Err(Box::new(RomAnalyzerError::new("N64 ROM too small.")));
    }

    let country_code = String::from_utf8_lossy(&data[0x3E..0x40])
        .trim_matches(char::from(0))
        .to_string();

    let region_name = match country_code.as_ref() {
        "E" => "USA / NTSC",
        "J" => "Japan / NTSC",
        "P" => "Europe / PAL",
        "D" => "Germany / PAL",
        "F" => "France / PAL",
        "U" => "USA (Legacy)",
        _ => "Unknown Code",
    }
    .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Nintendo 64 (N64)");
    println!("Region:       {}", region_name);
    println!("Code:         {}", country_code);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_analyze_n64_data_usa() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x40];
        data[0x3E..0x40].copy_from_slice(b"E\0"); // USA region
        analyze_n64_data(&data, "test_rom_us.n64")?;
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_japan() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x40];
        data[0x3E..0x40].copy_from_slice(b"J\0"); // Japan region
        analyze_n64_data(&data, "test_rom_jp.n64")?;
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_europe() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x40];
        data[0x3E..0x40].copy_from_slice(b"P\0"); // Europe region
        analyze_n64_data(&data, "test_rom_eur.n64")?;
        Ok(())
    }

    #[test]
    fn test_analyze_n64_data_unknown() -> Result<(), Box<dyn Error>> {
        let mut data = vec![0; 0x40];
        data[0x3E..0x40].copy_from_slice(b"X\0"); // Unknown region
        analyze_n64_data(&data, "test_rom.n64")?;
        Ok(())
    }
}
