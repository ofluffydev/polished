//! # Framebuffer Management
//!
//! This module provides structures and functions for initializing and describing a framebuffer.
//!
//! ## What is a Framebuffer?
//! A framebuffer is a region of memory that represents the pixels on a display. Each pixel is stored as a value (e.g., 32 bits for RGBA), and the display hardware reads this memory to show the image on the screen. By writing to the framebuffer, software can directly control what appears on the display.
//!
//! ## How Framebuffers Work
//! - The framebuffer is a contiguous block of memory mapped to the video hardware.
//! - Each pixel's color is represented by a value at a specific offset in this memory.
//! - The layout (stride, format) depends on the hardware and mode.
//! - To draw, software writes color values to the appropriate memory locations.
//!
//! This module provides a `FramebufferInfo` struct describing the framebuffer's location, size, and format, and a UEFI-specific function to initialize it.

#[cfg(feature = "uefi")]
use log::info;
#[cfg(feature = "uefi")]
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    proto::console::gop::{self, GraphicsOutput},
};

/// Information about the framebuffer's memory and display properties.
#[cfg_attr(not(feature = "uefi"), allow(dead_code))]
#[repr(C)]
#[derive(Debug)]
pub struct FramebufferInfo {
    /// Physical address of the framebuffer in memory.
    pub address: u64,
    /// Total size of the framebuffer in bytes.
    pub size: usize,
    /// Width of the framebuffer in pixels.
    pub width: usize,
    /// Height of the framebuffer in pixels.
    pub height: usize,
    /// Number of pixels per row (may be >= width due to padding).
    pub stride: usize,
    /// Pixel format used by the framebuffer.
    pub format: FramebufferFormat,
}

/// Supported framebuffer pixel formats.
#[repr(C)]
#[derive(Debug)]
pub enum FramebufferFormat {
    /// Red-Green-Blue pixel order.
    Rgb,
    /// Blue-Green-Red pixel order.
    Bgr,
    /// Bitmask pixel format (custom masks).
    Bitmask,
    /// Framebuffer only supports block transfers (no direct pixel access).
    BltOnly,
}

/// Initialize the framebuffer using UEFI's Graphics Output Protocol (GOP).
///
/// # Returns
/// A `FramebufferInfo` struct describing the framebuffer's memory and display properties.
///
/// # Panics
/// This function will panic if the GOP protocol cannot be accessed (should only be used in UEFI environments).
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
