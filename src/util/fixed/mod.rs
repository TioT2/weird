/// WEIRD Project
/// `File` utils/fixed/mod.rs
/// `Description` Fixed number and angle implementation module
/// `Author` TioT2
/// `Last changed` 07.05.2024

/// 16.16 fixed number representation structure
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Fixed {
    /// Fixed number bits
    value: i32,
} // struct Fixed

impl crate::math::numeric_traits::Sqrt for Fixed {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Default for Fixed {
    fn default() -> Self {
        Fixed::zero()
    }
}

impl From<f32> for Fixed {
    fn from(value: f32) -> Self {
        Fixed::from_f32(value)
    }
}

impl From<i16> for Fixed {
    fn from(value: i16) -> Self {
        Fixed::from_i16(value)
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

impl std::ops::AddAssign<Fixed> for Fixed {
    fn add_assign(&mut self, rhs: Fixed) {
        *self = *self + rhs;
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


impl std::ops::SubAssign<Fixed> for Fixed {
    fn sub_assign(&mut self, rhs: Fixed) {
        *self = *self - rhs;
    }
}

impl std::ops::Mul<Fixed> for Fixed {
    type Output = Fixed;
    fn mul(self, rhs: Fixed) -> Self::Output {
        Self::Output {
            value: ((self.value as i64).wrapping_mul(rhs.value as i64) >> 16) as i32
        }
    }
}

impl Fixed {
    /// Compile-time mul implementation function
    pub const fn mul_const(self, rhs: Fixed) -> Self {
        Self {
            value: ((self.value as i64).wrapping_mul(rhs.value as i64) >> 16) as i32
        }
    }
}

impl std::ops::MulAssign<Fixed> for Fixed {
    fn mul_assign(&mut self, rhs: Fixed) {
        *self = *self * rhs;
    }
}

impl std::ops::Div<Fixed> for Fixed {
    type Output = Fixed;
    fn div(self, rhs: Fixed) -> Self::Output {
        Self::Output {
            value: ((((self.value as i64) << 32) / if rhs.value == 0 { 1 } else { rhs.value as i64 }) >> 16) as i32,
        }
    }
}

impl Fixed {
    const fn div_const(self, rhs: Fixed) -> Self {
        Self {
            value: ((((self.value as i64) << 32) / if rhs.value == 0 { 1 } else { rhs.value as i64 }) >> 16) as i32,
        }
    }
}

impl std::ops::DivAssign<Fixed> for Fixed {
    fn div_assign(&mut self, rhs: Fixed) {
        *self = *self / rhs;
    }
}

impl std::ops::Neg for Fixed {
    type Output = Fixed;
    fn neg(self) -> Self::Output {
        Self::Output {
            value: self.value.wrapping_neg().wrapping_sub(1)
        }
    }
}

impl std::cmp::PartialOrd for Fixed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl std::cmp::Ord for Fixed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
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
            value: unsafe { std::mem::transmute::<u32, i32>(bits) },
        }
    } // pub fn from_bits

    /// Number into bits conversion function
    /// * Returns number bits
    pub const fn to_bits(self) -> u32 {
        unsafe { std::mem::transmute::<i32, u32>(self.value) }
    }// fn to_bits

    /// Fixed-point number from f32 constructor
    /// * `value` - floating-point number to construct Fixed from
    /// * Returns fixed that represents `value` number
    pub const fn from_f32(value: f32) -> Self {
        let exp = (unsafe { std::mem::transmute::<f32, u32>(value) } >> 23) & 0xFF;

        Fixed {
            value: unsafe { std::mem::transmute::<u32, i32>(unwrap_or(if exp < 134 {
                ((std::mem::transmute::<f32, u32>(value) & 0x7FFFFF) | 0x800000).checked_shr(134 - exp)
            } else {
                ((std::mem::transmute::<f32, u32>(value) & 0x7FFFFF) | 0x800000).checked_shl(exp - 134)
            }, 0) ^ std::mem::transmute::<i32, u32>(std::mem::transmute::<f32, i32>(value) >> 31)) }
        }
    } // fn from_f32

    /// Fixed to f32 conversion function
    /// * Returns number that represents this f32 as floating-point
    pub const fn into_f32(self) -> f32 {
        unsafe {
            let unsigned = std::mem::transmute::<i32, u32>(self.value ^ (self.value >> 31));
            let lz = unsigned.leading_zeros();

            std::mem::transmute::<u32, f32>((std::mem::transmute::<i32, u32>(self.value) & 0x80000000) | ((142 - lz) << 23) | (unwrap_or(unsigned.checked_shl(lz + 1), 0) >> 9))
        }
    } // fn into_f32

    /// Fixed-point number from u16 constructor
    /// * `value` - 16 bit signed integral value to construct Fixed from
    /// * Returns fixed that represents `value` number
    pub const fn from_i16(value: i16) -> Self {
        Fixed {
            value: (value as i32) << 16,
        }
    } // fn from_i16

    /// Fixed-point number from u16 constructor
    /// * `value` - 16 bit signed integral value to construct Fixed from
    /// * Returns fixed that represents `value` number
    pub const fn into_i16(self) -> i16 {
        (self.value >> 16) as i16
    } // fn from_i16

    /// Rounding function, rounds to nearest to zero
    /// * Returns rounded fixed-point number
    pub const fn round(self) -> Self {
        Self { value: self.value & -65536, }
    } // fn round

    /// Fractional part getting function
    /// * Returns fractional part.
    pub const fn fract(self) -> Self {
        Self { value: self.value & 0xFFFF, }
    } // fn fract

    /// Absolute value calculation function
    /// * Returns module.
    pub const fn abs(self) -> Fixed {
        Fixed {
            value: self.value ^ (self.value << 16 >> 31),
        }
    } // fn abs

    /// Arccosine calculation function
    /// * Returns acos
    pub const fn acos(self) -> Angle {
        return if self.value >= 0 {
            ACOS[self.value as usize]
        } else {
            Angle::from_bits(32768u16.wrapping_sub(ACOS[(-self.value - 1) as usize].value))
        };
    } // fn acos

    /// Signum calculation function
    /// * Returns self signum
    pub const fn signum(self) -> Self {
        Self {
            value: if self.value >= 0 { 65536 } else { -65536 }
        }
    } // fn signum

    /// Square root calculation function
    /// * Returns square root
    pub const fn sqrt(self) -> Self {
        // Assert on sqrt
        // debug_assert!(self.value >= 0);

        let mut r: u32 = unsafe { std::mem::transmute::<i32, u32>(self.value) };
        let mut q: u32 = 0;
        let mut b: u32 = 0x40000000u32;
        let mut t: u32;

        if r < 0x40000200 {
            while  b != 0x40 {
                t = q + b;
                if r >= t
                {
                    r -= t;
                    q = t + b; // equivalent to q += 2*b
                }
                r <<= 1;
                b >>= 1;
            }
            return Fixed::from_bits(q >> 8);
        }
        while b > 0x40 {
            t = q + b;
            if r >= t {
                r -= t;
                q = t + b; // equivalent to q += 2*b
            }
            if (r & 0x80000000) != 0 {
                q >>= 1;
                b >>= 1;
                r >>= 1;
                while b > 0x20 {
                    t = q + b;
                    if r >= t {
                        r -= t;
                        q = t + b;
                    }
                    r <<= 1;
                    b >>= 1;
                }
                return Fixed::from_bits(q >> 7);
            }
            r <<= 1;
            b >>= 1;
        }
        return Fixed::from_bits(q >> 8);
    } // fn sqrt

    /// Approximate distance calculation function
    /// * `dx` - delta by x axis
    /// * `dy` - delta by y axis
    /// * Returns approximate dx, dy vector length
    pub const fn approx_distance(mut dx: Fixed, mut dy: Fixed) -> Fixed {
        dx = dx.abs();
        dy = dy.abs();
        if dx.value < dy.value {
            Fixed { value: dx.value + dy.value - (dx.value >> 1) }
        } else {
            Fixed { value: dx.value + dy.value - (dy.value >> 1) }
        }
    }
} // impl Fixed


/// Minimal fixed value possible
pub const MIN: Fixed = Fixed::from_bits(0x80000000);
/// Maximal fixed value possible
pub const MAX: Fixed = Fixed::from_bits(0xFFFFFFFF);
/// Minimal fixed value possible
pub const EPSILON: Fixed = Fixed::from_bits(1);

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

/// Arccosine lookup table in [0..1] range represented by angle
const ACOS: [Angle; 65537] = {
    let acos: [u16; 65537] = include!("acos.txt");
    let mut dst = [Angle::zero(); 65537];

    let mut i = 0;
    while i < 65537 {
        dst[i] = Angle::from_bits(acos[i]);
        i += 1;
    }
    dst
};

/// Angle representaiton structure
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
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

    /// Into fixed point radians conversion function
    /// * Returns angle
    pub const fn into_radians_fixed(self) -> Fixed {
        Fixed { value: self.value as i32 }.mul_const(consts::PI.mul_const(Fixed::from_i16(2)))
    } // fn into_radians_fixed

    /// From fixed point radians conversion function
    /// * `fixed` - amount of radians to get
    /// * Returns angle
    pub const fn from_radians_fixed(fixed: Fixed) -> Self {
        Self {
            value: fixed.div_const(consts::PI.mul_const(Fixed::from_i16(2))).to_bits() as u16
        }
    } // fn from_radians_fixed

    /// Into raw value construction function
    /// * Returns raw value
    pub const fn to_bits(self) -> u16 {
        self.value
    } // fn to_bits

    /// From raw value construction function
    /// * `bits` - bits to construct angle from
    /// * Returns angle
    pub const fn from_bits(bits: u16) -> Self {
        Self { value: bits }
    } // fn from_bits

    /// From radians contained in float32 number angle construction function
    /// * `radians` - radians in floating point
    /// * Returns angle
    pub fn into_radians_f32(self) -> f32 {
        self.value as f32 * std::f32::consts::PI / 32768.0
    } // fn from_radians_f32

    /// Angle sine calculation function
    /// * Returns the angle sine
    pub const fn sin(mut self) -> Fixed {
        let fixed_xor_mask = unsafe { std::mem::transmute::<u32, i32>((self.value as u32) << 16) >> 31 };
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

impl std::ops::Add<Angle> for Angle {
    type Output = Angle;
    fn add(self, rhs: Self) -> Self::Output {
        Self { value: self.value.wrapping_add(rhs.value) }
    }
}

impl std::ops::AddAssign<Angle> for Angle {
    fn add_assign(&mut self, rhs: Angle) {
        self.value = self.value.wrapping_add(rhs.value);
    }
}

impl std::ops::Sub<Angle> for Angle {
    type Output = Angle;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { value: self.value.wrapping_sub(rhs.value) }
    }
}

impl std::ops::SubAssign<Angle> for Angle {
    fn sub_assign(&mut self, rhs: Angle) {
        self.value = self.value.wrapping_sub(rhs.value);
    }
}

impl std::ops::Mul<Fixed> for Angle {
    type Output = Angle;
    fn mul(self, rhs: Fixed) -> Self::Output {
        Self {
            value: ((Fixed::from_bits((self.value as u32) << 16) * rhs).value >> 16) as u16
        }
    }
}

// file mod.rs
