//! Provides utilities for inferring and normalizing geographical regions
//! from ROM filenames and header information.
//!
//! This module helps in identifying the target region (e.g., Japan, USA, Europe)
//! of a ROM, which is crucial for accurate analysis and categorization.

/// Infers the geographical region of a ROM from its filename.
///
/// This function examines the provided filename for common region indicators
/// (e.g., "JP", "USA", "EUR", "PAL", NTSC-J, NTSC-U, NTSC-E, (J), (U), (E), \[J\], \[U\], \[E\])
/// and returns a standardized region string if a match is found.
/// The search is case-insensitive.
///
/// # Arguments
///
/// * `name` - The filename of the ROM as a string slice.
///
/// # Returns
///
/// An `Option<&'static str>` which is:
/// - `Some("JAPAN")` if the filename indicates a Japanese region.
/// - `Some("USA")` if the filename indicates a USA region.
/// - `Some("EUROPE")` if the filename indicates a European region.
/// - `None` if no region could be inferred from the filename.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::region::infer_region_from_filename;
///
/// assert_eq!(infer_region_from_filename("MyGame (J).zip"), Some("JAPAN"));
/// assert_eq!(infer_region_from_filename("AnotherGame (USA).nes"), Some("USA"));
/// assert_eq!(infer_region_from_filename("PAL_Game.sfc"), Some("EUROPE"));
/// assert_eq!(infer_region_from_filename("UnknownGame.bin"), None);
/// ```
pub fn infer_region_from_filename(name: &str) -> Option<&'static str> {
    let lower_name = name.to_lowercase();

    if lower_name.contains("jap")
        || lower_name.contains("jp")
        || lower_name.contains("(j)")
        || lower_name.contains("[j]")
        || lower_name.contains("ntsc-j")
    {
        Some("JAPAN")
    } else if lower_name.contains("usa")
        || lower_name.contains("(u)")
        || lower_name.contains("[u]")
        || lower_name.contains("ntsc-u")
        || lower_name.contains("ntsc-us")
    {
        Some("USA")
    } else if lower_name.contains("eur")
        || lower_name.contains("(e)")
        || lower_name.contains("[e]")
        || lower_name.contains("pal")
        || lower_name.contains("ntsc-e")
    {
        Some("EUROPE")
    } else {
        None
    }
}

/// Normalizes a region string found in a ROM header to a standardized format.
///
/// This function takes a region string (e.g., from a ROM header) and attempts
/// to map it to one of the standardized regions: "JAPAN", "USA", or "EUROPE".
/// It handles various common spellings and codes (e.g., NTSC-J, SLUS, PAL)
/// and performs a case-insensitive match.
///
/// # Arguments
///
/// * `header_text` - The region string extracted from a ROM header.
///
/// # Returns
///
/// An `Option<&'static str>` which is:
/// - `Some("JAPAN")` if the header text indicates a Japanese region.
/// - `Some("USA")` if the header text indicates a USA region.
/// - `Some("EUROPE")` if the header text indicates a European region.
/// - `None` if the region could not be normalized.
///
/// # Examples
///
/// ```rust
/// use rom_analyzer::region::normalize_header_region;
///
/// assert_eq!(normalize_header_region("NTSC-J"), Some("JAPAN"));
/// assert_eq!(normalize_header_region("SLUS_000.00"), Some("USA"));
/// assert_eq!(normalize_header_region("PAL"), Some("EUROPE"));
/// assert_eq!(normalize_header_region("UNKNOWN"), None);
/// ```
pub fn normalize_header_region(header_text: &str) -> Option<&'static str> {
    let header_text = header_text.to_uppercase();

    if header_text.contains("JAPAN")
        || header_text.contains("NTSC-J")
        || header_text.contains("SLPS")
    {
        Some("JAPAN")
    } else if header_text.contains("USA")
        || header_text.contains("AMERICA")
        || header_text.contains("NTSC-U")
        || header_text.contains("SLUS")
        || header_text.contains("CANADA")
    {
        Some("USA")
    } else if header_text.contains("EUROPE")
        || header_text.contains("PAL")
        || header_text.contains("SLES")
        || header_text.contains("OCEANIA")
    {
        Some("EUROPE")
    } else {
        None
    }
}

/// Compare the inferred region (via filename) to the region in the ROM's header.
pub fn check_region_mismatch(source_name: &str, region_name: &str) -> bool {
    let inferred_region = infer_region_from_filename(source_name);
    let header_region_norm = normalize_header_region(region_name);

    inferred_region != header_region_norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_region_from_filename_japan() {
        assert_eq!(infer_region_from_filename("game (J).zip"), Some("JAPAN"));
        assert_eq!(infer_region_from_filename("game [J].zip"), Some("JAPAN"));
        assert_eq!(
            infer_region_from_filename("game (Japan).zip"),
            Some("JAPAN")
        );
        assert_eq!(
            infer_region_from_filename("game (NTSC-J).zip"),
            Some("JAPAN")
        );
    }

    #[test]
    fn test_infer_region_from_filename_usa() {
        assert_eq!(infer_region_from_filename("game (U).zip"), Some("USA"));
        assert_eq!(infer_region_from_filename("game [U].zip"), Some("USA"));
        assert_eq!(infer_region_from_filename("game (USA).zip"), Some("USA"));
        assert_eq!(infer_region_from_filename("game (NTSC-U).zip"), Some("USA"));
        assert_eq!(
            infer_region_from_filename("game (NTSC-US).zip"),
            Some("USA")
        );
    }

    #[test]
    fn test_infer_region_from_filename_europe() {
        assert_eq!(infer_region_from_filename("game (E).zip"), Some("EUROPE"));
        assert_eq!(infer_region_from_filename("game [E].zip"), Some("EUROPE"));
        assert_eq!(
            infer_region_from_filename("game (Europe).zip"),
            Some("EUROPE")
        );
        assert_eq!(infer_region_from_filename("game (PAL).zip"), Some("EUROPE"));
        assert_eq!(
            infer_region_from_filename("game (NTSC-E).zip"),
            Some("EUROPE")
        );
    }

    #[test]
    fn test_infer_region_from_filename_none() {
        assert_eq!(infer_region_from_filename("game (unmarked).zip"), None);
        assert_eq!(infer_region_from_filename("another game.zip"), None);
    }

    #[test]
    fn test_normalize_header_region_japan() {
        assert_eq!(normalize_header_region("JAPAN"), Some("JAPAN"));
        assert_eq!(normalize_header_region("NTSC-J"), Some("JAPAN"));
        assert_eq!(normalize_header_region("SLPS-00001"), Some("JAPAN"));
        assert_eq!(normalize_header_region("  japan  "), Some("JAPAN"));
    }

    #[test]
    fn test_normalize_header_region_usa() {
        assert_eq!(normalize_header_region("USA"), Some("USA"));
        assert_eq!(normalize_header_region("AMERICA"), Some("USA"));
        assert_eq!(normalize_header_region("NTSC-U"), Some("USA"));
        assert_eq!(normalize_header_region("SLUS-00001"), Some("USA"));
        assert_eq!(normalize_header_region("CANADA"), Some("USA"));
        assert_eq!(normalize_header_region("  usa  "), Some("USA"));
    }

    #[test]
    fn test_normalize_header_region_europe() {
        assert_eq!(normalize_header_region("EUROPE"), Some("EUROPE"));
        assert_eq!(normalize_header_region("PAL"), Some("EUROPE"));
        assert_eq!(normalize_header_region("SLES-00001"), Some("EUROPE"));
        assert_eq!(normalize_header_region("OCEANIA"), Some("EUROPE"));
        assert_eq!(normalize_header_region("  europe  "), Some("EUROPE"));
    }

    #[test]
    fn test_normalize_header_region_none() {
        assert_eq!(normalize_header_region("UNKNOWN"), None);
        assert_eq!(normalize_header_region("  random text  "), None);
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
}
