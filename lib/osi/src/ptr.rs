//! # Utilities to manage memory through raw pointers
//!
//! This module contains utilities to manage memory through raw pointers and
//! convert them to/from safe Rust types.
//!
//! ## Conversion
// ^This title is linked to via [](osi::ptr#conversion) so keep it stable.
//!
//! The Rust standard library defines rules for converting pointers to a
//! reference, as well as for dereferencing pointers (see
//! [core::ptr#Safety](core::ptr#safety), and
//! [core::ptr#Conversion](core::ptr#pointer-to-reference-conversion)).
//! Whenever the documentation here mentions `convertible to a reference`,
//! these documents define the precise requirements.
//!
//! In addition to the official documentation, if no specific reference type
//! is mentioned, then `convertible to a reference` does not include the
//! aliasing requirements. That is, those requirements need only to be upheld
//! if an explicit reference type is used (i.e., `convertible to a
//! {mutable,shared} reference`).

/// A [`core::ptr::NonNull`] but 4-byte aligned.
///
/// This transparently wraps [`core::ptr::NonNull`], but requires the embedded
/// pointer to be 4-byte aligned (on top of it being non-null). This invariant
/// is maintained.
///
/// Since every 4-byte aligned pointer has 2 unused bits, this wrapper exposes
/// an API to track additional metadata in those 2 bits.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct NonNull4<T: ?Sized> {
    ptr: core::ptr::NonNull<T>,
}

/// Lifetime annotated pointers behave like `NonNull<T>` but point to a valid
/// allocation for the given lifetime.
///
/// Unlike references, `Ptr` does not make any guarantees about immutability or
/// aliasing of the target allocation.
///
/// A `Ptr` is always [convertible to a reference](self#conversion).
#[repr(transparent)]
pub struct Ptr<'a, T: ?Sized> {
    inner: core::ptr::NonNull<T>,
    _ref: core::marker::PhantomData<&'a T>,
    _mut: core::marker::PhantomData<&'a mut T>,
}

/// A singular pointer behaving like an immutable reference.
///
/// A `OnceRef` can be used multiple times, but only exists once for a given
/// source reference, hence its name.
///
/// This type behaves like a normal reference `&'a T`, with the following
/// differences:
///
/// - It does not implement [`Clone`] or [`Copy`]. That is, any reference
///   created from it will necessarily borrow [`Self`] and thus cannot outlive
///   it, even if `'a` is a significantly greater lifetime. This means, any
///   function that consumes [`Self`] can safely assume that no reference
///   derived from [`Self`] can exist.
///   If other references to the same object existed before [`Self`] was
///   created, they can still be used. However, if no such references existed,
///   this instance will be guaranteed to be the only one.
/// - It is always rooted in a raw pointer, rather than a reference. This has
///   no visible effect, but might be relevant for compatibility with Stacked
///   Borrows.
/// - It is invariant over `T` (rather than covariant). This matches the
///   behavior of `&'a mut T`.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct OnceRef<'a, T: ?Sized> {
    ptr: core::ptr::NonNull<T>,
    _ref: core::marker::PhantomData<&'a T>,
    _mut: core::marker::PhantomData<&'a mut T>,
}

/// Convert an immutable reference to a `NonNull` pointer.
///
/// This is equivalent to [`core::ptr::NonNull::from_ref()`].
//
// MSRV(1.89): This is available in the Rust standard library as
//             [`core::ptr::NonNull::from_ref()`] but higher than our MSRV.
pub const fn nonnull_from_ref<T: ?Sized>(v: &T) -> core::ptr::NonNull<T> {
    // SAFETY: A reference cannot be null.
    unsafe { core::ptr::NonNull::new_unchecked(v as *const T as *mut T) }
}

/// Convert a mutable reference to a `NonNull` pointer.
///
/// This is equivalent to [`core::ptr::NonNull::from_mut()`].
//
// MSRV(1.89): This is available in the Rust standard library as
//             [`core::ptr::NonNull::from_mut()`] but higher than our MSRV.
pub const fn nonnull_from_mut<T: ?Sized>(v: &mut T) -> core::ptr::NonNull<T> {
    // SAFETY: A reference cannot be null.
    unsafe { core::ptr::NonNull::new_unchecked(v as *mut T) }
}

/// Convert a pinned reference to a `NonNull` pointer.
pub fn nonnull_from_pin_ref<T>(v: core::pin::Pin<T>) -> core::ptr::NonNull<T::Target>
where
    T: core::ops::Deref,
{
    // SAFETY: A pin cannot be null.
    unsafe { core::ptr::NonNull::new_unchecked(crate::pin::as_ptr(v) as *mut _) }
}

/// Convert a pinned mutable reference to a `NonNull` pointer.
///
/// This yields the same value as [`nonnull_from_pin_ref()`] but ensures the
/// pointer is created from a mutable reference via [`core::ops::DerefMut`].
/// This can be relevant when striving for compatibility with Stacked Borrows.
pub fn nonnull_from_pin_mut<T>(v: core::pin::Pin<T>) -> core::ptr::NonNull<T::Target>
where
    T: core::ops::DerefMut,
{
    // SAFETY: A pin cannot be null.
    unsafe { core::ptr::NonNull::new_unchecked(crate::pin::as_mut_ptr(v)) }
}

impl<T: ?Sized> NonNull4<T> {
    const MASK_META: usize = 0x3usize;
    const MASK_ADDR: usize = !Self::MASK_META;

    /// Create a new 4-byte aligned non-null pointer.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that the pointer is 4-byte aligned. That is,
    /// its lowest 2 bits must not be set.
    pub const unsafe fn new_unchecked(v: core::ptr::NonNull<T>) -> Self {
        Self {
            ptr: v,
        }
    }

    /// Create a new 4-byte aligned non-null pointer.
    ///
    /// If the provided non-null pointer is not aligned to 4-bytes, this will
    /// yield `None`. Otherwise, a new [`NonNull4`] is created.
    ///
    /// Use [`Self::new_unchecked()`] to skip the test.
    pub fn new(v: core::ptr::NonNull<T>) -> Option<Self> {
        if (v.addr().get() & Self::MASK_META) == 0 {
            // SAFETY: `v` is already guaranteed to be non-zero, so if it does
            //         not carry metadata, its address must be non-zero.
            unsafe { Some(Self::new_unchecked(v)) }
        } else {
            None
        }
    }

    /// Yield the pointer value.
    ///
    /// This will yield the embedded non-null pointer, with any metadata
    /// cleared. That is, the pointer value is guaranteed to be 4-byte aligned.
    pub fn ptr(&self) -> core::ptr::NonNull<T> {
        // SAFETY: The pointer value is a 4-byte aligned non-null pointer.
        //         Stripping the metadata cannot yield a zero value.
        self.ptr.map_addr(|v| unsafe {
            core::num::NonZero::new_unchecked(v.get() & Self::MASK_ADDR)
        })
    }

    /// Yield the metadata.
    ///
    /// This will yield the embedded metadata without the pointer value. That
    /// is, it will yield an integer smaller than 4.
    pub fn meta(&self) -> usize {
        self.ptr.addr().get() & Self::MASK_META
    }

    /// Yield a specific bit of the metadata.
    ///
    /// Yield only the bit at index `bit` of the metadata. This returns `true`
    /// if the bit is set, `false` otherwise.
    pub fn meta_bit(&self, bit: u8) -> bool {
        self.meta() & (1usize << bit) == 1usize << bit
    }

    /// Yield only bit 0 of the metadata.
    pub fn get0(&self) -> bool {
        self.meta_bit(0)
    }

    /// Yield only bit 1 of the metadata.
    pub fn get1(&self) -> bool {
        self.meta_bit(1)
    }

    /// Modify the pointer value while retaining the metadata.
    ///
    /// ## Safety
    ///
    /// The provided pointer must be 4-byte aligned.
    pub unsafe fn set_ptr_unchecked(&mut self, ptr: core::ptr::NonNull<T>) {
        // SAFETY: The caller must guarantee `ptr` is a non-null 4-byte aligned
        //         value, which thus is still non-zero if metadata is stripped.
        self.ptr = ptr.map_addr(|v| unsafe {
            core::num::NonZero::new_unchecked(
                (v.get() & Self::MASK_ADDR) | self.meta(),
            )
        });
    }

    /// Modify the metadata while retaining the pointer value.
    ///
    /// Any bits in `meta` outside of the metadata range is silently ignored.
    pub fn set_meta(&mut self, meta: usize) {
        // SAFETY: We know the pointer value is 4-byte aligned and non-zero,
        //         so the result is still non-zero when metadata is or'ed.
        self.ptr = self.ptr.map_addr(|v| unsafe {
            core::num::NonZero::new_unchecked(
                (v.get() & Self::MASK_ADDR) | (meta & Self::MASK_META),
            )
        });
    }

    /// Modify a specific metadata bit while retaining everything else.
    pub fn set_meta_bit(&mut self, bit: u8, flag: bool) {
        self.set_meta(
            (self.meta() & !(1usize << bit)) | ((flag as usize) << bit),
        )
    }

    /// Modify metadata bit 0 while retaining everything else.
    pub fn set0(&mut self, flag: bool) {
        self.set_meta_bit(0, flag)
    }

    /// Modify metadata bit 1 while retaining everything else.
    pub fn set1(&mut self, flag: bool) {
        self.set_meta_bit(1, flag)
    }
}

impl<'a, T: ?Sized> Ptr<'a, T> {
    /// Create a new instance from a [`NonNull`](core::ptr::NonNull).
    ///
    /// ## Safety
    ///
    /// `v` must be [convertible to a reference](self#conversion) for the
    /// lifetime `'a`.
    pub const unsafe fn new(v: core::ptr::NonNull<T>) -> Self {
        Self {
            inner: v,
            _ref: core::marker::PhantomData,
            _mut: core::marker::PhantomData,
        }
    }

    /// Create a new instance from a raw pointer.
    ///
    /// ## Safety
    ///
    /// `v` must be [convertible to a reference](self#conversion) for the
    /// lifetime `'a`.
    pub const unsafe fn from_ptr(v: *mut T) -> Self {
        unsafe { Self::new(core::ptr::NonNull::new_unchecked(v)) }
    }

    /// Create a new instance from a shared reference.
    pub const fn from_ref(v: &'a T) -> Self {
        // SAFETY: References are naturally convertible to a reference for
        //         their entire lifetime.
        unsafe { Self::new(crate::ptr::nonnull_from_ref(v)) }
    }

    /// Convert this into its underlying [`NonNull`](core::ptr::NonNull).
    pub const fn into_nonnull(self) -> core::ptr::NonNull<T> {
        self.inner
    }

    /// Convert this into its underlying raw pointer.
    pub const fn into_ptr(self) -> *mut T {
        self.inner.as_ptr()
    }

    /// Convert this into a proper reference.
    ///
    /// ## Safety
    ///
    /// The aliasing requirements of shared references must be guaranteed.
    pub const unsafe fn into_ref(self) -> &'a T {
        // SAFETY: Propagated to caller.
        unsafe { self.inner.as_ref() }
    }

    /// Convert this into a proper mutable reference.
    ///
    /// ## Safety
    ///
    /// The aliasing requirements of mutable references must be guaranteed.
    pub const unsafe fn into_mut(mut self) -> &'a mut T {
        // SAFETY: Propagated to caller.
        unsafe { self.inner.as_mut() }
    }

    /// Borrow the underlying [`NonNull`](core::ptr::NonNull).
    pub const fn as_nonnull(&self) -> core::ptr::NonNull<T> {
        self.inner
    }

    /// Borrow the underlying raw pointer.
    pub const fn as_ptr(&self) -> *mut T {
        self.inner.as_ptr()
    }

    /// Dereference this wrapper to the pointed object.
    ///
    /// ## Safety
    ///
    /// The aliasing requirements of shared references must be guaranteed.
    pub const unsafe fn as_ref(&self) -> &T {
        unsafe { self.inner.as_ref() }
    }

    /// Mutably dereference this wrapper to the pointed object.
    ///
    /// ## Safety
    ///
    /// The aliasing requirements of mutable references must be guaranteed.
    pub const unsafe fn as_mut(&mut self) -> &mut T {
        unsafe { self.inner.as_mut() }
    }
}

// `Ref` behaves like `&'a T` and `&'a mut T` combined.
unsafe impl<'a, T: ?Sized + Send + Sync> Send for Ptr<'a, T> {
}

// `Ref` behaves like `&'a T` and `&'a mut T` combined.
unsafe impl<'a, T: ?Sized + Sync> Sync for Ptr<'a, T> {
}

impl<'a, T: ?Sized> core::clone::Clone for Ptr<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: ?Sized> core::marker::Copy for Ptr<'a, T> {
}

impl<'a, T: ?Sized> core::fmt::Debug for Ptr<'a, T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("Ptr").field(&self.inner).finish()
    }
}

impl<'a, T: ?Sized> core::cmp::Eq for Ptr<'a, T> {
}

impl<'a, T: ?Sized> core::hash::Hash for Ptr<'a, T> {
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        self.inner.hash(state)
    }
}

impl<'a, T: ?Sized> core::cmp::Ord for Ptr<'a, T> {
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<'a, T: ?Sized> core::cmp::PartialEq for Ptr<'a, T> {
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<'a, T: ?Sized> core::cmp::PartialOrd for Ptr<'a, T> {
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ptr<'a, T> {
    fn from(v: &'a T) -> Self {
        Self::from_ref(v)
    }
}

impl<'a, T: ?Sized> From<Ptr<'a, T>> for core::ptr::NonNull<T> {
    fn from(v: Ptr<'a, T>) -> Self {
        v.into_nonnull()
    }
}

impl<'a, T: ?Sized> From<Ptr<'a, T>> for *mut T {
    fn from(v: Ptr<'a, T>) -> Self {
        v.into_ptr()
    }
}

impl<'a, T: ?Sized> From<Ptr<'a, T>> for *const T {
    fn from(v: Ptr<'a, T>) -> Self {
        v.into_ptr()
    }
}

impl<'a, T: ?Sized> OnceRef<'a, T> {
    /// Create a new instance from a [`NonNull`](core::ptr::NonNull).
    ///
    /// ## Safety
    ///
    /// When calling this method, you have to ensure that the pointer is
    /// [convertible to a reference](core::ptr#pointer-to-reference-conversion).
    pub unsafe fn from_nonnull(v: core::ptr::NonNull<T>) -> Self {
        Self {
            ptr: v,
            _ref: Default::default(),
            _mut: Default::default(),
        }
    }

    /// Create a new instance from a raw pointer.
    ///
    /// ## Safety
    ///
    /// When calling this method, you have to ensure that the pointer is
    /// [convertible to a reference](core::ptr#pointer-to-reference-conversion).
    pub unsafe fn from_ptr(v: *mut T) -> Self {
        unsafe { Self::from_nonnull(core::ptr::NonNull::new_unchecked(v)) }
    }

    /// Create a new instance from a reference.
    pub fn from_ref(v: &'a T) -> Self {
        unsafe { Self::from_nonnull(crate::ptr::nonnull_from_ref(v)) }
    }

    /// Create a new instance from a mutable reference.
    pub fn from_mut(v: &'a mut T) -> Self {
        unsafe { Self::from_nonnull(crate::ptr::nonnull_from_mut(v)) }
    }

    /// Create a new pinned instance from a pinned reference.
    pub fn pin_from_ref(v: core::pin::Pin<&'a T>) -> core::pin::Pin<Self> {
        // SAFETY: `OnceRef` honors pinning guarantees, so we can always wrap
        //         pinned references.
        unsafe { crate::pin::map_unchecked(v, Self::from_ref) }
    }

    /// Create a new pinned instance from a pinned mutable reference.
    pub fn pin_from_mut(v: core::pin::Pin<&'a mut T>) -> core::pin::Pin<Self> {
        // SAFETY: `OnceRef` honors pinning guarantees, so we can always wrap
        //         pinned references.
        unsafe { crate::pin::map_unchecked(v, Self::from_mut) }
    }

    /// Convert this into its underlying [`NonNull`](core::ptr::NonNull).
    pub fn into_nonnull(self) -> core::ptr::NonNull<T> {
        self.ptr
    }

    /// Convert this into its underlying raw pointer.
    pub fn into_ptr(self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Convert this into a proper reference.
    pub fn into_ref(self) -> &'a T {
        // SAFETY: The underlying pointer is guaranteed to be convertible to a
        //         reference.
        unsafe { self.ptr.as_ref() }
    }

    /// Convert this into a proper mutable reference.
    ///
    /// ## Safety
    ///
    /// While [`Self`] is guaranteed to be
    /// [convertible to a reference](core::ptr#pointer-to-reference-conversion),
    /// the caller must ensure sufficient exclusiveness guarantees.
    ///
    /// If [`Self`] was created from a mutable reference, this is always safe
    /// to call.
    pub unsafe fn into_mut(mut self) -> &'a mut T {
        // SAFETY: The underlying pointer is guaranteed to be convertible to a
        //         reference. Exclusiveness guarantees are propagated to the
        //         caller.
        unsafe { self.ptr.as_mut() }
    }

    /// Borrow the underlying [`NonNull`](core::ptr::NonNull).
    pub fn as_nonnull(&self) -> core::ptr::NonNull<T> {
        self.ptr
    }

    /// Borrow the underlying raw pointer.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Dereference this wrapper to the pointed object.
    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    /// Mutably dereference this wrapper to the pointed object.
    ///
    /// ## Safety
    ///
    /// While [`Self`] is guaranteed to be
    /// [convertible to a reference](core::ptr#pointer-to-reference-conversion),
    /// the caller must ensure sufficient exclusiveness guarantees.
    ///
    /// If [`Self`] was created from a mutable reference, this is always safe
    /// to call.
    pub unsafe fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

/// Since [`Self`] tries to preserve invariants of immutable and mutable
/// references, both their bounds are required for [`Self`] to be [`Send`].
unsafe impl<'a, T: ?Sized + Send + Sync> Send for OnceRef<'a, T> {
}

/// Since [`Sync`] is the required bound for both immutable and mutable
/// references to implement [`Sync`], it is a sufficient bound for [`Self`].
unsafe impl<'a, T: ?Sized + Sync> Sync for OnceRef<'a, T> {
}

impl<'a, T: ?Sized> core::ops::Deref for OnceRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T: ?Sized> From<&'a T> for OnceRef<'a, T> {
    fn from(v: &'a T) -> Self {
        Self::from_ref(v)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for OnceRef<'a, T> {
    fn from(v: &'a mut T) -> Self {
        Self::from_mut(v)
    }
}

impl<'a, T: ?Sized> From<core::pin::Pin<&'a T>> for core::pin::Pin<OnceRef<'a, T>> {
    fn from(v: core::pin::Pin<&'a T>) -> Self {
        OnceRef::pin_from_ref(v)
    }
}

impl<'a, T: ?Sized> From<core::pin::Pin<&'a mut T>> for core::pin::Pin<OnceRef<'a, T>> {
    fn from(v: core::pin::Pin<&'a mut T>) -> Self {
        OnceRef::pin_from_mut(v)
    }
}

impl<'a, T: ?Sized> From<OnceRef<'a, T>> for core::ptr::NonNull<T> {
    fn from(v: OnceRef<'a, T>) -> Self {
        v.into_nonnull()
    }
}

impl<'a, T: ?Sized> From<OnceRef<'a, T>> for *const T {
    fn from(v: OnceRef<'a, T>) -> Self {
        v.into_ptr()
    }
}

impl<'a, T: ?Sized> From<OnceRef<'a, T>> for *mut T {
    fn from(v: OnceRef<'a, T>) -> Self {
        v.into_ptr()
    }
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

    // Verify `nonnull_from_pin_ref()`.
    #[test]
    fn basic_nonnull_from_pin_ref() {
        let v = core::pin::pin!(71u16);
        let r0 = crate::pin::as_ptr(v.as_ref());
        let r1 = nonnull_from_pin_ref(v.as_ref());
        let r2 = nonnull_from_pin_ref(v);

        assert!(core::ptr::eq(r0, r1.as_ptr()));
        assert!(core::ptr::eq(r0, r2.as_ptr()));
    }

    // Verify `nonnull_from_pin_mut()`.
    #[test]
    fn basic_nonnull_from_pin_mut() {
        let mut v = core::pin::pin!(71u16);
        let r0 = crate::pin::as_mut_ptr(v.as_mut());
        let r1 = nonnull_from_pin_mut(v.as_mut());
        let r2 = nonnull_from_pin_mut(v);

        assert!(core::ptr::eq(r0, r1.as_ptr()));
        assert!(core::ptr::eq(r0, r2.as_ptr()));
    }

    #[test]
    fn basic_nonnull4() {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[repr(align(4))]
        struct Value {
            v: u32,
        }

        let x0: Value = Value { v: 71 };
        let x1: Value = Value { v: 73 };

        let nn_clean0 = nonnull_from_ref(&x0);
        let nn_clean1 = nonnull_from_ref(&x1);
        let nn_dirt1 = nn_clean0.map_addr(|v| v | 1);
        let nn_dirt2 = nn_clean0.map_addr(|v| v | 2);
        let nn_dirt3 = nn_clean0.map_addr(|v| v | 3);
        let nn_dirt4 = nn_clean0.map_addr(|v| v | 4);

        assert_ne!(nn_clean0, nn_clean1);
        assert_ne!(nn_clean0.addr(), nn_clean1.addr());

        assert!(NonNull4::new(nn_clean0).is_some());
        assert!(NonNull4::new(nn_clean1).is_some());
        assert!(NonNull4::new(nn_dirt1).is_none());
        assert!(NonNull4::new(nn_dirt2).is_none());
        assert!(NonNull4::new(nn_dirt3).is_none());
        assert!(NonNull4::new(nn_dirt4).is_some());

        let mut nn4 = unsafe { NonNull4::new_unchecked(nn_clean0) };
        assert_eq!(core::mem::align_of_val(&nn4), core::mem::align_of_val(&nn_clean0));
        assert_eq!(core::mem::size_of_val(&nn4), core::mem::size_of_val(&nn_clean0));
        assert_eq!(NonNull4::new(nn_clean0), Some(nn4));
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 0);
        assert!(!nn4.get0());
        assert!(!nn4.get1());

        nn4.set0(true);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 1);
        assert!(nn4.get0());
        assert!(!nn4.get1());

        nn4.set1(true);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 3);
        assert!(nn4.get0());
        assert!(nn4.get1());

        nn4.set0(false);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 2);
        assert!(!nn4.get0());
        assert!(nn4.get1());

        nn4.set1(false);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 0);
        assert!(!nn4.get0());
        assert!(!nn4.get1());

        unsafe { nn4.set_ptr_unchecked(nn_clean1) };
        assert_eq!(nn4.ptr(), nn_clean1);
        assert_eq!(nn4.meta(), 0);

        nn4.set_meta(3);
        assert_eq!(nn4.ptr(), nn_clean1);
        assert_eq!(nn4.meta(), 3);

        unsafe { nn4.set_ptr_unchecked(nn_clean0) };
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 3);

        nn4.set_meta(2);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 2);

        unsafe { nn4.set_ptr_unchecked(nn_clean1) };
        assert_eq!(nn4.ptr(), nn_clean1);
        assert_eq!(nn4.meta(), 2);
    }

    #[test]
    fn basic_ptr() {
        let v = 71;
        let p = &raw const v as *mut _;

        let r = Ptr::from_ref(&v);
        assert_eq!(unsafe { *r.as_ref() }, 71);
        assert_eq!(r, r);
        assert_eq!(r.as_ptr(), p);
        assert_eq!(unsafe { *r.into_ref() }, 71);
    }

    #[test]
    fn basic_onceref() {
        let mut v = 71;
        let p = &raw mut v;

        let r = OnceRef::from_mut(&mut v);
        assert_eq!(*r, 71);
        assert_eq!(r.as_ptr(), p);

        let r = OnceRef::from_ref(&v);
        assert_eq!(*r, 71);
        assert_eq!(r.as_ptr(), p);
        assert_eq!(*r.into_ref(), 71);
    }
}
