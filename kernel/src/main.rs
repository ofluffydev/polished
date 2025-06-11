#![no_std]
#![no_main]

extern crate alloc;

use polished_interrupts::init_idt;
use polished_memory as _;
use polished_panic_handler as _; // Import the panic handler // Import the memory module for memset, memcpy, etc.

use alloc::format;
use core::arch::{asm, naked_asm};
use linked_list_allocator::LockedHeap;
use polished_graphics::drawing::framebuffer_x_demo;
use polished_graphics::framebuffer::FramebufferInfo;
use polished_ps2::ps2_init;
use polished_serial_logging::{info, warn};
use polished_syscalls::syscall_handler;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn naked_start() {
    // Set up the stack pointer to the top of the stack
    naked_asm!(
        "cli",
        "lea rsp, STACK_TOP",
        "call kernel_entry",
        "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}

fn init_allocator() {
    let heap_start = 0x1000_0000; // Example heap start address
    let heap_size = 0x0100_0000; // Example heap size (16 MB)

    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

fn log_framebuffer_info(fb_info_ptr: *const FramebufferInfo) {
    if !fb_info_ptr.is_null() {
        let fb = unsafe { &*fb_info_ptr };
        let msg = format!(
            "FramebufferInfo: address=0x{:x}, size={}, {}x{}, stride={}, format={:?}",
            fb.address, fb.size, fb.width, fb.height, fb.stride, fb.format
        );
        info(&msg);
    } else {
        info("FramebufferInfo pointer is null");
    }
}

fn clear_framebuffer(fb_info_ptr: *const FramebufferInfo) {
    if !fb_info_ptr.is_null() {
        let fb = unsafe { &mut *(fb_info_ptr as *mut FramebufferInfo) };
        let buffer = unsafe { core::slice::from_raw_parts_mut(fb.address as *mut u8, fb.size) };
        for byte in buffer.iter_mut() {
            *byte = 0; // Fill with black
        }
        info("Framebuffer buffer filled with black");
        framebuffer_x_demo(fb);
    } else {
        warn("FramebufferInfo pointer is null, cannot fill buffer");
    }
}

fn init_interrupts() {
    info("Loading IDT...");
    init_idt();
    info("IDT loaded");
}

#[cfg(feature = "ext2")]
mod ext2_demo {
    use super::*;
    use polished_files::ext2::{BlockDevice, Ext2, Ext2Error};

    // Virtio block device for QEMU (MMIO, very minimal, assumes drive is at 0x100000)
    pub struct VirtioBlockDevice;
    impl BlockDevice for VirtioBlockDevice {
        fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), Ext2Error> {
            // For demo: map ext2.img at 0x100000 in guest memory (QEMU -drive if=virtio)
            // This is a hack: in real OS, use PCI/virtio driver. Here, just read from memory.
            let disk_base = 0x100000 as *const u8;
            let offset = lba as usize * 512;
            unsafe {
                let src = disk_base.add(offset);
                let dst = buf.as_mut_ptr();
                core::ptr::copy_nonoverlapping(src, dst, buf.len());
            }
            Ok(())
        }
    }

    // EXTREMELY MINIMAL: Only works for a file at a fixed inode (12) and block (for demo)
    pub fn try_mount_ext2() {
        let device = VirtioBlockDevice;
        match Ext2::new(&device) {
            Ok(fs) => {
                info("Mounted ext2 filesystem (virtio hack)");
                let mut buf = [0u8; 1024];
                match fs.read_file_first_block("myfile.txt", &mut buf) {
                    Ok(()) => {
                        let s = core::str::from_utf8(&buf).unwrap_or("<not utf8>");
                        info(&format!("Read myfile.txt block: {}", s));
                    }
                    Err(e) => info(&format!("Failed to read myfile.txt block: {:?}", e)),
                }
            }
            Err(e) => info(&format!("Failed to mount ext2: {:?}", e)),
        }
    }

    pub fn log_ext2_image_header() {
        use core::fmt::Write;
        use polished_serial_logging::info;
        struct HexBuf<'a>(&'a mut [u8], usize);
        impl<'a> Write for HexBuf<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let end = self.1 + bytes.len();
                if end > self.0.len() {
                    return Err(core::fmt::Error);
                }
                self.0[self.1..end].copy_from_slice(bytes);
                self.1 = end;
                Ok(())
            }
        }
        let ptr = 0x100000 as *const u8;
        let mut buf = [0u8; 128];
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, buf.as_mut_ptr(), 128);
        }
        for i in (0..128).step_by(16) {
            let chunk = &buf[i..i + 16];
            let mut hex = [0u8; 48];
            let hex_len = {
                let mut hexbuf = HexBuf(&mut hex, 0);
                for b in chunk.iter() {
                    let _ = write!(&mut hexbuf, "{:02x} ", b);
                }
                hexbuf.1
            };
            let hexstr = core::str::from_utf8(&hex[..hex_len]).unwrap_or("<hex error>");
            let mut msg = [0u8; 80];
            let msg_len = {
                let mut msgbuf = HexBuf(&mut msg, 0);
                let _ = write!(
                    &mut msgbuf,
                    "ext2 image @0x100000 [{:03}-{:03}]: {}",
                    i,
                    i + 15,
                    hexstr
                );
                msgbuf.1
            };
            let msgstr = core::str::from_utf8(&msg[..msg_len]).unwrap_or("<hex error>");
            info(msgstr);
        }
    }
}

/// # Safety
/// This function must be called only as the kernel entry point, and the provided
/// `fb_info_ptr` must be a valid pointer to a `FramebufferInfo` structure, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_entry(fb_info_ptr: *const FramebufferInfo) -> ! {
    init_allocator();
    info("Hello from the kernel!");
    info("Initializing GDT...");
    polished_gdt::init_gdt();
    info("GDT initialized");
    init_interrupts();
    ps2_init();
    log_framebuffer_info(fb_info_ptr);
    clear_framebuffer(fb_info_ptr);
    x86_64::instructions::interrupts::enable();
    #[cfg(feature = "ext2")]
    {
        crate::ext2_demo::log_ext2_image_header();
        crate::ext2_demo::try_mount_ext2();
    }
    // Only disable the PIC after confirming interrupts work, or comment out for now
    // info("Disabling legacy PIC...");
    // disable_pic();
    // info("Legacy PIC disabled");
    // simulate_divide_by_zero();

    // Loop forever to keep the kernel running
    info("Kernel initialized successfully, entering main loop...");
    unsafe {
        asm!("sti");
    }
    loop {
        unsafe { asm!("hlt") }; // Halt the CPU until the next interrupt
    }

    // panic!("Kernel halted");
}

// Example: Kernel-side syscall entry point for x86_64 (syscall instruction)
#[unsafe(no_mangle)]
pub extern "C" fn syscall_entry(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    // Safety: Only called from syscall context
    unsafe { syscall_handler(num, arg1, arg2, arg3, arg4, arg5, arg6) }
}
