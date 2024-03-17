/// Xorshift-based randomizer representaiton structure
#[derive(Copy, Clone, Debug)]
pub struct XorshiftRand {
    state: u32
} // struct XorshiftRand

impl XorshiftRand {
    /// New xorshift randomizer create function
    pub fn new(seed: u32) -> XorshiftRand {
        XorshiftRand { state: seed }
    } // fn new

    /// Random u32 generation function
    /// * Returns random u32
    pub fn rand_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;

        self.state
    } // fn rand_u32

    /// Random unit float generation function
    /// * Returns f32 in 0..1 range
    pub fn rand_unit_f32(&mut self) -> f32 {
        let u = self.rand_u32();

        (u as f32) / (u32::MAX as f32)
    } // fn rand_unit_f32

    /// Random f32 generation function
    /// * `begin` - generation range begin
    /// * `end` - generation range end
    /// * Returns random f32 in begin..end range
    pub fn rand_f32(&mut self, begin: f32, end: f32) -> f32 {
        self.rand_unit_f32() * (end - begin) + begin
    } // fn rand_f32
} // impl XorshiftRand

// file xorshift_rand.rs
