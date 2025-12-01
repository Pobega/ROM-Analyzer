use std::error::Error;
use crate::region::infer_region_from_filename;
use crate::print_separator;

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
