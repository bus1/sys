//! # Pseudo-Module based on libc
//!
//! This module exposes all supported interfaces of
//! [`ffi::linux`](crate::ffi::linux) for the native platform using definitions
//! of `libc`. This module is only available if the `libc` feature is selected.
//!
//! Note that use of this module does **not** require linking to the C library.
//! Only a compile-time dependency to the Rust wrappers of the C library
//! (i.e., the [`libc`] crate) is needed.
//!
//! This module does not provide any documentation for its exposed symbols.
//! Documentation for all exposed symbols is available in the
//! [`native`](crate::ffi::linux::native) module (or any of the platform
//! modules).

#![cfg(feature = "libc")]

pub use osi::ffi::abi::native as abi;

pub mod errno;
