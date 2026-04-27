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
//! ## Documentation
//!
//! This module does not provide any documentation for its exposed symbols.
//! Documentation for all exposed symbols is available in the
//! [`native`](crate::ffi::linux::native) module (or any of the platform
//! modules).

#![cfg(feature = "libc")]

pub mod abi {
    pub type I8 = i8;
    pub type I16 = libc::__s16;
    pub type I32 = libc::__s32;
    pub type I64 = libc::__s64;
    pub type I128 = i128;
    pub type Isize = libc::ssize_t;

    pub type U8 = libc::__u8;
    pub type U16 = libc::__u16;
    pub type U32 = libc::__u32;
    pub type U64 = libc::__u64;
    pub type U128 = u128;
    pub type Usize = libc::size_t;

    pub type F32 = f32;
    pub type F64 = f64;

    pub type Addr = osi::ffi::abi::native::Addr;
    pub type Ptr<Target> = osi::ffi::abi::native::Ptr<Target>;

    pub use osi::ffi::abi::shared::*;
}

pub mod errno;
