//! # Tests for the Linux FFI Definitions
//!
//! This module contains tests for all exported FFI definitions of the
//! `ffi::linux` module.

mod target;

use crate::ffi::linux::*;

// Verify that all supported platforms are available, by simply checking that
// they expose `abi::U16`.
#[test]
fn platform_availability() {
    assert_eq!(core::mem::size_of::<aarch64::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<x86::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<x86_64::abi::U16>(), 2);

    #[cfg(feature = "libc")]
    assert_eq!(core::mem::size_of::<libc::abi::U16>(), 2);
}
