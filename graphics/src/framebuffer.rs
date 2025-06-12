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

/// Information about the framebuffer's memory and display properties.
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
