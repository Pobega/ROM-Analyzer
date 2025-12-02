use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

pub fn get_nes_region_name(flag_9_byte: u8) -> &'static str {
    let is_pal = (flag_9_byte & 0x01) == 0x01;
    if is_pal {
        "PAL (Europe/Oceania)"
    } else {
        "NTSC (USA/Japan)"
    }
}

pub fn analyze_nes_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 16 {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain an iNES header (size: {} bytes).",
            data.len()
        ))));
    }

    let signature = &data[0..4];
    if signature != b"NES\x1a" {
        return Err(Box::new(RomAnalyzerError::new(
            "Invalid iNES header signature. Not a valid NES ROM.",
        )));
    }

    let flag_9 = data[9];
    let region_name = get_nes_region_name(flag_9);

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Nintendo Entertainment System (NES)");
    println!("Region:       {}", region_name);
    println!("iNES Flag 9:  0x{:02X}", flag_9);

    check_region_mismatch!(source_name, region_name);
    print_separator();
    Ok(())
}
