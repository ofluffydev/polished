#[cfg(feature = "uefi")]
use log::info;
#[cfg(feature = "uefi")]
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    proto::console::gop::{self, GraphicsOutput},
};

#[cfg_attr(not(feature = "uefi"), allow(dead_code))]
#[repr(C)]
#[derive(Debug)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: FramebufferFormat,
}

#[repr(C)]
#[derive(Debug)]
pub enum FramebufferFormat {
    Rgb,
    Bgr,
    Bitmask,
    BltOnly,
}

#[cfg(feature = "uefi")]
pub fn initialize_framebuffer() -> FramebufferInfo {
    let gop_handle = get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop_protocol = open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    let gop = gop_protocol.get_mut().unwrap();
    let mode_info = gop.current_mode_info();
    let resolution = mode_info.resolution();
    let stride = mode_info.stride();
    let pixel_format = mode_info.pixel_format();

    let mut gop_buffer = gop.frame_buffer();
    let gop_buffer_first_byte = gop_buffer.as_mut_ptr() as usize;

    info!("Framebuffer address: 0x{gop_buffer_first_byte:x}");
    info!("Framebuffer size: {} bytes", gop_buffer.size());

    FramebufferInfo {
        address: gop_buffer.as_mut_ptr() as u64,
        size: gop_buffer.size(),
        width: resolution.0,
        height: resolution.1,
        stride,
        format: match pixel_format {
            gop::PixelFormat::Rgb => FramebufferFormat::Rgb,
            gop::PixelFormat::Bgr => FramebufferFormat::Bgr,
            gop::PixelFormat::Bitmask => FramebufferFormat::Bitmask,
            gop::PixelFormat::BltOnly => FramebufferFormat::BltOnly,
        },
    }
}
