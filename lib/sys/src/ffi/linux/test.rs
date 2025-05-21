//! # Tests for the Linux FFI Definitions
//!
//! This module contains tests for all exported FFI definitions of the
//! `ffi::linux` module.

use super::*;

// If `libc` is not enabled, just alias it from `native` so the test
// can just use `libc` unconditionally.
#[cfg(not(feature = "libc"))]
use native as libc;

// Compare two `const` definitions for equality. This will compare their type
// layout and memory content for equality.
fn eq_def_const<A, B>(a: &A, b: &B) -> bool {
    core::mem::size_of::<A>() == core::mem::size_of::<B>()
    && core::mem::align_of::<A>() == core::mem::align_of::<B>()
    && osi::mem::eq(a, b)
}

// A 3-way variant of `eq_def_const()`.
fn eq3_def_const<A, B, C>(a: &A, b: &B, c: &C) -> bool {
    eq_def_const(a, b) && eq_def_const(a, c)
}

// Verify that all supported platforms are available, by simply checking that
// they expose `abi::U16`.
#[test]
fn platform_availability() {
    assert_eq!(core::mem::size_of::<x86::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<x86_64::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<target::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<native::abi::U16>(), 2);
    assert_eq!(core::mem::size_of::<libc::abi::U16>(), 2);
}

// Compare target APIs with native and libc APIs, and verify they match.
#[test]
fn comparison() {
    assert!(eq3_def_const(&target::errno::EPERM, &native::errno::EPERM, &libc::errno::EPERM));
}
