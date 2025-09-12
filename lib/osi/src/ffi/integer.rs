//! # Fixed ABI Integers
//!
//! This module provides [`Integer`], as well as related utilities.

use crate::{align, ffi};

/// A type to abstract over primitive integers of different size, alignment,
/// and endianness. It is meant to be used as a replacement for builtin
/// primitive integer types like `u32` or `i64`. Unlike the builtin types, this
/// type allows working on a wide range of integers with a single
/// implementation.
///
/// Most importantly, this type allows to explicitly define its properties:
///
/// - **Alignment**: The alignment always matches exactly that given by
///   `Alignment`.
///
/// - **Size**: The size and encoding of the type matches that of `Value`,
///   unless the requested alignment exceeds its size. In that case, trailing
///   padding bytes are added to ensure the size is a multiple of the
///   alignment.
///
/// - **Endianness*: The endianness is controlled by `Value` and always
///   converted to native endianness when accessed via the `from/to_native()`
///   accessors.
///
/// The non-zero property of `Value` is propagated through this type, allowing
/// for `Option<..>` optimizations and ffi-stability.
#[repr(C)]
pub struct Integer<Value, Alignment: align::Aligned> {
    value: ffi::Packed<Value>,
    alignment: [Alignment::Align; 0],
}

impl<Value, Alignment: align::Aligned> Integer<Value, Alignment> {
    /// Creates a new integer object from its value. The data is taken
    /// unmodified and embedded into the new object. `get()` will yield
    /// the same value again.
    #[inline]
    #[must_use]
    pub const fn new(v: Value) -> Self {
        Self {
            value: ffi::Packed::new(v),
            alignment: [],
        }
    }

    /// Returns a pointer to the unaligned value wrapped in this packed object.
    #[inline(always)]
    #[must_use]
    pub const fn as_ptr(&self) -> *const Value {
        self.value.as_ptr()
    }

    /// Returns a mutable pointer to the unaligned value wrapped in this
    /// packed object.
    #[inline(always)]
    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut Value {
        self.value.as_mut_ptr()
    }

    /// Unwraps this object and returns the inner value.
    #[inline(always)]
    #[must_use]
    pub const fn into_inner(self) -> Value {
        // Preferably, this would just be `{ self.value.into_inner() }`, but
        // this currently does not work in const-fn, since Rust cannot properly
        // check whether `Drop` would run. Hence, we instead move the inner
        // value out.
        unsafe {
            // SAFETY: Since we leak `self`, we can leave a copy behind
            //         without anyone ever getting access to it.
            let r: ffi::Packed<Value> = core::ptr::read(core::ptr::addr_of!(self.value));
            core::mem::forget(self);
            r.into_inner()
        }
    }

    /// Changes the value that is embedded in this object to the specified
    /// value. The previous value is lost unrecoverably.
    ///
    /// This operation is equivalent to assigning a new object of this wrapper
    /// to the location of `self`.
    #[inline]
    pub fn set(&mut self, v: Value) {
        self.value.set(v);
    }

    /// Replaces the contained value, and returns the old contained value.
    #[inline]
    pub fn replace(&mut self, v: Value) -> Value {
        self.value.replace(v)
    }
}

// Inherent methods that require `Copy`.
impl<Value: Copy, Alignment: align::Aligned> Integer<Value, Alignment> {
    /// Yield the value that is embedded in this object. The value is
    /// returned unmodified. See `new()` for the inverse operation.
    #[inline(always)]
    #[must_use]
    pub const fn get(&self) -> Value {
        self.value.get()
    }
}

// Inherent methods that require `Default`.
impl<Value: Default, Alignment: align::Aligned> Integer<Value, Alignment> {
    /// Takes the value, leaving Default::default() in its place.
    pub fn take(&mut self) -> Value {
        self.replace(Default::default())
    }
}

// Inherent convenience methods via `ffi::NativeEndian`.
impl<Value: Copy, Alignment: align::Aligned> Integer<Value, Alignment> {
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    #[inline]
    #[must_use]
    pub const fn from_raw<Raw>(raw: Raw) -> Self
    where
        Self: ffi::NativeEndian<Raw>,
        Raw: Copy,
    {
        ffi::endian::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    #[inline]
    #[must_use]
    pub fn to_raw<Raw>(self) -> Raw
    where
        Self: ffi::NativeEndian<Raw>,
        Raw: Copy,
    {
        ffi::endian::to_raw(self)
    }

    /// Creates the foreign-ordered value from a native value, converting the
    /// value before retaining it, if required.
    #[inline]
    #[must_use]
    pub fn from_native<Raw>(native: Raw) -> Self
    where
        Self: ffi::NativeEndian<Raw>,
        Raw: Copy,
    {
        ffi::endian::from_native(native)
    }

    /// Returns the native representation of the value behind this wrapping
    /// object. The value is converted to the native representation before it
    /// is returned, if required.
    #[inline]
    #[must_use]
    pub const fn to_native<Raw>(self) -> Raw
    where
        Self: ffi::NativeEndian<Raw>,
        Raw: Copy,
    {
        ffi::endian::to_native(self)
    }
}

// Implement clone via `Copy`. We cannot propagate clone as we cannot get a
// reference to the packed inner value, but have to rely on `Copy`.
impl<Value, Alignment> core::clone::Clone for Integer<Value, Alignment>
where
    Value: Copy,
    Alignment: align::Aligned,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

// Implement copy via propagation.
impl<Value, Alignment> core::marker::Copy for Integer<Value, Alignment>
where
    Value: Copy,
    Alignment: align::Aligned,
{
}

// For debugging simply print the values (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::fmt::Debug for Integer<Value, Alignment>
where
    Value: Copy + core::fmt::Debug,
    Alignment: align::Aligned,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("Integer").field(&self.get()).finish()
    }
}

// Implement `Default` via propagation.
impl<Value, Alignment> core::default::Default for Integer<Value, Alignment>
where
    Value: core::default::Default,
    Alignment: align::Aligned,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

// Implement `Display` via propagation (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::fmt::Display for Integer<Value, Alignment>
where
    Value: Copy + core::fmt::Display,
    Alignment: align::Aligned,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        <Value as core::fmt::Display>::fmt(&self.get(), fmt)
    }
}

// Implement `Eq` via propagation (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::cmp::Eq for Integer<Value, Alignment>
where
    Value: Copy + core::cmp::Eq,
    Alignment: align::Aligned,
{
}

// Implement `Hash` via propagation (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::hash::Hash for Integer<Value, Alignment>
where
    Value: Copy + core::hash::Hash,
    Alignment: align::Aligned,
{
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        self.get().hash(state)
    }
}

// Implement `Ord` via propagation (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::cmp::Ord for Integer<Value, Alignment>
where
    Value: Copy + core::cmp::Ord,
    Alignment: align::Aligned,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

// Implement `PartialEq` via propagation (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::cmp::PartialEq for Integer<Value, Alignment>
where
    Value: Copy + core::cmp::PartialEq,
    Alignment: align::Aligned,
{
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(&other.get())
    }
}

// Implement `PartialOrd` via propagation (needs `Copy` due to unaligned value).
impl<Value, Alignment> core::cmp::PartialOrd for Integer<Value, Alignment>
where
    Value: Copy + core::cmp::PartialOrd,
    Alignment: align::Aligned,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

// Implement `From` via propagation.
impl<Value, Alignment: align::Aligned> core::convert::From<Value> for Integer<Value, Alignment>
{
    #[inline]
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

// Propagate `ffi::NativeAddress` from the underlying value.
impl<Value, Alignment, Target> ffi::NativeAddress<Target> for Integer<Value, Alignment>
where
    Value: Copy + ffi::NativeAddress<Target>,
    Alignment: align::Aligned,
    Target: ?Sized,
{
    #[inline]
    unsafe fn from_usize_unchecked(v: usize) -> Self {
        unsafe {
            // SAFETY: delegated to caller
            Self::new(Value::from_usize_unchecked(v))
        }
    }

    #[inline(always)]
    fn to_usize(&self) -> usize {
        self.get().into_usize()
    }
}

// Propagate ffi::NativeEndian from the underlying address.
//
// SAFETY: With `repr(transparent)` byte-swaps and transmutations can be
//         propagated from the inner type.
unsafe impl<Value, Alignment, Native> ffi::NativeEndian<Native> for Integer<Value, Alignment>
where
    Value: ffi::NativeEndian<Native>,
    Alignment: align::Aligned,
    Native: Copy,
{
    const NEEDS_SWAP: bool = Value::NEEDS_SWAP;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify typeinfo of basic integers
    #[test]
    fn typeinfo_basic() {
        assert_eq!(size_of::<Integer<i8, align::AlignAs<1>>>(), 1);
        assert_eq!(align_of::<Integer<i8, align::AlignAs<1>>>(), 1);
        assert_eq!(size_of::<Integer<i16, align::AlignAs<2>>>(), 2);
        assert_eq!(align_of::<Integer<i16, align::AlignAs<2>>>(), 2);
        assert_eq!(size_of::<Integer<i32, align::AlignAs<4>>>(), 4);
        assert_eq!(align_of::<Integer<i32, align::AlignAs<4>>>(), 4);
        assert_eq!(size_of::<Integer<i64, align::AlignAs<8>>>(), 8);
        assert_eq!(align_of::<Integer<i64, align::AlignAs<8>>>(), 8);
        assert_eq!(size_of::<Integer<i128, align::AlignAs<16>>>(), 16);
        assert_eq!(align_of::<Integer<i128, align::AlignAs<16>>>(), 16);
        assert_eq!(size_of::<Integer<u8, align::AlignAs<1>>>(), 1);
        assert_eq!(align_of::<Integer<u8, align::AlignAs<1>>>(), 1);
        assert_eq!(size_of::<Integer<u16, align::AlignAs<2>>>(), 2);
        assert_eq!(align_of::<Integer<u16, align::AlignAs<2>>>(), 2);
        assert_eq!(size_of::<Integer<u32, align::AlignAs<4>>>(), 4);
        assert_eq!(align_of::<Integer<u32, align::AlignAs<4>>>(), 4);
        assert_eq!(size_of::<Integer<u64, align::AlignAs<8>>>(), 8);
        assert_eq!(align_of::<Integer<u64, align::AlignAs<8>>>(), 8);
        assert_eq!(size_of::<Integer<u128, align::AlignAs<16>>>(), 16);
        assert_eq!(align_of::<Integer<u128, align::AlignAs<16>>>(), 16);

        assert_eq!(size_of::<Option<Integer<core::num::NonZeroI8, align::AlignAs<1>>>>(), 1);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroI8, align::AlignAs<1>>>>(), 1);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroI16, align::AlignAs<2>>>>(), 2);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroI16, align::AlignAs<2>>>>(), 2);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroI32, align::AlignAs<4>>>>(), 4);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroI32, align::AlignAs<4>>>>(), 4);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroI64, align::AlignAs<8>>>>(), 8);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroI64, align::AlignAs<8>>>>(), 8);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroI128, align::AlignAs<16>>>>(), 16);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroI128, align::AlignAs<16>>>>(), 16);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroU8, align::AlignAs<1>>>>(), 1);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroU8, align::AlignAs<1>>>>(), 1);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroU16, align::AlignAs<2>>>>(), 2);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroU16, align::AlignAs<2>>>>(), 2);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroU32, align::AlignAs<4>>>>(), 4);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroU32, align::AlignAs<4>>>>(), 4);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroU64, align::AlignAs<8>>>>(), 8);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroU64, align::AlignAs<8>>>>(), 8);
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroU128, align::AlignAs<16>>>>(), 16);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroU128, align::AlignAs<16>>>>(), 16);
    }

    // Verify `Integer` advanced type layout
    //
    // Check some non-standard type-parameters for `Integer` and verify the
    // memory layout.
    #[test]
    fn typeinfo_complex() {
        // verify that the alignment honors the request
        assert_eq!(align_of::<Integer<u8, align::AlignAs<16>>>(), 16);
        assert_eq!(align_of::<Integer<u128, align::AlignAs<1>>>(), 1);

        // verify that high alignments cause padding
        assert_eq!(size_of::<Integer<u8, align::AlignAs<16>>>(), 16);

        // zero-optimization must propagate through `Integer<BigEndian<...>>`
        assert_eq!(size_of::<Option<Integer<core::num::NonZeroI64, align::AlignAs<8>>>>(), 8);
        assert_eq!(align_of::<Option<Integer<core::num::NonZeroI64, align::AlignAs<8>>>>(), 8);
    }

    // Verify basic behavior with `Copy`
    #[test]
    fn basic_copy() {
        type Test16 = Integer<u16, align::AlignAs<2>>;

        let mut v: Test16 = Test16::new(71);

        unsafe {
            // SAFETY: `u16` implements `Copy`, so we can copy values
            //         out of the unaligned field.
            assert_eq!(core::ptr::read_unaligned(v.as_ptr()), 71);
            assert_eq!(core::ptr::read_unaligned(v.as_mut_ptr()), 71);
        }

        assert_eq!(v.get(), 71);
        v.set(73);
        assert_eq!(v.get(), 73);

        assert_eq!(v.replace(71), 73);
        assert_eq!(v.get(), 71);

        assert_eq!(v.take(), 71);
        assert_eq!(v.get(), 0);

        v.set(71);
        assert_eq!(v.get(), 71);
        assert_eq!(v.into_inner(), 71);
    }

    // Verify basic behavior of packed types without `Copy`
    #[test]
    fn basic_noncopy() {
        use core::cell::RefCell; // a simple non-Copy type

        type Test16 = Integer<RefCell<u16>, align::AlignAs<2>>;

        let mut v: Test16 = Default::default();

        assert_eq!(v.take(), RefCell::new(0));
        assert_eq!(v.replace(RefCell::new(71)), RefCell::new(0));
        assert_eq!(v.into_inner(), RefCell::new(71));
    }

    // Verify traits
    #[test]
    fn traits() {
        type Test16 = Integer<u16, align::AlignAs<2>>;

        let v: Test16 = Test16::new(71);

        // `Clone`
        #[allow(clippy::clone_on_copy)]
        {
            assert_eq!(v.clone().into_inner(), 71);
        }

        // `Copy`
        let c: Test16 = v;
        assert_eq!(c, v);

        // `Debug`
        assert_eq!(std::format!("{:?}", v), "Integer(71)");

        // `Default`
        assert_eq!(Test16::new(0), <Test16 as Default>::default());

        // `Display`
        assert_eq!(std::format!("{}", v), "71");

        // `PartialEq` / `Eq`
        assert!(v == Test16::new(71));

        // `PartialOrd` / `Ord`
        assert!(v < Test16::new(73));

        // `From<T>`
        assert_eq!(v, 71.into());
    }

    // Verify hashing
    #[test]
    fn hash() {
        type Test16 = Integer<u16, align::AlignAs<2>>;

        fn hash<T: core::hash::Hash>(v: T) -> u64 {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            v.hash(&mut s);
            core::hash::Hasher::finish(&s)
        }

        assert_eq!(hash(Test16::new(1)), hash(1u16));
    }
}
