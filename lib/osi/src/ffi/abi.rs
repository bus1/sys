//! Common ABIs
//!
//! This module provides type definitions for a set of platform ABIs. This can
//! be used to introspect or synthesize objects of foreign platform ABIs.
//!
//! The individual sub-modules represent known ABIs of different platforms.
//! Each module exports the same set of symbols. Preferably, this would be
//! represented by a trait, which is implemented by each ABI. Unfortunately,
//! Rust traits are too limited right now to be suitable here (most
//! importantly, they do not allow constant methods). Hence, we instead export
//! a set of modules.
//!
//! [`native`] exports a special ABI which always represents the ABI of the
//! target platform and uses the Rust native data-types (i.e., it uses the
//! builtin primitive integers like `u16`, `i64`, and `usize`). Use this ABI
//! to get a native Rust experience. This is suitable if foreign data access
//! is not needed.
//!
//! [`auto`] is an alias of one of the other ABIs and represents the target
//! platform. Unlike [`native`], this does not necessarily use native Rust
//! data-types, but is a real alias to one of the other fixed definitions of
//! platform ABIs.
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
//! alignment of 8. Instead, [`crate::ffi::Integer`] can be used to model the
//! exact ABI of the foreign system.

// Little-endian integer with the given native type and alignment.
type Le<Native, Alignment> = crate::ffi::Integer<
    crate::ffi::LittleEndian<Native>,
    Alignment,
>;

// This module is imported by all ABIs and provides default symbols valid on
// all targets.
mod shared {
    /// Creates a number by converting the input value from native
    /// representation into the representation of the target type.
    ///
    /// This is the preferred method to initialize foreign ordered datatypes
    /// with a logical value. This method will take care of endian conversion,
    /// such that a machine of the foreign platform would read the same logical
    /// value.
    ///
    /// If this method is used to initialize non-foreign (native) datatypes,
    /// it will be an identity function and return the input unchanged.
    ///
    /// This works with any type that implements [`crate::ffi::NativeEndian`].
    pub const fn num<Endian: crate::ffi::NativeEndian<Raw>, Raw: Copy>(r: Raw) -> Endian {
        crate::ffi::from_native(r)
    }
}

/// # Native ABI
///
/// The native ABI uses the native primitive types of Rust, and thus represents
/// the ABI of the compilation target platform. This is the preferred ABI to
/// use when interfacing with native platform APIs, rather than foreign
/// platform APIs.
///
/// The types of this ABI directly alias Rust primitive types like `u16` or
/// `i64`. Hence, these should integrate nicely into native Rust code bases and
/// no special handling is needed.
pub mod native {
    pub type I8 = i8;
    pub type I16 = i16;
    pub type I32 = i32;
    pub type I64 = i64;
    pub type I128 = i128;
    pub type Isize = isize;

    pub type U8 = u8;
    pub type U16 = u16;
    pub type U32 = u32;
    pub type U64 = u64;
    pub type U128 = u128;
    pub type Usize = usize;

    pub type F32 = f32;
    pub type F64 = f64;

    pub type Addr = core::num::NonZeroUsize;
    pub type Ptr<Target> = core::ptr::NonNull<Target>;

    pub use super::shared::*;
}

/// # System-V x86 ABI
///
/// This ABI represents the 32-bit ABI of System-V for x86 systems. It is used
/// by most UNIX compatible systems, including Linux.
pub mod x86_sysv {
    use crate::align;

    pub type I8 = super::Le<i8, align::AlignAs<1>>;
    pub type I16 = super::Le<i16, align::AlignAs<2>>;
    pub type I32 = super::Le<i32, align::AlignAs<4>>;
    pub type I64 = super::Le<i64, align::AlignAs<4>>;
    pub type I128 = super::Le<i128, align::AlignAs<4>>;
    pub type Isize = super::Le<i32, align::AlignAs<4>>;

    pub type U8 = super::Le<u8, align::AlignAs<1>>;
    pub type U16 = super::Le<u16, align::AlignAs<2>>;
    pub type U32 = super::Le<u32, align::AlignAs<4>>;
    pub type U64 = super::Le<u64, align::AlignAs<4>>;
    pub type U128 = super::Le<u128, align::AlignAs<4>>;
    pub type Usize = super::Le<u32, align::AlignAs<4>>;

    pub type F32 = super::Le<f32, align::AlignAs<4>>;
    pub type F64 = super::Le<f64, align::AlignAs<4>>;

    pub type Addr = super::Le<core::num::NonZeroU32, align::AlignAs<4>>;
    pub type Ptr<Target> = crate::ffi::Pointer<Addr, Target>;

    pub use super::shared::*;
}

/// # System-V x86-64 ABI
///
/// This ABI represents the 64-bit ABI of System-V for x86 systems. It is used
/// by most UNIX compatible systems, including Linux.
pub mod x86_64_sysv {
    use crate::align;

    pub type I8 = super::Le<i8, align::AlignAs<1>>;
    pub type I16 = super::Le<i16, align::AlignAs<2>>;
    pub type I32 = super::Le<i32, align::AlignAs<4>>;
    pub type I64 = super::Le<i64, align::AlignAs<8>>;
    pub type I128 = super::Le<i128, align::AlignAs<16>>;
    pub type Isize = super::Le<i64, align::AlignAs<8>>;

    pub type U8 = super::Le<u8, align::AlignAs<1>>;
    pub type U16 = super::Le<u16, align::AlignAs<2>>;
    pub type U32 = super::Le<u32, align::AlignAs<4>>;
    pub type U64 = super::Le<u64, align::AlignAs<8>>;
    pub type U128 = super::Le<u128, align::AlignAs<16>>;
    pub type Usize = super::Le<u64, align::AlignAs<8>>;

    pub type F32 = super::Le<f32, align::AlignAs<4>>;
    pub type F64 = super::Le<f64, align::AlignAs<8>>;

    pub type Addr = super::Le<core::num::NonZeroU64, align::AlignAs<8>>;
    pub type Ptr<Target> = crate::ffi::Pointer<Addr, Target>;

    pub use super::shared::*;
}

#[cfg(all(
    target_arch = "x86",
    target_family = "unix",
))]
pub use x86_sysv as auto;

#[cfg(all(
    target_arch = "x86_64",
    target_family = "unix",
))]
pub use x86_64_sysv as auto;

#[cfg(all(
    target_arch = "x86",
    target_env = "msvc",
    target_family = "windows",
))]
pub use x86_win as auto;

#[cfg(all(
    target_arch = "x86_64",
    target_env = "msvc",
    target_family = "windows",
))]
pub use x86_64_win as auto;
