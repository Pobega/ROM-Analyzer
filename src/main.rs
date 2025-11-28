use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt;
use zip::ZipArchive;
use zip::result::ZipError;

#[derive(Debug)]
pub struct RomAnalyzerError {
    details: String,
}

impl RomAnalyzerError {
    fn new(msg: &str) -> RomAnalyzerError {
        RomAnalyzerError { details: msg.to_string() }
    }
}

impl fmt::Display for RomAnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RomAnalyzerError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<ZipError> for RomAnalyzerError {
    fn from(err: ZipError) -> RomAnalyzerError {
        RomAnalyzerError::new(&format!("Zip Error: {}", err))
    }
}

impl From<std::io::Error> for RomAnalyzerError {
    fn from(err: std::io::Error) -> RomAnalyzerError {
        RomAnalyzerError::new(&format!("IO Error: {}", err))
    }
}

const SUPPORTED_ROM_EXTENSIONS: &[&str] = &[
    ".nes",                        // NES
    ".smc", ".sfc",                // SNES
    ".n64", ".v64", ".z64",        // N64
    ".sms",                        // Sega Master System
    ".gg",                         // Sega Game Gear
    ".md", ".gen", ".32x",         // Sega Genesis / 32X
    ".gb", ".gbc",                 // Game Boy / Game Boy Color
    ".gba",                        // Game Boy Advance
    ".scd",                        // Sega CD
    ".iso", ".bin", ".img", ".psx" // CD Systems
];

fn print_separator() {
    println!("{}", "-".repeat(40));
}

fn infer_region_from_filename(name: &str) -> Option<&'static str> {
    let lower_name = name.to_lowercase();

    if lower_name.contains("jap") || lower_name.contains("(j)") || lower_name.contains("[j]") || lower_name.contains("ntsc-j") {
        Some("JAPAN")
    } else if lower_name.contains("usa") || lower_name.contains("(u)") || lower_name.contains("[u]") || lower_name.contains("ntsc-u") || lower_name.contains("ntsc-us") {
        Some("USA")
    } else if lower_name.contains("eur") || lower_name.contains("(e)") || lower_name.contains("[e]") || lower_name.contains("pal") || lower_name.contains("ntsc-e") {
        Some("EUROPE")
    } else {
        None
    }
}

fn normalize_header_region(header_text: &str) -> Option<&'static str> {
    let header_text = header_text.to_uppercase();

    if header_text.contains("JAPAN") || header_text.contains("NTSC-J") || header_text.contains("SLPS") {
        Some("JAPAN")
    } else if header_text.contains("USA") || header_text.contains("AMERICA") || header_text.contains("NTSC-U") || header_text.contains("SLUS") || header_text.contains("CANADA") {
        Some("USA")
    } else if header_text.contains("EUROPE") || header_text.contains("PAL") || header_text.contains("SLES") || header_text.contains("OCEANIA") {
        Some("EUROPE")
    } else {
        None
    }
}

/// Compare the inferred region (via filename) to the region in the ROM's header.
macro_rules! check_region_mismatch {
    ($source_name:expr, $region_name:expr) => {
        let inferred_region = infer_region_from_filename($source_name);
        let header_region_norm = normalize_header_region($region_name);

        if let (Some(inferred), Some(header)) = (inferred_region, header_region_norm) {
            if inferred != header {
                println!("\n*** WARNING: POSSIBLE REGION MISMATCH! ***");
                println!("Source File:  {}", Path::new($source_name).file_name().unwrap_or_default().to_string_lossy());
                println!("Filename suggests: {}", inferred);
                println!("ROM Header claims: {} (Header detail: '{}')", header, $region_name);
                println!("The ROM may be mislabeled or have been patched.");
                println!("*******************************************");
            }
        }
    };
}

fn process_zip_file(file: File, original_filename: &str) -> Result<(), Box<dyn Error>> {
    let mut archive = ZipArchive::new(file)?;
    let mut found_rom = false;

    println!("[+] Analyzing zip archive: {}", original_filename);

    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)?;
        let entry_name = file_in_zip.name().to_string();
        let lower_entry_name = entry_name.to_lowercase();

        if file_in_zip.is_dir() {
            continue;
        }

        let mut is_supported_rom = false;
        for ext in SUPPORTED_ROM_EXTENSIONS {
            if lower_entry_name.ends_with(ext) {
                is_supported_rom = true;
                break;
            }
        }

        if is_supported_rom {
            println!("[+] Found supported ROM in zip: {}", entry_name);
            found_rom = true;
            let mut data = Vec::new();
            file_in_zip.read_to_end(&mut data)?;
            process_rom_data(data, &entry_name)?;
        }
    }

    if !found_rom {
        return Err(Box::new(RomAnalyzerError::new(
            &format!("No supported ROM files found within the zip archive: {}", original_filename)
        )));
    }
    Ok(())
}

// --- NES Region Functions ---
fn get_nes_region_name(flag_9_byte: u8) -> &'static str {
    let is_pal = (flag_9_byte & 0x01) == 0x01;
    if is_pal { "PAL (Europe/Oceania)" } else { "NTSC (USA/Japan)" }
}

fn analyze_nes_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 16 {
        return Err(Box::new(RomAnalyzerError::new(&format!("ROM data is too small to contain an iNES header (size: {} bytes).", data.len()))));
    }

    let signature = &data[0..4];
    if signature != b"NES\x1a" {
        return Err(Box::new(RomAnalyzerError::new("Invalid iNES header signature. Not a valid NES ROM.")));
    }

    let flag_9 = data[9];
    let region_name = get_nes_region_name(flag_9);

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Nintendo Entertainment System (NES)");
    println!("Region:       {}", region_name);
    println!("iNES Flag 9:  0x{:02X}", flag_9);

    check_region_mismatch!(source_name, region_name);
    print_separator();
    Ok(())
}

// --- SNES Region Functions ---
fn get_snes_region_name(code: u8) -> String {
    let regions = vec![
        (0x00, "Japan (NTSC)"), (0x01, "USA / Canada (NTSC)"), (0x02, "Europe / Oceania / Asia (PAL)"),
        (0x03, "Sweden / Scandinavia (PAL)"), (0x04, "Finland (PAL)"), (0x05, "Denmark (PAL)"),
        (0x06, "France (PAL)"), (0x07, "Netherlands (PAL)"), (0x08, "Spain (PAL)"),
        (0x09, "Germany (PAL)"), (0x0A, "Italy (PAL)"), (0x0B, "China (PAL)"),
        (0x0C, "Indonesia (PAL)"), (0x0D, "South Korea (NTSC)"), (0x0E, "Common / International"),
        (0x0F, "Canada (NTSC)"), (0x10, "Brazil (NTSC)"), (0x11, "Australia (PAL)"),
        (0x12, "Other (Variation 1)"), (0x13, "Other (Variation 2)"), (0x14, "Other (Variation 3)"),
    ];
    for (c, name) in regions {
        if c == code { return name.to_string(); }
    }
    format!("Unknown Region (0x{:02X})", code)
}

fn validate_snes_checksum(rom_data: &[u8], header_offset: usize) -> bool {
    use std::convert::TryInto;
    if header_offset + 0x20 > rom_data.len() { return false; }

    let complement_bytes: [u8; 2] = match rom_data[header_offset + 0x1C .. header_offset + 0x1E].try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let checksum_bytes: [u8; 2] = match rom_data[header_offset + 0x1E .. header_offset + 0x20].try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };

    let complement = u16::from_le_bytes(complement_bytes);
    let checksum = u16::from_le_bytes(checksum_bytes);

    (checksum as u32 + complement as u32) == 0xFFFF
}

fn analyze_snes_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    let file_size = data.len();
    let mut header_offset = 0;

    if file_size % 1024 == 512 {
        header_offset = 512;
        println!("[*] Copier header detected (512 bytes). Offsetting reads...");
    }

    let lorom_addr = 0x7FC0 + header_offset;
    let hirom_addr = 0xFFC0 + header_offset;
    let valid_addr: usize;
    let mapping_type: &str;

    if validate_snes_checksum(data, lorom_addr) {
        valid_addr = lorom_addr;
        mapping_type = "LoROM";
    } else if validate_snes_checksum(data, hirom_addr) {
        valid_addr = hirom_addr;
        mapping_type = "HiROM";
    } else {
        println!("[!] Checksum validation failed. Attempting to read from LoROM location as fallback...");
        valid_addr = lorom_addr;
        mapping_type = "LoROM (Unverified)";
    }

    if valid_addr + 0x20 > file_size {
        return Err(Box::new(RomAnalyzerError::new(&format!("ROM data is too small or invalid (size: {} bytes).", file_size))));
    }

    let region_byte_offset = valid_addr + 0x19;
    let region_code = data[region_byte_offset];
    let region_name = get_snes_region_name(region_code);
    let game_title = String::from_utf8_lossy(&data[valid_addr .. valid_addr + 21]).trim().to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Super Nintendo (SNES)");
    println!("Game Title:   {}", game_title);
    println!("Mapping:      {}", mapping_type);
    println!("Region Code:  0x{:02X}", region_code);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- N64 Region Functions ---
fn analyze_n64_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x40 {
        return Err(Box::new(RomAnalyzerError::new("N64 ROM too small.")));
    }

    let country_code = String::from_utf8_lossy(&data[0x3E..0x40]).trim_matches(char::from(0)).to_string();

    let region_name = match country_code.as_ref() {
        "E" => "USA / NTSC", "J" => "Japan / NTSC", "P" => "Europe / PAL",
        "D" => "Germany / PAL", "F" => "France / PAL", "U" => "USA (Legacy)",
        _ => "Unknown Code",
    }.to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Nintendo 64 (N64)");
    println!("Region:       {}", region_name);
    println!("Code:         {}", country_code);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- Sega Master System Region Functions ---
fn analyze_mastersystem_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x7FFD {
        return Err(Box::new(RomAnalyzerError::new(&format!("ROM data is too small to contain a Master System header (size: {} bytes, requires at least 0x7FFD).", data.len()))));
    }

    // SMS Region/Language byte is at offset 0x7FFC
    let sms_region_byte = data[0x7FFC];
    let region_name = match sms_region_byte {
        0x30 => "Japan (NTSC)",
        0x4C => "Europe / Overseas (PAL/NTSC)",
        _ => "Unknown Code",
    }.to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sega Master System");
    println!("Region Code:  0x{:02X}", sms_region_byte);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- Sega Game Gear Region Functions ---
fn analyze_gamegear_data(_data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    // Sega Game Gear ROMs, like Master System, often lack a standardized region code in the header.
    // Region is typically inferred from filename.

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sega Game Gear");
    println!("Note:         Detailed region information often not available in header.");

    // Attempt to infer from filename if possible
    if let Some(inferred) = infer_region_from_filename(source_name) {
        println!("Region (inferred from filename): {}", inferred);
    } else {
        println!("Region:       Unknown (Filename inference failed).");
    }
    print_separator();
    Ok(())
}

// --- Game Boy / Game Boy Color Region Functions ---
fn analyze_gb_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x14B {
        return Err(Box::new(RomAnalyzerError::new(&format!("ROM data is too small to contain a Game Boy header (size: {} bytes, requires at least 0x14B).", data.len()))));
    }

    let system_type = if data[0x143] == 0x80 || data[0x143] == 0xC0 {
        "Game Boy Color (GBC)"
    } else {
        "Game Boy (GB)"
    };

    let game_title = String::from_utf8_lossy(&data[0x134..0x143]).trim_matches(char::from(0)).to_string();

    let destination_code = data[0x14A];
    let region_name = match destination_code {
        0x00 => "Japan",
        0x01 => "Non-Japan (International)",
        _ => "Unknown Code",
    }.to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       {}", system_type);
    println!("Game Title:   {}", game_title);
    println!("Region Code:  0x{:02X}", destination_code);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- Game Boy Advance (GBA) Region Functions ---
fn analyze_gba_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0xC0 {
        return Err(Box::new(RomAnalyzerError::new(&format!("ROM data is too small to contain a GBA header (size: {} bytes).", data.len()))));
    }

    // GBA header is at 0x0. Relevant info: Game Title (0xA0-0xAC), Game Code (0xAC-0xB0), Maker Code (0xB0-0xB2), Unit Code (0xB3), Region (0xB4).

    let game_title = String::from_utf8_lossy(&data[0xA0..0xAC]).trim_matches(char::from(0)).to_string();
    let game_code = String::from_utf8_lossy(&data[0xAC..0xB0]).trim_matches(char::from(0)).to_string();
    let maker_code = String::from_utf8_lossy(&data[0xB0..0xB2]).trim_matches(char::from(0)).to_string();
    let region_code = data[0xB4]; // Typically 0 for Japan, 1 for USA, 2 for Europe, etc. or ASCII representation

    let region_name = match region_code {
        0x00 => "Japan",
        0x01 => "USA",
        0x02 => "Europe",
        // Other common region codes as ASCII characters
        b'J' => "Japan",
        b'U' => "USA",
        b'E' => "Europe",
        b'P' => "Europe", // PAL
        _ => "Unknown",
    }.to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Game Boy Advance (GBA)");
    println!("Game Title:   {}", game_title);
    println!("Game Code:    {}", game_code);
    println!("Maker Code:   {}", maker_code);
    println!("Region Code:  0x{:02X} ('{}')", region_code, region_code as char);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- Sega Genesis / 32X Region Functions ---
fn analyze_sega_cartridge_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    // Sega Genesis header is at offset 0x100. It's 256 bytes long.
    // Region byte is at offset 0x1F0 relative to the start of the ROM (or 0xF0 relative to header start).

    if data.len() < 0x200 {
        return Err(Box::new(RomAnalyzerError::new(&format!("ROM data is too small to contain a Sega header (size: {} bytes).", data.len()))));
    }

    let header_start = 0x100;

    // Verify Sega header signature "SEGA MEGA DRIVE " or "SEGA GENESIS"
    let console_name = String::from_utf8_lossy(&data[header_start + 0x0..header_start + 0x10]).trim().to_string();
    if console_name != "SEGA MEGA DRIVE" && console_name != "SEGA GENESIS" {
        // For .bin files, this might be a false positive, so print a warning rather than erroring out.
        println!("[!] Warning: Sega header signature not found at 0x100 for {}. Console name: '{}'", source_name, console_name);
    }

    let game_title_domestic = String::from_utf8_lossy(&data[header_start + 0x10..header_start + 0x30]).trim().to_string();
    let game_title_international = String::from_utf8_lossy(&data[header_start + 0x30..header_start + 0x50]).trim().to_string();

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
    }.to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       {}", console_name);
    println!("Game Title (Domestic): {}", game_title_domestic);
    println!("Game Title (Int.):   {}", game_title_international);
    println!("Region Code:  0x{:02X} ('{}')", region_code_byte, region_code_byte as char);
    println!("Region:       {}", region_name);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- PlayStation (PSX) and Sega CD Disc Image Analysis ---
fn analyze_psx_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    // Check the first 128KB (0x20000 bytes)
    let check_size = std::cmp::min(data.len(), 0x20000);
    if check_size < 0x2000 { // Need enough data for Volume Descriptor/Boot file
        return Err(Box::new(RomAnalyzerError::new("PSX boot file too small for reliable analysis.")));
    }

    let data_sample = &data[..check_size].to_ascii_uppercase();

    let region_map = [
        ("SLUS".as_bytes(), "North America (NTSC-U)"),
        ("SLES".as_bytes(), "Europe (PAL)"),
        ("SLPS".as_bytes(), "Japan (NTSC-J)"),
    ];

    let mut found_prefix = None;
    let mut region_name = "Unknown";

    for (prefix, region) in region_map.iter() {
        if data_sample.windows(prefix.len()).any(|window| window == *prefix) {
            found_prefix = Some(prefix);
            region_name = region;
            break;
        }
    }

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sony PlayStation (PSX)");
    println!("Region:       {}", region_name);
    println!("Code:         {}", found_prefix.map(|p| String::from_utf8_lossy(p).to_string()).unwrap_or_else(|| "N/A".to_string()));

    if found_prefix.is_none() {
        println!("Note: Executable prefix (SLUS/SLES/SLPS) not found in header area. Requires main data track (.bin or .iso).");
    }

    check_region_mismatch!(source_name, region_name);
    print_separator();
    Ok(())
}


fn analyze_segacd_data(data: &[u8], source_name: &str) -> Result<(), Box<dyn Error>> {
    if data.len() < 0x200 {
        return Err(Box::new(RomAnalyzerError::new("Sega CD boot file too small.")));
    }

    let signature = String::from_utf8_lossy(&data[0x100..0x107]).trim().to_string();
    if signature != "SEGA CD" && signature != "SEGA MEGA" {
        println!("[!] Warning: File does not appear to be a standard Sega CD boot file (no SEGA CD signature at 0x100).");
    }

    // Region byte is at offset 0x10B in the boot program
    let region_code = data[0x10B];

    let region_name = match region_code {
        0x40 => "Japan (NTSC-J)",
        0x80 => "Europe (PAL)",
        0xC0 => "USA (NTSC-U)",
        0x00 => "Unrestricted/BIOS region",
        _ => "Unknown Code",
    }.to_string();

    print_separator();
    println!("Source:       {}", source_name);
    println!("System:       Sega CD / Mega CD");
    println!("Region:       {}", region_name);
    println!("Code:         0x{:02X}", region_code);

    check_region_mismatch!(source_name, &region_name);
    print_separator();
    Ok(())
}

// --- CHD Support (Placeholder for FFI) ---
fn analyze_chd_file(_filepath: &Path, source_name: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=======================================================");
    println!("  CHD ANALYSIS: Requires External Library (libchd)");
    println!("=======================================================");

    println!("In a real Rust environment, this function would use FFI (Foreign Function Interface) to bind to the MAME 'libchd' C library.");
    println!("This library would decompress the hunks of data and extract the raw contents (e.g., a .BIN file) from the archive.");

    // --- Conceptual Logic ---
    // 1. FFI Call: chd_api::open(filepath) -> chd_handle
    // 2. FFI Call: chd_api::read_raw_track(chd_handle) -> raw_data (Vec<u8>)

    // As a placeholder, we will simulate the failure expected due to the lack of FFI,
    // but clearly show the intended flow would route to the disc analysis.

    // 3. Routing: process_rom_data(raw_data, virtual_filename)

    // For demonstration, let's assume the CHD file contained PSX data.
    // We cannot proceed without the external dependency.
    return Err(Box::new(RomAnalyzerError::new(
        &format!("CHD analysis for {} failed: FFI library 'libchd' is missing.", source_name)
    )));
}

// --- Main Logic and Dispatcher ---
fn process_rom_data(data: Vec<u8>, name: &str) -> Result<(), Box<dyn Error>> {
    let lower_name = name.to_lowercase();

    if lower_name.ends_with(".nes") {
        analyze_nes_data(&data, name)
    } else if lower_name.ends_with(".smc") || lower_name.ends_with(".sfc") {
        analyze_snes_data(&data, name)
    } else if lower_name.ends_with(".n64") || lower_name.ends_with(".v64") || lower_name.ends_with(".z64") {
        analyze_n64_data(&data, name)
    } else if lower_name.ends_with(".sms") {
        analyze_mastersystem_data(&data, name)
    } else if lower_name.ends_with(".gg") {
        analyze_gamegear_data(&data, name)
    } else if lower_name.ends_with(".gb") || lower_name.ends_with(".gbc") {
        analyze_gb_data(&data, name)
    } else if lower_name.ends_with(".gba") {
        analyze_gba_data(&data, name)
    } else if lower_name.ends_with(".md") || lower_name.ends_with(".gen") || lower_name.ends_with(".32x") {
        analyze_sega_cartridge_data(&data, name)
    } else if lower_name.ends_with(".scd") {
        analyze_segacd_data(&data, name)
    } else if lower_name.ends_with(".iso") || lower_name.ends_with(".bin") || lower_name.ends_with(".img") || lower_name.ends_with(".psx") {
        // For .bin files, first check for Sega Genesis/32X header
        if data.len() > 0x110 && (data[0x100..0x110].starts_with(b"SEGA MEGA DRIVE") || data[0x100..0x110].starts_with(b"SEGA GENESIS")) {
            analyze_sega_cartridge_data(&data, name)
        } else if data.len() > 0x10A && &data[0x100..0x107].to_ascii_uppercase() == b"SEGA CD" {
            analyze_segacd_data(&data, name)
        } else {
            analyze_psx_data(&data, name)
        }
    } else {
        Err(Box::new(RomAnalyzerError::new(&format!("Unrecognized ROM file extension for dispatch: {}", name))))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: rom_analyzer <path_to_rom_or_chd>");
        return Ok(())
    }

    let filepath = PathBuf::from(&args[1]);
    let filename = filepath.file_name().unwrap_or_default().to_string_lossy().to_string();
    let lower_filename = filename.to_lowercase();

    if !filepath.exists() {
        return Err(Box::new(RomAnalyzerError::new(&format!("Error: File '{}' not found.", filename))));
    }

    // --- CHD File Handling ---
    if lower_filename.ends_with(".chd") {
        // This is the dedicated path for CHD.
        return analyze_chd_file(&filepath, &filename);
    }

    // --- Standard ROM File Handling ---
    if lower_filename.ends_with(".cue") {
        return Err(Box::new(RomAnalyzerError::new("Error: .cue files are metadata only. Please provide the corresponding .bin or .iso file, or the .chd archive.")));
    }

    if lower_filename.ends_with(".zip") {
        let file = File::open(&filepath)?;
        return process_zip_file(file, &filename);
    }

    let mut file = File::open(&filepath)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    println!("[+] Successfully read {} bytes from '{}'", data.len(), filename);

    process_rom_data(data, &filename)
}

// NOTE: This Rust code is designed to be runnable via `cargo run -- <filepath>`.
// To enable true CHD support, the `analyze_chd_file` function requires a third-party Rust crate
// or custom FFI bindings to the MAME 'libchd' C library.
