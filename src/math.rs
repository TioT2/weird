/// 2-component vector representation structure
#[derive(Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: std::fmt::Display> std::fmt::Display for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{}, {}>", self.x, self.y))
    }
} // impl std::fmt::Display for Vec2

#[derive(Copy, Clone, Debug)]
pub struct Ext2<T> {
    pub width: T,
    pub height: T,
}

pub type Vec2f = Vec2<f32>;
pub type Ext2zu = Vec2<f32>;
