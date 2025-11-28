

pub fn infer_region_from_filename(name: &str) -> Option<&'static str> {
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

pub fn normalize_header_region(header_text: &str) -> Option<&'static str> {
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
#[macro_export]
macro_rules! check_region_mismatch {
    ($source_name:expr, $region_name:expr) => {
        let inferred_region = crate::region::infer_region_from_filename($source_name);
        let header_region_norm = crate::region::normalize_header_region($region_name);

        if let (Some(inferred), Some(header)) = (inferred_region, header_region_norm) {
            if inferred != header {
                println!("\n*** WARNING: POSSIBLE REGION MISMATCH! ***");
                println!("Source File:  {}", ::std::path::Path::new($source_name).file_name().unwrap_or_default().to_string_lossy());
                println!("Filename suggests: {}", inferred);
                println!("ROM Header claims: {} (Header detail: '{}')", header, $region_name);
                println!("The ROM may be mislabeled or have been patched.");
                println!("*******************************************");
            }
        }
    };
}
