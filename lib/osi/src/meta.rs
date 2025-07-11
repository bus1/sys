//! # Meta Programming
//!
//! This module provides utilities for meta programming, including (limited)
//! runtime type information, type introspection, or even reflection.

/// Grant generic access to member fields.
///
/// This trait generalizes over member fields of structures. It allows granting
/// access to a specific field without requiring any other knowledge about the
/// containing type.
///
/// Any type that implements `Field<OFFSET, T>` is guaranteed to have a field
/// member at relative offset `OFFSET`. The type of the field member is `T`.
///
/// Note that multiple fields might share an offset if at least on of them
/// is a zero-sized type.
///
/// ## Safety
///
/// Any implementation must guarantee that the implementing type has a member
/// field of type `T` at offset `OFFSET` relative to its starting address. The
/// member field must follow standard alignment and sizing requirements.
///
/// The member field must be a direct member of the structure (rather than
/// nested in sub-structures). In particular, its type must match exactly the
/// type `T`.
///
/// ## Example
///
/// ```rust
/// use core::mem::offset_of;
/// use osi::mem::typed_offset_of;
/// use osi::meta::Field;
///
/// struct Position {
///     x: u32,
///     y: u64,
/// }
///
/// unsafe impl Field<{typed_offset_of!(Position, x, u32)}, u32> for Position {
/// }
///
/// unsafe impl Field<{typed_offset_of!(Position, y, u64)}, u64> for Position {
/// }
///
/// let pos = Position { x: 11, y: 1444 };
/// let pos_x = <_ as Field<{offset_of!{Position, x}}, _>>::field_of(&pos);
/// let pos_y = <_ as Field<{offset_of!{Position, y}}, _>>::field_of(&pos);
/// assert_eq!(*pos_x, pos.x);
/// assert_eq!(*pos_y, pos.y);
/// ```
pub unsafe trait Field<const OFFSET: usize, T>: Sized {
    /// Turn a container reference into a member field reference.
    fn field_of(container: &Self) -> &T {
        // SAFETY: The trait guarantees that there is a valid member field
        //         at the specified offset. Since we have the entire container
        //         borrowed, we can safely hand out sub-borrows.
        unsafe {
            &*(
                (container as *const Self)
                    .byte_offset(OFFSET as isize)
                    .cast::<T>()
            )
        }
    }

    /// Turn a member field reference back into a container reference.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that `field` points to a member field of a
    /// container of type `Self`. Furthermore, the entire container must be
    /// borrowed for at least the lifetime of `field`.
    ///
    /// Note that these guarantees are always given if `field` was acquired
    /// via `Self::field_of()`.
    unsafe fn container_of(field: &T) -> &Self {
        // SAFETY: The caller must guarantee that `field` was borrowed
        //         together with the entire container.
        unsafe {
            &*(
                (field as *const T)
                    .byte_offset(-(OFFSET as isize))
                    .cast::<Self>()
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mem;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    #[repr(C, align(4))]
    struct Test {
        a: u16,
        b: u8,
        c: u32,
    }

    unsafe impl Field<{mem::typed_offset_of!(Test, b, u8)}, u8> for Test {
    }

    // Basic functionality tests for `Field`.
    #[test]
    fn basic_field() {
        assert_eq!(core::mem::size_of::<Test>(), 8);

        let c = Test { a: 14, b: 11, c: 1444 };
        let f = Field::field_of(&c);

        assert_eq!(*f, 11);
        assert!(core::ptr::eq(&c, unsafe { Field::container_of(f) }));
        assert_eq!(c, unsafe { *Field::container_of(f) });
    }
}
