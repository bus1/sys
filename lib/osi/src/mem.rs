//! # Basic functions for dealing with memory
//!
//! This module contains functions to help dealing with direct memory
//! manipulation and inspection.

use core::mem::transmute_copy;

// Same as [`core::ptr::copy()`] but allows unaligned pointers.
const unsafe fn copy_unaligned<T>(src: *const T, dst: *mut T, count: usize) {
    // SAFETY: We can always alias raw-pointers temporarily. Rust has no
    //         restriction on raw-pointer aliasing.
    //         The size calculation is safe as the caller must guarantee
    //         the source is within a single allocated object, and those
    //         are limited in size to `isize::MAX`.
    unsafe {
        core::ptr::copy(
            src as *const u8,
            dst as *mut u8,
            count * size_of::<T>(),
        )
    }
}

/// Unsafely interprets a copy of `src` as `Dst`.
///
/// This function creates a byte-wise copy of the source data and unsafely
/// interpets the result as a value of type `Dst`. The data is truncated or
/// padded with uninitialized bytes, if necessary.
///
/// This is similar to [`core::mem:transmute_copy`] but allows the types to
/// differ in size.
///
/// ## Safety
///
/// The caller must guarantee that a value of type `Dst` can be safely created
/// with a byte-wise copy of `Src` (truncated or padded with uninitialized
/// bytes, if their size does not match).
#[inline]
#[must_use]
pub const unsafe fn transmute_copy_uninit<Src, Dst>(src: &Src) -> Dst {
    if size_of::<Src>() < size_of::<Dst>() {
        // The source is smaller in size than the destination. Hence, we need
        // an uninitialized buffer that we copy into, and then yield to the
        // caller. Trailing padding is left uninitialized.
        //
        // SAFETY: Delegated to the caller.
        unsafe {
            let mut dst = core::mem::MaybeUninit::<Dst>::uninit();
            copy_unaligned(
                src as *const Src,
                dst.as_mut_ptr() as *mut Src,
                1,
            );
            dst.assume_init()
        }
    } else {
        // The source is larger in size than, or equal to, the destination.
        // Hence, we can read out of the source value and ignore any trailing
        // data. If the source is not suitably aligned, we must ensure a proper
        // unaligned read instruction.
        //
        // SAFETY: Delegated to the caller.
        unsafe {
            if align_of::<Dst>() > align_of::<Src>() {
                core::ptr::read_unaligned(src as *const Src as *const Dst)
            } else {
                core::ptr::read(src as *const Src as *const Dst)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Verify `transmute_copy_uninit()` works in constant contexts.
    #[test]
    fn transmute_copy_uninit_const() {
        const C: u16 = unsafe { transmute_copy_uninit(&71u32) };

        assert_eq!(C, 71);
    }

    // Verify `transmute_copy_uninit()` works with padded/truncated
    // destinations.
    #[test]
    fn transmute_copy_uninit_mismatch() {
        #[derive(Clone, Copy)]
        #[repr(align(4))]
        struct Overaligned {
            v: u16,
        }

        #[derive(Clone, Copy)]
        #[repr(packed)]
        struct Underaligned {
            v: u16,
        }

        assert_eq!(size_of::<Overaligned>(), 4);
        assert_eq!(align_of::<Overaligned>(), 4);
        assert_eq!(size_of::<Underaligned>(), 2);
        assert_eq!(align_of::<Underaligned>(), 1);

        let s: u16 = 71;
        let o: Overaligned = unsafe { transmute_copy_uninit(&s) };
        let or: u16 = unsafe { transmute_copy_uninit(&o) };
        let u: Underaligned = unsafe { transmute_copy_uninit(&s) };
        let ur: u16 = unsafe { transmute_copy_uninit(&u) };

        assert_eq!(s, 71);
        assert_eq!(o.v, 71);
        assert_eq!(or, 71);
        assert_eq!(u.v as u16, 71); // prevent the macro from creating a ref
        assert_eq!(ur, 71);
    }

    }
}
