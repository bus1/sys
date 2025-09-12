//! # Capability-based Standard Interfaces
//!
//! This library provides _**O**perating **S**ystem **I**ndependent_ standard
//! interfaces following a **capability-based design**. It does not require any
//! particular runtime, but can optionally be combined with the Rust Standard
//! Library.

#![allow(clippy::identity_op)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::redundant_field_names)]

#![no_std]

extern crate alloc;
extern crate core;

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod align;
pub mod args;
pub mod brand;
pub mod cfg;
pub mod cmp;
pub mod compat;
pub mod convert;
pub mod error;
pub mod ffi;
pub mod hash;
pub mod hmac;
pub mod json;
pub mod marker;
pub mod mem;
pub mod meta;
pub mod never;
pub mod pin;
pub mod ptr;
pub mod str;
