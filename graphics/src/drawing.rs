//! # Drawing Routines
//!
//! This module provides basic drawing functions for the framebuffer, such as drawing lines using Bresenham's algorithm and demo patterns.
//!
//! ## How Drawing Works
//! Drawing to the screen is done by writing color values directly to the framebuffer memory. Each pixel is represented by a value at a specific offset, calculated from its (x, y) coordinates, the stride, and the pixel format. By setting these values, you control what appears on the display.

use crate::framebuffer::FramebufferInfo;
use libm::{floorf, roundf};

/// Computes the fractional part of a floating-point number.
fn fractf(x: f32) -> f32 {
    x - floorf(x)
}

/// Draws an 'X' across the entire framebuffer by drawing two diagonal lines.
///
/// # Arguments
/// * `fb` - Mutable reference to the framebuffer information struct.
///
/// This function demonstrates basic drawing by calling `draw_bresenham` for both diagonals.
pub fn framebuffer_x_demo(fb: &mut FramebufferInfo) {
    // Calculate the four corners of the framebuffer.
    let top_left = (0, 0);
    let top_right = (fb.width.saturating_sub(1), 0);
    let bottom_left = (0, fb.height.saturating_sub(1));
    let bottom_right = (fb.width.saturating_sub(1), fb.height.saturating_sub(1));

    // Draw both diagonals using Wu's anti-aliased algorithm.
    draw_wu_line(top_left.0, top_left.1, bottom_right.0, bottom_right.1, fb);
    draw_wu_line(top_right.0, top_right.1, bottom_left.0, bottom_left.1, fb);
}

/// Draws a line between two points using Bresenham's algorithm.
///
/// # Arguments
/// * `x0`, `y0` - Starting coordinates.
/// * `x1`, `y1` - Ending coordinates.
/// * `fb` - Mutable reference to the framebuffer information struct.
///
/// This function calculates the memory offset for each pixel along the line and writes a white pixel (0xFFFF_FFFF) directly to the framebuffer.
pub fn draw_bresenham(x0: usize, y0: usize, x1: usize, y1: usize, fb: &mut FramebufferInfo) {
    // Convert coordinates to signed integers for algorithm.
    let (mut x0, mut y0, x1, y1) = (x0 as isize, y0 as isize, x1 as isize, y1 as isize);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // Check bounds before writing to framebuffer.
        if x0 >= 0 && (x0 as usize) < fb.width && y0 >= 0 && (y0 as usize) < fb.height {
            // Calculate the memory offset for the pixel.
            // Each pixel is 4 bytes (assumed 32-bit color).
            let offset = fb.address as usize + (((y0 as usize) * fb.stride + (x0 as usize)) * 4);
            let pixel_ptr = offset as *mut u32;
            unsafe {
                // Write a white pixel (0xFFFF_FFFF) directly to framebuffer memory.
                pixel_ptr.write_volatile(0xFFFF_FFFF);
            }
        }
        // Stop if we've reached the end point.
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Blends a pixel at (x, y) with a given brightness (0.0 to 1.0).
fn blend_pixel(x: isize, y: isize, brightness: f32, fb: &mut FramebufferInfo) {
    if x >= 0 && (x as usize) < fb.width && y >= 0 && (y as usize) < fb.height {
        let offset = fb.address as usize + (((y as usize) * fb.stride + (x as usize)) * 4);
        let pixel_ptr = offset as *mut u32;
        unsafe {
            // Read the current pixel (assume black background)
            let bg = pixel_ptr.read_volatile();
            // Blend white (0xFFFFFFFF) with background based on brightness
            let alpha = (brightness.clamp(0.0, 1.0) * 255.0) as u32;
            let inv_alpha = 255 - alpha;
            let r = ((bg >> 16) & 0xFF) * inv_alpha / 255 + 255 * alpha / 255;
            let g = ((bg >> 8) & 0xFF) * inv_alpha / 255 + 255 * alpha / 255;
            let b = (bg & 0xFF) * inv_alpha / 255 + 255 * alpha / 255;
            let blended = (0xFF << 24) | (r << 16) | (g << 8) | b;
            pixel_ptr.write_volatile(blended);
        }
    }
}

/// Draws an anti-aliased line using Xiaolin Wu's algorithm.
pub fn draw_wu_line(x0: usize, y0: usize, x1: usize, y1: usize, fb: &mut FramebufferInfo) {
    let (mut x0, mut y0, mut x1, mut y1) = (x0 as f32, y0 as f32, x1 as f32, y1 as f32);
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    if steep {
        core::mem::swap(&mut x0, &mut y0);
        core::mem::swap(&mut x1, &mut y1);
    }
    if x0 > x1 {
        core::mem::swap(&mut x0, &mut x1);
        core::mem::swap(&mut y0, &mut y1);
    }
    let dx = x1 - x0;
    let dy = y1 - y0;
    let gradient = if dx == 0.0 { 1.0 } else { dy / dx };

    // handle first endpoint
    let xend = roundf(x0);
    let yend = y0 + gradient * (xend - x0);
    let xgap = 1.0 - fractf(x0 + 0.5);
    let xpxl1 = xend as isize;
    let ypxl1 = floorf(yend) as isize;
    if steep {
        blend_pixel(ypxl1, xpxl1, (1.0 - fractf(yend)) * xgap, fb);
        blend_pixel(ypxl1 + 1, xpxl1, fractf(yend) * xgap, fb);
    } else {
        blend_pixel(xpxl1, ypxl1, (1.0 - fractf(yend)) * xgap, fb);
        blend_pixel(xpxl1, ypxl1 + 1, fractf(yend) * xgap, fb);
    }
    let mut intery = yend + gradient;

    // handle second endpoint
    let xend = roundf(x1);
    let yend = y1 + gradient * (xend - x1);
    let xgap = fractf(x1 + 0.5);
    let xpxl2 = xend as isize;
    let ypxl2 = floorf(yend) as isize;
    if steep {
        blend_pixel(ypxl2, xpxl2, (1.0 - fractf(yend)) * xgap, fb);
        blend_pixel(ypxl2 + 1, xpxl2, fractf(yend) * xgap, fb);
    } else {
        blend_pixel(xpxl2, ypxl2, (1.0 - fractf(yend)) * xgap, fb);
        blend_pixel(xpxl2, ypxl2 + 1, fractf(yend) * xgap, fb);
    }

    // main loop
    if steep {
        for x in (xpxl1 + 1)..xpxl2 {
            let y = floorf(intery) as isize;
            blend_pixel(y, x, 1.0 - fractf(intery), fb);
            blend_pixel(y + 1, x, fractf(intery), fb);
            intery += gradient;
        }
    } else {
        for x in (xpxl1 + 1)..xpxl2 {
            let y = floorf(intery) as isize;
            blend_pixel(x, y, 1.0 - fractf(intery), fb);
            blend_pixel(x, y + 1, fractf(intery), fb);
            intery += gradient;
        }
    }
}
