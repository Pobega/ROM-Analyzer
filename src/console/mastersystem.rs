use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;
use std::error::Error;

pub fn analyze_mastersystem_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x7FFD {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a Master System header (size: {} bytes, requires at least 0x7FFD).",
            data.len()
        ))));
    }

    // SMS Region/Language byte is at offset 0x7FFC
    let sms_region_byte = data[0x7FFC];
    let region_name = match sms_region_byte {
        0x30 => "Japan (NTSC)",
        0x4C => "Europe / Overseas (PAL/NTSC)",
        _ => "Unknown Code",
    }
    .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sega Master System");
    println!("Region Code:  0x{:02X}", sms_region_byte);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}
