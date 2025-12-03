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
#[macro_export]
macro_rules! check_region_mismatch {
    ($source_name:expr, $region_name:expr) => {
        let inferred_region = crate::region::infer_region_from_filename($source_name);
        let header_region_norm = crate::region::normalize_header_region($region_name);

        if let (Some(inferred), Some(header)) = (inferred_region, header_region_norm) {
            if inferred != header {
                println!("\n*** WARNING: POSSIBLE REGION MISMATCH! ***");
                println!(
                    "Source File:  {}",
                    ::std::path::Path::new($source_name)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
                println!("Filename suggests: {}", inferred);
                println!(
                    "ROM Header claims: {} (Header detail: '{}')",
                    header, $region_name
                );
                println!("The ROM may be mislabeled or have been patched.");
                println!("*******************************************");
            }
        }
    };
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
}
