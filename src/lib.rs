pub mod archive;
pub mod console;
pub mod dispatcher;
pub mod error;
pub mod region;

use crate::console::gamegear::GameGearAnalysis;
use crate::console::gb::GbAnalysis;
use crate::console::gba::GbaAnalysis;
use crate::console::mastersystem::MasterSystemAnalysis;
use crate::console::n64::N64Analysis;
use crate::console::nes::NesAnalysis;
use crate::console::psx::PsxAnalysis;
use crate::console::sega_cartridge::SegaCartridgeAnalysis;
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

pub fn print_separator() {
    println!("{}", "-".repeat(40));
}

#[derive(Debug, PartialEq, Clone)]
pub enum RomAnalysisResult {
    GameGear(GameGearAnalysis),
    GB(GbAnalysis),
    GBA(GbaAnalysis),
    MasterSystem(MasterSystemAnalysis),
    N64(N64Analysis),
    NES(NesAnalysis),
    PSX(PsxAnalysis),
    SegaCartridge(SegaCartridgeAnalysis),
    SegaCD(SegaCdAnalysis),
    SNES(SnesAnalysis),
}

impl RomAnalysisResult {
    pub fn print(&self) {
        match self {
            RomAnalysisResult::GameGear(a) => a.print(),
            RomAnalysisResult::GB(a) => a.print(),
            RomAnalysisResult::GBA(a) => a.print(),
            RomAnalysisResult::MasterSystem(a) => a.print(),
            RomAnalysisResult::N64(a) => a.print(),
            RomAnalysisResult::NES(a) => a.print(),
            RomAnalysisResult::PSX(a) => a.print(),
            RomAnalysisResult::SegaCartridge(a) => a.print(),
            RomAnalysisResult::SegaCD(a) => a.print(),
            RomAnalysisResult::SNES(a) => a.print(),
        }
    }
}
