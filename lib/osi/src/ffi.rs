//! # Foreign Function Interfaces
//!
//! This module is a collection of utilities that aid implementation of foreign
//! function interfaces in Rust.
//!
//! ## Foreign ABI
//!
//! When accessing foreign ABIs, care must be taken to ensure datatypes have
//! the correct layout. The builtin primitives like `u32`, `i64`, etc., always
//! follow the native ABI, and thus cannot be reliably used to represent
//! data-structures of foreign ABIs. The utilities in this module can be used
//! instead.
//!
//! As an example, imagine a 32-bit Linux process that visualizes core-dumps of
//! crashed processes. If that process needs to read core-dumps of a 64-bit
//! system, it likely cannot use `u64` to model the structures used in that
//! core-dump, since it will have an alignment of 4, rather than the original
//! alignment of 8. Instead, [`Integer`] can be used to model the exact ABI of
//! the foreign system.

pub mod endian;
pub mod integer;
pub mod packed;
pub mod pointer;

pub use endian::{
    BigEndian,
    from_native,
    from_raw,
    LittleEndian,
    NativeEndian,
    to_native,
    to_raw,
};
pub use integer::Integer;
pub use packed::Packed;
pub use pointer::{NativeAddress, Pointer};
