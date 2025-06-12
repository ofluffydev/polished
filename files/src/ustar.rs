// Helper function to convert ASCII octal to binary
fn oct2bin(oct: &[u8]) -> usize {
    let mut n = 0;
    for &c in oct {
        if c == 0 || c == b' ' {
            break;
        }
        n *= 8;
        n += (c - b'0') as usize;
    }
    n
}

/// Looks up a file in a ustar archive.
/// Returns Some((file_data, file_size)) if found, else None.
pub fn tar_lookup<'a>(archive: &'a [u8], filename: &str) -> Option<(&'a [u8], usize)> {
    let mut ptr = 0;
    while ptr + 512 <= archive.len() && &archive[ptr + 257..ptr + 262] == b"ustar" {
        let filesize = oct2bin(&archive[ptr + 0x7c..ptr + 0x7c + 11]);
        let name_end = archive[ptr..].iter().position(|&b| b == 0).unwrap_or(100);
        let name = core::str::from_utf8(&archive[ptr..ptr + name_end]).unwrap_or("");
        if name == filename {
            let data_start = ptr + 512;
            let data_end = data_start + filesize;
            if data_end <= archive.len() {
                return Some((&archive[data_start..data_end], filesize));
            } else {
                return None;
            }
        }
        let blocks = filesize.div_ceil(512) + 1;
        ptr += blocks * 512;
    }
    None
}
