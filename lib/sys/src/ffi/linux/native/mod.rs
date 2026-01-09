//! # Pseudo-Module for the Native Platform
//!
//! This module behaves like an alias of the platform-module that matches the
//! compilation target. However, it is not a straight alias but a recompilation
//! of the platform-module with the Rust primitives as ABI.
//!
//! If no platform-module matches the compilation target, this will use the
//! `libc` module (if enabled). If the latter is not enabled, this module will
//! be empty.

pub use osi::ffi::abi::native as abi;

osi::cfg::cond! {
    (doc) {
        #[path = "../x86_64/mod.rs"]
        mod inner;
    },
    (target_arch = "x86") {
        #[path = "../x86/mod.rs"]
        mod inner;
    },
    (target_arch = "x86_64") {
        #[path = "../x86_64/mod.rs"]
        mod inner;
    },
    (feature = "libc") {
        use super::libc as inner;
    },
    {
        mod inner {}
    },
}

#[allow(unused)]
pub use inner::*;
