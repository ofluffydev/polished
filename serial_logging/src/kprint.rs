/// Minimal serial debug output for kernel/boot environments.
/// Provides `kprint!` macro and `DebugSerial` struct for direct serial port output.
#[macro_export]
macro_rules! kprint {
    ($($args:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::DebugSerial{}, $($args)*);
    });
}

pub struct DebugSerial;

impl core::fmt::Write for DebugSerial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            Self::put_byte(b);
        }
        Ok(())
    }
}

impl DebugSerial {
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
    pub fn put_byte(b: u8) {
        unsafe {
            core::arch::asm!("out dx, al", in("al") b, in("dx") 0x3f8 );
        }
    }
}
