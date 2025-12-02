use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;
use std::error::Error;

pub fn get_snes_region_name(code: u8) -> String {
    let regions = vec![
        (0x00, "Japan (NTSC)"),
        (0x01, "USA / Canada (NTSC)"),
        (0x02, "Europe / Oceania / Asia (PAL)"),
        (0x03, "Sweden / Scandinavia (PAL)"),
        (0x04, "Finland (PAL)"),
        (0x05, "Denmark (PAL)"),
        (0x06, "France (PAL)"),
        (0x07, "Netherlands (PAL)"),
        (0x08, "Spain (PAL)"),
        (0x09, "Germany (PAL)"),
        (0x0A, "Italy (PAL)"),
        (0x0B, "China (PAL)"),
        (0x0C, "Indonesia (PAL)"),
        (0x0D, "South Korea (NTSC)"),
        (0x0E, "Common / International"),
        (0x0F, "Canada (NTSC)"),
        (0x10, "Brazil (NTSC)"),
        (0x11, "Australia (PAL)"),
        (0x12, "Other (Variation 1)"),
        (0x13, "Other (Variation 2)"),
        (0x14, "Other (Variation 3)"),
    ];
    for (c, name) in regions {
        if c == code {
            return name.to_string();
        }
    }
    format!("Unknown Region (0x{:02X})", code)
}

pub fn validate_snes_checksum(rom_data: &[u8], header_offset: usize) -> bool {
    if header_offset + 0x20 > rom_data.len() {
        return false;
    }

    let complement_bytes: [u8; 2] =
        match rom_data[header_offset + 0x1C..header_offset + 0x1E].try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
    let checksum_bytes: [u8; 2] =
        match rom_data[header_offset + 0x1E..header_offset + 0x20].try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };

    let complement = u16::from_le_bytes(complement_bytes);
    let checksum = u16::from_le_bytes(checksum_bytes);

    (checksum as u32 + complement as u32) == 0xFFFF
}

pub fn analyze_snes_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    let file_size = data.len();
    let mut header_offset = 0;

    if file_size % 1024 == 512 {
        header_offset = 512;
        println!("[*] Copier header detected (512 bytes). Offsetting reads...");
    }

    let lorom_addr = 0x7FC0 + header_offset;
    let hirom_addr = 0xFFC0 + header_offset;
    let valid_addr: usize;
    let mapping_type: &str;

    if validate_snes_checksum(data, lorom_addr) {
        valid_addr = lorom_addr;
        mapping_type = "LoROM";
    } else if validate_snes_checksum(data, hirom_addr) {
        valid_addr = hirom_addr;
        mapping_type = "HiROM";
    } else {
        println!(
            "[!] Checksum validation failed. Attempting to read from LoROM location as fallback..."
        );
        valid_addr = lorom_addr;
        mapping_type = "LoROM (Unverified)";
    }

    if valid_addr + 0x20 > file_size {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small or invalid (size: {} bytes).",
            file_size
        ))));
    }

    let region_byte_offset = valid_addr + 0x19;
    let region_code = data[region_byte_offset];
    let region_name = get_snes_region_name(region_code);
    let game_title = String::from_utf8_lossy(&data[valid_addr..valid_addr + 21])
        .trim()
        .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Super Nintendo (SNES)");
    println!("Game Title:   {}", game_title);
    println!("Mapping:      {}", mapping_type);
    println!("Region Code:  0x{:02X}", region_code);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // Helper to create a dummy SNES ROM with a valid checksum
    fn create_snes_rom_data(
        size: usize,
        header_offset: usize,
        region_code: u8,
        is_hirom: bool,
    ) -> Vec<u8> {
        let mut data = vec![0; size];
        let header_start = if is_hirom { 0xFFC0 } else { 0x7FC0 } + header_offset;

        if header_start + 0x20 > size {
            panic!("Provided size is too small for SNES header at the given offset.");
        }

        // Set a valid checksum and its complement
        let checksum: u16 = 0xAAAA;
        let complement: u16 = 0x5555;
        data[header_start + 0x1C..header_start + 0x1E].copy_from_slice(&complement.to_le_bytes());
        data[header_start + 0x1E..header_start + 0x20].copy_from_slice(&checksum.to_le_bytes());

        // Set game title
        data[header_start..header_start + 21].copy_from_slice(b"TEST GAME TITLE      ");
        // Set region code
        data[header_start + 0x19] = region_code;

        data
    }

    #[test]
    fn test_analyze_snes_data_lorom_japan() -> Result<(), Box<dyn Error>> {
        let data = create_snes_rom_data(0x80000, 0, 0x00, false); // 512KB, LoROM, Japan
        analyze_snes_data(&data, "test_lorom_jp.sfc")?;
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_hirom_usa() -> Result<(), Box<dyn Error>> {
        let data = create_snes_rom_data(0x100000, 0, 0x01, true); // 1MB, HiROM, USA
        analyze_snes_data(&data, "test_hirom_us.sfc")?;
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_lorom_europe_copier_header() -> Result<(), Box<dyn Error>> {
        let data = create_snes_rom_data(0x80000 + 512, 512, 0x02, false); // LoROM, Europe, with 512-byte copier header
        analyze_snes_data(&data, "test_lorom_eur_copier.sfc")?;
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_hirom_canada_copier_header() -> Result<(), Box<dyn Error>> {
        let data = create_snes_rom_data(0x100000 + 512, 512, 0x0F, true); // HiROM, Canada, with 512-byte copier header
        analyze_snes_data(&data, "test_hirom_can_copier.sfc")?;
        Ok(())
    }

    #[test]
    fn test_analyze_snes_data_unknown_region() -> Result<(), Box<dyn Error>> {
        let data = create_snes_rom_data(0x80000, 0, 0xFF, false); // LoROM, Unknown region
        analyze_snes_data(&data, "test_lorom_unknown.sfc")?;
        Ok(())
    }
}
