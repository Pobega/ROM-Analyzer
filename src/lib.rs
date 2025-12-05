pub mod archive;
pub mod console;
pub mod error;
pub mod region;

use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use serde::Serialize;

use crate::archive::chd::analyze_chd_file;
use crate::archive::zip::process_zip_file;
use crate::console::gamegear::{self, GameGearAnalysis};
use crate::console::gb::{self, GbAnalysis};
use crate::console::gba::{self, GbaAnalysis};
use crate::console::genesis::{self, GenesisAnalysis};
use crate::console::mastersystem::{self, MasterSystemAnalysis};
use crate::console::n64::{self, N64Analysis};
use crate::console::nes::{self, NesAnalysis};
use crate::console::psx::{self, PsxAnalysis};
use crate::console::segacd::{self, SegaCdAnalysis};
use crate::console::snes::{self, SnesAnalysis};
use crate::error::RomAnalyzerError;

pub const SUPPORTED_ROM_EXTENSIONS: &[&str] = &[
    ".nes", // NES
    ".smc", ".sfc", // SNES
    ".n64", ".v64", ".z64", // N64
    ".sms", // Sega Master System
    ".gg",  // Sega Game Gear
    ".md", ".gen", ".32x", // Sega Genesis / 32X
    ".gb", ".gbc", // Game Boy / Game Boy Color
    ".gba", // Game Boy Advance
    ".scd", // Sega CD
    ".iso", ".bin", ".img", ".psx", // CD Systems
];

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "console")]
pub enum RomAnalysisResult {
    GameGear(GameGearAnalysis),
    GB(GbAnalysis),
    GBA(GbaAnalysis),
    Genesis(GenesisAnalysis),
    MasterSystem(MasterSystemAnalysis),
    N64(N64Analysis),
    NES(NesAnalysis),
    PSX(PsxAnalysis),
    SegaCD(SegaCdAnalysis),
    SNES(SnesAnalysis),
}

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
    CDSystem,
    Unknown,
}

fn get_file_extension_lowercase(file_path: &str) -> String {
    Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase()
}

fn get_rom_file_type(name: &str) -> RomFileType {
    let ext = get_file_extension_lowercase(name);

    match ext.as_str() {
        "nes" => RomFileType::Nes,
        "smc" | "sfc" => RomFileType::Snes,
        "n64" | "v64" | "z64" => RomFileType::N64,
        "sms" => RomFileType::MasterSystem,
        "gg" => RomFileType::GameGear,
        "gb" | "gbc" => RomFileType::GameBoy,
        "gba" => RomFileType::GameBoyAdvance,
        "md" | "gen" | "32x" => RomFileType::Genesis,
        "scd" => RomFileType::SegaCD,
        "iso" | "bin" | "img" | "psx" | "chd" => RomFileType::CDSystem,
        _ => RomFileType::Unknown,
    }
}

fn process_rom_data(data: Vec<u8>, rom_path: &str) -> Result<RomAnalysisResult, Box<dyn Error>> {
    match get_rom_file_type(rom_path) {
        RomFileType::Nes => nes::analyze_nes_data(&data, rom_path).map(RomAnalysisResult::NES),
        RomFileType::Snes => snes::analyze_snes_data(&data, rom_path).map(RomAnalysisResult::SNES),
        RomFileType::N64 => n64::analyze_n64_data(&data, rom_path).map(RomAnalysisResult::N64),
        RomFileType::MasterSystem => mastersystem::analyze_mastersystem_data(&data, rom_path)
            .map(RomAnalysisResult::MasterSystem),
        RomFileType::GameGear => {
            gamegear::analyze_gamegear_data(&data, rom_path).map(RomAnalysisResult::GameGear)
        }
        RomFileType::GameBoy => gb::analyze_gb_data(&data, rom_path).map(RomAnalysisResult::GB),
        RomFileType::GameBoyAdvance => {
            gba::analyze_gba_data(&data, rom_path).map(RomAnalysisResult::GBA)
        }
        RomFileType::Genesis => {
            genesis::analyze_genesis_data(&data, rom_path).map(RomAnalysisResult::Genesis)
        }
        RomFileType::SegaCD => {
            segacd::analyze_segacd_data(&data, rom_path).map(RomAnalysisResult::SegaCD)
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
                genesis::analyze_genesis_data(&data, rom_path).map(RomAnalysisResult::Genesis)
            } else if data.len() >= SEGA_CD_MIN_LEN
                && data[SEGA_HEADER_START..SEGA_CD_SIGNATURE_END].eq_ignore_ascii_case(b"SEGA CD")
            {
                segacd::analyze_segacd_data(&data, rom_path).map(RomAnalysisResult::SegaCD)
            } else {
                psx::analyze_psx_data(&data, rom_path).map(RomAnalysisResult::PSX)
            }
        }
        RomFileType::Unknown => Err(Box::new(RomAnalyzerError::new(&format!(
            "Unrecognized ROM file extension for dispatch: {}",
            rom_path
        )))
        .into()),
    }
}

/// Analyze the header data of a ROM file.
///
/// This function looks at the data of a ROM file and returns info based on the headers and
/// filetype.
pub fn analyze_rom_data(file_path: &str) -> Result<RomAnalysisResult, Box<dyn Error>> {
    match get_file_extension_lowercase(file_path).as_str() {
        "zip" => {
            let file = File::open(file_path)?;
            let (data, rom_file_name) = process_zip_file(file, file_path)?;
            process_rom_data(data, &rom_file_name)
        }
        "chd" => {
            let decompressed_chd = analyze_chd_file(Path::new(file_path))?;
            process_rom_data(decompressed_chd, file_path)
        }
        _ => {
            let data = fs::read(file_path)?;
            process_rom_data(data, file_path)
        }
    }
}

macro_rules! impl_rom_analysis_method {
    ($fn_name:ident, $return_type:ty) => {
        pub fn $fn_name(&self) -> $return_type {
            match self {
                RomAnalysisResult::GameGear(a) => a.$fn_name(),
                RomAnalysisResult::GB(a) => a.$fn_name(),
                RomAnalysisResult::GBA(a) => a.$fn_name(),
                RomAnalysisResult::Genesis(a) => a.$fn_name(),
                RomAnalysisResult::MasterSystem(a) => a.$fn_name(),
                RomAnalysisResult::N64(a) => a.$fn_name(),
                RomAnalysisResult::NES(a) => a.$fn_name(),
                RomAnalysisResult::PSX(a) => a.$fn_name(),
                RomAnalysisResult::SegaCD(a) => a.$fn_name(),
                RomAnalysisResult::SNES(a) => a.$fn_name(),
            }
        }
    };
}

macro_rules! impl_rom_analysis_accessor {
    ($fn_name:ident, $field:ident, &$return_type:ty) => {
        pub fn $fn_name(&self) -> &$return_type {
            match self {
                RomAnalysisResult::GameGear(a) => &a.$field,
                RomAnalysisResult::GB(a) => &a.$field,
                RomAnalysisResult::GBA(a) => &a.$field,
                RomAnalysisResult::Genesis(a) => &a.$field,
                RomAnalysisResult::MasterSystem(a) => &a.$field,
                RomAnalysisResult::N64(a) => &a.$field,
                RomAnalysisResult::NES(a) => &a.$field,
                RomAnalysisResult::PSX(a) => &a.$field,
                RomAnalysisResult::SegaCD(a) => &a.$field,
                RomAnalysisResult::SNES(a) => &a.$field,
            }
        }
    };
    ($fn_name:ident, $field:ident, $return_type:ty) => {
        pub fn $fn_name(&self) -> $return_type {
            match self {
                RomAnalysisResult::GameGear(a) => a.$field,
                RomAnalysisResult::GB(a) => a.$field,
                RomAnalysisResult::GBA(a) => a.$field,
                RomAnalysisResult::Genesis(a) => a.$field,
                RomAnalysisResult::MasterSystem(a) => a.$field,
                RomAnalysisResult::N64(a) => a.$field,
                RomAnalysisResult::NES(a) => a.$field,
                RomAnalysisResult::PSX(a) => a.$field,
                RomAnalysisResult::SegaCD(a) => a.$field,
                RomAnalysisResult::SNES(a) => a.$field,
            }
        }
    };
}

impl RomAnalysisResult {
    impl_rom_analysis_method!(print, String);
    impl_rom_analysis_accessor!(source_name, source_name, &str);
    impl_rom_analysis_accessor!(region, region, &str);
    impl_rom_analysis_accessor!(region_mismatch, region_mismatch, bool);
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
        assert_eq!(get_rom_file_type("game.chd"), RomFileType::CDSystem);
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
