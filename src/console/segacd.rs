use crate::print_separator;
use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;

pub fn analyze_segacd_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x200 {
        return Err(Box::new(RomAnalyzerError::new(
            "Sega CD boot file too small.",
        )));
    }

    let signature = String::from_utf8_lossy(&data[0x100..0x107])
        .trim()
        .to_string();
    if signature != "SEGA CD" && signature != "SEGA MEGA" {
        println!(
            "[!] Warning: File does not appear to be a standard Sega CD boot file (no SEGA CD signature at 0x100)."
        );
    }

    // Region byte is at offset 0x10B in the boot program
    let region_code = data[0x10B];

    let region_name = match region_code {
        0x40 => "Japan (NTSC-J)",
        0x80 => "Europe (PAL)",
        0xC0 => "USA (NTSC-U)",
        0x00 => "Unrestricted/BIOS region",
        _ => "Unknown Code",
    }
    .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sega CD / Mega CD");
    println!("Region:       {}", region_name);
    println!("Code:         0x{:02X}", region_code);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}
