//! # Utilities for conversions between types
//!
//! This module contains utilities to help dealing with conversions between
//! types.

use core::pin;
use crate::ptr::OnceRef;

/// Convert a value into a wrapped pointer to its dereferenced value.
///
/// This trait extends [`core::ops::Deref`] and allows converting a value
/// into a wrapped pointer to its dereferenced value. While
/// [`core::ops::Deref`] retains the original value and merely borrows to
/// dereference it, [`IntoDeref`] actually converts the value into a
/// transparent wrapper of a pointer to the dereferenced value without
/// retaining the original.
///
/// Note that in many cases this will leak the original value if no extra
/// steps are taken. Usually, you want to restore the original value to ensure
/// the correct drop handlers are run (see [`FromDeref`]).
///
/// This trait is a generic version of
/// [`Box::into_raw()`](alloc::boxed::Box::into_raw),
/// [`Rc::into_raw()`](alloc::rc::Rc::into_raw),
/// [`Arc::into_raw()`](alloc::sync::Arc::into_raw), and more.
///
/// ## Mutability
///
/// [`IntoDeref`] serves for immutable and mutable conversions. The returned
/// wrapper type properly wraps both mutable and immutable references. It never
/// hands out safe mutable access, but also prevents safe creation of copies.
///
/// See [`OnceRef`] for details.
///
/// ## Safety
///
/// The implementations of [`Deref`](core::ops::Deref) and [`IntoDeref`] must
/// be compatible. That is, for a given instance, both must agree on what
/// object a deref resolves to.
///
/// Furthermore, for types that provide [pinned](core::pin) variants,
/// [`IntoDeref`] is part of the safety requirements of
/// [`core::pin::Pin::new_unchecked()`] just like
/// [`Deref`](core::ops::Deref) is.
pub unsafe trait IntoDeref: Sized + core::ops::Deref {
    /// Convert a value into a wrapped pointer to its dereferenced value.
    ///
    /// This consumes any dereferencable value and yields a wrapped pointer
    /// to the dereferenced value. The lifetime of the yielded wrapper is
    /// chosen by the caller, but cannot outlive [`Self`].
    ///
    /// While the returned wrapper can be treated as a `&'a Self::Target`, the
    /// wrapper provides additional guarantees that the reference cannot. See
    /// its definition for details.
    fn into_deref<'a>(v: Self) -> OnceRef<'a, Self::Target> where Self: 'a;

    /// Convert a pinned value into a wrapped pointer to its dereferenced
    /// value.
    ///
    /// This is the pinned equivalent of [`Self::into_deref()`].
    fn pin_into_deref<'a>(v: pin::Pin<Self>) -> pin::Pin<OnceRef<'a, Self::Target>>
    where
        pin::Pin<Self>: 'a,
    {
        // SAFETY: Pinned types must ensure they uphold pinning guarantees
        //         just like `Deref` does (see trait requirements).
        unsafe {
            crate::pin::map_unchecked(v, Self::into_deref)
        }
    }
}

/// Convert a wrapped pointer to a dereferenced value back to its original
/// value.
///
/// This trait provides the inverse operation of [`IntoDeref`]. It takes a
/// pointer to a dereferenced value and restores the original pointer.
/// This operation is unsafe and requires the caller to guarantee that the
/// pointer was acquired via [`IntoDeref`] or similar means.
///
/// This trait is a generic version of
/// [`Box::from_raw()`](alloc::boxed::Box::from_raw),
/// [`Rc::from_raw()`](alloc::rc::Rc::from_raw),
/// [`Arc::from_raw()`](alloc::sync::Arc::from_raw), and more.
///
/// ## Safety
///
/// The implementations of [`Deref`](core::ops::Deref) and [`FromDeref`] must
/// be compatible. That is, for a given instance, both must agree on what
/// object a deref resolves to.
///
/// Furthermore, for types that provide [pinned](core::pin) variants,
/// [`IntoDeref`] is part of the safety requirements of
/// [`core::pin::Pin::new_unchecked()`] just like
/// [`Deref`](core::ops::Deref) is.
pub unsafe trait FromDeref: Sized + core::ops::Deref {
    /// Convert a wrapped pointer to a dereferenced value back to its original
    /// value.
    ///
    /// ## Safety
    ///
    /// The wrapped pointer must have been acquired via [`IntoDeref`] or a
    /// matching equivalent (i.e., the wrapped pointer must be a valid pointer
    /// for the smart pointer [`Self`]). This implies that it must be valid
    /// for a suitable lifetime for [`Self`], regardless which lifetime `'a` is
    /// picked.
    ///
    /// If `Self` requires exclusive access to the wrapped pointer, the caller
    /// must guarantee that they do not make use of any retained copies of the
    /// wrapped pointer (note that such copies cannot be created in safe code).
    ///
    /// It is always safe to call this on values obtained via [`IntoDeref`].
    unsafe fn from_deref<'a>(v: OnceRef<'a, Self::Target>) -> Self;

    /// Convert a wrapped pointer to a dereferenced value back to its original
    /// pinned value.
    ///
    /// This is the pinned equivalent of [`Self::from_deref()`].
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that the original value was a pinned pointer.
    /// Furthermore, all requirements of [`Self::from_deref()`] apply.
    unsafe fn pin_from_deref<'a>(v: pin::Pin<OnceRef<'a, Self::Target>>) -> pin::Pin<Self> {
        // SAFETY: Pinned types must ensure they uphold pinning guarantees
        //         just like `Deref` does (see trait requirements). Also, the
        //         caller must ensure the original value was pinned.
        unsafe { crate::pin::map_unchecked(v, |v| Self::from_deref(v)) }
    }
}

mod lib {
    use super::*;
    use alloc::{boxed::Box, rc::Rc, sync::Arc};

    unsafe impl<T: ?Sized> IntoDeref for &T {
        fn into_deref<'a>(v: Self) -> OnceRef<'a, Self::Target> where Self: 'a {
            OnceRef::from_ref(v)
        }
    }

    unsafe impl<T: ?Sized> FromDeref for &T {
        unsafe fn from_deref<'a>(v: OnceRef<'a, Self::Target>) -> Self {
            unsafe { v.into_nonnull().as_ref() }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for &mut T {
        fn into_deref<'a>(v: Self) -> OnceRef<'a, Self::Target> where Self: 'a {
            OnceRef::from_mut(v)
        }
    }

    unsafe impl<T: ?Sized> FromDeref for &mut T {
        unsafe fn from_deref<'a>(v: OnceRef<'a, Self::Target>) -> Self {
            unsafe { v.into_nonnull().as_mut() }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for Box<T> {
        fn into_deref<'a>(v: Self) -> OnceRef<'a, Self::Target> where Self: 'a {
            unsafe {
                OnceRef::from_ptr(Box::into_raw(v))
            }
        }
    }

    unsafe impl<T: ?Sized> FromDeref for Box<T> {
        unsafe fn from_deref<'a>(v: OnceRef<'a, Self::Target>) -> Self {
            unsafe { Box::from_raw(v.into_nonnull().as_ptr()) }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for Rc<T> {
        fn into_deref<'a>(v: Self) -> OnceRef<'a, Self::Target> where Self: 'a {
            unsafe {
                OnceRef::from_ptr(Rc::into_raw(v) as *mut _)
            }
        }
    }

    unsafe impl<T: ?Sized> FromDeref for Rc<T> {
        unsafe fn from_deref<'a>(v: OnceRef<'a, Self::Target>) -> Self {
            unsafe { Rc::from_raw(v.into_nonnull().as_ptr()) }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for Arc<T> {
        fn into_deref<'a>(v: Self) -> OnceRef<'a, Self::Target> where Self: 'a {
            unsafe { OnceRef::from_ptr(Arc::into_raw(v) as *mut _) }
        }
    }

    unsafe impl<T: ?Sized> FromDeref for Arc<T> {
        unsafe fn from_deref<'a>(v: OnceRef<'a, Self::Target>) -> Self {
            unsafe { Arc::from_raw(v.into_nonnull().as_ptr()) }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::{boxed::Box, rc::Rc, sync::Arc};

    #[test]
    fn basic_into_from_deref() {
        let mut v: u64 = 71;

        {
            let p: *const u64 = &raw const v;
            let f: &u64 = &v;

            let d: OnceRef<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d.as_ref());
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: &u64 = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, r));
        }

        {
            let p: *mut u64 = &raw mut v;
            let f: &mut u64 = &mut v;

            let d: OnceRef<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d.as_ref());
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: &mut u64 = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, r));
        }

        {
            let f: Box<u64> = Box::new(v);
            let p: *const u64 = &raw const *f;

            let d: OnceRef<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d);
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: Box<u64> = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, &raw const *r));
        }

        {
            let f: Rc<u64> = Rc::new(v);
            let p: *const u64 = &raw const *f;

            let d: OnceRef<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d);
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: Rc<u64> = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, &raw const *r));
        }

        {
            let f: Arc<u64> = Arc::new(v);
            let p: *const u64 = &raw const *f;

            let d: OnceRef<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d);
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: Arc<u64> = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, &raw const *r));
        }
    }
}
