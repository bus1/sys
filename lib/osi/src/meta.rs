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
/// Any type that implements `Field<OFFSET, T>` is guaranteed to have a member
/// field at relative offset `OFFSET`. The type of the member field is `T`.
///
/// Note that multiple fields might share an offset if at least on of them
/// is a zero-sized type.
///
/// ## Dynamically-sized Types (DSTs)
///
/// This trait works for dynamically sized types (both for the container as
/// well as the field). However, the offset of the field in question must be
/// statically known. Furthermore, in case of dynamically-sized types, offset
/// calculations will not be able to deduce the size of the container or member
/// field, and thus cannot create raw pointers or even references (since Rust
/// uses metadata for DSTs even on raw pointers).
///
/// Long story short: If your types are not `Sized`, the provided helpers will
/// likely not apply.
///
/// ## Safety
///
/// Any implementation must guarantee that the implementing type has a member
/// field of type `T` at offset `OFFSET` relative to its starting address. The
/// member field must follow standard alignment and sizing requirements.
///
/// The member field must be a direct member of the structure (rather than
/// nested in sub-structures). In particular, its type must match exactly `T`.
///
/// ## Example
///
/// ```rust
/// use core::mem::offset_of;
/// use osi::mem::typed_offset_of;
/// use osi::meta::Field;
///
/// struct Position {
///     x: u8,
///     y: u16,
///     z: u8,
/// }
///
/// unsafe impl Field<{typed_offset_of!(Position, x, u8)}, u8> for Position {
/// }
///
/// unsafe impl Field<{typed_offset_of!(Position, y, u16)}, u16> for Position {
/// }
///
/// unsafe impl Field<{typed_offset_of!(Position, z, u8)}, u8> for Position {
/// }
///
/// let pos = Position { x: 11, y: 1444, z: 71 };
/// let pos_x = osi::meta::field_of::<{typed_offset_of!(Position, x, u8)}, u8, _>(&pos);
/// let pos_y = osi::meta::field_of::<{typed_offset_of!(Position, y, u16)}, u16, _>(&pos);
/// let pos_z = osi::meta::field_of::<{typed_offset_of!(Position, z, u8)}, u8, _>(&pos);
///
/// // With Rust-1.89 and later, you can infer the offset if unambiguous:
/// // let pos_y = osi::meta::field_of::<_, u16, _>(&pos);
///
/// assert_eq!(*pos_x, pos.x);
/// assert_eq!(*pos_y, pos.y);
/// assert_eq!(*pos_z, pos.z);
/// ```
pub unsafe trait Field<const OFFSET: usize, T: ?Sized> {
}

/// Turn a container pointer into a member field pointer.
///
/// This is equivalent to taking a raw pointer to a member field
/// `&raw mut (*container).field`. Note that the container is not
/// dereferenced for this operation, since this merely needs a place
/// expression.
pub fn field_of_ptr<const OFFSET: usize, F, C>(v: *mut C) -> *mut F
where
    C: ?Sized + Field<OFFSET, F>,
{
    v.wrapping_byte_offset(OFFSET as isize).cast()
}

/// Turn a container reference into a member field reference.
///
/// This is equivalent to taking a reference to a member field
/// `&container.field`. However, this function will always borrow the entire
/// container for as long as the field is borrowed (i.e., you cannot split
/// borrows across multiple fields).
///
/// This function uses the [`Field`] trait internally.
pub fn field_of<const OFFSET: usize, F, C>(v: &C) -> &F
where
    C: ?Sized + Field<OFFSET, F>,
{
    // SAFETY: `Field` guarantees that there is a valid member field at the
    //         specified offset. Hence, `field_of_ptr()` cannot wrap around nor
    //         leave the allocation.
    //         Since we have the entire container borrowed, we can safely hand
    //         out sub-borrows.
    unsafe { &*field_of_ptr(v as *const C as *mut C) }
}

/// Turn a mutable container reference into a mutable member field reference.
///
/// This is equivalent to taking a mutable reference to a member field
/// `&mut container.field`. However, this function will always mutably borrow
/// the entire container for as long as the field is borrowed (i.e., you cannot
/// split borrows across multiple fields).
///
/// This function uses the [`Field`] trait internally.
pub fn field_of_mut<const OFFSET: usize, F, C>(v: &mut C) -> &mut F
where
    C: ?Sized + Field<OFFSET, F>,
{
    // SAFETY: `Field` guarantees that there is a valid member field at the
    //         specified offset. Hence, `field_of_ptr()` cannot wrap around nor
    //         leave the allocation.
    //         Since we have the entire container borrowed, we can safely hand
    //         out sub-borrows.
    unsafe { &mut *(field_of_ptr(v)) }
}

/// Turn a field pointer into a container pointer
///
/// This is the inverse of [`field_of_ptr()`]. It recreates the container
/// pointer from the member field pointer.
///
/// ## Miri Stacked & Tree Borrows
///
/// If you require compatibility with Stacked Borrows as used in Miri, you must
/// ensure that the field pointer was created from a reference to the
/// container, rather than from a reference to the field. In other words, make
/// sure that you use [`field_of_ptr()`], rather than [`field_of()`], and then
/// retain that raw field pointer until you need it for [`container_of_ptr()`].
/// Otherwise, your code will likely not be compatible with Stacked Borrows.
///
/// If you only require compatibility with Tree Borrows, this is not an issue.
pub fn container_of_ptr<const OFFSET: usize, F, C>(v: *mut F) -> *mut C
where
    C: Field<OFFSET, F>,
    F: ?Sized,
{
    v.wrapping_byte_offset(-(OFFSET as isize)).cast()
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    #[repr(C, align(4))]
    struct Test {
        a: u16,
        b: u8,
        c: u32,
    }

    unsafe impl Field<{crate::mem::typed_offset_of!(Test, b, u8)}, u8> for Test {
    }

    // Basic functionality tests for `Field`.
    #[test]
    fn basic_field() {
        assert_eq!(core::mem::size_of::<Test>(), 8);

        let o = Test { a: 14, b: 11, c: 1444 };
        let o_p = &raw const o as *mut Test;

        let f = field_of(&o);
        assert_eq!(*f, 11);

        let f_p = field_of_ptr(o_p);
        let f_r = unsafe { &*f_p };
        let c_p = container_of_ptr(f_p);
        let c_r = unsafe { &*c_p };

        assert!(core::ptr::eq(o_p, c_p));
        assert_eq!(*f_r, 11);
        assert_eq!(c_r.b, 11);
    }
}
