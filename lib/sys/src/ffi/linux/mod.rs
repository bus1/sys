//! # Definitions of Linux System Interfaces
//!
//! This module exposes the raw definitions of the Linux kernel system
//! interfaces, including interface constants, structures, types, and more.
//! It allows direct access to Linux system interfaces bypassing any high-level
//! abstraction. The module does not strive for completeness, but any
//! interfaces that are part of the Linux kernel `uapi` are in-scope.
//!
//! ## Platforms
//!
//! Since Linux system interfaces differ depending on the target platform, the
//! interfaces are provided once for each supported platform. A separate
//! sub-module exists for each platform, but they all expose the same APIs (but
//! with differences specific to the architecture).
//!
//! When fixed access to a specific platform is required, the modules can be
//! used directly (e.g., reading core-dumps of a specific platform, independent
//! of the host platform). But if access to the target platform of the
//! compilation is needed, then one of the following aliases is recommended:
//!
//! - `native`: Exposes the system interfaces using the native
//!   compiler primitives for the target platform. That is, integers are
//!   represented with the builtin types `u32`, `i64`, ... They use the
//!   native endianness and alignment. This module is **not** an alias, but
//!   compiled as a unique, separate platform.
//! - `libc`: Exposes the system interfaces using the types of the `libc`
//!   crate. Only available under the `libc` feature. Like `native`, this is
//!   compiled as a unique, separate platform.
//! - `target`: Alias for the platform module matching the target platform. If
//!   no such platform module exists, the alias is not available.

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
//    Note that this mandates `./<platform>/mod.rs` rather than
//    `./<platform>.rs`, to ensure the platform directory exists and our usage
//    of `path = "../[...]"` works.

#![allow(clippy::duplicate_mod)]

#[cfg(test)]
mod test;

pub mod libc;

macro_rules! impl_platform {
    (
        $platform:ident,
        $cfg:meta,
        $path:meta,
        $abi:path
        $(,)?
    ) => {
        #[doc = concat!("# Platform Module for ", stringify!($platform))]
        ///
        /// This module exposes all supported interfaces of
        /// [`crate::ffi::linux`] for the
        #[doc = concat!(stringify!($platform), " platform.")]
        pub mod $platform {
            pub use $abi as abi;

            #[$path]
            mod inner;

            pub use inner::*;
        }

        // # Platform Module for the target platform
        //
        // This module is a straight alias of the platform module that matches
        // the compilation target. If no platform module exists for the
        // compilation target, this alias is not provided.
        #[$cfg]
        pub use $platform as target;

        /// # Pseudo-Module for the Native Platform
        ///
        /// This module behaves like an alias of the platform module that
        /// matches the compilation target. However, it is not a straight alias
        /// but a recompilation of the platform module with the Rust primitives
        /// as ABI.
        ///
        /// If no platform module matches the compilation target, this will use
        /// the `libc` module (if enabled). If the latter is not enabled, this
        /// module will not exist.
        #[$cfg]
        pub mod native {
            pub use osi::ffi::abi::native as abi;

            #[$path]
            mod inner;

            #[allow(unused)]
            pub use inner::*;
        }
    };
}

impl_platform!(
    aarch64,
    cfg(target_arch = "aarch64"),
    path = "../aarch64/mod.rs",
    osi::ffi::abi::aarch64_sysv,
);

impl_platform!(
    x86,
    cfg(target_arch = "x86"),
    path = "../x86/mod.rs",
    osi::ffi::abi::x86_sysv,
);

impl_platform!(
    x86_64,
    cfg(target_arch = "x86_64"),
    path = "../x86_64/mod.rs",
    osi::ffi::abi::x86_64_sysv,
);
