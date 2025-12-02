use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

pub fn analyze_gba_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0xC0 {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a GBA header (size: {} bytes).",
            data.len()
        ))));
    }

    // GBA header is at 0x0. Relevant info: Game Title (0xA0-0xAC), Game Code (0xAC-0xB0), Maker Code (0xB0-0xB2), Unit Code (0xB3), Region (0xB4).

    let game_title = String::from_utf8_lossy(&data[0xA0..0xAC])
        .trim_matches(char::from(0))
        .to_string();
    let game_code = String::from_utf8_lossy(&data[0xAC..0xB0])
        .trim_matches(char::from(0))
        .to_string();
    let maker_code = String::from_utf8_lossy(&data[0xB0..0xB2])
        .trim_matches(char::from(0))
        .to_string();
    let region_code = data[0xB4]; // Typically 0 for Japan, 1 for USA, 2 for Europe, etc. or ASCII representation

    let region_name = match region_code {
        0x00 => "Japan",
        0x01 => "USA",
        0x02 => "Europe",
        // Other common region codes as ASCII characters
        b'J' => "Japan",
        b'U' => "USA",
        b'E' => "Europe",
        b'P' => "Europe", // PAL
        _ => "Unknown",
    }
    .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Game Boy Advance (GBA)");
    println!("Game Title:   {}", game_title);
    println!("Game Code:    {}", game_code);
    println!("Maker Code:   {}", maker_code);
    println!(
        "Region Code:  0x{:02X} ('{}')",
        region_code, region_code as char
    );
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}
