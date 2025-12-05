//! This module contains console-specific analysis logic for various retro gaming systems.
//!
//! Each submodule corresponds to a different console and provides functions
//! and data structures for parsing ROM headers, extracting metadata, and performing
//! other console-specific analyses.

pub mod gamegear;
pub mod gb;
pub mod gba;
pub mod genesis;
pub mod mastersystem;
pub mod n64;
pub mod nes;
pub mod psx;
pub mod segacd;
pub mod snes;
