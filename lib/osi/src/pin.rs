//! # Pin Utilities
//!
//! This module provides utilities to simplify use of [`core::pin::Pin`], as
//! well as provide pinned alternatives for the other APIs provide by this
//! crate.

use core::pin;
use crate::meta;

/// A pinned view into a container for a specific member field.
///
/// This trait is a pinned alternative to [`meta::FieldView`].
#[repr(transparent)]
pub struct PinnedFieldView<'this, Container, const OFFSET: usize>
where
    Container: PinnedField<OFFSET>,
{
    field: pin::Pin<&'this Container::Value>,
}

/// A pinned mutable view into a container for a specific member field.
///
/// This trait is a pinned alternative to [`meta::FieldViewMut`].
#[repr(transparent)]
pub struct PinnedFieldViewMut<'this, Container, const OFFSET: usize>
where
    Container: PinnedField<OFFSET>,
{
    field: pin::Pin<&'this mut Container::Value>,
}

/// Grant generic access to pinned member fields.
///
/// This trait is a pinned alternative to [`meta::Field`].
///
/// ## Safety
///
/// Any implementation must uphold the requirements of [`meta::Field`]. On top,
/// the member field at offset `OFFSET` must follow the rules of
/// `structural pinning` as described in [`core::pin::Pin`]
pub unsafe trait PinnedField<const OFFSET: usize>
where
    Self: Sized + meta::Field<OFFSET>,
{
    /// Turn a pinned container reference into a pinned member field reference.
    fn pinned_field_of(container: pin::Pin<&Self>) -> pin::Pin<&Self::Value> {
        // SAFETY: The trait guarantees structural pinning for the member
        //         field. Therefore, we can safely pin the field.
        unsafe {
            container.map_unchecked(Self::field_of)
        }
    }

    /// Turn a pinned mutable container reference into a pinned mutable member
    /// field reference.
    fn pinned_field_of_mut(container: pin::Pin<&mut Self>) -> pin::Pin<&mut Self::Value> {
        // SAFETY: The trait guarantees structural pinning for the member
        //         field. Therefore, we can safely pin the field.
        unsafe {
            container.map_unchecked_mut(Self::field_of_mut)
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
    unsafe fn pinned_container_of(field: pin::Pin<&Self::Value>) -> pin::Pin<&Self> {
        // SAFETY: The caller guarantees that the entire container is borrowed
        //         and pinned.
        unsafe {
            field.map_unchecked(|v| Self::container_of(v))
        }
    }

    /// Turn a pinned mutable member field reference back into a pinned mutable
    /// container reference.
    ///
    /// ## Safety
    ///
    /// The caller must guarantee that `field` points to a member field of a
    /// container of type `Self`. Furthermore, the entire container must be
    /// mutably borrowed and pinned for at least the lifetime of `field`.
    ///
    /// Note that these guarantees are always given if `field` was acquired
    /// via `Self::pinned_field_of_mut()`.
    unsafe fn pinned_container_of_mut(field: pin::Pin<&mut Self::Value>) -> pin::Pin<&mut Self> {
        // SAFETY: The caller guarantees that the entire container is mutably
        //         borrowed and pinned.
        unsafe {
            field.map_unchecked_mut(|v| Self::container_of_mut(v))
        }
    }

    /// Return a pinned reference to the member field of the container.
    fn pinned_field(self: pin::Pin<&Self>) -> pin::Pin<&Self::Value> {
        Self::pinned_field_of(self)
    }

    /// Return a pinned mutable reference to the member field of the container.
    fn pinned_field_mut(self: pin::Pin<&mut Self>) -> pin::Pin<&mut Self::Value> {
        Self::pinned_field_of_mut(self)
    }

    /// Provide a pinned view to the member field of the container.
    fn pinned_view(self: pin::Pin<&Self>) -> PinnedFieldView<'_, Self, OFFSET> {
        PinnedFieldView::with_container(self)
    }

    /// Provide a pinned mutable view to the member field of the container.
    fn pinned_view_mut(self: pin::Pin<&mut Self>) -> PinnedFieldViewMut<'_, Self, OFFSET> {
        PinnedFieldViewMut::with_container(self)
    }
}

impl<'this, Container, const OFFSET: usize>
    PinnedFieldView<'this, Container, OFFSET>
where
    Container: PinnedField<OFFSET>,
{
    /// Create a new pinned field view from the container.
    ///
    /// This takes a container by pinned reference, and creates a pinned view
    /// to the member field at offset `OFFSET`. This view allows accessing the
    /// member field, but also borrowing the container again.
    pub fn with_container(container: pin::Pin<&'this Container>) -> Self {
        Self {
            field: PinnedField::pinned_field_of(container),
        }
    }

    /// Get a pinned reference to the member field.
    pub fn field(&self) -> pin::Pin<&Container::Value> {
        self.field
    }

    /// Get a reference to the container.
    pub fn container(&self) -> pin::Pin<&Container> {
        // SAFETY: We know `self.field` was gained via
        //         `PinnedField::pinned_field_of()`, hence we know the entire
        //         container is borrowed.
        unsafe {
            PinnedField::pinned_container_of(self.field)
        }
    }

    /// Turn this into a pinned reference to the member field.
    pub fn into_ref(self) -> pin::Pin<&'this Container::Value> {
        self.field
    }

    /// Turn this back into a pinned reference to the container.
    pub fn into_container(self) -> pin::Pin<&'this Container> {
        // SAFETY: We know `self.field` was gained via `Field::field_of()`,
        //         hence we know the entire container is borrowed. This is
        //         required for `container_of()`.
        unsafe {
            PinnedField::pinned_container_of(self.field)
        }
    }
}

impl<'this, Container, const OFFSET: usize>
    PinnedFieldViewMut<'this, Container, OFFSET>
where
    Container: PinnedField<OFFSET>,
{
    /// Create a new pinned mutable field view from the container.
    ///
    /// This takes a container by pinned mutable reference, and creates a
    /// pinned mutable view to the member field at offset `OFFSET`. This view
    /// allows access to both the member field and the container.
    pub fn with_container(container: pin::Pin<&'this mut Container>) -> Self {
        Self {
            field: PinnedField::pinned_field_of_mut(container),
        }
    }

    /// Get a pinned reference to the member field.
    pub fn field(&self) -> pin::Pin<&Container::Value> {
        self.field.as_ref()
    }

    /// Get a pinned mutable reference to the member field.
    pub fn field_mut(&mut self) -> pin::Pin<&mut Container::Value> {
        self.field.as_mut()
    }

    /// Get a pinned reference to the container.
    pub fn container(&self) -> pin::Pin<&Container> {
        // SAFETY: We know `self.field` was gained via
        //         `PinnedField::pinned_field_of_mut()`, hence we know the
        //         entire container is mutably borrowed and pinned.
        unsafe {
            PinnedField::pinned_container_of(self.field.as_ref())
        }
    }

    /// Get a pinned mutable reference to the container.
    pub fn container_mut(&mut self) -> pin::Pin<&mut Container> {
        // SAFETY: We know `self.field` was gained via
        //         `PinnedField::pinned_field_of_mut()`, hence we know the
        //         entire container is mutably borrowed and pinned.
        unsafe {
            PinnedField::pinned_container_of_mut(self.field.as_mut())
        }
    }

    /// Turn this into a pinned reference to the member field.
    pub fn into_ref(self) -> pin::Pin<&'this Container::Value> {
        self.field.into_ref()
    }

    /// Turn this into a pinned mutable reference to the member field.
    pub fn into_mut(self) -> pin::Pin<&'this mut Container::Value> {
        self.field
    }

    /// Turn this back into a pinned reference to the container.
    pub fn into_container(self) -> pin::Pin<&'this mut Container> {
        // SAFETY: We know `self.field` was gained via
        //         `PinnedField::pinned_field_of_mut()`, hence we know the
        //         entire container is mutably borrowed and pinned.
        unsafe {
            PinnedField::pinned_container_of_mut(self.field)
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

    unsafe impl meta::Field<{core::mem::offset_of!(Test, b)}> for Test {
        type Value = u8;
    }
    unsafe impl PinnedField<{core::mem::offset_of!(Test, b)}> for Test {
    }

    // Basic functionality tests for `Field`.
    #[test]
    fn basic_field() {
        assert_eq!(core::mem::size_of::<Test>(), 12);

        let t = pin::pin!(Test { a: 14, b: 11, c: 1444 });
        let v = t.as_ref().pinned_view();
        assert_eq!(*v.field(), 11);
    }
}
