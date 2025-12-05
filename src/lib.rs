pub mod archive;
pub mod console;
pub mod dispatcher;
pub mod error;
pub mod region;

use serde::Serialize;

use crate::console::gamegear::GameGearAnalysis;
use crate::console::gb::GbAnalysis;
use crate::console::gba::GbaAnalysis;
use crate::console::genesis::GenesisAnalysis;
use crate::console::mastersystem::MasterSystemAnalysis;
use crate::console::n64::N64Analysis;
use crate::console::nes::NesAnalysis;
use crate::console::psx::PsxAnalysis;
use crate::console::segacd::SegaCdAnalysis;
use crate::console::snes::SnesAnalysis;

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
