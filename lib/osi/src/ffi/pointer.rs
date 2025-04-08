//! # Fixed ABI Pointers
//!
//! This module provides [`NativeAddress`], [`Pointer`], as well as related
//! utilities.

use crate::ffi;

/// A trait to annotate types that effectively wrap a native memory address. It
/// provides easy converters to/from pointer types, as well as some utilities
/// to treat the underlying type as a pointer or even reference.
///
/// This trait combines `T: From<usize>` and `*const Target: From<usize>` (as
/// well as their inverse) in a single trait and makes these easily accessible.
///
/// This type should only be implemented for types that can be represented as
/// a `usize` on the target platform. That is, this type assumes that the
/// addresses it deals with are native memory addresses.
pub trait NativeAddress<Target: ?Sized> {
    /// Creates a new instance of this type from its address given as a `usize`
    /// value. The given value must not be 0.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that the address is not zero.
    #[must_use]
    unsafe fn from_usize_unchecked(v: usize) -> Self;

    /// Creates a new instance of this type with the address specified as a
    /// `usize` value. If the address is 0, this will yield `None`.
    #[inline]
    #[must_use]
    fn from_usize(v: usize) -> Option<Self>
    where
        Self: Sized,
    {
        if v == 0 {
            None
        } else {
            // SAFETY: verified to be non-zero
            unsafe { Some(Self::from_usize_unchecked(v)) }
        }
    }

    /// Yields the address of this instance as a `usize` value. The returned
    /// address is guaranteed to be non-zero.
    #[must_use]
    fn to_usize(&self) -> usize;

    /// Yields the address of this instance as a `usize` value, consuming the
    /// original object. The returned address is guaranteed to be non-zero.
    #[must_use]
    fn into_usize(self) -> usize
    where
        Self: Sized,
    {
        self.to_usize()
    }

    /// Creates a new instance of this type with a dangling address.
    /// This address is guaranteed not to be 0. However, the address is
    /// not necessarily unique and might match a valid address of
    /// another allocated object.
    #[inline]
    #[must_use]
    fn dangling() -> Self
    where
        Self: Sized,
        Target: Sized,
    {
        // SAFETY: Alignments cannot be 0.
        unsafe { Self::from_usize_unchecked(core::mem::align_of::<Target>()) }
    }

    /// Returns the underlying address of this type as a raw pointer type. This
    /// pointer is guaranteed not to be NULL.
    #[inline(always)]
    #[must_use]
    fn as_ptr(&self) -> *const Target
    where
        Target: Sized,
    {
        self.to_usize() as *const Target
    }

    /// Returns the underlying address of this type as a raw pointer pointer
    /// type. This pointer is guaranteed not to be NULL.
    #[inline(always)]
    #[must_use]
    fn as_mut_ptr(&self) -> *mut Target
    where
        Target: Sized,
    {
        self.to_usize() as *mut Target
    }

    /// Returns the underlying address of this type as a reference to the
    /// target type.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the underlying address can be safely cast
    /// into a reference, following the usual requirements of the Rust
    /// language.
    #[inline(always)]
    #[must_use]
    unsafe fn as_ref<'a>(&self) -> &'a Target
    where
        Target: Sized,
    {
        // SAFETY: Delegated to caller.
        unsafe { &*self.as_ptr() }
    }

    /// Returns the underlying address of this pointer as a mutable
    /// reference to the target type.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the underlying address can be safely cast
    /// into a mutable reference, following the usual requirements of the Rust
    /// language.
    #[inline(always)]
    #[must_use]
    unsafe fn as_mut<'a>(&self) -> &'a mut Target
    where
        Target: Sized,
    {
        // SAFETY: Delegated to caller.
        unsafe { &mut *self.as_mut_ptr() }
    }
}

/// A type designed as alternative to `core::ptr::NonNull` but with a generic
/// address type. It allows representing 32-bit pointers on 64-bit machines,
/// and vice-versa, with correct alignment and size.
#[repr(transparent)]
pub struct Pointer<Address, Target: ?Sized> {
    address: Address,
    target: core::marker::PhantomData<*const Target>,
}

// Implement `NativeAddress` on native-sized primitive integers.
macro_rules! implement_native_address {
    ( $self:ty ) => {
        impl<Target: ?Sized> NativeAddress<Target> for $self {
            #[inline]
            unsafe fn from_usize_unchecked(v: usize) -> Self {
                assert!(size_of::<usize>() <= size_of::<$self>());
                // SAFETY: as-cast never folds to 0
                v as _
            }

            #[inline(always)]
            fn to_usize(&self) -> usize {
                assert!(size_of::<$self>() <= size_of::<usize>());
                *self as _
            }
        }
    };
}

// Implement `NativeAddress` on native-sized non-zero integers.
macro_rules! implement_native_address_nonzero {
    ( $self:ty ) => {
        impl<Target: ?Sized> NativeAddress<Target> for $self {
            #[inline]
            unsafe fn from_usize_unchecked(v: usize) -> Self {
                assert!(size_of::<usize>() <= size_of::<$self>());
                unsafe {
                    // SAFETY: delegated to caller
                    Self::new_unchecked(v as _)
                }
            }

            #[inline(always)]
            fn to_usize(&self) -> usize {
                assert!(size_of::<$self>() <= size_of::<usize>());
                self.get() as _
            }
        }
    };
}

// Lets ensure we know when Rust gains support for other pointer widths.
// `unexpected_cfgs` "prevents" us from supporting them ahead of time, so
// lets abide and error out when other values start popping up.
#[cfg(not(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64",
)))]
compile_error!("Target platform has an unsupported pointer-width.");

implement_native_address!(usize);
implement_native_address_nonzero!(core::num::NonZeroUsize);

#[cfg(target_pointer_width = "16")]
implement_native_address!(u16);
#[cfg(target_pointer_width = "16")]
implement_native_address_nonzero!(core::num::NonZeroU16);

#[cfg(target_pointer_width = "32")]
implement_native_address!(u32);
#[cfg(target_pointer_width = "32")]
implement_native_address_nonzero!(core::num::NonZeroU32);

#[cfg(target_pointer_width = "64")]
implement_native_address!(u64);
#[cfg(target_pointer_width = "64")]
implement_native_address_nonzero!(core::num::NonZeroU64);

impl<Address, Target: ?Sized> Pointer<Address, Target> {
    /// Creates a new instance of this pointer type from the provided address.
    /// The address is taken verbatim.
    #[inline]
    #[must_use]
    pub const fn new(v: Address) -> Self {
        Self {
            address: v,
            target: core::marker::PhantomData,
        }
    }

    /// Unwraps this object and returns the inner value.
    #[inline(always)]
    #[must_use]
    pub const fn into_inner(self) -> Address {
        // Preferably, this would just be `{ self.0 }`, but this currently does
        // not work in const-fn, since Rust cannot properly check whether
        // `Drop` would run. Hence, we instead move the inner value out.
        unsafe {
            // SAFETY: Since we leak `self`, we can leave a copy of `Value`
            //         behind without anyone ever getting access to it.
            let r: Address = core::ptr::read(core::ptr::addr_of!(self.address));
            core::mem::forget(self);
            r
        }
    }

    /// Returns the address underlying this pointer type.
    #[inline(always)]
    #[must_use]
    pub const fn address(&self) -> &Address {
        &self.address
    }

    /// Changes the underlying value of the wrapped type to the new value.
    /// This is equivalent to assigning a new wrapped object to this instance.
    #[inline]
    pub fn set(&mut self, v: Address) {
        self.address = v;
    }

    /// Replaces the contained value, and returns the old contained value.
    #[inline]
    pub fn replace(&mut self, mut v: Address) -> Address {
        core::mem::swap(&mut self.address, &mut v);
        v
    }

    /// Changes the target pointer type to the specified type. This does not
    /// change the underlying address value.
    #[inline]
    #[must_use]
    pub fn cast_into<Other>(self) -> Pointer<Address, Other> {
        let Self { address: v, .. } = self;
        Pointer::<Address, Other>::new(v)
    }
}

// Inherent methods that require `Copy`.
impl<Address: Copy, Target: ?Sized> Pointer<Address, Target> {
    /// Returns a copy of the wrapped value.
    #[inline(always)]
    #[must_use]
    pub const fn get(&self) -> Address {
        self.address
    }

    /// Changes the target pointer type to the specified type. This does not
    /// change the underlying address value.
    #[inline]
    #[must_use]
    pub const fn cast<Other>(&self) -> Pointer<Address, Other> {
        Pointer::<Address, Other>::new(*self.address())
    }
}

// Inherent methods that require `NativeAddress`.
impl<Address, Target> Pointer<Address, Target>
where
    Self: NativeAddress<Target>,
    Target: ?Sized,
{
    /// Creates a new instance of this type from its address given as a `usize`
    /// value. The given value must not be 0.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that the address is not zero.
    #[must_use]
    pub unsafe fn from_usize_unchecked(v: usize) -> Self {
        unsafe {
            // SAFETY: propagated to caller
            <Self as NativeAddress<Target>>::from_usize_unchecked(v)
        }
    }

    /// Creates a new instance of this type with the address specified as a
    /// `usize` value. If the address is 0, this will yield `None`.
    #[inline]
    #[must_use]
    pub fn from_usize(v: usize) -> Option<Self> {
        <Self as NativeAddress<Target>>::from_usize(v)
    }

    /// Yields the address of this instance as a `usize` value. The returned
    /// address is guaranteed to be non-zero.
    #[must_use]
    pub fn to_usize(&self) -> usize {
        <Self as NativeAddress<Target>>::to_usize(self)
    }

    /// Yields the address of this instance as a `usize` value, consuming the
    /// original object. The returned address is guaranteed to be non-zero.
    #[must_use]
    pub fn into_usize(self) -> usize {
        <Self as NativeAddress<Target>>::into_usize(self)
    }

    /// Creates a new instance of this type with a dangling address.
    /// This address is guaranteed not to be 0. However, the address is
    /// not necessarily unique and might match a valid address of
    /// another allocated object.
    #[inline]
    #[must_use]
    pub fn dangling() -> Self
    where
        Target: Sized,
    {
        <Self as NativeAddress<Target>>::dangling()
    }

    /// Returns the underlying address of this type as a raw pointer type. This
    /// pointer is guaranteed not to be NULL.
    #[inline(always)]
    #[must_use]
    pub fn as_ptr(&self) -> *const Target
    where
        Target: Sized,
    {
        <Self as NativeAddress<Target>>::as_ptr(self)
    }

    /// Returns the underlying address of this type as a raw pointer pointer
    /// type. This pointer is guaranteed not to be NULL.
    #[inline(always)]
    #[must_use]
    pub fn as_mut_ptr(&self) -> *mut Target
    where
        Target: Sized,
    {
        <Self as NativeAddress<Target>>::as_mut_ptr(self)
    }

    /// Returns the underlying address of this type as a reference to the
    /// target type.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the underlying address can be safely cast
    /// into a reference, following the usual requirements of the Rust
    /// language.
    #[inline(always)]
    #[must_use]
    pub unsafe fn as_ref<'a>(&self) -> &'a Target
    where
        Target: Sized,
    {
        // SAFETY: Delegated to caller.
        unsafe { <Self as NativeAddress<Target>>::as_ref(self) }
    }

    /// Returns the underlying address of this pointer as a mutable
    /// reference to the target type.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the underlying address can be safely cast
    /// into a mutable reference, following the usual requirements of the Rust
    /// language.
    #[inline(always)]
    #[must_use]
    pub unsafe fn as_mut<'a>(&self) -> &'a mut Target
    where
        Target: Sized,
    {
        // SAFETY: delegated to caller
        unsafe { <Self as NativeAddress<Target>>::as_mut(self) }
    }
}

// Inherent methods that require `NativeEndian`.
impl<Address, Target: ?Sized> Pointer<Address, Target> {
    /// Takes the raw, possibly foreign-ordered value `raw` and creates a
    /// wrapping object that protects the value from unguarded access.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[inline]
    #[must_use]
    pub fn from_raw<Raw>(raw: Raw) -> Self
    where
        Self: ffi::NativeEndian<Raw>,
        Raw: Copy,
    {
        ffi::endian::from_raw(raw)
    }

    /// Returns the underlying raw, possibly foreign-ordered value behind this
    /// wrapping object.
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
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
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
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
    ///
    /// This is a convenience accessor via the `NativeEndian` trait.
    #[inline]
    #[must_use]
    pub fn to_native<Raw>(self) -> Raw
    where
        Self: ffi::NativeEndian<Raw>,
        Raw: Copy,
    {
        ffi::endian::to_native(self)
    }
}

// Implement `Clone` via propagation.
impl<Address: Clone, Target: ?Sized> core::clone::Clone for Pointer<Address, Target> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.address().clone())
    }
}

// Implement `Copy` via propagation.
impl<Address: Copy, Target: ?Sized> core::marker::Copy for Pointer<Address, Target> {
}

// Implement `Debug` via propagation.
impl<Address, Target> core::fmt::Debug for Pointer<Address, Target>
where
    Address: core::fmt::Debug,
    Target: ?Sized,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("Pointer").field(self.address()).finish()
    }
}

// Implement `Display` via propagation.
impl<Address, Target> core::fmt::Display for Pointer<Address, Target>
where
    Address: core::fmt::Display,
    Target: ?Sized,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        <Address as core::fmt::Display>::fmt(self.address(), fmt)
    }
}

// Implement `Eq` via propagation.
impl<Address, Target> core::cmp::Eq for Pointer<Address, Target>
where
    Address: core::cmp::Eq,
    Target: ?Sized,
{
}

// Implement `Hash` via propagation.
impl<Address, Target> core::hash::Hash for Pointer<Address, Target>
where
    Address: core::hash::Hash,
    Target: ?Sized,
{
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        self.address().hash(state)
    }
}

// Implement `Ord` via propagation.
impl<Address, Target> core::cmp::Ord for Pointer<Address, Target>
where
    Address: core::cmp::Ord,
    Target: ?Sized,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.address().cmp(other.address())
    }
}

// Implement `PartialEq` via propagation.
impl<Address, Target> core::cmp::PartialEq for Pointer<Address, Target>
where
    Address: core::cmp::PartialEq,
    Target: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.address().eq(other.address())
    }
}

// Implement `PartialOrd` via propagation.
impl<Address, Target> core::cmp::PartialOrd for Pointer<Address, Target>
where
    Address: core::cmp::PartialOrd,
    Target: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.address().partial_cmp(other.address())
    }
}

// Propagate `NativeAddress` from the underlying address.
impl<Address, Target> NativeAddress<Target> for Pointer<Address, Target>
where
    Address: NativeAddress<Target>,
    Target: ?Sized,
{
    #[inline]
    unsafe fn from_usize_unchecked(v: usize) -> Self {
        unsafe {
            // SAFETY: propagated to caller
            Self::new(Address::from_usize_unchecked(v))
        }
    }

    #[inline(always)]
    fn to_usize(&self) -> usize {
        self.address().to_usize()
    }
}

// Propagate `ffi::NativeEndian` from the underlying address.
//
// SAFETY: With `repr(transparent)` byte-swaps and transmutations can be
//         propagated from the inner type.
unsafe impl<Address, Target, Native> ffi::NativeEndian<Native> for Pointer<Address, Target>
where
    Address: ffi::NativeEndian<Native>,
    Target: ?Sized,
    Native: Copy,
{
    const NEEDS_SWAP: bool = Address::NEEDS_SWAP;
}

// Implement import from usize based on NativeAddress.
impl<Address, Target> TryFrom<usize> for Pointer<Address, Target>
where
    Self: NativeAddress<Target>,
    Target: ?Sized,
{
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        Self::from_usize(v).ok_or(())
    }
}

// Implement import from reference based on NativeAddress.
impl<Address, Target> From<&Target> for Pointer<Address, Target>
where
    Self: NativeAddress<Target>,
    Target: Sized,
{
    #[inline]
    fn from(v: &Target) -> Self {
        // SAFETY: References cannot be NULL.
        unsafe { Self::from_usize_unchecked(v as *const Target as usize) }
    }
}

// Implement import from mutable reference based on NativeAddress.
impl<Address, Target> From<&mut Target> for Pointer<Address, Target>
where
    Self: NativeAddress<Target>,
    Target: Sized,
{
    #[inline]
    fn from(v: &mut Target) -> Self {
        // SAFETY: References cannot be NULL.
        unsafe { Self::from_usize_unchecked(v as *mut Target as usize) }
    }
}

// Implement import from pointer based on NativeAddress.
impl<Address, Target> TryFrom<*const Target> for Pointer<Address, Target>
where
    Self: NativeAddress<Target>,
    Target: Sized,
{
    type Error = ();

    fn try_from(v: *const Target) -> Result<Self, Self::Error> {
        Self::from_usize(v as usize).ok_or(())
    }
}

// Implement import from pointer based on NativeAddress.
impl<Address, Target> TryFrom<*mut Target> for Pointer<Address, Target>
where
    Self: NativeAddress<Target>,
    Target: Sized,
{
    type Error = ();

    fn try_from(v: *mut Target) -> Result<Self, Self::Error> {
        Self::from_usize(v as usize).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify typeinfo of basic types
    //
    // Check for size and alignment constaints on all helper types that have a
    // guaranteed layout.
    #[test]
    fn typeinfo() {
        assert_eq!(align_of::<Option<Pointer<core::num::NonZeroU32, ()>>>(), align_of::<u32>());
        assert_eq!(size_of::<Option<Pointer<core::num::NonZeroU32, ()>>>(), size_of::<u32>());
    }

    // Verify basic behavior with `Copy`
    #[test]
    fn basic_copy() {
        let mut v: Pointer<u64, ()> = Pointer::new(71);

        assert_eq!(v.get(), 71);
        v.set(73);
        assert_eq!(v.get(), 73);

        assert_eq!(v.replace(71), 73);
        assert_eq!(v.get(), 71);

        assert_eq!(v.get(), 71);
        assert_eq!(v.into_inner(), 71);
    }

    // Verify basic behavior without `Copy`
    #[test]
    fn basic_noncopy() {
        use core::cell::RefCell; // a simple non-Copy type

        let mut v: Pointer<RefCell<u64>, ()> = Pointer::new(RefCell::new(73));

        assert_eq!(v.replace(RefCell::new(71)), RefCell::new(73));
        assert_eq!(v.into_inner(), RefCell::new(71));
    }

    // Verify traits of packed types
    #[test]
    fn traits() {
        let v: Pointer<u64, ()> = Pointer::new(71);

        // `Clone`
        assert_eq!(v.clone().into_inner(), 71);

        // `Copy`
        let c: Pointer<u64, ()> = v;
        assert_eq!(c, v);

        // `Debug`
        assert_eq!(std::format!("{:?}", v), "Pointer(71)");

        // `Display`
        assert_eq!(std::format!("{}", v), "71");

        // `PartialEq` / `Eq`
        assert!(v == Pointer::new(71));

        // `PartialOrd` / `Ord`
        assert!(v < Pointer::new(73));
    }
}
