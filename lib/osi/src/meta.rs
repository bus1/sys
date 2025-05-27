//! # Meta Programming
//!
//! This module provides utilities for meta programming, including (limited)
//! runtime type information, type introspection, or even reflection.

/// A view into a container for a specific member field.
///
/// The `FieldView` type wraps a reference to a member field of a container,
/// granting access to both the member field as well as the container.
///
/// Views are usually created via the [`Field`] trait.
#[repr(transparent)]
pub struct FieldView<'this, Container, const OFFSET: usize>
where
    Container: Field<OFFSET>,
{
    field: &'this Container::Value,
}

/// A mutable view into a container for a specific member field.
///
/// The `FieldViewMut` type wraps a mutable reference to a member field of a
/// container, granting access to both the member field as well as the
/// container.
///
/// Views are usually created via the [`Field`] trait.
#[repr(transparent)]
pub struct FieldViewMut<'this, Container, const OFFSET: usize>
where
    Container: Field<OFFSET>,
{
    field: &'this mut Container::Value,
}

/// Grant generic access to member fields.
///
/// This trait generalizes over member fields of structures. It allows granting
/// access to a specific field without requiring any other knowledge about the
/// containing type.
///
/// Any type that implements `Field<OFFSET>` is guaranteed to have a field
/// member at relative offset `OFFSET`. The type of the field member is given
/// as `Field<OFFSET>::Value`.
///
/// ## Safety
///
/// Any implementation must guarantee that the implementing type has a member
/// field of type `Self::Value` at offset `OFFSET` relative to its starting
/// address. The member field must follow standard alignment and sizing
/// requirements.
///
/// ## Example
///
/// ```rust
/// use core::mem::offset_of;
/// use osi::meta::Field;
///
/// struct Position {
///     x: u32,
///     y: u64,
/// }
///
/// unsafe impl Field<{offset_of!(Position, x)}> for Position {
///     type Value = u32;
/// }
///
/// unsafe impl Field<{offset_of!(Position, y)}> for Position {
///     type Value = u64;
/// }
///
/// let pos = Position { x: 11, y: 1444 };
/// let pos_x = <_ as Field<{offset_of!{Position, x}}>>::field(&pos);
/// let pos_y = <_ as Field<{offset_of!{Position, y}}>>::field(&pos);
/// assert_eq!(*pos_x, pos.x);
/// assert_eq!(*pos_y, pos.y);
/// ```
pub unsafe trait Field<const OFFSET: usize>
where
    Self: Sized,
{
    type Value;

    /// Turn a container reference into a member field reference.
    fn field_of(container: &Self) -> &Self::Value {
        // SAFETY: The trait guarantees that there is a valid member field
        //         at the specified offset. Since we have the entire container
        //         borrowed, we can safely hand out sub-borrows.
        unsafe {
            &*(
                (container as *const Self)
                    .cast::<u8>()
                    .offset(OFFSET as isize)
                    .cast::<Self::Value>()
            )
        }
    }

    /// Turn a mutable container reference into a mutable member field
    /// reference.
    fn field_of_mut(container: &mut Self) -> &mut Self::Value {
        // SAFETY: The trait guarantees that there is a valid member field
        //         at the specified offset. Since we have the entire container
        //         borrowed, we can safely hand out sub-borrows.
        unsafe {
            &mut *(
                (container as *mut Self)
                    .cast::<u8>()
                    .offset(OFFSET as isize)
                    .cast::<Self::Value>()
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
    unsafe fn container_of(field: &Self::Value) -> &Self {
        // SAFETY: The caller must guarantee that `field` was acquired via
        //         `field_of()`, hence we know that the entire container is
        //         borrowed. Thus it is safe to reverse the operation.
        unsafe {
            &*(
                (field as *const Self::Value)
                    .cast::<u8>()
                    .offset(OFFSET as isize)
                    .cast::<Self>()
            )
        }
    }

    /// Turn a mutable member field reference back into a mutable container
    /// reference.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that `field` points to a member field of a
    /// container of type `Self`. Furthermore, the entire container must be
    /// mutably borrowed for at least the lifetime of `field`.
    ///
    /// Note that these guarantees are always given if `field` was acquired
    /// via `Self::field_of_mut()`.
    unsafe fn container_of_mut(field: &mut Self::Value) -> &mut Self {
        // SAFETY: The caller must guarantee that `field` was acquired via
        //         `field_of_mut()`, hence we know that the entire container is
        //         mutably borrowed. Thus it is safe to reverse the operation.
        unsafe {
            &mut *(
                (field as *mut Self::Value)
                    .cast::<u8>()
                    .offset(OFFSET as isize)
                    .cast::<Self>()
            )
        }
    }

    /// Return a reference to the member field of the container.
    fn field(&self) -> &Self::Value {
        Self::field_of(self)
    }

    /// Return a mutable reference to the member field of the container.
    fn field_mut(&mut self) -> &mut Self::Value {
        Self::field_of_mut(self)
    }

    /// Provide a view to the member field of the container.
    fn view(&self) -> FieldView<'_, Self, OFFSET> {
        FieldView::with_container(self)
    }

    /// Provide a mutable view to the member field of the container.
    fn view_mut(&mut self) -> FieldViewMut<'_, Self, OFFSET> {
        FieldViewMut::with_container(self)
    }
}

impl<'this, Container, const OFFSET: usize>
    FieldView<'this, Container, OFFSET>
where
    Container: Field<OFFSET>,
{
    /// Create a new field view from the container.
    ///
    /// This takes a container by reference, and creates a view to the member
    /// field at offset `OFFSET`. This view allows accessing the member field,
    /// but also borrowing the container again.
    pub fn with_container(container: &'this Container) -> Self {
        Self {
            field: Field::field_of(container),
        }
    }

    /// Get a reference to the member field.
    pub fn field(&self) -> &Container::Value {
        self.field
    }

    /// Get a reference to the container.
    pub fn container(&self) -> &Container {
        // SAFETY: We know `self.field` was gained via `Field::field_of()`,
        //         hence we know the entire container is borrowed. This is
        //         required for `container_of()`.
        unsafe {
            Field::container_of(self.field)
        }
    }

    /// Turn this into a reference to the member field.
    pub fn into_ref(self) -> &'this Container::Value {
        self.field
    }

    /// Turn this back into a reference to the container.
    pub fn into_container(self) -> &'this Container {
        // SAFETY: We know `self.field` was gained via `Field::field_of()`,
        //         hence we know the entire container is borrowed. This is
        //         required for `container_of()`.
        unsafe {
            Field::container_of(self.field)
        }
    }
}

impl<'this, Container, const OFFSET: usize>
    FieldViewMut<'this, Container, OFFSET>
where
    Container: Field<OFFSET>,
{
    /// Create a new mutable field view from the container.
    ///
    /// This takes a container by mutable reference, and creates a mutable view
    /// to the member field at offset `OFFSET`. This view allows access to both
    /// the member field and the container.
    pub fn with_container(container: &'this mut Container) -> Self {
        Self {
            field: Field::field_of_mut(container),
        }
    }

    /// Get a reference to the member field.
    pub fn field(&self) -> &Container::Value {
        self.field
    }

    /// Get a mutable reference to the member field.
    pub fn field_mut(&mut self) -> &mut Container::Value {
        self.field
    }

    /// Get a reference to the container.
    pub fn container(&self) -> &Container {
        // SAFETY: We know `self.field` was gained via `Field::field_of_mut()`,
        //         hence we know the entire container is mutably borrowed. This
        //         is enough for `container_of()`.
        unsafe {
            Field::container_of(self.field)
        }
    }

    /// Get a mutable reference to the container.
    pub fn container_mut(&mut self) -> &mut Container {
        // SAFETY: We know `self.field` was gained via `Field::field_of_mut()`,
        //         hence we know the entire container is mutably borrowed. This
        //         is required for `container_of_mut()`.
        unsafe {
            Field::container_of_mut(self.field)
        }
    }

    /// Turn this into a reference to the member field.
    pub fn into_ref(self) -> &'this Container::Value {
        self.field
    }

    /// Turn this into a mutable reference to the member field.
    pub fn into_mut(self) -> &'this mut Container::Value {
        self.field
    }

    /// Turn this back into a reference to the container.
    pub fn into_container(self) -> &'this mut Container {
        unsafe {
            Field::container_of_mut(self.field)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default)]
    #[repr(C, align(4))]
    struct Test {
        a: u32,
        b: u8,
        c: u32,
    }

    unsafe impl Field<{core::mem::offset_of!(Test, b)}> for Test {
        type Value = u8;
    }

    // Basic functionality tests for `Field`.
    #[test]
    fn basic_field() {
        assert_eq!(core::mem::size_of::<Test>(), 12);

        let t = Test { a: 14, b: 11, c: 1444 };
        let v = t.view();
        assert_eq!(*v.field(), 11);
    }
}
