use std::error::Error;
use crate::print_separator;

use crate::error::RomAnalyzerError;
use crate::check_region_mismatch;

pub fn analyze_psx_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    // Check the first 128KB (0x20000 bytes)
    let check_size = std::cmp::min(data.len(), 0x20000);
    if check_size < 0x2000 { // Need enough data for Volume Descriptor/Boot file
        return Err(Box::new(RomAnalyzerError::new("PSX boot file too small for reliable analysis.")));
    }

    let data_sample = &data[..check_size].to_ascii_uppercase();

    let region_map = [
        ("SLUS".as_bytes(), "North America (NTSC-U)"),
        ("SLES".as_bytes(), "Europe (PAL)"),
        ("SLPS".as_bytes(), "Japan (NTSC-J)"),
    ];

    let mut found_prefix = None;
    let mut region_name = "Unknown";

    for (prefix, region) in region_map.iter() {
        if data_sample.windows(prefix.len()).any(|window| window == *prefix) {
            found_prefix = Some(prefix);
            region_name = region;
            break;
        }
    }

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sony PlayStation (PSX)");
    println!("Region:       {}", region_name);
    println!("Code:         {}", found_prefix.map(|p| String::from_utf8_lossy(p).to_string()).unwrap_or_else(|| "N/A".to_string()));

    if found_prefix.is_none() {
        println!("Note: Executable prefix (SLUS/SLES/SLPS) not found in header area. Requires main data track (.bin or .iso).");
    }

    check_region_mismatch!(source_name, region_name);
    print_separator();
    Ok(())
}
