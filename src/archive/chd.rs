use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use chd::Chd;

use crate::console::psx;
use crate::console::segacd;

// We only need the first few KB for header analysis for PSX and SegaCD.
const MAX_HEADER_SIZE: usize = 0x20000; // 128KB

pub fn analyze_chd_file(filepath: &Path, source_name: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=======================================================");
    println!("  CHD ANALYSIS: {}", source_name);
    println!("=======================================================");

    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut chd = Chd::open(&mut reader, None)?;

    let hunk_count = chd.header().hunk_count();
    let hunk_size = chd.header().hunk_size();

    let mut decompressed_data = Vec::new();
    decompressed_data.reserve_exact(
        ((hunk_count as u64) * (hunk_size as u64)).min(MAX_HEADER_SIZE as u64) as usize,
    );

    let mut out_buf = chd.get_hunksized_buffer();
    let mut temp_buf = Vec::new();

    for hunk_num in 0..hunk_count {
        if decompressed_data.len() >= MAX_HEADER_SIZE {
            break;
        }

        let mut hunk = chd.hunk(hunk_num)?;
        hunk.read_hunk_in(&mut temp_buf, &mut out_buf)?;

        let remaining_capacity = MAX_HEADER_SIZE - decompressed_data.len();
        let data_to_add = out_buf.len().min(remaining_capacity);
        decompressed_data.extend_from_slice(&out_buf[..data_to_add]);
    }

    println!(
        "Decompressed first {} bytes for header analysis.",
        decompressed_data.len()
    );

    // --- Console Dispatch Logic ---
    // Check for "SEGA CD" signature at offset 0x100 for Sega CD
    const SEGA_CD_SIGNATURE_OFFSET: usize = 0x100;
    const SEGA_CD_SIGNATURE: &[u8] = b"SEGA CD";
    if decompressed_data.len() >= SEGA_CD_SIGNATURE_OFFSET + SEGA_CD_SIGNATURE.len()
        && decompressed_data
            [SEGA_CD_SIGNATURE_OFFSET..SEGA_CD_SIGNATURE_OFFSET + SEGA_CD_SIGNATURE.len()]
            == *SEGA_CD_SIGNATURE
    {
        println!("Detected Sega CD signature. Analyzing as Sega CD.");
        segacd::analyze_segacd_data(&decompressed_data, source_name)
    } else {
        println!("No Sega CD signature. Analyzing as PSX.");
        psx::analyze_psx_data(&decompressed_data, source_name)
    }
}
