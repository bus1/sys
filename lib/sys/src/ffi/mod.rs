//! # Definitions of System Interfaces
//!
//! For all system interfaces the respective raw definitions of constants,
//! structures, types, and more are provided in this module. This allows use of
//! these definitions outside of possible higher abstractions.
//!
//! The definitions are transposed into Rust following a set of rules and
//! guidelines, thus yielding predictable type names and definitions. The idea
//! is to produce the same predictable result, as if a tool like `bindgen`
//! was used.
//!
//! This module only provides the definitions of the system interfaces, but no
//! implementation. This is left to other modules (or the user).
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
//!  * `no incomplete types`: Several structures use incomplete structure types
//!    by using an unbound array as last member. While rust can easily
//!    represent those within its type-system, such structures become DSTs,
//!    hence even raw pointers to them become fat-pointers, and would thus
//!    violate the UEFI ABI.
//!
//!    Instead, we use const-generics to allow compile-time adjustment of the
//!    variable-sized structures, with a default value of 0. This allows
//!    computing different sizes of the structures without any runtime
//!    overhead.
//!
//! ## Foreign Architectures
//!
//! Generally, definitions are provided in an architecture independent format.
//! This means, the structures and definitions will be the same regardless of
//! the compilation target platform. This allows accessing definitions of
//! foreign architectures without any extra care (e.g., inspecting core dumps
//! of another machine). Unfortunately, this makes most primitive types
//! unsuitable for those definitions. Therefore, some aliases are provided, if
//! possible.
//!
//! If an FFI module is specific to an architecture, it is provided once for
//! each architecture, using the ABI of the respective architecture.
//! Additionally, is is also provided as a module called _"native"_, using the
//! compiler primitives. This _"native"_ module is only provided if the FFI
//! module is available for the target platform.
//!
//! For instance, `ffi::linux` exposes the linux definitions for each
//! architecture separately (e.g., `ffi::linux::x86`), but provides
//! `ffi::linux::native` as an alias using native compiler primitives.
//!
//! If an FFI module is otherwise specific to a given property, a similar style
//! is followed. However, if the number of options is fixed, it might be used
//! as suffix instead (e.g., `elf32`, `elf64`, `elfn`, where the latter is the
//! equivalent of _"native"_).

pub mod elf;
pub mod linux;
