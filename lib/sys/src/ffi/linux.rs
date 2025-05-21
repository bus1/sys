//! # Definitions of Linux System Interfaces
//!
//! For all high-level abstractions of Linux system interfaces that are
//! provided by this crate, the respective raw definitions of Linux kernel
//! interface constants, structures, types, and more are provided in this
//! module. This allows use of these definitions independent of higher
//! abstractions.
//!
//! This module exposes a set of platform-modules, which are all syntactically
//! equivalent, but define the interfaces for different platforms. Several
//! pseudo-modules are provided, which either alias another platform-module or
//! provide a virtual platform based on another one. In most cases, you want
//! to use the [`native`] platform-module to get access to all the interfaces
//! for the platform of the compilation target.
//!
//! ## Completeness
//!
//! This module does not claim complete coverage of all Linux kernel
//! interfaces. However, all Linux kernel interfaces are in-scope of this
//! module (usually this means the interface is defined in the `uapi` headers
//! of the linux kernel). Feel free to add more interfaces if needed.

// We provide FFI definitions for all supported platforms simultaneously. This
// is especially useful for debugging utilities which need to access foreign
// platform data, and thus the foreign ABI will be different to the native one.
// This, however, requires manually defining all the interfaces for all
// platforms. To aid in this, we use a few tricks:
//
//  - Each platform exposes a sub-module `abi`, which defines the basic
//    data-types of the platform. This is taken from [`osi::ffi::abi`] and
//    simply re-exported.
//  - Common definitions are shared in the [`common`] sub-module. This
//    sub-module is not exposed by itself, but has to be included manually by
//    each platform. The module makes use of other definitions of the platform
//    via `use super::XYZ`. See `common/mod.rs` for details on which
//    definitions it requires. This is also the reason why the module is
//    recompiled for each platform, since its implementation depends on other
//    platform details.
//  - The `libc` platform is available if the `libc` feature is selected. It is
//    not a real platform, but rather defines the same API as the other
//    platforms but via aliases to the types of `libc`. It can be used for
//    platforms that are not natively supported by this crate, yet. While the
//    FFI definitions do not require the C library to be linked, dependent
//    users will likely pull in the C library if the `libc` feature is used.
//  - The `native` platform is also not a real platform. Instead it is an
//    alias for one of the other platforms but using the native Rust primitives
//    via [`osi::ffi::abi::native`]. If a platform is not supported, this will
//    even use the `libc` platform.
//  - Ideally, we would provide FFI definitions via Rust traits, with the
//    platform ABI as type generic. Unfortunately, this would severly limit the
//    usability due to rustc restrictions (e.g., no inherently const methods in
//    traits, no associated types for inherent impls, ...). Furthermore, since
//    there are no module-generics in Rust, we instead recompile the FFI
//    definitions for each platform, sharing as much as possible.
//    Note that this is why we use `./<platform>/mod.rs` rather than
//    `./<platform>.rs`, to ensure the platform directory exists and our usage
//    of `path = "../[...]"` works.

#[cfg(test)]
mod test;

pub mod libc;
pub mod native;

/// # Platform Module for x86
///
/// This module exposes all supported interfaces of [`crate::ffi::linux`] for
/// the x86 platform.
pub mod x86 {
    pub use osi::ffi::abi::x86_sysv as abi;

    #[path = "mod.rs"]
    mod inner;

    pub use inner::*;
}

/// # Platform Module for x86_64
///
/// This module exposes all supported interfaces of [`crate::ffi::linux`] for
/// the x86_64 platform.
pub mod x86_64 {
    pub use osi::ffi::abi::x86_64_sysv as abi;

    #[path = "mod.rs"]
    mod inner;

    pub use inner::*;
}

osi::cfg::cond! {
    (doc) {
        /// # Pseudo-Module for the Target Platform
        ///
        /// This module is a straight alias of the platform-module that matches
        /// the compilation target. If no platform-module exists for the
        /// compilation target, this will be an alias of `native`.
        ///
        /// For documentation purposes, this is always an alias of `native`.
        pub mod target {
            #[doc(inline)]
            pub use super::native::*;
        }
    },
    (target_arch = "x86") {
        pub use x86 as target;
    },
    (target_arch = "x86_64") {
        pub use x86_64 as target;
    },
    {
        pub use native as target;
    },
}
