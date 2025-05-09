//! # Foreign Function Interfaces
//!
//! This module is a collection of utilities that aid implementation of foreign
//! function interfaces in Rust.

pub mod abi;
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
