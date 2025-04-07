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

// Manually create a copy of `v`, reversing the order of all underlying bytes.
// This is the slow-path that manually iterates the individual bytes, but works
// with any data-type, as long as the data-type stays valid with swapped bytes.
const unsafe fn bswap_slow<T: Copy>(v: T) -> T {
    let mut r = core::mem::MaybeUninit::<T>::uninit();
    let src = &v as *const T as *const u8;
    let dst = r.as_mut_ptr() as *mut u8;

    unsafe {
        let mut i = 0;
        while i < size_of::<T>() {
            core::ptr::copy(
                src.add(size_of::<T>() - i - 1),
                dst.add(i),
                1,
            );
            i += 1;
        }

        r.assume_init()
    }
}

/// Reverse the order of all bytes in `v`.
///
/// This reverses the order of all bytes underlying the object `v`. That is,
/// its last byte will be swapped with the first, its second to last byte
/// will be swapped with the second, and so on. In case of an odd number of
/// bytes, the middle byte will stay untouched.
///
/// ## Safety
///
/// The caller must guarantee that `T` remains valid after all bytes were
/// swapped.
pub const unsafe fn bswap<T: Copy>(v: T) -> T {
    // SAFETY: The caller guarantees that `T` is valid with all bytes swapped.
    //         And due to `T: Copy`, we can safely create memory copies.
    unsafe {
        match size_of::<T>() {
            1 => transmute_copy(&v),
            2 => transmute_copy(&u16::swap_bytes(transmute_copy(&v))),
            4 => transmute_copy(&u32::swap_bytes(transmute_copy(&v))),
            8 => transmute_copy(&u64::swap_bytes(transmute_copy(&v))),
            16 => transmute_copy(&u128::swap_bytes(transmute_copy(&v))),
            _ => bswap_slow(v),
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

    // Verify basic byte swapping
    #[test]
    fn bswap_basic() {
        unsafe {
            assert_eq!(bswap(0x12u8), 0x12u8);
            assert_eq!(bswap(0x1234u16), 0x3412u16);
            assert_eq!(bswap(0x12345678u32), 0x78563412u32);
            assert_eq!(bswap(0x0011223344556677u64), 0x7766554433221100u64);
            assert_eq!(bswap(0x00112233445566778899101112131415u128), 0x15141312111099887766554433221100u128);
            assert_eq!(bswap([0x00u8, 0x11u8, 0x22u8]), [0x22u8, 0x11u8, 0x00u8]);
        }
    }
}
