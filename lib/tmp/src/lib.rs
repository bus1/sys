//! # Capability-based Preview Interfaces
//!
//! This library contains unstable preview interfaces of the [`osi`] and
//! [`sys`] crates.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::identity_op)]
#![allow(clippy::inherent_to_string_shadow_display)]
#![allow(clippy::len_zero)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::new_without_default)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::redundant_field_names)]

#![no_std]

extern crate alloc;
extern crate core;

#[cfg(test)]
extern crate std;

pub mod fmt;
pub mod io;
pub mod msdosmz;
pub mod pecoff;
