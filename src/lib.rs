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

impl RomAnalysisResult {
    pub fn print(&self) -> String {
        match self {
            RomAnalysisResult::GameGear(a) => a.print(),
            RomAnalysisResult::GB(a) => a.print(),
            RomAnalysisResult::GBA(a) => a.print(),
            RomAnalysisResult::Genesis(a) => a.print(),
            RomAnalysisResult::MasterSystem(a) => a.print(),
            RomAnalysisResult::N64(a) => a.print(),
            RomAnalysisResult::NES(a) => a.print(),
            RomAnalysisResult::PSX(a) => a.print(),
            RomAnalysisResult::SegaCD(a) => a.print(),
            RomAnalysisResult::SNES(a) => a.print(),
        }
    }

    pub fn json(&self) -> String {
        match self {
            RomAnalysisResult::GameGear(a) => a.json(),
            RomAnalysisResult::GB(a) => a.json(),
            RomAnalysisResult::GBA(a) => a.json(),
            RomAnalysisResult::Genesis(a) => a.json(),
            RomAnalysisResult::MasterSystem(a) => a.json(),
            RomAnalysisResult::N64(a) => a.json(),
            RomAnalysisResult::NES(a) => a.json(),
            RomAnalysisResult::PSX(a) => a.json(),
            RomAnalysisResult::SegaCD(a) => a.json(),
            RomAnalysisResult::SNES(a) => a.json(),
        }
    }

    pub fn region(&self) -> &str {
        match self {
            RomAnalysisResult::GameGear(a) => &a.region,
            RomAnalysisResult::GB(a) => &a.region,
            RomAnalysisResult::GBA(a) => &a.region,
            RomAnalysisResult::Genesis(a) => &a.region,
            RomAnalysisResult::MasterSystem(a) => &a.region,
            RomAnalysisResult::N64(a) => &a.region,
            RomAnalysisResult::NES(a) => &a.region,
            RomAnalysisResult::PSX(a) => &a.region,
            RomAnalysisResult::SegaCD(a) => &a.region,
            RomAnalysisResult::SNES(a) => &a.region,
        }
    }

    pub fn source_name(&self) -> &str {
        match self {
            RomAnalysisResult::GameGear(a) => &a.source_name,
            RomAnalysisResult::GB(a) => &a.source_name,
            RomAnalysisResult::GBA(a) => &a.source_name,
            RomAnalysisResult::Genesis(a) => &a.source_name,
            RomAnalysisResult::MasterSystem(a) => &a.source_name,
            RomAnalysisResult::N64(a) => &a.source_name,
            RomAnalysisResult::NES(a) => &a.source_name,
            RomAnalysisResult::PSX(a) => &a.source_name,
            RomAnalysisResult::SegaCD(a) => &a.source_name,
            RomAnalysisResult::SNES(a) => &a.source_name,
        }
    }
}
