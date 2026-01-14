//! # Capability-based Standard Interfaces
//!
//! This library provides _**O**perating **S**ystem **I**ndependent_ standard
//! interfaces following a **capability-based design**. It does not require any
//! particular runtime, but can optionally be combined with the Rust Standard
//! Library.

// MSRV(unknown): This is a wish-list for Rust features that either have no
//     clear path to stabilization, or just do not exist. Only features
//     relevant to the development of this project are listed.
//
// - rustdoc self-type: Jumping to the documentation of a method in rustdoc
//     html renderings does not show the self-type. It is often cumbersome to
//     scroll up to the exact place where the `impl` is shown. It would be nice
//     if the self-type was shown, or there was another way to quickly jump to
//     it via an anchor.
//
// - rustfmt optout: There is no global opt-out for rustfmt. While individual
//     items can be annotated with `#[rustfmt::skip]`, the root module of a
//     crate cannot be annotated like this (NB: inner attributes like
//     `#![rustfmt::skip]` are not stable).
//     While projects can decide to not run `rustfmt`, it would be nice to
//     annotate the code-base so IDEs will also not format the code
//     automatically.
//
// - maybe-owned types: When objects borrow data, the data cannot live in the
//     same object, since Rust does not allow self-referential types. A common
//     workaround is to make such objects own the data instead, but this is
//     sub-optimal and leads to unneeded copies. It would be nice to have some
//     official support to represent data that can be either owned or borrowed,
//     similar to `core::borrow::Cow`, but without the requirement for `Clone`.
//
// - memchr(), memmem(): While strings have two-way search support in the Rust
//     standard library, no such features are exposed for searching u8. Given
//     that these can be greatly optimized by the compiler, they seem a worthy
//     fit for the standard library.

#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

// Used by macros via `$crate::{alloc,core}::*`, explicitly part of the public
// API. Usually of little use to code outside of this crate, though.
pub extern crate alloc;
pub extern crate core;

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
pub mod mown;
pub mod never;
pub mod pin;
pub mod ptr;
pub mod str;
