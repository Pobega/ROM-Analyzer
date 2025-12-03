use std::error::Error;

use crate::RomAnalysisResult;
use crate::console::gamegear;
use crate::console::gb;
use crate::console::gba;
use crate::console::genesis;
use crate::console::mastersystem;
use crate::console::n64;
use crate::console::nes;
use crate::console::psx;
use crate::console::segacd;
use crate::console::snes;
use crate::error::RomAnalyzerError;

#[derive(Debug, PartialEq, Eq)]
enum RomFileType {
    Nes,
    Snes,
    N64,
    MasterSystem,
    GameGear,
    GameBoy,
    GameBoyAdvance,
    Genesis,
    SegaCD,
    CDSystem, // For .iso, .bin, .img, .psx which require further inspection
    Unknown,
}

fn get_rom_file_type(name: &str) -> RomFileType {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    match ext.to_lowercase().as_str() {
        "nes" => RomFileType::Nes,
        "smc" | "sfc" => RomFileType::Snes,
        "n64" | "v64" | "z64" => RomFileType::N64,
        "sms" => RomFileType::MasterSystem,
        "gg" => RomFileType::GameGear,
        "gb" | "gbc" => RomFileType::GameBoy,
        "gba" => RomFileType::GameBoyAdvance,
        "md" | "gen" | "32x" => RomFileType::Genesis,
        "scd" => RomFileType::SegaCD,
        "iso" | "bin" | "img" | "psx" => RomFileType::CDSystem,
        _ => RomFileType::Unknown,
    }
}

pub fn process_rom_data(data: Vec<u8>, name: &str) -> Result<RomAnalysisResult, Box<dyn Error>> {
    let rom_data = match get_rom_file_type(name) {
        RomFileType::Nes => nes::analyze_nes_data(&data, name).map(RomAnalysisResult::NES),
        RomFileType::Snes => snes::analyze_snes_data(&data, name).map(RomAnalysisResult::SNES),
        RomFileType::N64 => n64::analyze_n64_data(&data, name).map(RomAnalysisResult::N64),
        RomFileType::MasterSystem => mastersystem::analyze_mastersystem_data(&data, name)
            .map(RomAnalysisResult::MasterSystem),
        RomFileType::GameGear => {
            gamegear::analyze_gamegear_data(&data, name).map(RomAnalysisResult::GameGear)
        }
        RomFileType::GameBoy => gb::analyze_gb_data(&data, name).map(RomAnalysisResult::GB),
        RomFileType::GameBoyAdvance => {
            gba::analyze_gba_data(&data, name).map(RomAnalysisResult::GBA)
        }
        RomFileType::Genesis => {
            genesis::analyze_genesis_data(&data, name).map(RomAnalysisResult::Genesis)
        }
        RomFileType::SegaCD => {
            segacd::analyze_segacd_data(&data, name).map(RomAnalysisResult::SegaCD)
        }
        RomFileType::CDSystem => {
            // Some cartridge formats (like Sega Genesis) use the .bin extension, which
            // conflicts with CD image formats. This checks for cartridge headers inside
            // files that might otherwise be treated as CD images.
            const SEGA_HEADER_START: usize = 0x100;
            const SEGA_GENESIS_HEADER_END: usize = 0x110;
            const SEGA_CD_SIGNATURE_END: usize = 0x107;
            const SEGA_CD_MIN_LEN: usize = 0x10C; // To read region code at 0x10B

            if data.len() >= SEGA_GENESIS_HEADER_END
                && (data[SEGA_HEADER_START..SEGA_GENESIS_HEADER_END]
                    .starts_with(b"SEGA MEGA DRIVE")
                    || data[SEGA_HEADER_START..SEGA_GENESIS_HEADER_END]
                        .starts_with(b"SEGA GENESIS"))
            {
                genesis::analyze_genesis_data(&data, name).map(RomAnalysisResult::Genesis)
            } else if data.len() >= SEGA_CD_MIN_LEN
                && data[SEGA_HEADER_START..SEGA_CD_SIGNATURE_END].eq_ignore_ascii_case(b"SEGA CD")
            {
                segacd::analyze_segacd_data(&data, name).map(RomAnalysisResult::SegaCD)
            } else {
                psx::analyze_psx_data(&data, name).map(RomAnalysisResult::PSX)
            }
        }
        RomFileType::Unknown => Err(Box::new(RomAnalyzerError::new(&format!(
            "Unrecognized ROM file extension for dispatch: {}",
            name
        )))
        .into()),
    };
    Ok(rom_data?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rom_file_type() {
        assert_eq!(get_rom_file_type("game.nes"), RomFileType::Nes);
        assert_eq!(get_rom_file_type("game.smc"), RomFileType::Snes);
        assert_eq!(get_rom_file_type("game.sfc"), RomFileType::Snes);
        assert_eq!(get_rom_file_type("game.n64"), RomFileType::N64);
        assert_eq!(get_rom_file_type("game.v64"), RomFileType::N64);
        assert_eq!(get_rom_file_type("game.z64"), RomFileType::N64);
        assert_eq!(get_rom_file_type("game.sms"), RomFileType::MasterSystem);
        assert_eq!(get_rom_file_type("game.gg"), RomFileType::GameGear);
        assert_eq!(get_rom_file_type("game.gb"), RomFileType::GameBoy);
        assert_eq!(get_rom_file_type("game.gbc"), RomFileType::GameBoy);
        assert_eq!(get_rom_file_type("game.gba"), RomFileType::GameBoyAdvance);
        assert_eq!(get_rom_file_type("game.md"), RomFileType::Genesis);
        assert_eq!(get_rom_file_type("game.gen"), RomFileType::Genesis);
        assert_eq!(get_rom_file_type("game.32x"), RomFileType::Genesis);
        assert_eq!(get_rom_file_type("game.scd"), RomFileType::SegaCD);
        assert_eq!(get_rom_file_type("game.iso"), RomFileType::CDSystem);
        assert_eq!(get_rom_file_type("game.bin"), RomFileType::CDSystem);
        assert_eq!(get_rom_file_type("game.img"), RomFileType::CDSystem);
        assert_eq!(get_rom_file_type("game.psx"), RomFileType::CDSystem);
        assert_eq!(get_rom_file_type("game.zip"), RomFileType::Unknown);
        assert_eq!(get_rom_file_type("game.txt"), RomFileType::Unknown);
    }

    #[test]
    fn test_process_rom_data_unrecognized_extension() {
        let data = vec![];
        let name = "game.xyz";
        let result = process_rom_data(data, name);
        let err = result.expect_err(
            "process_rom_data should have returned an error for unrecognized extension",
        );
        assert!(err.to_string().contains("Unrecognized ROM file extension"));
    }

    #[test]
    fn test_process_rom_data_cd_system_sega_genesis_header() {
        let mut data = vec![0; 0x120];
        data[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE\0"); // Padded to 16 bytes
        let name = "game.bin";
        // This will attempt to call genesis::analyze_genesis_data
        // Since we don't have a full mock, we'll assert it doesn't return an unknown error
        // A successful return indicates it dispatched to a recognized console analyzer.
        let result = process_rom_data(data, name);
        // Expect an error from the analyzer itself if the data isn't valid for a Sega Cartridge, not an 'Unknown' dispatch error.
        assert!(result.is_err());
        let err = result.expect_err("process_rom_data should have returned an error for mock data");
        assert!(!err.to_string().contains("Unrecognized ROM file extension"));
    }

    #[test]
    fn test_process_rom_data_cd_system_sega_cd_header() {
        let mut data = vec![0; 0x120];
        data[0x100..0x107].copy_from_slice(b"SEGA CD");
        let name = "game.iso";
        let result = process_rom_data(data, name);
        let err = result.expect_err("process_rom_data should have returned an error for mock data");
        assert!(!err.to_string().contains("Unrecognized ROM file extension"));
    }

    #[test]
    fn test_process_rom_data_cd_system_psx() {
        let data = vec![0; 0x100]; // Not enough for Sega headers, should fall through to PSX
        let name = "game.bin";
        let result = process_rom_data(data, name);
        let err = result.expect_err("process_rom_data should have returned an error for mock data");
        assert!(!err.to_string().contains("Unrecognized ROM file extension"));
    }
}
