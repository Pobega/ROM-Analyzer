pub mod archive;
pub mod console;
pub mod dispatcher;
pub mod error;
pub mod region;

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
