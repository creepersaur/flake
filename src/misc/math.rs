#![allow(unused)]
use cgmath::prelude::*;
use std::ops::Neg;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type Quaternion = cgmath::Quaternion<f32>;

pub trait FlakeMath: Sized {
    fn normalize_or_zero(&self) -> Self;
    fn one() -> Self;
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

    fn neg_one() -> Self {
        Self::from_value(T::Scalar::neg(T::Scalar::one()))
    }
}
