//! # Pin Utilities
//!
//! This module provides utilities to simplify use of [`core::pin::Pin`], as
//! well as pinned alternatives for traits defined elsewhere.

use core::pin;

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

/// Grant generic access to pinned member fields.
///
/// This trait is a pinned alternative to [`crate::meta::Field`].
///
/// ## Safety
///
/// Any implementation must uphold the requirements of [`crate::meta::Field`].
/// On top, the member field at offset `OFFSET` must follow the rules of
/// `structural pinning` as described in [`core::pin::Pin`]
pub unsafe trait PinnedField<const OFFSET: usize, T>: Sized {
    /// Turn a pinned container reference into a pinned member field reference.
    fn pinned_field_of(container: pin::Pin<&Self>) -> pin::Pin<&T> {
        // SAFETY: The trait guarantees structural pinning for the member
        //         field. Therefore, we can safely pin the field.
        unsafe {
            container.map_unchecked(|v| &*(
                (v as *const Self)
                    .byte_offset(OFFSET as isize)
                    .cast::<T>()
            ))
        }
    }

    /// Turn a pinned member field reference back into a pinned container
    /// reference.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that `field` points to a member field of a
    /// container of type `Self`. Furthermore, the entire container must be
    /// borrowed and pinned for at least the lifetime of `field`.
    ///
    /// Note that these guarantees are always given if `field` was acquired
    /// via `Self::pinned_field_of()`.
    unsafe fn pinned_container_of(field: pin::Pin<&T>) -> pin::Pin<&Self> {
        // SAFETY: The caller guarantees that the entire container is borrowed
        //         and pinned.
        unsafe {
            field.map_unchecked(|v| &*(
                (v as *const T)
                    .byte_offset(-(OFFSET as isize))
                    .cast::<Self>()
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mem;

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

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    #[repr(C, align(4))]
    struct Test {
        a: u16,
        b: u8,
        c: u32,
    }

    unsafe impl PinnedField<{mem::typed_offset_of!(Test, b, u8)}, u8> for Test {
    }

    // Basic functionality tests for `PinnedField`.
    #[test]
    fn basic_pinned_field() {
        assert_eq!(core::mem::size_of::<Test>(), 8);

        let v = pin::pin!(Test { a: 14, b: 11, c: 1444 });
        let c = v.into_ref();
        let f = PinnedField::pinned_field_of(c);

        assert_eq!(*f, 11);
        assert!(core::ptr::eq(&*c, unsafe { &*PinnedField::pinned_container_of(f) }));
        assert_eq!(c, unsafe { PinnedField::pinned_container_of(f) });
    }
}
