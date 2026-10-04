#![allow(unused)]
pub use cgmath::num_traits::*;
pub use cgmath::prelude::*;
pub use std::ops::Neg;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type Quaternion = cgmath::Quaternion<f32>;

pub trait FlakeMath: Sized {
    fn normalize_or_zero(&self) -> Self;
    fn one() -> Self;
    fn default() -> Self;
    fn neg_one() -> Self;
}

impl<T> FlakeMath for T
where
    T: InnerSpace + Array<Element = <T as VectorSpace>::Scalar> + Copy,
    T::Scalar: cgmath::BaseFloat,
{
    fn normalize_or_zero(&self) -> Self {
        if self.magnitude2().is_zero() {
            Self::zero()
        } else {
            self.normalize()
        }
    }

    fn one() -> Self {
        Self::from_value(T::Scalar::one())
    }

    fn default() -> Self {
        Self::zero()
    }

    fn neg_one() -> Self {
        Self::from_value(T::Scalar::neg(T::Scalar::one()))
    }
}

use std::ops::{Add, Div, Mul, Sub};

pub trait Number:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    fn to_f32(self) -> f32;
    fn zero() -> Self;
    fn one() -> Self;
}

macro_rules! impl_number {
    ($($t:ty),*) => {
        $(impl Number for $t {
            fn to_f32(self) -> f32 { self as f32 }
            fn zero() -> Self { 0 as $t }
            fn one() -> Self { 1 as $t }
        })*
    };
}

impl_number!(f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);
