/// WEIRD Project
/// `File` utils/fixed/mod.rs
/// `Description` Fixed number and angle implementation module
/// `Author` TioT2
/// `Last changed` 07.05.2024

/// 16.16 fixed number representation structure
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Fixed {
    /// Fixed number bits
    value: u32,
} // struct Fixed


impl From<f32> for Fixed {
    fn from(value: f32) -> Self {
        Fixed::from_f32(value)
    }
}

impl Into<f32> for Fixed {
    fn into(self) -> f32 {
        self.into_f32()
    }
}

impl std::ops::Add<Fixed> for Fixed {
    type Output = Fixed;
    fn add(self, rhs: Fixed) -> Self::Output {
        Self::Output {
            value: self.value.wrapping_add(rhs.value)
        }
    }
}

impl std::ops::Sub<Fixed> for Fixed {
    type Output = Fixed;
    fn sub(self, rhs: Fixed) -> Self::Output {
        Self::Output {
            value: self.value.wrapping_sub(rhs.value)
        }
    }
}

impl std::ops::Mul<Fixed> for Fixed {
    type Output = Fixed;
    fn mul(self, rhs: Fixed) -> Self::Output {
        Self::Output {
            value: ((self.value as u64).wrapping_mul(rhs.value as u64) >> 16) as u32
        }
    }
}

impl std::ops::Div<Fixed> for Fixed {
    type Output = Fixed;
    fn div(self, rhs: Fixed) -> Self::Output {
        Self::Output {
            value: (((self.value as u64) << 16).wrapping_div(rhs.value as u64)) as u32
        }
    }
}

impl std::ops::Neg for Fixed {
    type Output = Fixed;
    fn neg(self) -> Self::Output {
        Self::Output {
            value: self.value.wrapping_neg()
        }
    }
}

impl std::cmp::PartialOrd for Fixed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe {
            std::mem::transmute::<u32, i32>(self.value).partial_cmp(&std::mem::transmute::<u32, i32>(other.value))
        }
    }
}

impl std::cmp::Ord for Fixed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe {
            std::mem::transmute::<u32, i32>(self.value).cmp(&std::mem::transmute::<u32, i32>(other.value))
        }
    }
}

/// Utilitary constant-evaluated unwrapping function
/// * `val` - value to use if no values contained
/// * Returns unwrapped value
const fn unwrap_or<T: Copy>(opt: Option<T>, val: T) -> T {
    match opt {
        Some(v) => v,
        None => val
    }
} // fn unwrap_or


impl Fixed {
    /// Fixed number constant zero getting function
    /// * Returns zero fixed number
    pub const fn zero() -> Self {
        Self {
            value: 0,
        }
    } // fn zero

    /// Fixed number from it's u32 representation constructor
    /// * `bits` - bits to construct u32 with
    /// * Returns fixed point number
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            value: bits,
        }
    } // pub fn from_bits

    /// Number into bits conversion function
    /// * Returns number bits
    pub const fn to_bits(self) -> u32 {
        self.value
    }// fn to_bits

    /// Fixed-point number from f32 constructor
    /// * `value` - floating-point number to construct Fixed from
    /// * Returns fixed that represents `value` number
    pub const fn from_f32(value: f32) -> Self {
        let exp = (unsafe { std::mem::transmute::<f32, u32>(value) } >> 23) & 0xFF;

        Fixed {
            value: unwrap_or(if exp < 134 {
                ((unsafe { std::mem::transmute::<f32, u32>(value) } & 0x7FFFFF) | 0x800000).checked_shr(134 - exp)
            } else {
                ((unsafe { std::mem::transmute::<f32, u32>(value) } & 0x7FFFFF) | 0x800000).checked_shl(exp - 134)
            }, 0) ^ unsafe { std::mem::transmute::<i32, u32>(std::mem::transmute::<f32, i32>(value) >> 31) }
        }
    } // fn from_f32

    /// Fixed to f32 conversion function
    /// * Returns number that represents this f32 as floating-point
    pub const fn into_f32(self) -> f32 {
        unsafe {
            let unsigned = self.value ^ std::mem::transmute::<i32, u32>(std::mem::transmute::<u32, i32>(self.value) >> 31);
            let lz = unsigned.leading_zeros();

            std::mem::transmute::<u32, f32>((self.value & 0x80000000) | ((142 - lz) << 23) | (unwrap_or(unsigned.checked_shl(lz + 1), 0) >> 9))
        }
    } // fn into_f32

    /// Rounding function, rounds to nearest to zero
    /// * Returns rounded fixed-point number
    pub const fn round(self) -> Self {
        Self { value: self.value & 0xFFFF0000, }
    } // fn round

    /// Fractional part getting function
    /// * Returns fractional part.
    pub const fn fract(self) -> Self {
        Self { value: self.value & 0xFFFF, }
    } // fn fract
} // impl Fixed


/// Minimal fixed value possible
pub const MIN: Fixed = Fixed::from_bits(0x80000000);
/// Maximal fixed value possible
pub const MAX: Fixed = Fixed::from_bits(0xFFFFFFFF);

/// Fixed point constant module
pub mod consts {
    use super::Fixed;

    /// PI Number
    pub const PI: Fixed = Fixed::from_f32(std::f32::consts::PI);
    /// E Number
    pub const E: Fixed = Fixed::from_f32(std::f32::consts::E);
} // mod consts

/// Sine lookup table in [0..pi/4] range represented by fixed-point number
const SIN_QUART: [Fixed; 16384] = {
    let sinq: [f32; 16384] = include!("sin_quart.txt");
    let mut dst = [Fixed::zero(); 16384];

    let mut i = 0;
    while i < 16384 {
        dst[i] = Fixed::from_f32(sinq[i]);
        i += 1;
    }

    dst
};

/// Angle representaiton structure
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Angle {
    /// Angle actual value
    value: u16,
} // struct Angle

impl Angle {
    /// Zero angle getting function
    /// * Returns zero angle
    pub const fn zero() -> Self {
        Self {
            value: 0
        }
    } // fn zero

    /// From degrees contained in float32 number angle construction function
    /// * `degrees` - degrees in floating point
    /// * Returns angle
    pub fn from_degrees_f32(degrees: f32) -> Self {
        Self::from_radians_f32(degrees * std::f32::consts::PI / 180.0)
    } // fn from_degrees_f32

    /// From radians contained in float32 number angle construction function
    /// * `radians` - radians in floating point
    /// * Returns angle
    pub fn from_radians_f32(radians: f32) -> Self {
        Self {
            value: (((radians / (std::f32::consts::PI * 2.0)).fract() + 1.0).fract() * 65536.0) as u16
        }
    } // fn from_radians_f32

    /// Angle sine calculation function
    /// * Returns the angle sine
    pub const fn sin(mut self) -> Fixed {
        let fixed_xor_mask = unsafe { std::mem::transmute::<i32, u32>(std::mem::transmute::<u32, i32>((self.value as u32) << 16) >> 31) };
        self.value &= 0x7FFF;
        if self.value > 0x3FFF {
            self.value = 0x7FFFu16.wrapping_sub(self.value);
        }

        Fixed {
            value: SIN_QUART[self.value as usize].value ^ fixed_xor_mask
        }
    } // fn sin

    /// Angle cosine calculation function
    /// * Returns the angle cosine
    pub const fn cos(mut self) -> Fixed {
        self.value = self.value.wrapping_neg().wrapping_add(0x3FFF);
        self.sin()
    } // fn cos
} // impl Angle

// file mod.rs
