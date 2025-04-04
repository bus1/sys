//! # Capability-based Standard Interfaces
//!
//! This library provides operating system independent standard interfaces
//! following a capability-based design. It does not require any particular
//! runtime, but can optionally be combined with the Rust Standard Library.

#![no_std]

extern crate alloc;
extern crate core;

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod align;
pub mod args;
pub mod compat;
pub mod error;
pub mod ffi;
pub mod hash;
pub mod hmac;
pub mod json;
pub mod str;
