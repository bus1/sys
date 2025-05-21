// Common Linux Definitions
//
// This module provides a common implementation of the vast majority of Linux
// system interfaces. It is meant to be used by the platform-modules via:
//
// ```rust,ignore
// #[path = "../common/mod.rs"]
// mod common;
// pub use common::*;
// ```
//
// This module references the including module via `super::xyz`, and requires
// the following definitions to be provided:
//
//  * `super::abi`: This must be an ABI module with the same symbols as
//    defined by `osi::ffi::abi`.
//
// Sub-modules of this module never reference the including module directly,
// but only ever use the symbols exported here. Hence, this module re-exports
// all required symbols for internal use only.

use super::abi as abi;

pub mod errno;
