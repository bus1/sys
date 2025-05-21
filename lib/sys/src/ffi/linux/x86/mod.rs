// Platform Module for x86
//
// This module is included multiple times by `../../linux.rs`, using outer
// comments for documentation. It is also the responsibility of the caller
// to define the ABI to use. This module simply re-uses it via
// `use super::abi`.

use super::abi;

#[path = "../common/mod.rs"]
mod common;

pub use common::*;
