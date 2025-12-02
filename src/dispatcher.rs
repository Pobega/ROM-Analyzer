use std::error::Error;

use crate::console::gamegear;
use crate::console::gb;
use crate::console::gba;
use crate::console::mastersystem;
use crate::console::n64;
use crate::console::nes;
use crate::console::psx;
use crate::console::sega_cartridge;
use crate::console::segacd;
use crate::console::snes;
use crate::error::RomAnalyzerError;

pub fn process_rom_data(data: Vec<u8>, name: &str) -> Result<(), Box<dyn Error>> {
    let lower_name = name.to_lowercase();

    if lower_name.ends_with(".nes") {
        nes::analyze_nes_data(&data, name)
    } else if lower_name.ends_with(".smc") || lower_name.ends_with(".sfc") {
        snes::analyze_snes_data(&data, name)
    } else if lower_name.ends_with(".n64")
        || lower_name.ends_with(".v64")
        || lower_name.ends_with(".z64")
    {
        n64::analyze_n64_data(&data, name)
    } else if lower_name.ends_with(".sms") {
        mastersystem::analyze_mastersystem_data(&data, name)
    } else if lower_name.ends_with(".gg") {
        gamegear::analyze_gamegear_data(&data, name)
    } else if lower_name.ends_with(".gb") || lower_name.ends_with(".gbc") {
        gb::analyze_gb_data(&data, name)
    } else if lower_name.ends_with(".gba") {
        gba::analyze_gba_data(&data, name)
    } else if lower_name.ends_with(".md")
        || lower_name.ends_with(".gen")
        || lower_name.ends_with(".32x")
    {
        sega_cartridge::analyze_sega_cartridge_data(&data, name)
    } else if lower_name.ends_with(".scd") {
        segacd::analyze_segacd_data(&data, name)
    } else if lower_name.ends_with(".iso")
        || lower_name.ends_with(".bin")
        || lower_name.ends_with(".img")
        || lower_name.ends_with(".psx")
    {
        // For .bin files, first check for Sega Genesis/32X header
        if data.len() > 0x110
            && (data[0x100..0x110].starts_with(b"SEGA MEGA DRIVE")
                || data[0x100..0x110].starts_with(b"SEGA GENESIS"))
        {
            sega_cartridge::analyze_sega_cartridge_data(&data, name)
        } else if data.len() > 0x10A && &data[0x100..0x107].to_ascii_uppercase() == b"SEGA CD" {
            segacd::analyze_segacd_data(&data, name)
        } else {
            psx::analyze_psx_data(&data, name)
        }
    } else {
        Err(Box::new(RomAnalyzerError::new(&format!(
            "Unrecognized ROM file extension for dispatch: {}",
            name
        ))))
    }
}
