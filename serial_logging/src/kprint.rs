/// Minimal serial debug output for kernel/boot environments.
///
/// The `kprint!` macro and `DebugSerial` struct provide direct, dependency-free serial port output.
/// This is ideal for very early boot stages or when only core formatting is available.
///
/// # When to Use
/// - Use `kprint!` when you cannot rely on external crates or need output before the main serial driver is initialized.
/// - For most kernel logging, prefer the higher-level `serial_print!`/`serial_log!` macros.
///
/// # QEMU Usage
/// QEMU will display all output sent to the serial port (0x3F8) if run with `-serial stdio`.
#[macro_export]
macro_rules! kprint {
    ($($args:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::DebugSerial{}, $($args)*);
    });
}

/// A minimal serial port writer for direct output.
///
/// Implements `core::fmt::Write` for use with formatting macros.
pub struct DebugSerial;

impl core::fmt::Write for DebugSerial {
    /// Writes a string to the serial port, byte by byte.
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            Self::put_byte(b);
        }
        Ok(())
    }
}

impl DebugSerial {
    /// Reads a byte from the serial port if available.
    ///
    /// Returns `Some(u8)` if a byte is ready, or `None` otherwise.
    ///
    /// # Safety
    /// Uses inline assembly to access the port directly.
    pub fn get_byte() -> Option<u8> {
        #[allow(unused_assignments)]
        let mut byte = 0;
        unsafe {
            core::arch::asm!("in al, dx", out("al") byte, in("dx") 0x3f8 + 5);
            if byte & 0x01 != 0 {
                core::arch::asm!("in al, dx", out("al") byte, in("dx") 0x3f8);
                Some(byte)
            } else {
                None
            }
        }
    }
    /// Writes a byte directly to the serial port (0x3F8).
    ///
    /// # Safety
    /// Uses inline assembly for direct port output.
    pub fn put_byte(b: u8) {
        unsafe {
            core::arch::asm!("out dx, al", in("al") b, in("dx") 0x3f8 );
        }
    }
}
