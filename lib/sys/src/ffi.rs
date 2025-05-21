//! # Definitions of System Interfaces
//!
//! For all system interfaces the respective raw definitions of constants,
//! structures, types, and more are provided in this module. This allows use of
//! these definitions outside of possible higher abstractions.
//!
//! The definitions are transposed into Rust following a set of rules and
//! guidelines, thus yielding predictable type names and definitions. The idea
//! is to produce the same predictable as result, as if a tool like `bindgen`
//! was used.
//!
//! This module only provides the definitions of the system interfaces, but no
//! implementation. This is left to other modules (or the user).
//!
//! Unless explicitly specified, the definitions are provided in an
//! architecture independent format. They are suitable for access of foreign
//! system architectures, as is common for introspection or debugging.
//!
//! ## Transpose Rules
//!
//! While this module attempts to be a direct mapping to the respective
//! protocols and specifications, slight adjustments are usually necessary to
//! account for the peculiarities of Rust:
//!
//!  * All names follow the standard Rust naming scheme, using `CamelCase` for
//!    types, `UPPER_CASE` for constants, and `snake_case` for everything else.
//!
//!  * Prefixes are stripped if the Rust module or type-system provides a
//!    suitable prefix.
//!
//!  * C-enums are always provided as raw integer type, rather than Rust enum
//!    to allow arbitrary discriminants to be used. This is particularly
//!    important when the interface allows for custom/vendor extensions, since
//!    then Rust enums would be unable to represent the unused ranges.
//!
//!  * Pointers are always represented as `NonNull` or `Option<NonNull>` and
//!    thus strip any `const` annotations. This is on purpose, since the
//!    classic C-const annotations cannot be transposed to Rust in a sensible
//!    way. For architecture-independent representations, see `osi::ffi`.
//!
//! ## Native Alias
//!
//! If suitable, a module will expose the types native to the compilation
//! target under a `native` alias (or with `*n` suffix). This allows easy
//! interaction with each module on the running system. However, it will
//! prevent any cross-architecture interaction, or interaction with non-native
//! actors.

pub mod linux;
