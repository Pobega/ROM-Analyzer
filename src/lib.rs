//! The `rom_analyzer` crate provides functionality to analyze various ROM file formats from
//! classic video game consoles. It aims to extract metadata such as region, game title, publisher,
//! and other console-specific details from ROM headers and file names.
//!
//! This library supports a range of console ROMs, including but not limited to NES, SNES, N64,
//! Sega Master System, Game Gear, Game Boy, Game Boy Advance, Sega Genesis, and Sega CD.  It can
//! also handle ROMs packaged as ZIP or CHD (Compressed Hunks of Data) archives.
//!
//! The primary entry point for analysis is the [`analyze_rom_data`] function, which takes a file
//! path and returns a [`RomAnalysisResult`] enum containing console-specific analysis data.

pub mod archive;
pub mod console;
pub mod error;
pub mod region;

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

/// A list of file extensions that the ROM analyzer supports.
/// These extensions are used to determine the type of ROM file being processed.
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

/// Represents the analysis result for a ROM file.
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

/// Represents the type of ROM file based on its extension.
/// This enum is used internally to dispatch to the correct analysis logic.
#[derive(Debug, PartialEq, Eq)]
pub enum RomFileType {
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

/// Extracts the file extension from a given file path and converts it to lowercase.
///
/// # Arguments
///
/// * `file_path` - The path to the file.
///
/// # Returns
///
/// A `String` containing the lowercase file extension, or an empty string if no
/// extension is found.
fn get_file_extension_lowercase(file_path: &str) -> String {
    Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase()
}

/// Maps a file's **extension** to the corresponding [`RomFileType`] for supported consoles.
///
/// The file extension is extracted from the provided name, converted to lowercase
/// and matched against a predefined list of extensions for different retro gaming systems.
///
/// # Arguments
///
/// * `name` - The full file name, which may or may not include a path (e.g., `"game/zelda.nes"`).
///
/// # Returns
///
/// A [`RomFileType`] variant corresponding to the file extension:
///
/// * [`RomFileType::Nes`] for `nes`
/// * [`RomFileType::Snes`] for `smc` or `sfc`
/// * [`RomFileType::N64`] for `n64`, `v64`, or `z64`
/// * [`RomFileType::MasterSystem`] for `sms`
/// * [`RomFileType::GameGear`] for `gg`
/// * [`RomFileType::GameBoy`] for `gb` or `gbc`
/// * [`RomFileType::GameBoyAdvance`] for `gba`
/// * [`RomFileType::Genesis`] for `md`, `gen`, or `32x`
/// * [`RomFileType::SegaCD`] for `scd`
/// * [`RomFileType::CDSystem`] for `iso`, `bin`, `img`, `psx`, or `chd`
/// * [`RomFileType::Unknown`] for any other extension.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::{get_rom_file_type, RomFileType};
///
/// let rom_type_nes = get_rom_file_type("game.NES");
/// assert_eq!(rom_type_nes, RomFileType::Nes);
///
/// let rom_type_snes = get_rom_file_type("chrono.sfc");
/// assert_eq!(rom_type_snes, RomFileType::Snes);
///
/// let unknown = get_rom_file_type("document.txt");
/// assert_eq!(unknown, RomFileType::Unknown);
/// ```
pub fn get_rom_file_type(name: &str) -> RomFileType {
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

/// Processes raw ROM data based on its determined file type.
///
/// This function takes the raw byte data of a ROM file and its path, determines
/// the console type using [`get_rom_file_type`] and then dispatches the data to
/// the appropriate console-specific analysis function.
///
/// # Arguments
///
/// * `data` - A `Vec<u8>` containing the raw bytes of the ROM file.
/// * `rom_path` - The path to the ROM file, used to infer the file type.
///
/// # Returns
///
/// A `Result` containing either a [`RomAnalysisResult`] with the analysis data
/// or a [`RomAnalyzerError`].
fn process_rom_data(data: Vec<u8>, rom_path: &str) -> Result<RomAnalysisResult, RomAnalyzerError> {
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
        RomFileType::Unknown => Err(RomAnalyzerError::UnsupportedFormat(format!(
            "Unrecognized ROM file extension for dispatch: {}",
            rom_path
        ))),
    }
}

/// Analyze the header data of a ROM file.
///
/// This is the primary public function for analyzing ROM files. It handles different
/// file types (including archives like ZIP and CHD) by first processing them to
/// extract the ROM data, and then dispatches the data to `process_rom_data` for
/// console-specific analysis.
///
/// # Arguments
///
/// * `file_path` - The path to the ROM file or archive.
///
/// # Returns
///
/// A `Result` containing either a [`RomAnalysisResult`] with the analysis data
/// or a [`RomAnalyzerError`].
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::analyze_rom_data;
///
/// let result = analyze_rom_data("path/to/your/rom.nes");
/// match result {
///     Ok(analysis) => println!("Analysis successful!"),
///     Err(e) => eprintln!("Error analyzing ROM: {}", e),
/// }
/// ```
pub fn analyze_rom_data(file_path: &str) -> Result<RomAnalysisResult, RomAnalyzerError> {
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
        /// Calls the `$fn_name` method on the inner console-specific analysis struct.
        /// This allows a common interface for accessing console-specific data.
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
        /// Provides read-only access to the `$field` field of the inner console-specific analysis struct.
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
        /// Provides access to the `$field` field of the inner console-specific analysis struct.
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
    impl_rom_analysis_accessor!(region, region_string, &str);
    impl_rom_analysis_accessor!(region_mismatch, region_mismatch, bool);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::{FileOptions, ZipWriter};

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
        assert!(!err.to_string().contains("PSX"));
    }

    #[test]
    fn test_process_rom_data_cd_system_sega_genesis_header_genesis() {
        let mut data = vec![0; 0x120];
        data[0x100..0x110].copy_from_slice(b"SEGA GENESIS    ");
        let name = "game.bin";
        let result = process_rom_data(data, name);
        assert!(result.is_err());
        let err = result.expect_err("process_rom_data should have returned an error for mock data");
        assert!(!err.to_string().contains("Unrecognized ROM file extension"));
        assert!(!err.to_string().contains("PSX"));
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

    #[test]
    fn test_analyze_rom_data_zip() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        zip.start_file("game.nes", FileOptions::default()).unwrap();
        zip.write_all(b"NES ROM DATA").unwrap();
        zip.finish().unwrap();
        let zip_path_str = zip_path.to_str().unwrap();
        let result = analyze_rom_data(zip_path_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.to_string().contains("Unrecognized ROM file extension"));
    }

    #[test]
    fn test_analyze_rom_data_chd() {
        let dir = tempdir().unwrap();
        let chd_path = dir.path().join("test.chd");
        std::fs::write(&chd_path, b"invalid chd data").unwrap();
        let chd_path_str = chd_path.to_str().unwrap();
        let result = analyze_rom_data(chd_path_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.to_string().contains("Unrecognized ROM file extension"));
        assert!(!err.to_string().contains("PSX"));
    }
}
