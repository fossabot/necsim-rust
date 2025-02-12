#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate serde_derive_state;

mod arguments;

pub mod classical;
pub mod gillespie;
pub mod skipping_gillespie;
