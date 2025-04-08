//! # Endianness Utilities
//!
//! This module provides utilities to safely deal with foreign-endian types.

/// A trait to convert to and from the native endianness to the endianness of a
/// specific type. If a type is already encoded in the native endianness, this
/// trait becomes an identity function for this type. For other types, it
/// converts from and to native endianness.
///
/// The trait-generic `Raw` defines the type of the native representation. It
/// must be suitable to represent native **and** foreign values. Furthermore,
/// the trait is designed for `Copy` types (in particular primitive integers).
/// Bigger or more complex types are not suitable.
///
/// This trait provides default implementations for all its methods, which can
/// also be accessed as static associated `const fn` functions. There is no
/// need to override the default implementations, except for performance
/// reasons.
///
/// ## Safety
///
/// An implementation must guarantee that it is safe to create memory copies
/// from `Raw` to create `Self` (and vice versa). If their size does not match,
/// memory is truncated, or padded with uninitialized bytes.
///
/// Furthermore, if [`Self::NEEDS_SWAP`] is [`true`], it must be valid to
/// reverse the order of all bytes in `Raw` to convert from, and to, the native
/// representation.
pub unsafe trait NativeEndian<Raw: Copy>: Copy {
    /// This marker shows whether the native encoding matches the encoding of
    /// the type ([`false`]), or whether a byte-swap is needed ([`true`]).
    const NEEDS_SWAP: bool = false;

    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    #[inline]
    #[must_use]
    fn from_raw(raw: Raw) -> Self {
        self::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    #[inline]
    #[must_use]
    fn to_raw(self) -> Raw {
        self::to_raw(self)
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    #[inline]
    #[must_use]
    fn from_native(native: Raw) -> Self {
        self::from_native(native)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    #[inline]
    #[must_use]
    fn to_native(self) -> Raw {
        self::to_native(self)
    }
}

/// A type to represent values encoded as big-endian. It is a simple
/// wrapping-structure with the same alignment and size requirements as the
/// type it wraps.
///
/// The `NativeEndian` trait is implemented for this type if `Raw` is a
/// primitive integer. Thus, conversion from and to native endianness is
/// provided, as well as default values, ordering, and other properties
/// reliant on the native value.
#[repr(transparent)]
pub struct BigEndian<Raw>(Raw);

/// A type to represent values encoded as little-endian. It is a simple
/// wrapping-structure with the same alignment and size requirements as the
/// type it wraps.
///
/// The `NativeEndian` trait is implemented for this type if `Raw` is a
/// primitive integer. Thus, conversion from and to native endianness is
/// provided, as well as default values, ordering, and other properties
/// reliant on the native value.
#[repr(transparent)]
pub struct LittleEndian<Raw>(Raw);

// Provide static implementations of the default methods in `NativeEndian`, so
// they can be accessed in `const fn`. This can be dropped once inherent const
// methods are allowed in traits.

/// Takes the raw, possibly foreign-ordered value `raw` and creates a
/// wrapping object that protects the value from unguarded access.
#[inline]
#[must_use]
pub const fn from_raw<Endian: NativeEndian<Raw>, Raw: Copy>(r: Raw) -> Endian {
    // SAFETY: The trait guarantees that `Endian` and `Raw` can be interchanged
    //         freely with truncated/uninitialized padding.
    unsafe { crate::mem::transmute_copy_uninit(&r) }
}

/// Returns the underlying raw, possibly foreign-ordered value behind this
/// wrapping object.
#[inline]
#[must_use]
pub const fn to_raw<Endian: NativeEndian<Raw>, Raw: Copy>(e: Endian) -> Raw {
    // SAFETY: The trait guarantees that `Endian` and `Raw` can be interchanged
    //         freely with truncated/uninitialized padding.
    unsafe { crate::mem::transmute_copy_uninit(&e) }
}

/// Creates the foreign-ordered value from a native value, converting the
/// value before retaining it, if required.
#[inline]
#[must_use]
pub const fn from_native<Endian: NativeEndian<Raw>, Raw: Copy>(r: Raw) -> Endian {
    if Endian::NEEDS_SWAP {
        // SAFETY: The trait guarantees that byte-swaps are allowed on the raw
        //         representation.
        unsafe { from_raw(crate::mem::bswap_copy(&r)) }
    } else {
        from_raw(r)
    }
}

/// Returns the native representation of the value behind this wrapping
/// object. The value is converted to the native representation before it
/// is returned, if required.
#[inline]
#[must_use]
pub const fn to_native<Endian: NativeEndian<Raw>, Raw: Copy>(e: Endian) -> Raw {
    if Endian::NEEDS_SWAP {
        // SAFETY: The trait guarantees that byte-swaps are allowed on the raw
        //         representation.
        unsafe { crate::mem::bswap_copy(&to_raw(e)) }
    } else {
        to_raw(e)
    }
}

unsafe impl NativeEndian<i8> for i8 { }
unsafe impl NativeEndian<i16> for i16 { }
unsafe impl NativeEndian<i32> for i32 { }
unsafe impl NativeEndian<i64> for i64 { }
unsafe impl NativeEndian<i128> for i128 { }
unsafe impl NativeEndian<isize> for isize { }
unsafe impl NativeEndian<u8> for u8 { }
unsafe impl NativeEndian<u16> for u16 { }
unsafe impl NativeEndian<u32> for u32 { }
unsafe impl NativeEndian<u64> for u64 { }
unsafe impl NativeEndian<u128> for u128 { }
unsafe impl NativeEndian<usize> for usize { }
unsafe impl NativeEndian<core::num::NonZeroI8> for core::num::NonZeroI8 { }
unsafe impl NativeEndian<core::num::NonZeroI16> for core::num::NonZeroI16 { }
unsafe impl NativeEndian<core::num::NonZeroI32> for core::num::NonZeroI32 { }
unsafe impl NativeEndian<core::num::NonZeroI64> for core::num::NonZeroI64 { }
unsafe impl NativeEndian<core::num::NonZeroI128> for core::num::NonZeroI128 { }
unsafe impl NativeEndian<core::num::NonZeroIsize> for core::num::NonZeroIsize { }
unsafe impl NativeEndian<core::num::NonZeroU8> for core::num::NonZeroU8 { }
unsafe impl NativeEndian<core::num::NonZeroU16> for core::num::NonZeroU16 { }
unsafe impl NativeEndian<core::num::NonZeroU32> for core::num::NonZeroU32 { }
unsafe impl NativeEndian<core::num::NonZeroU64> for core::num::NonZeroU64 { }
unsafe impl NativeEndian<core::num::NonZeroU128> for core::num::NonZeroU128 { }
unsafe impl NativeEndian<core::num::NonZeroUsize> for core::num::NonZeroUsize { }

impl<Raw> BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy,
{
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: Raw) -> Self {
        self::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    #[inline]
    #[must_use]
    pub fn to_raw(self) -> Raw {
        self::to_raw(self)
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    #[inline]
    #[must_use]
    pub fn from_native(native: Raw) -> Self {
        self::from_native(native)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    #[inline]
    #[must_use]
    pub fn to_native(self) -> Raw {
        self::to_native(self)
    }
}

// Implement clone via propagation.
impl<Raw: core::clone::Clone> core::clone::Clone for BigEndian<Raw> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement copy via propagation.
impl<Raw: Copy> core::marker::Copy for BigEndian<Raw> {
}

// For debugging simply print the raw values.
impl<Raw: core::fmt::Debug> core::fmt::Debug for BigEndian<Raw> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("BigEndian").field(&self.0).finish()
    }
}

// Choose defaults based on the native defaults.
impl<Raw> core::default::Default for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::default::Default,
{
    fn default() -> Self {
        Self::from_native(Default::default())
    }
}

// Convert to native for basic formatting.
impl<Raw> core::fmt::Display for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::fmt::Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        <Raw as core::fmt::Display>::fmt(&self.to_native(), fmt)
    }
}

// Compare based on native value.
impl<Raw> core::cmp::Eq for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::Eq,
{
}

// Hash based on native value.
impl<Raw> core::hash::Hash for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::hash::Hash,
{
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        self.to_native().hash(state)
    }
}

// Order based on native value.
impl<Raw> core::cmp::Ord for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_native().cmp(&other.to_native())
    }
}

// Compare based on native value.
impl<Raw> core::cmp::PartialEq for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.to_native().eq(&other.to_native())
    }
}

// Order based on native value.
impl<Raw> core::cmp::PartialOrd for BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_native().partial_cmp(&other.to_native())
    }
}

impl<Raw> LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy,
{
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    #[inline]
    #[must_use]
    pub fn from_raw(raw: Raw) -> Self {
        self::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    #[inline]
    #[must_use]
    pub fn to_raw(self) -> Raw {
        self::to_raw(self)
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    #[inline]
    #[must_use]
    pub fn from_native(native: Raw) -> Self {
        self::from_native(native)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    #[inline]
    #[must_use]
    pub fn to_native(self) -> Raw {
        self::to_native(self)
    }
}

// Implement clone via propagation.
impl<Raw: core::clone::Clone> core::clone::Clone for LittleEndian<Raw> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement copy via propagation.
impl<Raw: Copy> core::marker::Copy for LittleEndian<Raw> {
}

// For debugging simply print the raw values.
impl<Raw: core::fmt::Debug> core::fmt::Debug for LittleEndian<Raw> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("LittleEndian").field(&self.0).finish()
    }
}

// Choose defaults based on the native defaults.
impl<Raw> core::default::Default for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::default::Default,
{
    fn default() -> Self {
        Self::from_native(Default::default())
    }
}

// Convert to native for basic formatting.
impl<Raw> core::fmt::Display for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::fmt::Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        <Raw as core::fmt::Display>::fmt(&self.to_native(), fmt)
    }
}

// Compare based on native value.
impl<Raw> core::cmp::Eq for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::Eq,
{
}

// Hash based on native value.
impl<Raw> core::hash::Hash for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::hash::Hash,
{
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        self.to_native().hash(state)
    }
}

// Order based on native value.
impl<Raw> core::cmp::Ord for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_native().cmp(&other.to_native())
    }
}

// Compare based on native value.
impl<Raw> core::cmp::PartialEq for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.to_native().eq(&other.to_native())
    }
}

// Order based on native value.
impl<Raw> core::cmp::PartialOrd for LittleEndian<Raw>
where
    Self: NativeEndian<Raw>,
    Raw: Copy + core::cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_native().partial_cmp(&other.to_native())
    }
}

#[cfg(target_endian = "big")]
mod impl_big {
    use super::*;

    unsafe impl NativeEndian<i8> for BigEndian<i8> { }
    unsafe impl NativeEndian<i16> for BigEndian<i16> { }
    unsafe impl NativeEndian<i32> for BigEndian<i32> { }
    unsafe impl NativeEndian<i64> for BigEndian<i64> { }
    unsafe impl NativeEndian<i128> for BigEndian<i128> { }
    unsafe impl NativeEndian<isize> for BigEndian<isize> { }
    unsafe impl NativeEndian<u8> for BigEndian<u8> { }
    unsafe impl NativeEndian<u16> for BigEndian<u16> { }
    unsafe impl NativeEndian<u32> for BigEndian<u32> { }
    unsafe impl NativeEndian<u64> for BigEndian<u64> { }
    unsafe impl NativeEndian<u128> for BigEndian<u128> { }
    unsafe impl NativeEndian<usize> for BigEndian<usize> { }
    unsafe impl NativeEndian<core::num::NonZeroI8> for BigEndian<core::num::NonZeroI8> { }
    unsafe impl NativeEndian<core::num::NonZeroI16> for BigEndian<core::num::NonZeroI16> { }
    unsafe impl NativeEndian<core::num::NonZeroI32> for BigEndian<core::num::NonZeroI32> { }
    unsafe impl NativeEndian<core::num::NonZeroI64> for BigEndian<core::num::NonZeroI64> { }
    unsafe impl NativeEndian<core::num::NonZeroI128> for BigEndian<core::num::NonZeroI128> { }
    unsafe impl NativeEndian<core::num::NonZeroIsize> for BigEndian<core::num::NonZeroIsize> { }
    unsafe impl NativeEndian<core::num::NonZeroU8> for BigEndian<core::num::NonZeroU8> { }
    unsafe impl NativeEndian<core::num::NonZeroU16> for BigEndian<core::num::NonZeroU16> { }
    unsafe impl NativeEndian<core::num::NonZeroU32> for BigEndian<core::num::NonZeroU32> { }
    unsafe impl NativeEndian<core::num::NonZeroU64> for BigEndian<core::num::NonZeroU64> { }
    unsafe impl NativeEndian<core::num::NonZeroU128> for BigEndian<core::num::NonZeroU128> { }
    unsafe impl NativeEndian<core::num::NonZeroUsize> for BigEndian<core::num::NonZeroUsize> { }

    unsafe impl NativeEndian<i8> for LittleEndian<i8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i16> for LittleEndian<i16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i32> for LittleEndian<i32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i64> for LittleEndian<i64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i128> for LittleEndian<i128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<isize> for LittleEndian<isize> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u8> for LittleEndian<u8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u16> for LittleEndian<u16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u32> for LittleEndian<u32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u64> for LittleEndian<u64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u128> for LittleEndian<u128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<usize> for LittleEndian<usize> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI8> for LittleEndian<core::num::NonZeroI8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI16> for LittleEndian<core::num::NonZeroI16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI32> for LittleEndian<core::num::NonZeroI32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI64> for LittleEndian<core::num::NonZeroI64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI128> for LittleEndian<core::num::NonZeroI128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroIsize> for LittleEndian<core::num::NonZeroIsize> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU8> for LittleEndian<core::num::NonZeroU8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU16> for LittleEndian<core::num::NonZeroU16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU32> for LittleEndian<core::num::NonZeroU32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU64> for LittleEndian<core::num::NonZeroU64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU128> for LittleEndian<core::num::NonZeroU128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroUsize> for LittleEndian<core::num::NonZeroUsize> { const NEEDS_SWAP: bool = true; }
}

#[cfg(target_endian = "little")]
mod impl_big {
    use super::*;

    unsafe impl NativeEndian<i8> for BigEndian<i8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i16> for BigEndian<i16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i32> for BigEndian<i32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i64> for BigEndian<i64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<i128> for BigEndian<i128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<isize> for BigEndian<isize> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u8> for BigEndian<u8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u16> for BigEndian<u16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u32> for BigEndian<u32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u64> for BigEndian<u64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<u128> for BigEndian<u128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<usize> for BigEndian<usize> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI8> for BigEndian<core::num::NonZeroI8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI16> for BigEndian<core::num::NonZeroI16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI32> for BigEndian<core::num::NonZeroI32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI64> for BigEndian<core::num::NonZeroI64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroI128> for BigEndian<core::num::NonZeroI128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroIsize> for BigEndian<core::num::NonZeroIsize> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU8> for BigEndian<core::num::NonZeroU8> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU16> for BigEndian<core::num::NonZeroU16> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU32> for BigEndian<core::num::NonZeroU32> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU64> for BigEndian<core::num::NonZeroU64> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroU128> for BigEndian<core::num::NonZeroU128> { const NEEDS_SWAP: bool = true; }
    unsafe impl NativeEndian<core::num::NonZeroUsize> for BigEndian<core::num::NonZeroUsize> { const NEEDS_SWAP: bool = true; }

    unsafe impl NativeEndian<i8> for LittleEndian<i8> { }
    unsafe impl NativeEndian<i16> for LittleEndian<i16> { }
    unsafe impl NativeEndian<i32> for LittleEndian<i32> { }
    unsafe impl NativeEndian<i64> for LittleEndian<i64> { }
    unsafe impl NativeEndian<i128> for LittleEndian<i128> { }
    unsafe impl NativeEndian<isize> for LittleEndian<isize> { }
    unsafe impl NativeEndian<u8> for LittleEndian<u8> { }
    unsafe impl NativeEndian<u16> for LittleEndian<u16> { }
    unsafe impl NativeEndian<u32> for LittleEndian<u32> { }
    unsafe impl NativeEndian<u64> for LittleEndian<u64> { }
    unsafe impl NativeEndian<u128> for LittleEndian<u128> { }
    unsafe impl NativeEndian<usize> for LittleEndian<usize> { }
    unsafe impl NativeEndian<core::num::NonZeroI8> for LittleEndian<core::num::NonZeroI8> { }
    unsafe impl NativeEndian<core::num::NonZeroI16> for LittleEndian<core::num::NonZeroI16> { }
    unsafe impl NativeEndian<core::num::NonZeroI32> for LittleEndian<core::num::NonZeroI32> { }
    unsafe impl NativeEndian<core::num::NonZeroI64> for LittleEndian<core::num::NonZeroI64> { }
    unsafe impl NativeEndian<core::num::NonZeroI128> for LittleEndian<core::num::NonZeroI128> { }
    unsafe impl NativeEndian<core::num::NonZeroIsize> for LittleEndian<core::num::NonZeroIsize> { }
    unsafe impl NativeEndian<core::num::NonZeroU8> for LittleEndian<core::num::NonZeroU8> { }
    unsafe impl NativeEndian<core::num::NonZeroU16> for LittleEndian<core::num::NonZeroU16> { }
    unsafe impl NativeEndian<core::num::NonZeroU32> for LittleEndian<core::num::NonZeroU32> { }
    unsafe impl NativeEndian<core::num::NonZeroU64> for LittleEndian<core::num::NonZeroU64> { }
    unsafe impl NativeEndian<core::num::NonZeroU128> for LittleEndian<core::num::NonZeroU128> { }
    unsafe impl NativeEndian<core::num::NonZeroUsize> for LittleEndian<core::num::NonZeroUsize> { }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{align, ffi};

    // Verify typeinfo
    #[test]
    fn typeinfo() {
        assert_eq!(size_of::<BigEndian<u8>>(), 1);
        assert_eq!(size_of::<BigEndian<u64>>(), 8);
        assert_eq!(size_of::<LittleEndian<u8>>(), 1);
        assert_eq!(size_of::<LittleEndian<u64>>(), 8);

        assert_eq!(align_of::<BigEndian<u8>>(), 1);
        assert_eq!(align_of::<BigEndian<u64>>(), align_of::<u64>());
        assert_eq!(align_of::<LittleEndian<u8>>(), 1);
        assert_eq!(align_of::<LittleEndian<u64>>(), align_of::<u64>());
    }

    // Verify basic behavior
    #[test]
    fn basic() {
        let r: u32 = 1020304050;

        {
            let b: BigEndian<u32> = BigEndian::from_raw(r);
            let l: LittleEndian<u32> = LittleEndian::from_raw(r);

            assert_eq!(b.to_raw(), r);
            assert_eq!(l.to_raw(), r);
            assert_eq!(b.to_raw(), r);
            assert_eq!(l.to_raw(), r);

            assert!(b.to_native() != l.to_native());
        }

        {
            let b: BigEndian<u32> = BigEndian::from_native(r);
            let l: LittleEndian<u32> = LittleEndian::from_native(r);

            assert_eq!(b.to_native(), r);
            assert_eq!(l.to_native(), r);
            assert_eq!(b.to_native(), r);
            assert_eq!(l.to_native(), r);

            assert!(b.to_raw() != l.to_raw());
        }
    }

    // Verify unaligned trait conversions
    #[test]
    fn unaligned() {
        // Use an underaligned implementation of `NativeEndian` and verify
        // the generic implementations of `from_raw()` and `to_raw()` work
        // as expected.
        {
            type Big32 = ffi::Integer<BigEndian<u32>, align::AlignAs<1>>;
            type Little32 = ffi::Integer<LittleEndian<u32>, align::AlignAs<1>>;

            let r: u32 = 1020304050;
            let bn: Big32 = Big32::new(BigEndian::from_native(r));
            let br: Big32 = Big32::new(BigEndian::from_raw(r));
            let ln: Little32 = Little32::new(LittleEndian::from_native(r));
            let lr: Little32 = Little32::new(LittleEndian::from_raw(r));

            assert_eq!(bn.to_native(), r);
            assert_eq!(br.to_raw(), r);
            assert_eq!(ln.to_native(), r);
            assert_eq!(lr.to_raw(), r);
        }

        // Use an overaligned implementation of `NativeEndian` and verify
        // the generic implementations of `from_raw()` and `to_raw()` work
        // as expected.
        {
            type Big32 = ffi::Integer<BigEndian<u32>, align::AlignAs<8>>;
            type Little32 = ffi::Integer<LittleEndian<u32>, align::AlignAs<8>>;

            let r: u32 = 1020304050;
            let bn: Big32 = Big32::new(BigEndian::from_native(r));
            let br: Big32 = Big32::new(BigEndian::from_raw(r));
            let ln: Little32 = Little32::new(LittleEndian::from_native(r));
            let lr: Little32 = Little32::new(LittleEndian::from_raw(r));

            assert_eq!(bn.to_native(), r);
            assert_eq!(br.to_raw(), r);
            assert_eq!(ln.to_native(), r);
            assert_eq!(lr.to_raw(), r);
        }
    }

    // Verify traits
    #[test]
    fn traits() {
        let r: u32 = 1020304050;
        let b: BigEndian<u32> = BigEndian::from_native(r);
        let l: LittleEndian<u32> = LittleEndian::from_native(r);

        // `Clone`
        assert_eq!(b.clone().to_native(), r);
        assert_eq!(l.clone().to_native(), r);

        // `Copy`
        let bc: BigEndian<u32> = b;
        let lc: LittleEndian<u32> = l;
        assert_eq!(bc, b);
        assert_eq!(lc, l);

        // `Debug`
        assert_eq!(
            std::format!("{:?}", BigEndian::from_raw(r)),
            "BigEndian(1020304050)",
        );
        assert_eq!(
            std::format!("{:?}", LittleEndian::from_raw(r)),
            "LittleEndian(1020304050)",
        );

        // `Default`
        assert_eq!(
            BigEndian::from_native(0),
            <BigEndian<u32> as Default>::default(),
        );
        assert_eq!(
            LittleEndian::from_native(0),
            <LittleEndian<u32> as Default>::default(),
        );

        // `Display`
        assert_eq!(std::format!("{}", b), "1020304050");
        assert_eq!(std::format!("{}", l), "1020304050");

        // `PartialEq` / `Eq`
        assert!(b == BigEndian::from_native(r));
        assert!(l == LittleEndian::from_native(r));

        // `PartialOrd` / `Ord`
        assert!(b < BigEndian::from_native(r + 1));
        assert!(l < LittleEndian::from_native(r + 1));
    }
}
