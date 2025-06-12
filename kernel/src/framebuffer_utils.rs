use alloc::format;
use polished_graphics::drawing::framebuffer_x_demo;
use polished_graphics::framebuffer::FramebufferInfo;
use polished_serial_logging::info;

pub fn log_framebuffer_info(fb: &FramebufferInfo) {
    let msg = format!(
        "FramebufferInfo: address=0x{:x}, size={}, {}x{}, stride={}, format=RAW",
        fb.address, fb.size, fb.width, fb.height, fb.stride
    );
    info(&msg);
}

pub fn clear_framebuffer(fb: &FramebufferInfo) {
    let buffer = unsafe { core::slice::from_raw_parts_mut(fb.address as *mut u8, fb.size) };
    for byte in buffer.iter_mut() {
        *byte = 0; // Fill with black
    }
    info("Framebuffer buffer filled with black");
    let mut fb_mut = FramebufferInfo {
        address: fb.address,
        size: fb.size,
        width: fb.width,
        height: fb.height,
        stride: fb.stride,
        format: match fb.format {
            polished_graphics::framebuffer::FramebufferFormat::Rgb => {
                polished_graphics::framebuffer::FramebufferFormat::Rgb
            }
            polished_graphics::framebuffer::FramebufferFormat::Bgr => {
                polished_graphics::framebuffer::FramebufferFormat::Bgr
            }
            polished_graphics::framebuffer::FramebufferFormat::Bitmask => {
                polished_graphics::framebuffer::FramebufferFormat::Bitmask
            }
            polished_graphics::framebuffer::FramebufferFormat::BltOnly => {
                polished_graphics::framebuffer::FramebufferFormat::BltOnly
            }
        },
    };
    framebuffer_x_demo(&mut fb_mut);
}

pub fn make_framebuffer_info(
    boot_info: &polished_bootloader::BootInfo,
) -> polished_graphics::framebuffer::FramebufferInfo {
    if boot_info.framebuffer_bpp == 0 {
        polished_serial_logging::warn(
            "BootInfo.framebuffer_bpp is zero! Defaulting stride to 0 to avoid division by zero.",
        );
        polished_graphics::framebuffer::FramebufferInfo {
            address: boot_info.framebuffer_addr,
            size: (boot_info.framebuffer_pitch as usize) * (boot_info.framebuffer_height as usize),
            width: boot_info.framebuffer_width as usize,
            height: boot_info.framebuffer_height as usize,
            stride: 0,
            format: polished_graphics::framebuffer::FramebufferFormat::Rgb, // TODO: Use correct format if needed
        }
    } else {
        polished_graphics::framebuffer::FramebufferInfo {
            address: boot_info.framebuffer_addr,
            size: (boot_info.framebuffer_pitch as usize) * (boot_info.framebuffer_height as usize),
            width: boot_info.framebuffer_width as usize,
            height: boot_info.framebuffer_height as usize,
            stride: boot_info.framebuffer_pitch as usize / (boot_info.framebuffer_bpp as usize / 8),
            format: polished_graphics::framebuffer::FramebufferFormat::Rgb, // TODO: Use correct format if needed
        }
    }
}
