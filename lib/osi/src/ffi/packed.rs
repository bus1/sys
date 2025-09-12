//! # Packed Types
//!
//! This module provides [`Packed`], as well as related utilities.

/// A wrapper type that applies `repr(packed)`. This means the packed type
/// has a minimum alignment of 1, but the same size as the wrapped type.
///
/// Similar to [`core::cell::Cell`], a lot of the inherent methods require
/// the embedded type to implement `Copy`. This is, because packed (and thus
/// possibly unaligned) values cannot be referenced safely but have to be
/// copied to be accessed.
#[derive(Copy, Default)]
#[repr(C, packed)]
pub struct Packed<Value>(Value);

impl<Value> Packed<Value> {
    /// Creates a new packed object with the specified value.
    #[inline]
    #[must_use]
    pub const fn new(v: Value) -> Self {
        Self(v)
    }

    /// Returns a pointer to the unaligned value wrapped in this packed object.
    #[inline(always)]
    #[must_use]
    pub const fn as_ptr(&self) -> *const Value {
        core::ptr::addr_of!(self.0)
    }

    /// Returns a mutable pointer to the unaligned value wrapped in this
    /// packed object.
    #[inline(always)]
    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut Value {
        core::ptr::addr_of_mut!(self.0)
    }

    /// Unwraps this object and returns the inner value.
    #[inline(always)]
    #[must_use]
    pub const fn into_inner(self) -> Value {
        // Preferably, this would just be `{ self.0 }`, but this currently does
        // not work in const-fn, since Rust cannot properly check whether
        // `Drop` would run. Hence, we instead move the inner value out.
        unsafe {
            // SAFETY: Since we leak `self`, we can leave a copy of `Value`
            //         behind without anyone ever getting access to it.
            let r: Value = core::ptr::read_unaligned(core::ptr::addr_of!(self.0));
            core::mem::forget(self);
            r
        }
    }

    /// Changes the underlying value of the wrapped type to the new value.
    /// This is equivalent to assigning a new wrapped object to this instance.
    #[inline]
    pub fn set(&mut self, v: Value) {
        self.0 = v;
    }

    /// Replaces the contained value, and returns the old contained value.
    #[inline]
    pub fn replace(&mut self, v: Value) -> Value {
        unsafe {
            // SAFETY: We implement `core::mem::swap()` but with unaligned
            //         pointers. This requires both source and destination to
            //         be valid allocations and available for read and write.
            //         This is given, since we have ownership or mutable access
            //         to both.
            let r: Value = core::ptr::read_unaligned(self.as_ptr());
            core::ptr::write_unaligned(self.as_mut_ptr(), v);
            r
        }
    }
}

// Inherent methods that require `Copy`.
impl<Value: Copy> Packed<Value> {
    /// Returns a copy of the wrapped value. The returned value will be
    /// properly aligned with all restrictions lifted.
    #[inline(always)]
    #[must_use]
    pub const fn get(&self) -> Value {
        self.0
    }
}

// Inherent methods that require `Default`.
impl<Value: Default> Packed<Value> {
    /// Takes the value, leaving Default::default() in its place.
    pub fn take(&mut self) -> Value {
        self.replace(Default::default())
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value: Copy> Clone for Packed<Value> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::fmt::Debug for Packed<Value>
where
    Value: Copy + core::fmt::Debug,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("Packed").field(&self.get()).finish()
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::fmt::Display for Packed<Value>
where
    Value: Copy + core::fmt::Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        <Value as core::fmt::Display>::fmt(&self.get(), fmt)
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::cmp::Eq for Packed<Value>
where
    Value: Copy + core::cmp::Eq,
{
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::hash::Hash for Packed<Value>
where
    Value: Copy + core::hash::Hash,
{
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        self.get().hash(state)
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::cmp::Ord for Packed<Value>
where
    Value: Copy + core::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::cmp::PartialEq for Packed<Value>
where
    Value: Copy + core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(&other.get())
    }
}

// Rely on `Copy` since we cannot get a reference to an unaligned value.
impl<Value> core::cmp::PartialOrd for Packed<Value>
where
    Value: Copy + core::cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

// Implement `From` via propagation.
impl<Value: Copy> core::convert::From<Value> for Packed<Value> {
    #[inline]
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify typeinfo of packed types
    #[test]
    fn typeinfo() {
        assert_eq!(size_of::<Packed<u8>>(), 1);
        assert_eq!(size_of::<Packed<u64>>(), 8);
        assert_eq!(align_of::<Packed<u8>>(), 1);
        assert_eq!(align_of::<Packed<u64>>(), 1);
    }

    // Verify basic behavior of packed types with `Copy`
    #[test]
    fn basic_copy() {
        let mut v: Packed<u64> = Packed::new(71);

        unsafe {
            // SAFETY: `u64` implements `Copy`, so we can copy values
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

        let mut v: Packed<RefCell<u64>> = Default::default();

        assert_eq!(v.take(), RefCell::new(0));
        assert_eq!(v.replace(RefCell::new(71)), RefCell::new(0));
        assert_eq!(v.into_inner(), RefCell::new(71));
    }

    // Verify traits of packed types
    #[test]
    fn traits() {
        let v: Packed<u64> = Packed::new(71);

        // `Clone`
        #[allow(clippy::clone_on_copy)]
        {
            assert_eq!(v.clone().into_inner(), 71);
        }

        // `Copy`
        let c: Packed<u64> = v;
        assert_eq!(c, v);

        // `Debug`
        assert_eq!(std::format!("{:?}", v), "Packed(71)");

        // `Default`
        assert_eq!(Packed::new(0), <Packed<u64> as Default>::default());

        // `Display`
        assert_eq!(std::format!("{}", v), "71");

        // `PartialEq` / `Eq`
        assert!(v == Packed::new(71));

        // `PartialOrd` / `Ord`
        assert!(v < Packed::new(73));

        // `From<T>`
        assert_eq!(v, 71.into());
    }
}
