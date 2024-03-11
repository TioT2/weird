
/// Unordered pair representation strucutre 
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct UnorderedPair<T> {
    first: T,
    second: T,
} // struct UnorderedPair


impl<T: std::cmp::PartialOrd> UnorderedPair<T> {
    pub fn new(first: T, second: T) -> Self {
        if first < second {
            Self {
                first,
                second,
            }
        } else {
            Self {
                first: second,
                second: first,
            }
        }
    }
} // impl UnorderedPair

impl<T> UnorderedPair<T> {
    pub fn first(&self) -> &T {
        &self.first
    }

    pub fn second(&self) -> &T {
        &self.second
    }
} // impl UnorderedPair

impl<T: std::cmp::PartialOrd> From<(T, T)> for UnorderedPair<T> {
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, value.1)
    }
} // impl From for UnorderedPair

impl<T> Into<(T, T)> for UnorderedPair<T> {
    fn into(self) -> (T, T) {
        (self.first, self.second)
    }
} // impl From for UnorderedPair

// file unordered_pair.rs

