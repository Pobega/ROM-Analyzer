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
