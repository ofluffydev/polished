// Example demo code for using the ustar archive in the kernel
use polished_files::ustar::tar_lookup;
use polished_serial_logging::{info, warn};

extern crate alloc;
use alloc::format;
use core::option::Option::None;
use core::option::Option::Some;

/// Demonstrates looking up and printing files from a ustar archive.
pub fn demo_ustar_archive(ustar_archive: &'static [u8]) {
    // Example usage of the ustar module
    let file = match tar_lookup(ustar_archive, "ustar-files/mymessage.txt") {
        Some(file) => file,
        None => {
            warn("File not found in archive: mytext.txt");
            (b"" as &[u8], 0)
        }
    };
    info(&format!("Found file: mytext.txt, size: {}", file.1));
    let file = match tar_lookup(ustar_archive, "ustar-files/hello_world.rs") {
        Some(file) => file,
        None => {
            warn("File not found in archive: hello_world.rs");
            (b"" as &[u8], 0)
        }
    };
    info(&format!("Found file: hello_world.rs, size: {}", file.1));

    // Print the contents of the file
    let file_contents = core::str::from_utf8(file.0).expect("Invalid UTF-8 in file");
    info(&format!("File contents:\n{file_contents}"));
}
