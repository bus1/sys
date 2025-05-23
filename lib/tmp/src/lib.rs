//! # Capability-based Preview Interfaces
//!
//! This library contains unstable preview interfaces of the [`osi`] and
//! [`sys`] crates.

#![no_std]

extern crate alloc;
extern crate core;

#[cfg(test)]
extern crate std;

pub mod msdosmz;
pub mod pecoff;
