//! A deliberately small prime field for readable protocol experiments.
use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

/// The prime modulus used throughout the lab.
pub const MODULUS: u64 = 97;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Fp(pub u64);

impl Fp {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub const fn new(value: u64) -> Self {
        Self(value % MODULUS)
    }
    pub fn pow(self, mut exponent: u64) -> Self {
        let mut base = self;
        let mut result = Self::ONE;
        while exponent > 0 {
            if exponent & 1 == 1 {
                result *= base;
            }
            base *= base;
            exponent >>= 1;
        }
        result
    }
    pub fn inverse(self) -> Option<Self> {
        (self != Self::ZERO).then(|| self.pow(MODULUS - 2))
    }
    pub fn value(self) -> u64 {
        self.0
    }
}

impl From<i32> for Fp {
    fn from(value: i32) -> Self {
        Self::new(value.rem_euclid(MODULUS as i32) as u64)
    }
}
impl From<u64> for Fp {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
impl From<usize> for Fp {
    fn from(value: usize) -> Self {
        Self::new(value as u64)
    }
}
impl Add for Fp {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.0 + rhs.0)
    }
}
impl AddAssign for Fp {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl Sub for Fp {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(MODULUS + self.0 - rhs.0)
    }
}
impl SubAssign for Fp {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl Mul for Fp {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.0 * rhs.0)
    }
}
impl MulAssign for Fp {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
#[allow(clippy::suspicious_arithmetic_impl)]
impl Div for Fp {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self::new(self.0 * rhs.inverse().expect("division by zero").0)
    }
}
impl Neg for Fp {
    type Output = Self;
    fn neg(self) -> Self {
        Self::ZERO - self
    }
}
impl fmt::Debug for Fp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl fmt::Display for Fp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl core::iter::Sum for Fp {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}
impl core::iter::Product for Fp {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, Mul::mul)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arithmetic_wraps() {
        assert_eq!(Fp::from(96) + Fp::from(3), Fp::from(2));
    }
    #[test]
    fn inverses() {
        for i in 1..MODULUS {
            let x = Fp::from(i);
            assert_eq!(x * x.inverse().unwrap(), Fp::ONE);
        }
    }
}
