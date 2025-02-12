#![deny(clippy::pedantic)]
#![no_std]
#![feature(rustc_attrs)]
#![feature(total_cmp)]

mod closed_unit_f64;
mod non_negative_f64;
mod non_zero_one_u64;
mod partition;
mod positive_f64;
mod positive_unit_f64;

pub use closed_unit_f64::ClosedUnitF64;
pub use non_negative_f64::NonNegativeF64;
pub use non_zero_one_u64::NonZeroOneU64;
pub use partition::Partition;
pub use positive_f64::PositiveF64;
pub use positive_unit_f64::PositiveUnitF64;
