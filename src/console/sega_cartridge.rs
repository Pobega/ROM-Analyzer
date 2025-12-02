use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

pub fn analyze_sega_cartridge_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    // Sega Genesis header is at offset 0x100. It's 256 bytes long.
    // Region byte is at offset 0x1F0 relative to the start of the ROM (or 0xF0 relative to header start).

    if data.len() < 0x200 {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain a Sega header (size: {} bytes).",
            data.len()
        ))));
    }

    let header_start = 0x100;

    // Verify Sega header signature "SEGA MEGA DRIVE " or "SEGA GENESIS"
    let console_name = String::from_utf8_lossy(&data[header_start + 0x0..header_start + 0x10])
        .trim()
        .to_string();
    if console_name != "SEGA MEGA DRIVE" && console_name != "SEGA GENESIS" {
        // For .bin files, this might be a false positive, so print a warning rather than erroring out.
        println!(
            "[!] Warning: Sega header signature not found at 0x100 for {}. Console name: '{}'",
            source_name, console_name
        );
    }

    let game_title_domestic =
        String::from_utf8_lossy(&data[header_start + 0x10..header_start + 0x30])
            .trim()
            .to_string();
    let game_title_international =
        String::from_utf8_lossy(&data[header_start + 0x30..header_start + 0x50])
            .trim()
            .to_string();

    let region_code_byte = data[0x1F0]; // 0xF0 relative to header_start

    let region_name = match region_code_byte {
        b'J' => "Japan (NTSC-J)",
        b'U' => "USA (NTSC-U)",
        b'E' => "Europe (PAL)",
        b'A' => "Asia (NTSC)",
        b'B' => "Brazil (PAL-M)", // Technically Brazil often uses NTSC-M but some releases were PAL-M
        b'C' => "China (NTSC)",
        b'F' => "France (PAL)",
        b'K' => "Korea (NTSC)",
        b'L' => "UK (PAL)",
        b'S' => "Scandinavia (PAL)",
        b'T' => "Taiwan (NTSC)",
        b'4' => "USA/Europe (NTSC/PAL)", // Combined region for some releases
        _ => "Unknown Code",
    }
    .to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       {}", console_name);
    println!("Game Title (Domestic): {}", game_title_domestic);
    println!("Game Title (Int.):   {}", game_title_international);
    println!(
        "Region Code:  0x{:02X} ('{}')",
        region_code_byte, region_code_byte as char
    );
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}
