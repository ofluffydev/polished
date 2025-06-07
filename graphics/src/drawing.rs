use crate::framebuffer::FramebufferInfo;

pub fn framebuffer_x_demo(fb: &mut FramebufferInfo) {
    let top_left = (0, 0);
    let top_right = (fb.width.saturating_sub(1), 0);
    let bottom_left = (0, fb.height.saturating_sub(1));
    let bottom_right = (fb.width.saturating_sub(1), fb.height.saturating_sub(1));

    draw_bresenham(top_left.0, top_left.1, bottom_right.0, bottom_right.1, fb);
    draw_bresenham(top_right.0, top_right.1, bottom_left.0, bottom_left.1, fb);
}

pub fn draw_bresenham(x0: usize, y0: usize, x1: usize, y1: usize, fb: &mut FramebufferInfo) {
    let (mut x0, mut y0, x1, y1) = (x0 as isize, y0 as isize, x1 as isize, y1 as isize);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && (x0 as usize) < fb.width && y0 >= 0 && (y0 as usize) < fb.height {
            let offset = fb.address as usize + (((y0 as usize) * fb.stride + (x0 as usize)) * 4);
            let pixel_ptr = offset as *mut u32;
            unsafe {
                pixel_ptr.write_volatile(0xFFFF_FFFF);
            }
        }
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
