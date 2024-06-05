use std::ops::{Add, AddAssign, BitXor, Div, DivAssign, Mul, MulAssign, Neg, Range, Rem, RemAssign, Sub, SubAssign};

pub mod numeric_traits {
    pub trait Sqrt {
        fn sqrt(self) -> Self;
    }

    impl Sqrt for f32 {
        fn sqrt(self) -> Self {
            self.sqrt()
        }
    }

    impl Sqrt for f64 {
        fn sqrt(self) -> Self {
            self.sqrt()
        }
    }
}

macro_rules! consume_ident {
    ($type: ty, $i: ident) => { $type };
}

macro_rules! impl_vecn_base {
    ($struct_name: ident, $template_type: ident, $value_type: ty, $($x: ident),*) => {
        #[derive(Debug, Default, PartialEq)]
        pub struct $struct_name<$template_type> {
            $( pub $x : $value_type, )*
        }

        impl<$template_type: Clone> Clone for $struct_name<$template_type> where $value_type: Clone {
            fn clone(&self) -> Self {
                Self {
                    $( $x: self.$x.clone() ),*
                }
            }
        }

        impl<$template_type: Copy> Copy for $struct_name<$template_type> where $value_type: Copy {

        }

        impl<$template_type> $struct_name<$template_type> {
            pub fn new($($x: $value_type,)*) -> Self {
                Self { $($x,)* }
            }

            pub fn from_tuple(t: ( $( consume_ident!($value_type, $x) ),* )) -> Self {
                Self::from(t)
            }

            pub fn into_tuple(self) -> ( $( consume_ident!($value_type, $x) ),* ) {
                self.into()
            }
        }

        impl<$template_type> Into<( $( consume_ident!($value_type, $x) ),* )> for $struct_name<$template_type> {
            fn into(self) -> ( $( consume_ident!($value_type, $x) ),* ) {
                ( $( self.$x ),* )
            }
        }

        impl<$template_type> From<( $( consume_ident!($value_type, $x) ),* )> for $struct_name<$template_type> {
            fn from(t: ( $( consume_ident!($value_type, $x) ),* )) -> Self {
                let ($($x),*) = t;

                Self { $($x),* }
            }
        }
    }
}

macro_rules! impl_vecn_binary_operator {
    ($op_name: ident, $op_fn_name: ident, $struct_name: ident, $($x: ident),*) => {
        impl<A: $op_name<Output = A>> $op_name<$struct_name<A>> for $struct_name<A> {
            type Output = $struct_name<A>;

            fn $op_fn_name(self, rhs: $struct_name<A>) -> Self::Output {
                Self::Output {
                    $( $x: $op_name::$op_fn_name(self.$x, rhs.$x), )*
                }
            }
        }

        impl<T: Clone + $op_name<Output = T>> $op_name<T> for $struct_name<T> {
            type Output = $struct_name<T>;

            fn $op_fn_name(self, rhs: T) -> Self::Output {
                Self::Output {
                    $( $x: $op_name::$op_fn_name(self.$x, rhs.clone()), )*
                }
            }
        }
    }
}

macro_rules! impl_vecn_assignment_operator {
    ($op_name: ident, $op_fn_name: ident, $struct_name: ident, $($x: ident),*) => {
        impl<T: $op_name> $op_name<$struct_name<T>> for $struct_name<T> {
            fn $op_fn_name(&mut self, rhs: $struct_name<T>) {
                $( $op_name::<T>::$op_fn_name(&mut self.$x, rhs.$x); )*
            }
        }

        impl<T: Clone + $op_name> $op_name<T> for $struct_name<T> {
            fn $op_fn_name(&mut self, rhs: T) {
                $( $op_name::<T>::$op_fn_name(&mut self.$x, rhs.clone()); )*
            }
        }
    }
}

macro_rules! impl_vecn_unary_operator {
    ($op_name: ident, $op_fn_name: ident, $struct_name: ident, $($x: ident),*) => {
        impl<T: $op_name<Output = T>> $op_name for $struct_name<T> {
            type Output = $struct_name<T>;

            fn $op_fn_name(self) -> Self::Output {
                Self::Output {
                    $( $x: $op_name::$op_fn_name(self.$x), )*
                }
            }
        }
    }
}

macro_rules! operator_on_variadic {
    ($operator: tt, $first: expr) => {
        $first
    };

    ($operator: tt, $first: expr, $($rest: expr),*) => {
        $first $operator operator_on_variadic!($operator, $($rest),*)
    };
}

macro_rules! impl_vecn {
    ($struct_name: ident, $($x: ident),*) => {
        impl_vecn_base!($struct_name, T, T, $($x),*);

        impl<T: Add<T, Output = T> + Mul<T, Output = T>> BitXor for $struct_name<T> {
            type Output = T;

            fn bitxor(self, rhs: $struct_name<T>) -> Self::Output {
                operator_on_variadic!(+, $(self.$x * rhs.$x),*)
            }
        }

        impl<T: Add<T, Output = T> + Mul<T, Output = T> + Clone> $struct_name<T> {
            pub fn length2(&self) -> T {
                self.clone() ^ self.clone()
            }
        }

        impl<T: Add<T, Output = T> + Mul<T, Output = T> + Clone + numeric_traits::Sqrt> $struct_name<T> {
            pub fn length(&self) -> T {
                self.length2().sqrt()
            }
        }

        impl<T: Add<T, Output = T> + Mul<T, Output = T> + Div<T, Output = T> + Clone + numeric_traits::Sqrt> $struct_name<T> {
            pub fn normalized(&self) -> Self {
                let len = self.length();

                Self { $( $x: self.$x.clone() / len.clone() ),* }
            }

            pub fn normalize(&mut self) {
                let len = self.length();

                $( self.$x = self.$x.clone() / len.clone(); )*
            }
        }

        impl_vecn_binary_operator!(Add, add, $struct_name, $($x),*);
        impl_vecn_binary_operator!(Sub, sub, $struct_name, $($x),*);
        impl_vecn_binary_operator!(Mul, mul, $struct_name, $($x),*);
        impl_vecn_binary_operator!(Div, div, $struct_name, $($x),*);

        impl_vecn_unary_operator!(Neg, neg, $struct_name, $($x),*);

        impl_vecn_assignment_operator!(AddAssign, add_assign, $struct_name, $($x),*);
        impl_vecn_assignment_operator!(SubAssign, sub_assign, $struct_name, $($x),*);
        impl_vecn_assignment_operator!(MulAssign, mul_assign, $struct_name, $($x),*);
        impl_vecn_assignment_operator!(DivAssign, div_assign, $struct_name, $($x),*);
    }
}

macro_rules! impl_extn {
    ($struct_name: ident, $($x: ident),*) => {
        impl_vecn_base!($struct_name, T, T, $($x),*);
    }
}

macro_rules! impl_rectn {
    ($struct_name: ident, $point_name: ident, $ext_name: ident, $($x: ident),*) => {
        impl_vecn_base!($struct_name, T, Range<T>, $($x),*);

        impl<T> $struct_name<T> where Range<T>: ExactSizeIterator {
            pub fn extent(&self) -> $ext_name<usize> {
                $ext_name::<usize>::new($( self.$x.len() ),*)
            }
        }

        impl<T: Clone> $struct_name<T> {
            pub fn start(&self) -> $point_name<T> {
                $point_name::<T>::new($( self.$x.start.clone() ),*)
            }

            pub fn end(&self) -> $point_name<T> {
                $point_name::<T>::new($( self.$x.end.clone() ),*)
            }
        }
    }
}

impl_vecn!(Vec2, x, y);
impl_vecn!(Vec3, x, y, z);
impl_vecn!(Vec4, x, y, z, w);

impl_extn!(Ext2, w, h);
impl_extn!(Ext3, w, h, d);

impl_rectn!(Rect, Vec2, Ext2, x, y);
impl_rectn!(Box, Vec3, Ext3, x, y, z);

pub type Ext2f = Ext2<f32>;
pub type Vec2f = Vec2<f32>;
pub type Vec3f = Vec3<f32>;

pub type Vec2u32 = Vec2<u32>;
pub type Ext2u32 = Ext2<u32>;

pub type Vec2us = Vec2<usize>;
pub type Ext2us = Ext2<usize>;



impl<T: Clone + Mul<T, Output = T> + Sub<T, Output = T>> Rem for Vec3<T> {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.y.clone() * rhs.z.clone() - self.z.clone() * rhs.y.clone(),
            y: self.z * rhs.x.clone() - self.x.clone() * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl<T: Clone + Mul<T, Output = T> + Sub<T, Output = T>> RemAssign for Vec3<T> {
    fn rem_assign(&mut self, rhs: Self) {
        *self = self.clone() % rhs;
    }
}

impl<T: Clone + Mul<T, Output = T> + Sub<T, Output = T>> Rem for Vec2<T> {
    type Output = T;
    fn rem(self, rhs: Self) -> Self::Output {
        self.x * rhs.y - self.y * rhs.x
    }
}
