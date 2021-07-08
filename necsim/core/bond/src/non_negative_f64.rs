use core::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul},
};

use serde::{Deserialize, Serialize};

use crate::{ClosedUnitF64, PositiveF64};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct NonNegativeF64Error(f64);

impl fmt::Display for NonNegativeF64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is negative.", self.0)
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", rustacuda(core = "rust_cuda::rustacuda_core"))]
#[cfg_attr(feature = "mpi", derive(mpi::traits::Equivalence))]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct NonNegativeF64(f64);

impl TryFrom<f64> for NonNegativeF64 {
    type Error = NonNegativeF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Debug for NonNegativeF64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct NonNegativeF64Range(f64);

        impl fmt::Debug for NonNegativeF64Range {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "0.0 <= {}", self.0)
            }
        }

        fmt.debug_tuple("NonNegativeF64")
            .field(&NonNegativeF64Range(self.0))
            .finish()
    }
}

impl NonNegativeF64 {
    /// # Errors
    ///
    /// Returns `NonNegativeF64Error` if not `0.0 <= value`
    pub fn new(value: f64) -> Result<Self, NonNegativeF64Error> {
        if value >= 0.0 {
            Ok(Self(value))
        } else {
            Err(NonNegativeF64Error(value))
        }
    }

    /// # Safety
    ///
    /// Only safe iff `0.0 <= value`
    #[must_use]
    pub unsafe fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn zero() -> Self {
        Self(0.0_f64)
    }

    #[must_use]
    pub fn one() -> Self {
        Self(1.0_f64)
    }

    #[must_use]
    pub fn infinity() -> Self {
        Self(f64::INFINITY)
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

impl From<u32> for NonNegativeF64 {
    fn from(value: u32) -> Self {
        Self(f64::from(value))
    }
}

impl From<u64> for NonNegativeF64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: u64) -> Self {
        Self(value as f64)
    }
}

impl From<usize> for NonNegativeF64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}

impl From<PositiveF64> for NonNegativeF64 {
    fn from(value: PositiveF64) -> Self {
        Self(value.get())
    }
}

impl From<ClosedUnitF64> for NonNegativeF64 {
    fn from(value: ClosedUnitF64) -> Self {
        Self(value.get())
    }
}

impl PartialEq for NonNegativeF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for NonNegativeF64 {}

impl PartialOrd for NonNegativeF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for NonNegativeF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for NonNegativeF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialEq<PositiveF64> for NonNegativeF64 {
    fn eq(&self, other: &PositiveF64) -> bool {
        self.0.eq(&other.get())
    }
}

impl PartialOrd<PositiveF64> for NonNegativeF64 {
    fn partial_cmp(&self, other: &PositiveF64) -> Option<Ordering> {
        self.0.partial_cmp(&other.get())
    }
}

impl PartialEq<f64> for NonNegativeF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<f64> for NonNegativeF64 {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Add for NonNegativeF64 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for NonNegativeF64 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Mul for NonNegativeF64 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl Div for NonNegativeF64 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl Mul<PositiveF64> for NonNegativeF64 {
    type Output = Self;

    fn mul(self, other: PositiveF64) -> Self {
        Self(self.0 * other.get())
    }
}

impl Add<ClosedUnitF64> for NonNegativeF64 {
    type Output = Self;

    fn add(self, other: ClosedUnitF64) -> Self {
        Self(self.0 + other.get())
    }
}

impl Mul<ClosedUnitF64> for NonNegativeF64 {
    type Output = Self;

    fn mul(self, other: ClosedUnitF64) -> Self {
        Self(self.0 * other.get())
    }
}

impl Sum for NonNegativeF64 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(NonNegativeF64::zero(), |a, b| a + b)
    }
}
