//! # Utilities to manage memory through raw pointers
//!
//! This module contains utilities to manage memory through raw pointers and
//! convert them to/from safe Rust types.

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
        assert_eq!(nn4.get0(), false);
        assert_eq!(nn4.get1(), false);

        nn4.set0(true);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 1);
        assert_eq!(nn4.get0(), true);
        assert_eq!(nn4.get1(), false);

        nn4.set1(true);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 3);
        assert_eq!(nn4.get0(), true);
        assert_eq!(nn4.get1(), true);

        nn4.set0(false);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 2);
        assert_eq!(nn4.get0(), false);
        assert_eq!(nn4.get1(), true);

        nn4.set1(false);
        assert_eq!(nn4.ptr(), nn_clean0);
        assert_eq!(nn4.meta(), 0);
        assert_eq!(nn4.get0(), false);
        assert_eq!(nn4.get1(), false);

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
}
