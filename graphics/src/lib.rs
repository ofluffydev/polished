//! # Graphics Library
//!
//! This module provides basic framebuffer management and drawing routines for low-level graphics output, such as in an OS kernel or bootloader. It is designed to be used in environments without a standard library (`no_std`).
//!
//! ## Modules
//! - `framebuffer`: Framebuffer initialization and information structures.
//! - `drawing`: Basic drawing routines (e.g., lines) using the framebuffer.

#![no_std]

/// Drawing routines for the framebuffer, such as lines and demo patterns.
pub mod drawing;
/// Framebuffer initialization and information structures.
pub mod framebuffer;

#[cfg(feature = "uefi")]
pub use framebuffer::initialize_framebuffer;
