use crate::Surface;
use crate::math::Ext2;

/// Font representation structure
pub struct Font {
    width: usize,
    height: usize,
    stride: usize,
    letter_stride: usize,
    bits: Vec<u8>,
} // struct Font

/// Font loading error
#[derive(Clone, Debug, PartialEq)]
pub enum FntLoadingError {
    InappropriateDataSize { required: usize },
    InappropriateStride { minimal_required: usize },
    TooLargeStride,
    Other(String),
}

impl Default for Font {
    fn default() -> Self {
        let default_font_data = include_bytes!("../fonts/8x8t.fnt");
        Self::from_fnt_bytes(8, 8, 1, default_font_data).unwrap()
    }
}

impl Font {
    /// Fron .FNT file bytes construction function
    /// * `width_bits` - font width in bits
    /// * `height` - font height
    /// * `stride` - font stride bytes
    /// * `fnt_bytes` - font byte data
    /// * Returns font or .FNT file loading error
    pub fn from_fnt_bytes(width_bits: u32, height: u32, stride: u32, fnt_bytes: &[u8]) -> Result<Self, FntLoadingError> {
        if stride > 8 {
            return Err(FntLoadingError::TooLargeStride);
        }

        if (height * stride) as usize * 256 != fnt_bytes.len() {
            return Err(FntLoadingError::InappropriateDataSize { required: (height * stride) as usize });
        }

        if width_bits > stride * 8 {
            return Err(FntLoadingError::InappropriateStride { minimal_required: width_bits as usize / 8 + ((width_bits % 8) != 0) as usize })
        }

        Ok(Font {
            width: width_bits as usize,
            height: height as usize,
            stride: stride as usize,
            letter_stride: (height * stride) as usize,
            bits: {
                let mut bits = Vec::<u8>::with_capacity(fnt_bytes.len() / 2 + 7);

                bits.extend_from_slice(&fnt_bytes[0..fnt_bytes.len() / 2]);
                for b in &mut bits {
                    let mut rb = *b;
                    rb = (rb >> 4) | (rb << 4);
                    rb = ((rb >> 2) & 0x33) | ((rb << 2) & 0xCC);
                    rb = ((rb >> 1) & 0x55) | ((rb << 1) & 0xAA);
                    *b = rb;
                }
                bits.resize(bits.len() + 7, 0);

                bits
            },
        })
    } // fn from_fnt_bytes

    /// String to surface putting function
    /// * `surface` - surface to render string to
    /// * `x` - string x coordinate
    /// * `y` - string y coordinate
    /// * `line` - string to put
    /// * `color` - text color
    pub fn put_string(&self, surface: &mut Surface, x: usize, y: usize, line: &str, color: u32) {
        // base pointer
        let stride = surface.get_stride();
        let ext = surface.get_extent();
        let mut base_ptr = unsafe { surface.get_data_mut().as_mut_ptr().add(y * stride + x) };

        for (index, ch_unicode) in line.chars().enumerate() {
            let ch = if ch_unicode.is_ascii() {
                ch_unicode as u8
            } else {
                b'?'
            };

            // Break the loop if have to enough space to print next letter
            if (index + 1) * (self.width + 1) - 1 + x >= ext.width {
                break;
            }

            let mut y_ptr = base_ptr;
            for y in 0..self.height {
                let mut line_bits: u64 = unsafe { std::mem::transmute::<*const u8, *const u64>(self.bits.as_ptr().add(ch as usize * self.letter_stride + y * self.stride)).read_unaligned() };
                let mut x_ptr = y_ptr;

                for _ in 0..self.width {
                    if line_bits & 1 == 1 {
                        unsafe {
                            *x_ptr = color;
                        }
                    }

                    line_bits >>= 1;
                    x_ptr = unsafe { x_ptr.add(1) };
                }

                y_ptr = unsafe { y_ptr.add(surface.get_stride()) };
            }

            // put character
            base_ptr = unsafe { base_ptr.add(self.width + 1) };
        }
    } // fn put_line

    /// Font size getting function
    /// * Returns letter extnet
    pub fn get_letter_size(&self) -> Ext2<usize> {
        Ext2 {
            width: self.width,
            height: self.height,
        }
    } // pub fn size
} // impl Font

// file font.rs
