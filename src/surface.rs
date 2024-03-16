
#[derive(Copy, Clone)]
pub struct Surface {
    pub data: *mut u32,
    pub width: usize,
    pub stride: usize,
    pub height: usize,
}

// file surface.rs