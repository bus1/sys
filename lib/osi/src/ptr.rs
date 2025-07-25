//! # Utilities to manage memory through raw pointers
//!
//! This module contains utilities to manage memory through raw pointers and
//! convert them to/from safe Rust types.

/// Convert an immutable reference to a `NonNull` pointer.
///
/// This is equivalent to [`core::ptr::NonNull::from_ref()`].
///
// MSRV(1.89): This is available in the Rust standard library as
//             [`core::ptr::NonNull::from_ref()`] but higher than our MSRV.
pub const fn nonnull_from_ref<T>(v: &T) -> core::ptr::NonNull<T> {
    // SAFETY: A reference cannot be null.
    unsafe { core::ptr::NonNull::new_unchecked(v as *const T as *mut T) }
}

/// Convert a mutable reference to a `NonNull` pointer.
///
/// This is equivalent to [`core::ptr::NonNull::from_mut()`].
///
// MSRV(1.89): This is available in the Rust standard library as
//             [`core::ptr::NonNull::from_mut()`] but higher than our MSRV.
pub const fn nonnull_from_mut<T>(v: &mut T) -> core::ptr::NonNull<T> {
    // SAFETY: A reference cannot be null.
    unsafe { core::ptr::NonNull::new_unchecked(v as *mut T) }
}

#[cfg(test)]
mod test {
    use super::*;

    // Verify `nonnull_from_ref()`.
    #[test]
    fn basic_nonnull_from_ref() {
        let v = 71u16;
        let r0 = &v;
        let r1 = nonnull_from_ref(r0);
        let r2 = unsafe { r1.as_ref() };

        assert_eq!(*r0, 71);
        assert_eq!(unsafe { *r1.as_ref() }, 71);
        assert_eq!(*r2, 71);
        assert!(core::ptr::eq(r0, r1.as_ptr()));
        assert!(core::ptr::eq(r2, r1.as_ptr()));
    }

    // Verify `nonnull_from_mut()`.
    #[test]
    fn basic_nonnull_from_mut() {
        let mut v = 71u16;
        let r0 = &mut v;
        let r1 = nonnull_from_mut(r0);

        assert_eq!(*r0, 71);
        assert!(core::ptr::eq(r0, r1.as_ptr()));
    }
}
