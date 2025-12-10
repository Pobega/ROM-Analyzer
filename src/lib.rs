//! The `rom_analyzer` crate provides functionality to analyze various ROM file formats from
//! classic video game consoles. It aims to extract metadata such as region, game title, publisher,
//! and other console-specific details from ROM headers and file names.
//!
//! This library supports a range of console ROMs, including but not limited to NES, SNES, N64,
//! Sega Master System, Game Gear, Game Boy, Game Boy Advance, Sega Genesis, and Sega CD.  It can
//! also handle ROMs packaged as ZIP or CHD (Compressed Hunks of Data) archives.
//!
//! The primary entry point for analysis is the `analyze_rom_data` function, which takes a file
//! path and returns a `RomAnalysisResult` enum containing console-specific analysis data.

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

/// # Architecture Overview
///
/// The ROM-Analyzer library follows a modular architecture designed for extensibility
/// and maintainability. The analysis pipeline consists of several key components:
///
/// ## Main Components
///
/// 1. **Entry Point**: [`analyze_rom_data`] - The primary public function that handles
///    file type detection and delegates to appropriate processors.
///
/// 2. **File Processing Layer**: Handles different file formats:
///    - **Raw ROM files**: Processed by console-specific analyzers
///    - **ZIP archives**: Processed by [`crate::archive::zip::process_zip_file`]
///    - **CHD archives**: Processed by [`crate::archive::chd::analyze_chd_file`]
///
/// 3. **Dispatch Layer**: Determines console type using [`get_rom_file_type`] and
///    routes to console-specific analyzers.
///
/// 4. **Console Analyzers**: Module-specific analysis functions in [`console`] module:
///    - [`crate::console::nes::analyze_nes_data`]
///    - [`crate::console::snes::analyze_snes_data`]
///    - [`crate::console::n64::analyze_n64_data`]
///    - And other console-specific analyzers
///
/// 5. **Result Unification**: [`RomAnalysisResult`] enum that provides a unified
///    interface for all console-specific analysis results.
///
/// ## Data Flow
///
/// ```plaintext
/// File Input (e.g., "game.nes")
///     ↓
/// analyze_rom_data (entry point)
///     ↓
/// File Type Detection (by extension)
///     ↓
/// Archive Processing (if ZIP/CHD)
///     ↓
/// process_rom_data (dispatch)
///     ↓
/// Console-Specific Analyzer
///     ↓
/// RomAnalysisResult (unified output)
/// ```
///
/// ## Key Design Patterns
///
/// 1. **Strategy Pattern**: Different console analyzers implement the same interface
///    for analyzing ROM data.
///
/// 2. **Factory Pattern**: The dispatch layer acts as a factory that creates
///    appropriate analyzer instances based on file type.
///
/// 3. **Adapter Pattern**: Archive processors adapt ZIP/CHD formats to raw ROM data
///    that can be processed by console analyzers.
///
/// 4. **Unified Interface**: [`RomAnalysisResult`] provides a common interface for
///    accessing console-specific data through trait-like methods.
///
/// ## Extensibility
///
/// The architecture is designed to be easily extended:
///
/// 1. **Adding new console support**: Create a new module in [`console`] with
///    appropriate analysis functions and add a new variant to [`RomAnalysisResult`].
///
/// 2. **Adding new archive formats**: Create a new module in [`archive`] and
///    add a new branch to the match statement in [`analyze_rom_data`].
///
/// 3. **Adding new analysis features**: Extend the console-specific analysis structs
///    and update the [`RomAnalysisResult`] methods accordingly.
///
/// ## Error Handling
///
/// The library uses Rust's `Result` type throughout, with `Box<dyn Error>` as the
/// error type to allow for different error types from various components. Custom
/// errors are defined in the [`error`] module.

/// A list of file extensions that the ROM analyzer supports.
///
/// This constant contains all the file extensions recognized by the ROM analyzer,
/// organized by console system. These extensions are used by [`get_rom_file_type`]
/// to determine how to process each file.
///
/// # Supported Extensions by Console
///
/// * **Nintendo Entertainment System (NES)**: `.nes`
/// * **Super Nintendo (SNES)**: `.smc`, `.sfc`
/// * **Nintendo 64 (N64)**: `.n64`, `.v64`, `.z64`
/// * **Sega Master System**: `.sms`
/// * **Sega Game Gear**: `.gg`
/// * **Sega Genesis/Mega Drive**: `.md`, `.gen`, `.32x`
/// * **Nintendo Game Boy**: `.gb`, `.gbc`
/// * **Nintendo Game Boy Advance**: `.gba`
/// * **Sega CD**: `.scd`
/// * **CD-based Systems** (PlayStation, Sega CD, etc.): `.iso`, `.bin`, `.img`, `.psx`
/// * **Archives**: `.zip`, `.chd` (not included in this list but supported by [`analyze_rom_data`])
///
/// # Usage
///
/// This constant can be used to validate file extensions or to display supported formats
/// in user interfaces:
///
/// ```rust
/// use rom_analyzer::SUPPORTED_ROM_EXTENSIONS;
///
/// fn is_supported_extension(extension: &str) -> bool {
///     SUPPORTED_ROM_EXTENSIONS.contains(&extension)
/// }
///
/// assert!(is_supported_extension(".nes"));
/// assert!(is_supported_extension(".gba"));
/// assert!(!is_supported_extension(".exe"));
/// ```
///
/// # Note
///
/// Some extensions like `.bin` can be ambiguous and may require additional header
/// analysis to determine the exact console type. The analyzer handles this automatically
/// during the analysis process.
///
/// # See Also
///
/// * [`get_rom_file_type`] - Converts extensions to console types
/// * [`analyze_rom_data`] - Main analysis function that uses these extensions
/// * [`RomFileType`] - Enum representing the console types
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
///
/// This enum serves as a unified return type for ROM analysis, containing console-specific
/// analysis data. Each variant corresponds to a different gaming console and contains
/// a struct with detailed metadata extracted from the ROM header.
///
/// The enum uses Serde's `tag` attribute with `"console"` to enable serialization that
/// includes the console type as a discriminator field, making it suitable for JSON
/// and other serialization formats.
///
/// # Variants
///
/// * `GameGear(GameGearAnalysis)` - Sega Game Gear ROM analysis
/// * `GB(GbAnalysis)` - Nintendo Game Boy (including Color) ROM analysis
/// * `GBA(GbaAnalysis)` - Nintendo Game Boy Advance ROM analysis
/// * `Genesis(GenesisAnalysis)` - Sega Genesis/Mega Drive ROM analysis
/// * `MasterSystem(MasterSystemAnalysis)` - Sega Master System ROM analysis
/// * `N64(N64Analysis)` - Nintendo 64 ROM analysis
/// * `NES(NesAnalysis)` - Nintendo Entertainment System ROM analysis
/// * `PSX(PsxAnalysis)` - Sony PlayStation ROM analysis
/// * `SegaCD(SegaCdAnalysis)` - Sega CD ROM analysis
/// * `SNES(SnesAnalysis)` - Super Nintendo Entertainment System ROM analysis
///
/// # Methods
///
/// The enum provides several convenience methods for accessing common fields across
/// all console types:
///
/// * [`print()`](RomAnalysisResult::print) - Returns a formatted string of the analysis
/// * [`source_name()`](RomAnalysisResult::source_name) - Gets the original file name
/// * [`region()`](RomAnalysisResult::region) - Gets the region string
/// * [`region_mismatch()`](RomAnalysisResult::region_mismatch) - Checks for region conflicts
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::{RomAnalysisResult, SUPPORTED_ROM_EXTENSIONS};
///
/// // Check if a file extension is supported
/// fn is_supported_rom(extension: &str) -> bool {
///     SUPPORTED_ROM_EXTENSIONS.contains(&extension)
/// }
///
/// assert!(is_supported_rom(".nes"));
/// assert!(is_supported_rom(".gba"));
/// assert!(!is_supported_rom(".exe"));
///
/// // Example of handling different console types (conceptual)
/// fn handle_analysis_result(result: RomAnalysisResult) {
///     match result {
///         RomAnalysisResult::NES(analysis) => {
///             println!("NES ROM: {}", analysis.region_string);
///         }
///         RomAnalysisResult::SNES(analysis) => {
///             println!("SNES ROM: {}", analysis.game_title);
///         }
///         // Handle other console types...
///         _ => println!("Other console ROM"),
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`analyze_rom_data`] - The main function that returns this enum
/// * Console-specific analysis structs like [`crate::console::nes::NesAnalysis`]
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
///
/// This enum is used internally by the ROM analyzer to categorize files and dispatch
/// them to the appropriate console-specific analysis functions. The classification is
/// primarily based on file extensions, with some additional header analysis for
/// ambiguous cases (e.g., CD-based systems).
///
/// # Variants
///
/// * `Nes` - Nintendo Entertainment System (`.nes`)
/// * `Snes` - Super Nintendo Entertainment System (`.smc`, `.sfc`)
/// * `N64` - Nintendo 64 (`.n64`, `.v64`, `.z64`)
/// * `MasterSystem` - Sega Master System (`.sms`)
/// * `GameGear` - Sega Game Gear (`.gg`)
/// * `GameBoy` - Nintendo Game Boy / Game Boy Color (`.gb`, `.gbc`)
/// * `GameBoyAdvance` - Nintendo Game Boy Advance (`.gba`)
/// * `Genesis` - Sega Genesis/Mega Drive (`.md`, `.gen`, `.32x`)
/// * `SegaCD` - Sega CD (`.scd`)
/// * `CDSystem` - CD-based systems (`.iso`, `.bin`, `.img`, `.psx`, `.chd`) - requires header analysis
/// * `Unknown` - Unrecognized or unsupported file types
///
/// # Usage
///
/// This enum is primarily used by [`get_rom_file_type`] for determining which
/// console-specific analyzer to use. The `CDSystem` variant is special as it requires
/// additional header analysis to distinguish between different consoles that use
/// similar file extensions.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::{get_rom_file_type, RomFileType};
///
/// assert_eq!(get_rom_file_type("game.nes"), RomFileType::Nes);
/// assert_eq!(get_rom_file_type("game.smc"), RomFileType::Snes);
/// assert_eq!(get_rom_file_type("game.bin"), RomFileType::CDSystem);
/// assert_eq!(get_rom_file_type("game.txt"), RomFileType::Unknown);
/// ```
///
/// # See Also
///
/// * [`get_rom_file_type`] - Converts file extensions to this enum
/// * [`analyze_rom_data`] - Uses this enum for dispatching to analyzers
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
/// This function is used internally to normalize file extensions for consistent
/// comparison when determining ROM file types. It handles the conversion from
/// `OsStr` to UTF-8 strings and ensures case-insensitive matching.
///
/// # Arguments
///
/// * `file_path` - The path to the file as a string slice. This can be a full
///   path, relative path, or just a filename.
///
/// # Returns
///
/// A `String` containing the lowercase file extension without the leading dot,
/// or an empty string if:
/// * The file has no extension
/// * The extension cannot be converted to a valid UTF-8 string
///
/// # Note
///
/// This function is used internally and not exposed in the public API. For public
/// API usage, see [`get_rom_file_type`] which uses this function internally.
///
/// # See Also
///
/// * [`get_rom_file_type`] - Uses this function to determine the ROM file type
/// * [`process_rom_data`] - Uses file extensions for console-specific dispatching
fn get_file_extension_lowercase(file_path: &str) -> String {
    Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase()
}

/// Maps a file's **extension** to the corresponding **RomFileType** for supported consoles.
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
/// A **RomFileType** variant corresponding to the file extension:
///
/// * **RomFileType::Nes** for `nes`
/// * **RomFileType::Snes** for `smc` or `sfc`
/// * **RomFileType::N64** for `n64`, `v64`, or `z64`
/// * **RomFileType::MasterSystem** for `sms`
/// * **RomFileType::GameGear** for `gg`
/// * **RomFileType::GameBoy** for `gb` or `gbc`
/// * **RomFileType::GameBoyAdvance** for `gba`
/// * **RomFileType::Genesis** for `md`, `gen`, or `32x`
/// * **RomFileType::SegaCD** for `scd`
/// * **RomFileType::CDSystem** for `iso`, `bin`, `img`, `psx`, or `chd`
/// * **RomFileType::Unknown** for any other extension.
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
/// This function serves as the core dispatch mechanism for ROM analysis. It determines
/// the console type from the file extension using [`get_rom_file_type`], then routes
/// the raw ROM data to the appropriate console-specific analysis module.
///
/// For CD-based systems (identified by extensions like `.iso`, `.bin`, `.img`, `.psx`, `.chd`),
/// this function performs additional header analysis to distinguish between different
/// console types that may share the same file extensions (e.g., Sega Genesis vs Sega CD
/// vs PlayStation).
///
/// # Arguments
///
/// * `data` - A `Vec<u8>` containing the raw bytes of the ROM file. This should contain
///   the complete ROM data including any headers.
/// * `rom_path` - The path to the ROM file, used to infer the file type through its
///   extension. This is used for dispatching to the correct analysis function.
///
/// # Returns
///
/// A `Result` containing either:
/// * `Ok(RomAnalysisResult)` - The analysis result with console-specific metadata
/// * `Err(Box<dyn Error>)` - An error if:
///   * The file extension is unrecognized
///   * The console-specific analyzer encounters an error (e.g., invalid header data)
///   * The ROM data is too small for header analysis
///
/// # Errors
///
/// This function can return errors in several scenarios:
///
/// * **Unrecognized extension**: If the file extension doesn't match any supported console
/// * **Invalid ROM data**: If the ROM data is too small or has invalid headers
/// * **Console-specific errors**: If the dispatched analyzer encounters format-specific issues
///
/// # Note
///
/// This function is used internally and not exposed in the public API. For public
/// API usage, see [`analyze_rom_data`] which calls this function internally.
///
/// # See Also
///
/// * [`get_rom_file_type`] - Determines the console type from file extension
/// * [`analyze_rom_data`] - The main public entry point that calls this function
/// * Console-specific analyzers like [`crate::console::nes::analyze_nes_data`]
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
/// This is the primary public function for analyzing ROM files and serves as the main
/// entry point to the ROM-Analyzer library. It provides a unified interface for analyzing
/// various ROM file formats, including both raw ROM files and archive formats.
///
/// The function automatically detects the file type and applies the appropriate analysis:
///
/// * **Raw ROM files**: Directly analyzes the ROM data using console-specific analyzers
/// * **ZIP archives**: Extracts and analyzes the first ROM file found in the archive
/// * **CHD archives**: Decompresses and analyzes the ROM data from CHD format
///
/// # Supported File Formats
///
/// This function supports the following file extensions:
///
/// * **Nintendo**: `.nes`, `.smc`, `.sfc`, `.n64`, `.v64`, `.z64`, `.gb`, `.gbc`, `.gba`
/// * **Sega**: `.sms`, `.gg`, `.md`, `.gen`, `.32x`, `.scd`
/// * **Sony**: `.iso`, `.bin`, `.img`, `.psx`
/// * **Archives**: `.zip`, `.chd`
///
/// # Arguments
///
/// * `file_path` - The path to the ROM file or archive as a string slice. This can be
///   an absolute path, relative path, or just a filename.
///
/// # Returns
///
/// A `Result` containing either:
/// * `Ok(RomAnalysisResult)` - The analysis result with console-specific metadata
/// * `Err(Box<dyn Error>)` - An error if:
///   * The file cannot be opened or read
///   * The file format is not supported
///   * The ROM data is invalid or corrupted
///   * The archive contains no valid ROM files
///
/// # Errors
///
/// This function can return errors in several scenarios:
///
/// * **File I/O errors**: If the file cannot be opened or read (permission issues, missing file)
/// * **Unsupported format**: If the file extension is not recognized
/// * **Archive errors**: If ZIP/CHD archives are corrupted or contain no valid ROMs
/// * **ROM analysis errors**: If the ROM data has invalid headers or is too small
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::analyze_rom_data;
///
/// // Analyze a raw NES ROM file
/// let result = analyze_rom_data("path/to/game.nes");
/// match result {
///     Ok(analysis) => println!("Analysis successful: {:?}", analysis),
///     Err(e) => eprintln!("Error analyzing ROM: {}", e),
/// }
///
/// // Analyze a ROM inside a ZIP archive
/// let result = analyze_rom_data("path/to/roms.zip");
/// match result {
///     Ok(analysis) => println!("ZIP analysis successful"),
///     Err(e) => eprintln!("ZIP analysis failed: {}", e),
/// }
/// ```
///
/// # See Also
///
/// * [`RomAnalysisResult`] - The enum containing console-specific analysis results
/// * [`analyze_rom_data`] - The main function that performs console-specific analysis
/// * [`SUPPORTED_ROM_EXTENSIONS`] - List of all supported file extensions
///
/// # Panics
///
/// This function should not panic under normal circumstances. However, it may panic if
/// there are severe system-level issues (e.g., out of memory when reading large files).
///
/// # Performance
///
/// Performance varies by file type:
/// * **Raw ROM files**: Fast, direct memory mapping
/// * **ZIP archives**: Moderate, depends on compression and archive size
/// * **CHD archives**: Slower, requires decompression of the entire CHD structure
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
