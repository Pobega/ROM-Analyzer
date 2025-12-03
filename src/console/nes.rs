use std::error::Error;

use crate::check_region_mismatch;
use crate::error::RomAnalyzerError;
use crate::print_separator;

const INES_REGION_MASK: u8 = 0x01;

const NES2_REGION_MASK: u8 = 0x03;
const NES2_FORMAT_MASK: u8 = 0x0C;
const NES2_FORMAT_EXPECTED_VALUE: u8 = 0x08;

pub fn get_nes_region_name(region_byte: u8, nes2_format: bool) -> &'static str {
    if nes2_format {
        // NES 2.0 headers store region data in the CPU/PPU timing bit
        // in byte 12.
        match region_byte & NES2_REGION_MASK {
            0 => "NTSC (USA/Japan)",
            1 => "PAL (Europe/Oceania)",
            2 => "Multi-region",
            3 => "Dendy (Russia)",
            _ => "Unknown",
        }
    } else {
        // iNES headers store region data in byte 9.
        // It is only the lowest-order bit for NTSC vs PAL.
        // NTSC covers USA and Japan.
        match region_byte & INES_REGION_MASK {
            0 => "NTSC (USA/Japan)",
            1 => "PAL (Europe/Oceania)",
            _ => "Unknown",
        }
    }
}

pub fn analyze_nes_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 16 {
        return Err(Box::new(RomAnalyzerError::new(&format!(
            "ROM data is too small to contain an iNES header (size: {} bytes).",
            data.len()
        ))));
    }

    // All headered NES ROMs should begin with 'NES<EOF>'
    let signature = &data[0..4];
    if signature != b"NES\x1a" {
        return Err(Box::new(RomAnalyzerError::new(
            "Invalid iNES header signature. Not a valid NES ROM.",
        )));
    }

    let mut region_byte = data[9]; // iNES region byte (in lowest order bit)
    let nes2_format = data[7] & NES2_FORMAT_MASK == NES2_FORMAT_EXPECTED_VALUE;
    if nes2_format {
        region_byte = data[12]; // NES 2.0 region byte (in two lowest order bits)
    }
    let region_name = get_nes_region_name(region_byte, nes2_format);

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Nintendo Entertainment System (NES)");
    println!("Region:       {}", region_name);
    if nes2_format {
        println!("NES2.0 Flag 12: 0x{:02X}", region_byte);
    } else {
        println!("iNES Flag 9:  0x{:02X}", region_byte);
    }

    // Don't bother checking for mismatches on iNES headers, as it
    // is unused by most modern emulators and ROM dumps.
    if nes2_format {
        check_region_mismatch!(source_name, region_name);
    }
    print_separator();
    Ok(())
}
