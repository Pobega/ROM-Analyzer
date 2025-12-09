//! Provides utilities for inferring and normalizing geographical regions
//! from ROM filenames and header information.
//!
//! This module helps in identifying the target region (e.g., Japan, USA, Europe)
//! of a ROM, which is crucial for accurate analysis and categorization.

use std::fmt;

use bitflags::bitflags;
use serde::Serialize;

bitflags! {
    /// A bitflag struct representing geographical regions.
    /// Allows a ROM to belong to multiple regions (e.g., NES NTSC = USA + JAPAN).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    pub struct Region: u8 {

        const UNKNOWN = 0;
        const JAPAN = 1 << 0;
        const USA = 1 << 1;
        const EUROPE = 1 << 2;
        const RUSSIA = 1 << 3;
        const ASIA = 1 << 4;
        const CHINA = 1 << 5;
        const KOREA = 1 << 6;

        // Dynamic "WORLD" that matches all available regions and is safe.
        const WORLD = Self::JAPAN.bits() | Self::USA.bits() | Self::EUROPE.bits() | Self::RUSSIA.bits();
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "Unknown");
        }

        // Handle the composite constant WORLD for cleaner output
        if self.bits() == Region::WORLD.bits() {
            return write!(f, "World");
        }

        // Collect the string names using a match statement
        let regions: Vec<&str> = self
            .iter()
            .map(|flag| match flag {
                Region::JAPAN => "Japan",
                Region::USA => "USA",
                Region::EUROPE => "Europe",
                Region::RUSSIA => "Russia",
                Region::ASIA => "Asia",
                Region::CHINA => "China",
                Region::KOREA => "Korea",
                _ => "",
            })
            .filter(|s| !s.is_empty())
            .collect();

        // Join multiple regions with forward slash (e.g. "Japan/USA")
        write!(f, "{}", regions.join("/"))
    }
}

/// Infers the geographical region of a ROM from its filename.
///
/// This function examines the provided filename for common region indicators (e.g., "JP", "USA",
/// "EUR", "PAL", NTSC-J, NTSC-U, NTSC-E, (J), (U), (E), \[J\], \[U\], \[E\]) and returns a
/// standardized region string if a match is found. The search is case-insensitive.
///
/// # Arguments
///
/// * `name` - The filename of the ROM as a string slice.
///
/// # Returns
///
/// Returns a `Region` bitmask. If no region is found, returns `Region::UNKNOWN`.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::region::{infer_region_from_filename, Region};
///
/// assert_eq!(infer_region_from_filename("MyGame (J).zip"), Region::JAPAN);
/// assert_eq!(infer_region_from_filename("AnotherGame (USA).nes"), Region::USA);
/// assert_eq!(infer_region_from_filename("PAL_Game.sfc"), Region::EUROPE);
/// assert_eq!(infer_region_from_filename("UnknownGame.bin"), Region::UNKNOWN);
/// ```
pub fn infer_region_from_filename(name: &str) -> Region {
    let lower_name = name.to_lowercase();
    let mut region = Region::UNKNOWN;

    // Define region patterns with their corresponding flags
    let region_patterns = [
        (vec!["jap", "jp", "(j)", "[j]", "ntsc-j"], Region::JAPAN),
        (vec!["usa", "(u)", "[u]", "ntsc-u", "ntsc-us"], Region::USA),
        (vec!["eur", "(e)", "[e]", "pal", "ntsc-e"], Region::EUROPE),
        (vec!["russia", "dendy"], Region::RUSSIA),
        (vec!["(world)", "[world]", "(w)", "[w]"], Region::WORLD),
    ];

    // Check each pattern and set the corresponding region flag
    for (patterns, flag) in region_patterns {
        for pattern in patterns {
            if lower_name.contains(pattern) {
                region |= flag;
                break;
            }
        }
    }

    region
}

/// Normalizes a region string found in a ROM header to a standardized Region bitmask.
///
/// This function takes a region string (e.g., from a ROM header) and attempts
/// to map it to any of the standardized regions: "JAPAN", "USA", or "EUROPE".
/// It handles various common spellings and codes (e.g., NTSC-J, SLUS, PAL)
/// and performs a case-insensitive match.
///
/// Handles composite regions (like "NTSC" implying both USA and Japan).
///
/// # Arguments
///
/// * `header_text` - The region string extracted from a ROM header.
///
/// # Returns
///
/// Returns a `Region` bitmask. If no region is found, returns `Region::UNKNOWN`.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::region::{normalize_header_region, Region};
///
/// assert_eq!(normalize_header_region("NTSC-J"), Region::JAPAN);
/// assert_eq!(normalize_header_region("SLUS_000.00"), Region::USA);
/// assert_eq!(normalize_header_region("PAL"), Region::EUROPE);
/// assert_eq!(normalize_header_region("UNKNOWN"), Region::UNKNOWN);
/// ```
// TODO: Gameboy has a 'Non-Japan' region, we need that to not match Japan somehow.
// Perhaps we either detect that specifically and flip the Japan bit back over, or
// we use bounding regexes ('^Japan$') to match everything.
// TODO: we should also probably move some of this code into the consoles themselves.
// We could make them match as "SLUS" -> Region::USA, and add a 'header_region' to
// explicitly expose the region name from the header. This would also resolve the above
// TODO, since we can map 'Non-Japan' to Region::USA and Region::EUROPE.
pub fn normalize_header_region(header_text: &str) -> Region {
    let header_text = header_text.to_uppercase();
    let mut region = Region::UNKNOWN;

    // Define region patterns with their corresponding flags
    let region_patterns = [
        (
            vec!["JAPAN", "NTSC-J", "SLPS", "NTSC USA/Japan"],
            Region::JAPAN,
        ),
        (
            vec![
                "USA",
                "AMERICA",
                "NTSC-U",
                "CANADA",
                "SLUS",                   // PSX specific
                "NTSC USA/Japan",         // NES specific
                "SMS EXPORT",             // GameGear specific
                "GAMEGEAR EXPORT",        // GameGear specific
                "GAMEGEAR INTERNATIONAL", // GameGear specific
            ],
            Region::USA,
        ),
        (
            vec![
                "EUROPE",
                "PAL",
                "OCEANIA",
                "SLES",                   // PSX specific
                "SMS EXPORT",             // GameGear specific
                "GAMEGEAR EXPORT",        // GameGear specific
                "GAMEGEAR INTERNATIONAL", // GameGear specific
            ],
            Region::EUROPE,
        ),
        (vec!["RUSSIA", "DENDY"], Region::RUSSIA),
        (vec!["ASIA (NTSC)", "TAIWAN (NTSC)"], Region::ASIA), // SNES specific
        (vec!["CHINA (NTSC)"], Region::CHINA),                // SNES specific
        (vec!["KOREA (NTSC)"], Region::KOREA),                // SNES specific
        (vec!["MULTI-REGION"], Region::WORLD),
    ];

    // Check each pattern and set the corresponding region flag
    for (patterns, flag) in region_patterns {
        for pattern in patterns {
            if header_text.contains(pattern) {
                region |= flag;
                break;
            }
        }
    }

    region
}

/// Compare the inferred region (via filename) to the region in the ROM's header.
///
/// Returns `true` (mismatch) if:
/// 1. Both filename and header have known regions.
/// 2. They share NO common regions (intersection is empty).
///
/// If either is UNKNOWN, this returns `true` (mismatch).
// TODO: consider if UNKNOWN should be a mismatch? Perhaps it should be surfaced as its own
// separate error.
pub fn check_region_mismatch(source_name: &str, region_name: &str) -> bool {
    let inferred_region = infer_region_from_filename(source_name);
    let header_region = normalize_header_region(region_name);

    // If neither region can be found, avoid a mismatch by returning early.
    if inferred_region.is_empty() && header_region.is_empty() {
        return false;
    }

    // If either region is unknown, return a mismatch.
    if inferred_region.is_empty() || header_region.is_empty() {
        return true;
    }

    !inferred_region.intersects(header_region)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_region_from_filename_japan() {
        assert_eq!(infer_region_from_filename("game (J).zip"), Region::JAPAN);
        assert_eq!(infer_region_from_filename("game [J].zip"), Region::JAPAN);
        assert_eq!(
            infer_region_from_filename("game (Japan).zip"),
            Region::JAPAN
        );
        assert_eq!(
            infer_region_from_filename("game (NTSC-J).zip"),
            Region::JAPAN
        );
    }

    #[test]
    fn test_infer_region_from_filename_usa() {
        assert_eq!(infer_region_from_filename("game (U).zip"), Region::USA);
        assert_eq!(infer_region_from_filename("game [U].zip"), Region::USA);
        assert_eq!(infer_region_from_filename("game (USA).zip"), Region::USA);
        assert_eq!(infer_region_from_filename("game (NTSC-U).zip"), Region::USA);
        assert_eq!(
            infer_region_from_filename("game (NTSC-US).zip"),
            Region::USA
        );
    }

    #[test]
    fn test_infer_region_from_filename_europe() {
        assert_eq!(infer_region_from_filename("game (E).zip"), Region::EUROPE);
        assert_eq!(infer_region_from_filename("game [E].zip"), Region::EUROPE);
        assert_eq!(
            infer_region_from_filename("game (Europe).zip"),
            Region::EUROPE
        );
        assert_eq!(infer_region_from_filename("game (PAL).zip"), Region::EUROPE);
        assert_eq!(
            infer_region_from_filename("game (NTSC-E).zip"),
            Region::EUROPE
        );
    }

    #[test]
    fn test_infer_region_from_filename_none() {
        assert_eq!(
            infer_region_from_filename("game (unmarked).zip"),
            Region::UNKNOWN
        );
        assert_eq!(
            infer_region_from_filename("another game.zip"),
            Region::UNKNOWN
        );
    }

    #[test]
    fn test_normalize_header_region_japan() {
        assert_eq!(normalize_header_region("JAPAN"), Region::JAPAN);
        assert_eq!(normalize_header_region("NTSC-J"), Region::JAPAN);
        assert_eq!(normalize_header_region("SLPS-00001"), Region::JAPAN);
        assert_eq!(normalize_header_region("  japan  "), Region::JAPAN);
    }

    #[test]
    fn test_normalize_header_region_usa() {
        assert_eq!(normalize_header_region("USA"), Region::USA);
        assert_eq!(normalize_header_region("AMERICA"), Region::USA);
        assert_eq!(normalize_header_region("NTSC-U"), Region::USA);
        assert_eq!(normalize_header_region("SLUS-00001"), Region::USA);
        assert_eq!(normalize_header_region("CANADA"), Region::USA);
        assert_eq!(normalize_header_region("  usa  "), Region::USA);
    }

    #[test]
    fn test_normalize_header_region_europe() {
        assert_eq!(normalize_header_region("EUROPE"), Region::EUROPE);
        assert_eq!(normalize_header_region("PAL"), Region::EUROPE);
        assert_eq!(normalize_header_region("SLES-00001"), Region::EUROPE);
        assert_eq!(normalize_header_region("OCEANIA"), Region::EUROPE);
        assert_eq!(normalize_header_region("  europe  "), Region::EUROPE);
    }

    #[test]
    fn test_normalize_header_region_none() {
        assert_eq!(normalize_header_region("UNKNOWN"), Region::UNKNOWN);
        assert_eq!(normalize_header_region("  random text  "), Region::UNKNOWN);
    }

    #[test]
    fn test_check_region_mismatch_no_mismatch_japan() {
        // Filename indicates Japan, header is also Japan
        assert_eq!(check_region_mismatch("game (J).zip", "JAPAN"), false);
        assert_eq!(check_region_mismatch("game (Japan).zip", "NTSC-J"), false);
        assert_eq!(check_region_mismatch("game (J).zip", "SLPS-00001"), false);
    }

    #[test]
    fn test_check_region_mismatch_no_mismatch_usa() {
        // Filename indicates USA, header is also USA
        assert_eq!(check_region_mismatch("game (U).zip", "USA"), false);
        assert_eq!(check_region_mismatch("game (USA).zip", "AMERICA"), false);
        assert_eq!(check_region_mismatch("game (U).zip", "SLUS-00001"), false);
    }

    #[test]
    fn test_check_region_mismatch_no_mismatch_europe() {
        // Filename indicates Europe, header is also Europe
        assert_eq!(check_region_mismatch("game (E).zip", "EUROPE"), false);
        assert_eq!(check_region_mismatch("game (Europe).zip", "PAL"), false);
        assert_eq!(check_region_mismatch("game (E).zip", "SLES-00001"), false);
    }

    #[test]
    fn test_check_region_mismatch_mismatch_japan_usa() {
        // Filename indicates Japan, header indicates USA
        assert_eq!(check_region_mismatch("game (J).zip", "USA"), true);
        assert_eq!(check_region_mismatch("game (Japan).zip", "AMERICA"), true);
    }

    #[test]
    fn test_check_region_mismatch_mismatch_usa_europe() {
        // Filename indicates USA, header indicates Europe
        assert_eq!(check_region_mismatch("game (U).zip", "EUROPE"), true);
        assert_eq!(check_region_mismatch("game (USA).zip", "PAL"), true);
    }

    #[test]
    fn test_check_region_mismatch_mismatch_europe_japan() {
        // Filename indicates Europe, header indicates Japan
        assert_eq!(check_region_mismatch("game (E).zip", "JAPAN"), true);
        assert_eq!(check_region_mismatch("game (Europe).zip", "NTSC-J"), true);
    }

    #[test]
    fn test_check_region_mismatch_filename_has_region_header_unknown() {
        // Filename indicates a region, but header is unknown/unnormalized
        assert_eq!(
            check_region_mismatch("game (J).zip", "Some random text"),
            true
        );
        assert_eq!(
            check_region_mismatch("game (U).zip", "REGION_UNKNOWN"),
            true
        );
        assert_eq!(check_region_mismatch("game (E).zip", "NO_MATCH"), true);
    }

    #[test]
    fn test_check_region_mismatch_filename_unknown_header_has_region() {
        // Filename is generic, header indicates a region
        assert_eq!(check_region_mismatch("game.zip", "JAPAN"), true);
        assert_eq!(check_region_mismatch("another game.zip", "USA"), true);
        assert_eq!(check_region_mismatch("game_title", "EUROPE"), true);
    }

    #[test]
    fn test_check_region_mismatch_both_unknown() {
        // Neither filename nor header can be normalized to a region
        assert_eq!(check_region_mismatch("game.zip", "Some random text"), false);
        assert_eq!(check_region_mismatch("another game.zip", "UNKNOWN"), false);
        assert_eq!(check_region_mismatch("game_title", "NO_MATCH"), false);
    }

    #[test]
    fn test_check_region_mismatch_case_insensitivity_filename() {
        // Test case insensitivity for filename inference
        assert_eq!(check_region_mismatch("game (JapAn).zip", "JAPAN"), false);
        assert_eq!(check_region_mismatch("game (uSa).zip", "USA"), false);
        assert_eq!(check_region_mismatch("game (EuRoPe).zip", "EUROPE"), false);
    }

    #[test]
    fn test_overlap_logic() {
        // NES Example: Header says "NTSC", Filename says "(U)"
        let filename_region = infer_region_from_filename("Contra (U).nes"); // USA
        let header_region = normalize_header_region("NTSC (USA/Japan)"); // USA | JAPAN

        // They should intersect (match), so mismatch is false
        assert!(filename_region.intersects(header_region));
        assert_eq!(
            check_region_mismatch("Contra (U).nes", "NTSC (USA/Japan)"),
            false
        );
    }

    #[test]
    fn test_strict_mismatch() {
        // Filename says (E), Header says NTSC (USA|Japan)
        let filename_region = infer_region_from_filename("Contra (E).nes"); // EUROPE
        let header_region = normalize_header_region("NTSC"); // USA | JAPAN

        // No intersection, so mismatch is true
        assert!(!filename_region.intersects(header_region));
        assert_eq!(check_region_mismatch("Contra (E).nes", "NTSC"), true);
    }

    #[test]
    fn test_world_rom() {
        // Filename says (W), Header says USA
        // (W) implies USA | JAPAN | EUROPE
        assert_eq!(check_region_mismatch("Game (W).bin", "USA"), false);
    }

    #[test]
    fn test_multiple_region_filename_display() {
        let filename = "Super Game (U) (J).nes";
        let region = infer_region_from_filename(filename).to_string();
        assert_eq!(region, "Japan/USA")
    }
}
