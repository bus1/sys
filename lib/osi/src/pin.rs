//! # Pin Utilities
//!
//! This module provides utilities to simplify use of [`core::pin::Pin`], as
//! well as pinned alternatives for traits defined elsewhere.

use core::pin;

/// Borrow the inner pointer of a pinned pointer.
///
/// This exposes the inner pointer of a pinned pointer. This is unsafe, as it
/// allows circumventing the pinning guarantees.
///
/// ## Safety
///
/// The caller must uphold the pinning guarantees while using the returned
/// reference.
pub unsafe fn as_inner<T>(v: &pin::Pin<T>) -> &T {
    // SAFETY: `Pin` is guaranteed to be `repr(transparent)`. All other
    //         safety requirements are propagated.
    core::mem::transmute(v)
}

/// Yield a const raw pointer to the target.
pub fn as_ptr<T>(v: pin::Pin<T>) -> *mut T::Target
where
    T: core::ops::Deref,
{
    // SAFETY: `pin::Pin` guarantees that `Deref` upholds its guarantees.
    unsafe { &raw const *pin::Pin::into_inner_unchecked(v) as *mut _ }
}

/// Yield a mut raw pointer to the target.
pub fn as_mut_ptr<T>(v: pin::Pin<T>) -> *mut T::Target
where
    T: core::ops::DerefMut,
{
    // SAFETY: `pin::Pin` guarantees that `DerefMut` upholds its guarantees.
    unsafe { &raw mut *pin::Pin::into_inner_unchecked(v) }
}

/// Constructs a new pin by mapping the pointer.
///
/// This is an alternative to [`pin::Pin::map_unchecked`] but operates on the
/// pointer rather than the pointee.
///
/// ## Safety
///
/// The caller must uphold the pinning guarantees as if the closure received
/// a pinned value rather than an unpinned one, and as if it returned a pinned
/// value. This is similar to the pin requirements for [`Drop::drop`].
pub unsafe fn map_unchecked<T, U, F>(v: pin::Pin<T>, f: F) -> pin::Pin<U>
where
    T: core::ops::Deref,
    U: core::ops::Deref,
    F: FnOnce(T) -> U,
{
    // SAFETY: All requirements are relayed to the caller.
    unsafe { pin::Pin::new_unchecked(f(pin::Pin::into_inner_unchecked(v))) }
}

#[cfg(test)]
mod test {
    use super::*;

    // Verify behavior of `as_inner()`.
    #[test]
    fn basic_as_inner() {
        let v = alloc::boxed::Box::pin(71u16);
        assert!(
            core::ptr::eq(
                as_ptr(v.as_ref()),
                unsafe { &raw const **as_inner(&v) },
            ),
        );
    }

    // Verify behavior of `as_[mut_]ptr()`.
    #[test]
    fn basic_as_ptr() {
        let v = pin::pin!(71u16);
        assert!(core::ptr::eq(&raw const *v, as_ptr(v)));

        let mut v = pin::pin!(71u16);
        assert!(core::ptr::eq(&raw mut *v, as_mut_ptr(v)));
    }

    // Verify behavior of `map_unchecked()`.
    #[test]
    fn basic_map() {
        let v = pin::pin!(71u16);
        let r0: pin::Pin<&mut u16> = v;
        assert_eq!(*r0, 71);

        let r1: pin::Pin<&u16> = unsafe {
            map_unchecked(r0, |v: &mut u16| -> &u16 { v })
        };
        assert_eq!(*r1, 71);

        let r2: pin::Pin<&[u8; 2]> = unsafe {
            map_unchecked(r1, |v: &u16| -> &[u8; 2] { &*(v as *const u16 as *const _) })
        };
        assert!(r2[0] == 71 || r2[1] == 71);
    }
}
