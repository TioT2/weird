use crate::Ext2;


#[derive(Copy, Clone)]
pub struct Surface {
    pub data: *mut u32,
    pub width: usize,
    pub stride: usize,
    pub height: usize,
}

impl Surface {
    pub unsafe fn draw_bar_unchecked(&self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let mut yptr = self.data.add(y0 * self.stride + x0);
        let yeptr = yptr.add((y1 - y0) * self.stride);
        let dx = x1 - x0;

        while yptr != yeptr {
            let mut xptr = yptr;
            let xeptr = xptr.add(dx);

            while xptr != xeptr {
                *xptr = color;
                xptr = xptr.add(1);
            }

            yptr = yptr.add(self.stride);
        }
    }

    pub fn draw_bar(&self, x0: isize, y0: isize, x1: isize, y1: isize, color: u32) {
        unsafe {
            let x0 = std::mem::transmute::<isize, usize>(x0.clamp(0, self.width as isize));
            let y0 = std::mem::transmute::<isize, usize>(y0.clamp(0, self.height as isize));

            let x1 = std::mem::transmute::<isize, usize>(x1.clamp(0, self.width as isize));
            let y1 = std::mem::transmute::<isize, usize>(y1.clamp(0, self.height as isize));

            self.draw_bar_unchecked(x0, y0, x1, y1, color);
        }
    }

    fn clip_line(mut x0: isize, mut y0: isize, mut x1: isize, mut y1: isize, w: usize, h: usize) -> Option<(usize, usize, usize, usize)> {
        // Clip line
        const LOC_INSIDE: u32 = 0;
        const LOC_LEFT: u32 = 1;
        const LOC_RIGHT: u32 = 2;
        const LOC_BOTTOM: u32 = 4;
        const LOC_TOP: u32 = 8;

        let w = w as isize - 1;
        let h = h as isize - 1;

        let get_point_code = |x: isize, y: isize| {
            let mut code = LOC_INSIDE;

            if x < 0 { code |= LOC_LEFT; }
            if x > w { code |= LOC_RIGHT; }
            if y < 0 { code |= LOC_TOP; }
            if y > h { code |= LOC_BOTTOM; }

            code
        };

        let mut code_0 = get_point_code(x0, y0);
        let mut code_1 = get_point_code(x1, y1);

        if 'intersection_loop: loop {
            if (code_0 | code_1) == LOC_INSIDE {
                break 'intersection_loop true;
            }

            if (code_0 & code_1) != LOC_INSIDE {
                break 'intersection_loop false;
            }

            let out_code = if code_0 != LOC_INSIDE {
                code_0
            } else {
                code_1
            };

            let dx = x1 - x0;
            let dy = y1 - y0;

            let px;
            let py;

            if out_code & LOC_TOP != 0 {
                px = x0 - dx * y0 / dy;
                py = 0;
            } else if out_code & LOC_BOTTOM != 0 {
                px = x0 + dx * (h - y0) / dy;
                py = h;
            } else if out_code & LOC_LEFT != 0 {
                px = 0;
                py = y0 - dy * x0 / dx;
            } else if out_code & LOC_RIGHT != 0 {
                px = w;
                py = y0 + dy * (w - x0) / dx;
            } else {
                break 'intersection_loop false;
            }

            if out_code == code_0 {
                x0 = px;
                y0 = py;
                code_0 = get_point_code(x0, y0);
            } else {
                x1 = px;
                y1 = py;
                code_1 = get_point_code(x1, y1);
            }
        } {
            // Clipped values are safe to transmute into unsigned
            unsafe {
                Some((
                    std::mem::transmute::<isize, usize>(x0),
                    std::mem::transmute::<isize, usize>(y0),
                    std::mem::transmute::<isize, usize>(x1),
                    std::mem::transmute::<isize, usize>(y1),
                ))
            }
        } else {
            None
        }
    }

    pub fn draw_line(&self, x0: isize, y0: isize, x1: isize, y1: isize, color: u32) {
        if let Some((x0, y0, x1, y1)) = Self::clip_line(x0, y0, x1, y1, self.width, self.height) {
            unsafe {
                self.draw_line_unchecked(x0, y0, x1, y1, color);
            }
        }
    }

    pub unsafe fn draw_line_unchecked(&self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let (mut dy, sy): (usize, usize) = if y1 < y0 {
            (y0 - y1, self.stride.wrapping_neg())
        } else {
            (y1 - y0, self.stride)
        };
        let (mut dx, sx): (usize, usize) = if x1 < x0 {
            (x0 - x1, 1usize.wrapping_neg())
        } else {
            (x1 - x0, 1usize)
        };

        let mut pptr = self.data.wrapping_add(y0 * self.stride + x0);
        pptr.write(color);

        if dx >= dy {
            let ie = 2 * dy;
            let mut f = ie.wrapping_sub(dx);
            let ine = ie.wrapping_sub(2 * dx);

            while dx != 0 {
                pptr = pptr.wrapping_add(sx);
                pptr.write(color);
                dx -= 1;
                if f < std::mem::transmute(isize::MIN) {
                    pptr = pptr.wrapping_add(sy);
                    f = f.wrapping_add(ine);
                } else {
                    f = f.wrapping_add(ie);
                }
            }
        } else {
            let ie = 2 * dx;
            let mut f = ie.wrapping_sub(dy);
            let ine = ie.wrapping_sub(2 * dy);

            while dy != 0 {
                pptr = pptr.wrapping_add(sy);
                pptr.write(color);
                dy -= 1;

                if f < std::mem::transmute(isize::MIN) {
                    pptr = pptr.wrapping_add(sx);
                    f = f.wrapping_add(ine);
                } else {
                    f = f.wrapping_add(ie);
                }
            }
        }
    }

    pub fn get_data_mut<'a>(&'a mut self) -> &'a mut [u32] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data, self.stride * self.height)
        }
    }

    pub fn get_extent(&self) -> Ext2<usize> {
        Ext2 {
            width: self.width,
            height: self.height,
        }
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }
}

// file self.rs