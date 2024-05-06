/// WEIRD Project
/// `File` math.rs
/// `Description` Math utilities implementation module
/// `Author` TioT2
/// `Last changed` 05.05.2024

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: std::fmt::Display> std::fmt::Display for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{}, {}>", self.x, self.y))
    }
} // impl std::fmt::Display for Vec2

impl<T: std::fmt::Display> std::fmt::Display for Vec3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{}, {}, {}>", self.x, self.y, self.z))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Ext2<T> {
    pub width: T,
    pub height: T,
}

pub type Vec2f = Vec2<f32>;
pub type Vec2si = Vec2<isize>;
pub type Ext2su = Ext2<usize>;
