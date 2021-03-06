//! Trait definitions and type aliases.

use image::{
    Rgb,
    Luma,
    Pixel,
    ImageBuffer
};
use std::{
    u8,
    u16,
    i16
};

/// An ImageBuffer containing Pixels of type P with storage
/// Vec<P::Subpixel>.
// TODO: This produces a compiler warning about trait bounds
// TODO: not being enforced in type definitions. In this case
// TODO: they are. Can we get rid of the warning?
pub type VecBuffer<P: Pixel> = ImageBuffer<P, Vec<P::Subpixel>>;

/// Pixels which have a named Black value.
pub trait HasBlack {
    fn black() -> Self;
}

/// Pixels which have a named White value.
pub trait HasWhite {
    fn white() -> Self;
}

macro_rules! impl_black_white {
    ($for_:ty, $min:expr, $max:expr) => (
        impl HasBlack for $for_ {
            fn black() -> Self {
                $min
            }
        }

        impl HasWhite for $for_ {
            fn white() -> Self {
                $max
            }
        }
    )
}

impl_black_white!(Luma<u8>, Luma([u8::MIN]), Luma([u8::MAX]));
impl_black_white!(Luma<u16>, Luma([u16::MIN]), Luma([u16::MAX]));
impl_black_white!(Rgb<u8>, Rgb([u8::MIN; 3]), Rgb([u8::MAX; 3]));

/// Something with a 2d position.
pub trait Position {
    fn x(&self) -> u32;
    fn y(&self) -> u32;
}

/// Something with a score.
pub trait Score {
    fn score(&self) -> f32;
}

/// A type to which we can clamp a value of type T.
/// Implementations are not required to handle NaNs gracefully.
pub trait Clamp<T> {
    fn clamp(x: T) -> Self;
}

/// Creates an implementation of Clamp<From> for type To.
macro_rules! implement_clamp {
    ($from:ty, $to:ty, $min:expr, $max:expr, $min_from:expr, $max_from:expr) => (
        impl Clamp<$from> for $to {
            fn clamp(x: $from) -> $to {
                if x < $max_from as $from {
                    if x > $min_from as $from {
                        x as $to
                    } else {
                        $min
                    }
                } else {
                    $max
                }
            }
        }
    )
}

implement_clamp!(f32, u8, u8::MIN, u8::MAX, u8::MIN as f32, u8::MAX as f32);
implement_clamp!(f32, u16, u16::MIN, u16::MAX, u16::MIN as f32, u16::MAX as f32);
implement_clamp!(f64, u8, u8::MIN, u8::MAX, u8::MIN as f64, u8::MAX as f64);
implement_clamp!(f64, u16, u16::MIN, u16::MAX, u16::MIN as f64, u16::MAX as f64);
implement_clamp!(i32, u8, u8::MIN, u8::MAX, u8::MIN as i32, u8::MAX as i32);
implement_clamp!(u32, u8, u8::MIN, u8::MAX, u8::MIN as u32, u8::MAX as u32);
implement_clamp!(i32, u16, u16::MIN, u16::MAX, u16::MIN as i32, u16::MAX as i32);
implement_clamp!(i32, i16, i16::MIN, i16::MAX, i16::MIN as i32, i16::MAX as i32);
implement_clamp!(u16, u8, u8::MIN, u8::MAX, u8::MIN as u16, u8::MAX as u16);

#[cfg(test)]
mod test {

    use super::Clamp;

    #[test]
    fn test_clamp_f32_u8() {
        let t: u8 = Clamp::clamp(255f32);
        assert_eq!(t, 255u8);
        let u: u8 = Clamp::clamp(300f32);
        assert_eq!(u, 255u8);
        let v: u8 = Clamp::clamp(0f32);
        assert_eq!(v, 0u8);
        let w: u8 = Clamp::clamp(-5f32);
        assert_eq!(w, 0u8);
    }

    #[test]
    fn test_clamp_f32_u16() {
        let t: u16 = Clamp::clamp(65535f32);
        assert_eq!(t, 65535u16);
        let u: u16 = Clamp::clamp(300000f32);
        assert_eq!(u, 65535u16);
        let v: u16 = Clamp::clamp(0f32);
        assert_eq!(v, 0u16);
        let w: u16 = Clamp::clamp(-5f32);
        assert_eq!(w, 0u16);
    }
}
