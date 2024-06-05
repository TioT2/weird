/// Xorshift-based randomizer representaiton structure
#[derive(Copy, Clone, Debug)]
pub struct Xorshift32 {
    state: u32
} // struct XorshiftRand

impl Xorshift32 {
    /// New xorshift randomizer create function
    pub fn new(seed: u32) -> Xorshift32 {
        Xorshift32 { state: seed }
    } // fn new

    /// Next number yielding function
    /// Returns next random value
    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        return x;
    } // pub fn next
} // impl XorshiftRand

impl Iterator for Xorshift32 {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    } // fn next
} // impl Iterator for Xorshift32

// file xorshift_rand.rs
