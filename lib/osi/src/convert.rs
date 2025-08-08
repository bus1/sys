//! # Utilities for conversions between types
//!
//! This module contains utilities to help dealing with conversions between
//! types.

use core::pin;

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
/// ## Mutability
///
/// [`IntoDeref`] serves for immutable and mutable conversions. The returned
/// wrapper type properly wraps both mutable and immutable references. It never
/// hands out safe mutable access, but also prevents safe creation of copies.
///
/// See [`DerefPtr`] for details.
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
    fn into_deref<'a>(v: Self) -> DerefPtr<'a, Self::Target> where Self: 'a;

    /// Convert a pinned value into a wrapped pointer to its dereferenced
    /// value.
    ///
    /// This is the pinned equivalent of [`Self::into_deref()`].
    fn pin_into_deref<'a>(v: pin::Pin<Self>) -> DerefPtr<'a, Self::Target>
    where
        pin::Pin<Self>: 'a,
    {
        // SAFETY: Pinned types must ensure they uphold pinning guarantees
        //         just like `Deref` does (see trait requirements).
        Self::into_deref(unsafe { pin::Pin::into_inner_unchecked(v) })
    }
}

/// Convert a wrapped pointer to a dereferenced value back to its original
/// value.
///
/// This trait provides the inverse operation of [`IntoDeref`]. It takes a
/// pointer to a dereferenced value and restores the original smart pointer.
/// This operation is unsafe and requires the caller to guarantee that the
/// pointer was acquired via [`IntoDeref`] or similar means.
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
    unsafe fn from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> Self;

    /// Convert a wrapped pointer to a dereferenced value back to its original
    /// pinned value.
    ///
    /// This is the pinned equivalent of [`Self::from_deref()`].
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that the original value was a pinned pointer.
    /// Furthermore, all requirements of [`Self::from_deref()`] apply.
    unsafe fn pin_from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> pin::Pin<Self> {
        // SAFETY: Pinned types must ensure they uphold pinning guarantees
        //         just like `Deref` does (see trait requirements). Also, the
        //         caller must ensure the original value was pinned.
        unsafe { core::pin::Pin::new_unchecked(Self::from_deref(v)) }
    }
}

/// A wrapped pointer to a dereferenced value.
///
/// In and of itself, this type is a wrapper around a
/// [`core::ptr::NonNull<T>`] for a fixed lifetime `'a`. It can be safely
/// dereferenced to a `&'a T`. However, this type is not equivalent to a mere
/// reference. In particular, it differs in the following ways:
///
///  1) It cannot be cloned or copied. This ensures that if it was
///     created from a mutable reference, no copies of it can be created.
///  2) It is always rooted in a raw pointer. That is, any
///     reference it hands out is always temporary and cannot outlive it. This
///     allows better compatibility with Miri Stacked Borrows and Tree Borrows.
///
/// [`DerefPtr`] can be used on its own, but note that it partakes in
/// [`IntoDeref`] and [`FromDeref`] and is designed exactly for those traits.
/// Hence, it might have unsuitable restrictions for other use-cases.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct DerefPtr<'a, T: ?Sized> {
    ptr: core::ptr::NonNull<T>,
    _ref: core::marker::PhantomData<&'a T>,
}

unsafe impl<'a, T: ?Sized + Send + Sync> Send for DerefPtr<'a, T> {
}

unsafe impl<'a, T: ?Sized + Send + Sync> Sync for DerefPtr<'a, T> {
}

impl<'a, T: ?Sized> DerefPtr<'a, T> {
    unsafe fn new(v: core::ptr::NonNull<T>) -> Self {
        Self {
            ptr: v,
            _ref: Default::default(),
        }
    }

    /// Create a new instance from a [`NonNull`](core::ptr::NonNull).
    ///
    /// ## Safety
    ///
    /// The non-null pointer must pointer to a valid object of type `T`, and it
    /// must be valid for at least lifetime `'a`.
    ///
    /// No mutable reference to the object must be live for at least `'a`. That
    /// is, it must be safe for this wrapper to immutably borrow the object for
    /// `'a`.
    pub unsafe fn from_nonnull(v: core::ptr::NonNull<T>) -> Self {
        unsafe { Self::new(v) }
    }

    /// Create a new instance from a raw pointer.
    ///
    /// ## Safety
    ///
    /// See [`Self::from_nonnull()`].
    pub unsafe fn from_ptr(v: *const T) -> Self {
        unsafe { Self::new(core::ptr::NonNull::new_unchecked(v as *mut _)) }
    }

    /// Create a new instance from a reference.
    pub fn from_ref(v: &'a T) -> Self {
        unsafe { Self::new(crate::ptr::nonnull_from_ref(v)) }
    }

    /// Create a new instance from a mutable reference.
    pub fn from_mut(v: &'a mut T) -> Self {
        unsafe { Self::new(crate::ptr::nonnull_from_mut(v)) }
    }

    /// Convert this into its underlying [`NonNull`](core::ptr::NonNull).
    pub fn into_nonnull(self) -> core::ptr::NonNull<T> {
        self.ptr
    }

    /// Convert this into its underlying raw pointer.
    pub fn into_ptr(self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Borrow the underlying [`NonNull`](core::ptr::NonNull).
    pub fn as_nonnull(&self) -> core::ptr::NonNull<T> {
        self.ptr
    }

    /// Borrow the underlying raw pointer.
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Dereference this wrapper to the pointed object.
    pub fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T: ?Sized> core::ops::Deref for DerefPtr<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.deref()
    }
}

impl<'a, T> From<T> for DerefPtr<'a, T::Target>
where
    T: 'a + core::ops::Deref + IntoDeref,
{
    fn from(v: T) -> Self {
        IntoDeref::into_deref(v)
    }
}

impl<'a, T: ?Sized> From<DerefPtr<'a, T>> for core::ptr::NonNull<T> {
    fn from(v: DerefPtr<'a, T>) -> Self {
        v.into_nonnull()
    }
}

impl<'a, T: ?Sized> From<DerefPtr<'a, T>> for *const T {
    fn from(v: DerefPtr<'a, T>) -> Self {
        v.into_ptr()
    }
}

mod lib {
    use super::*;
    use alloc::{boxed::Box, rc::Rc, sync::Arc};

    unsafe impl<T: ?Sized> IntoDeref for &T {
        fn into_deref<'a>(v: Self) -> DerefPtr<'a, Self::Target> where Self: 'a {
            DerefPtr::from_ref(v)
        }
    }

    unsafe impl<T: ?Sized> FromDeref for &T {
        unsafe fn from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> Self {
            unsafe { v.into_nonnull().as_ref() }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for &mut T {
        fn into_deref<'a>(v: Self) -> DerefPtr<'a, Self::Target> where Self: 'a {
            DerefPtr::from_mut(v)
        }
    }

    unsafe impl<T: ?Sized> FromDeref for &mut T {
        unsafe fn from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> Self {
            unsafe { v.into_nonnull().as_mut() }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for Box<T> {
        fn into_deref<'a>(v: Self) -> DerefPtr<'a, Self::Target> where Self: 'a {
            unsafe {
                DerefPtr::from_ptr(Box::into_raw(v))
            }
        }
    }

    unsafe impl<T: ?Sized> FromDeref for Box<T> {
        unsafe fn from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> Self {
            unsafe { Box::from_raw(v.into_nonnull().as_ptr()) }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for Rc<T> {
        fn into_deref<'a>(v: Self) -> DerefPtr<'a, Self::Target> where Self: 'a {
            unsafe {
                DerefPtr::from_ptr(Rc::into_raw(v))
            }
        }
    }

    unsafe impl<T: ?Sized> FromDeref for Rc<T> {
        unsafe fn from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> Self {
            unsafe { Rc::from_raw(v.into_nonnull().as_ptr()) }
        }
    }

    unsafe impl<T: ?Sized> IntoDeref for Arc<T> {
        fn into_deref<'a>(v: Self) -> DerefPtr<'a, Self::Target> where Self: 'a {
            unsafe { DerefPtr::from_ptr(Arc::into_raw(v)) }
        }
    }

    unsafe impl<T: ?Sized> FromDeref for Arc<T> {
        unsafe fn from_deref<'a>(v: DerefPtr<'a, Self::Target>) -> Self {
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

            let d: DerefPtr<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d.deref());
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: &u64 = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, r));
        }

        {
            let p: *mut u64 = &raw mut v;
            let f: &mut u64 = &mut v;

            let d: DerefPtr<u64> = IntoDeref::into_deref(f);
            assert_eq!(71, *d.deref());
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: &mut u64 = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, r));
        }

        {
            let f: Box<u64> = Box::new(v);
            let p: *const u64 = &raw const *f;

            let d: DerefPtr<u64> = f.into();
            assert_eq!(71, *d);
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: Box<u64> = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, &raw const *r));
        }

        {
            let f: Rc<u64> = Rc::new(v);
            let p: *const u64 = &raw const *f;

            let d: DerefPtr<u64> = f.into();
            assert_eq!(71, *d);
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: Rc<u64> = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, &raw const *r));
        }

        {
            let f: Arc<u64> = Arc::new(v);
            let p: *const u64 = &raw const *f;

            let d: DerefPtr<u64> = f.into();
            assert_eq!(71, *d);
            assert!(core::ptr::eq(p, d.as_ptr()));

            let r: Arc<u64> = unsafe { FromDeref::from_deref(d) };
            assert_eq!(71, *r);
            assert!(core::ptr::eq(p, &raw const *r));
        }
    }
}
