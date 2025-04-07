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
/// ## Safety
///
/// This trait requires the implementation to guarantee the size of `Self` (but
/// not necessarily its alignment) is larger than, or equal to, that of `Raw`,
/// and it must support initialization `Self` from the memory contents of `Raw`
/// (possibly with following uninitialized padding), and vice versa.
///
/// This effectively allows creating `Self` from `Raw` in a generic way, by
/// simply copying the contents into a new instance of `Self`, and filling the
/// remaining bytes with 0 (if any).
pub unsafe trait NativeEndian<Raw> {
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    #[must_use]
    fn from_raw(raw: Raw) -> Self
    where
        Self: Sized,
    {
        assert!(size_of::<Raw>() <= size_of::<Self>());

        let mut v = core::mem::MaybeUninit::<Self>::uninit();

        unsafe {
            // SAFETY: The trait guarantees that `Self` can be initialized with
            //         the value of `Raw` plus possible trailing padding.
            if align_of::<Self>() >= align_of::<Raw>() {
                core::ptr::write(v.as_mut_ptr() as *mut Raw, raw);
            } else {
                core::ptr::write_unaligned(v.as_mut_ptr() as *mut Raw, raw);
            }

            v.assume_init()
        }
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    #[must_use]
    fn into_raw(self) -> Raw
    where
        Self: Sized,
    {
        assert!(size_of::<Raw>() <= size_of::<Self>());

        let v = unsafe {
            // SAFETY: The trait guarantees `Raw` can be initialized by `Self`
            //         by simple copy and stripping possible trailing padding.
            if align_of::<Self>() >= align_of::<Raw>() {
                core::ptr::read(&self as *const Self as *const Raw)
            } else {
                core::ptr::read_unaligned(&self as *const Self as *const Raw)
            }
        };

        // We do **not** call `core::mem::forget(self)`, since we did not move
        // the inner value, but created `v` from a memory copy.
        core::mem::drop(self);

        v
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    #[must_use]
    fn to_raw(&self) -> Raw
    where
        Self: Copy + Sized,
    {
        self.into_raw()
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    #[must_use]
    fn from_native(native: Raw) -> Self;

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    #[must_use]
    fn into_native(self) -> Raw;

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    #[must_use]
    fn to_native(&self) -> Raw
    where
        Self: Copy + Sized,
    {
        self.into_native()
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

// Implement `NativeEndian` on all primitive integers via identity mappings.
macro_rules! implement_endian_identity {
    ( $self:ty ) => {
        // SAFETY: Transmuting from/to itself is always safe.
        unsafe impl NativeEndian<$self> for $self {
            #[inline]
            fn from_native(native: Self) -> Self {
                native
            }

            #[inline(always)]
            fn into_native(self) -> Self {
                self
            }
        }
    };
}

implement_endian_identity!(i8);
implement_endian_identity!(i16);
implement_endian_identity!(i32);
implement_endian_identity!(i64);
implement_endian_identity!(i128);
implement_endian_identity!(isize);
implement_endian_identity!(u8);
implement_endian_identity!(u16);
implement_endian_identity!(u32);
implement_endian_identity!(u64);
implement_endian_identity!(u128);
implement_endian_identity!(usize);
implement_endian_identity!(core::num::NonZeroI8);
implement_endian_identity!(core::num::NonZeroI16);
implement_endian_identity!(core::num::NonZeroI32);
implement_endian_identity!(core::num::NonZeroI64);
implement_endian_identity!(core::num::NonZeroI128);
implement_endian_identity!(core::num::NonZeroIsize);
implement_endian_identity!(core::num::NonZeroU8);
implement_endian_identity!(core::num::NonZeroU16);
implement_endian_identity!(core::num::NonZeroU32);
implement_endian_identity!(core::num::NonZeroU64);
implement_endian_identity!(core::num::NonZeroU128);
implement_endian_identity!(core::num::NonZeroUsize);

impl<Raw> BigEndian<Raw>
where
    Self: NativeEndian<Raw>,
{
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn from_raw(raw: Raw) -> Self {
        <Self as NativeEndian<Raw>>::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn into_raw(self) -> Raw {
        <Self as NativeEndian<Raw>>::into_raw(self)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn to_raw(&self) -> Raw
    where
        Self: Copy,
    {
        <Self as NativeEndian<Raw>>::to_raw(self)
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[inline]
    #[must_use]
    pub fn from_native(native: Raw) -> Self {
        <Self as NativeEndian<Raw>>::from_native(native)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn into_native(self) -> Raw {
        <Self as NativeEndian<Raw>>::into_native(self)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn to_native(&self) -> Raw
    where
        Self: Copy,
    {
        <Self as NativeEndian<Raw>>::to_native(self)
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
{
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn from_raw(raw: Raw) -> Self {
        <Self as NativeEndian<Raw>>::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn into_raw(self) -> Raw {
        <Self as NativeEndian<Raw>>::into_raw(self)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn to_raw(&self) -> Raw
    where
        Self: Copy,
    {
        <Self as NativeEndian<Raw>>::to_raw(self)
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[inline]
    #[must_use]
    pub fn from_native(native: Raw) -> Self {
        <Self as NativeEndian<Raw>>::from_native(native)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn into_native(self) -> Raw {
        <Self as NativeEndian<Raw>>::into_native(self)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[must_use]
    pub fn to_native(&self) -> Raw
    where
        Self: Copy,
    {
        <Self as NativeEndian<Raw>>::to_native(self)
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

// Implement `NativeEndian` on big-endian integers via `from/to_be()`.
macro_rules! implement_endian_be {
    ( $self:ty, $raw:ty ) => {
        // SAFETY: `BigEndian<T>` is `repr(transparent)` and `T` is `Copy`
        //         for all implementors, so copy-initialization is safe.
        unsafe impl NativeEndian<$raw> for $self {
            #[inline]
            fn from_native(native: $raw) -> Self {
                Self::from_raw(native.to_be())
            }

            #[inline(always)]
            fn into_native(self) -> $raw {
                <$raw>::from_be(self.to_raw())
            }
        }
    };
}

// Implement `NativeEndian` on big-endian non-zeros via `from/to_be()`.
macro_rules! implement_endian_be_nonzero {
    ( $self:ty, $raw:ty, $prim:ty ) => {
        // SAFETY: `BigEndian<T>` is `repr(transparent)` and `T` is `Copy`
        //         for all implementors, so copy-initialization is safe.
        unsafe impl NativeEndian<$raw> for $self {
            #[inline]
            fn from_native(native: $raw) -> Self {
                Self::from_raw(
                    // SAFETY: endian conversion never folds to 0
                    unsafe { <$raw>::new_unchecked(native.get().to_be()) },
                )
            }

            #[inline(always)]
            fn into_native(self) -> $raw {
                // SAFETY: endian conversion never folds to 0
                unsafe { <$raw>::new_unchecked(<$prim>::from_be(self.to_raw().get())) }
            }
        }
    };
}

// Implement `NativeEndian` on little-endian integers via `from/to_le()`.
macro_rules! implement_endian_le {
    ( $self:ty, $raw:ty ) => {
        // SAFETY: `LittleEndian<T>` is `repr(transparent)` and `T` is `Copy`
        //         for all implementors, so copy-initialization is safe.
        unsafe impl NativeEndian<$raw> for $self {
            #[inline]
            fn from_native(native: $raw) -> Self {
                Self::from_raw(native.to_le())
            }

            #[inline(always)]
            fn into_native(self) -> $raw {
                <$raw>::from_le(self.to_raw())
            }
        }
    };
}

// Implement `NativeEndian` on little-endian non-zeros via `from/to_le()`.
macro_rules! implement_endian_le_nonzero {
    ( $self:ty, $raw:ty, $prim:ty ) => {
        // SAFETY: `LittleEndian<T>` is `repr(transparent)` and `T` is `Copy`
        //         for all implementors, so copy-initialization is safe.
        unsafe impl NativeEndian<$raw> for $self {
            #[inline]
            fn from_native(native: $raw) -> Self {
                Self::from_raw(
                    // SAFETY: endian conversion never folds to 0
                    unsafe { <$raw>::new_unchecked(native.get().to_le()) },
                )
            }

            #[inline(always)]
            fn into_native(self) -> $raw {
                // SAFETY: endian conversion never folds to 0
                unsafe { <$raw>::new_unchecked(<$prim>::from_le(self.to_raw().get())) }
            }
        }
    };
}

implement_endian_be!(BigEndian<i8>, i8);
implement_endian_be!(BigEndian<i16>, i16);
implement_endian_be!(BigEndian<i32>, i32);
implement_endian_be!(BigEndian<i64>, i64);
implement_endian_be!(BigEndian<i128>, i128);
implement_endian_be!(BigEndian<isize>, isize);
implement_endian_be!(BigEndian<u8>, u8);
implement_endian_be!(BigEndian<u16>, u16);
implement_endian_be!(BigEndian<u32>, u32);
implement_endian_be!(BigEndian<u64>, u64);
implement_endian_be!(BigEndian<u128>, u128);
implement_endian_be!(BigEndian<usize>, usize);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroI8>, core::num::NonZeroI8, i8);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroI16>, core::num::NonZeroI16, i16);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroI32>, core::num::NonZeroI32, i32);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroI64>, core::num::NonZeroI64, i64);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroI128>, core::num::NonZeroI128, i128);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroIsize>, core::num::NonZeroIsize, isize);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroU8>, core::num::NonZeroU8, u8);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroU16>, core::num::NonZeroU16, u16);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroU32>, core::num::NonZeroU32, u32);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroU64>, core::num::NonZeroU64, u64);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroU128>, core::num::NonZeroU128, u128);
implement_endian_be_nonzero!(BigEndian<core::num::NonZeroUsize>, core::num::NonZeroUsize, usize);

implement_endian_le!(LittleEndian<i8>, i8);
implement_endian_le!(LittleEndian<i16>, i16);
implement_endian_le!(LittleEndian<i32>, i32);
implement_endian_le!(LittleEndian<i64>, i64);
implement_endian_le!(LittleEndian<i128>, i128);
implement_endian_le!(LittleEndian<isize>, isize);
implement_endian_le!(LittleEndian<u8>, u8);
implement_endian_le!(LittleEndian<u16>, u16);
implement_endian_le!(LittleEndian<u32>, u32);
implement_endian_le!(LittleEndian<u64>, u64);
implement_endian_le!(LittleEndian<u128>, u128);
implement_endian_le!(LittleEndian<usize>, usize);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroI8>, core::num::NonZeroI8, i8);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroI16>, core::num::NonZeroI16, i16);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroI32>, core::num::NonZeroI32, i32);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroI64>, core::num::NonZeroI64, i64);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroI128>, core::num::NonZeroI128, i128);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroIsize>, core::num::NonZeroIsize, isize);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroU8>, core::num::NonZeroU8, u8);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroU16>, core::num::NonZeroU16, u16);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroU32>, core::num::NonZeroU32, u32);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroU64>, core::num::NonZeroU64, u64);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroU128>, core::num::NonZeroU128, u128);
implement_endian_le_nonzero!(LittleEndian<core::num::NonZeroUsize>, core::num::NonZeroUsize, usize);

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

            assert_eq!(b.into_raw(), r);
            assert_eq!(l.into_raw(), r);
            assert_eq!(b.to_raw(), r);
            assert_eq!(l.to_raw(), r);

            assert!(b.to_native() != l.to_native());
        }

        {
            let b: BigEndian<u32> = BigEndian::from_native(r);
            let l: LittleEndian<u32> = LittleEndian::from_native(r);

            assert_eq!(b.into_native(), r);
            assert_eq!(l.into_native(), r);
            assert_eq!(b.to_native(), r);
            assert_eq!(l.to_native(), r);

            assert!(b.to_raw() != l.to_raw());
        }
    }

    // Verify unaligned trait conversions
    #[test]
    fn unaligned() {
        // Use an underaligned implementation of `NativeEndian` and verify
        // the generic implementations of `from_raw()` and `into_raw()` work
        // as expected.
        {
            type Big32 = ffi::Integer<BigEndian<u32>, align::AlignAs<1>>;
            type Little32 = ffi::Integer<LittleEndian<u32>, align::AlignAs<1>>;

            let r: u32 = 1020304050;
            let bn: Big32 = Big32::new(BigEndian::from_native(r));
            let br: Big32 = Big32::new(BigEndian::from_raw(r));
            let ln: Little32 = Little32::new(LittleEndian::from_native(r));
            let lr: Little32 = Little32::new(LittleEndian::from_raw(r));

            assert_eq!(bn.into_native(), r);
            assert_eq!(br.into_raw(), r);
            assert_eq!(ln.into_native(), r);
            assert_eq!(lr.into_raw(), r);
        }

        // Use an overaligned implementation of `NativeEndian` and verify
        // the generic implementations of `from_raw()` and `into_raw()` work
        // as expected.
        {
            type Big32 = ffi::Integer<BigEndian<u32>, align::AlignAs<8>>;
            type Little32 = ffi::Integer<LittleEndian<u32>, align::AlignAs<8>>;

            let r: u32 = 1020304050;
            let bn: Big32 = Big32::new(BigEndian::from_native(r));
            let br: Big32 = Big32::new(BigEndian::from_raw(r));
            let ln: Little32 = Little32::new(LittleEndian::from_native(r));
            let lr: Little32 = Little32::new(LittleEndian::from_raw(r));

            assert_eq!(bn.into_native(), r);
            assert_eq!(br.into_raw(), r);
            assert_eq!(ln.into_native(), r);
            assert_eq!(lr.into_raw(), r);
        }
    }

    // Verify traits
    #[test]
    fn traits() {
        let r: u32 = 1020304050;
        let b: BigEndian<u32> = BigEndian::from_native(r);
        let l: LittleEndian<u32> = LittleEndian::from_native(r);

        // `Clone`
        assert_eq!(b.clone().into_native(), r);
        assert_eq!(l.clone().into_native(), r);

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
