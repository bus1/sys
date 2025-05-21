//! # Capability-based System Interfaces
//!
//! This library provides _**Sys**tem Interfaces_ following a
//! **capability-based design**. It does not require any particular runtime,
//! but can optionally be combined with the Rust Standard Library.

#![no_std]

extern crate alloc;
extern crate core;

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod ffi;
