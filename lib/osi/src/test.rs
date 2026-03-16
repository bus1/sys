//! # Utilities for Testing
//!
//! This module exposes utilities primarily meant for testing of Rust code. It
//! can be used for other purposes, if desired.

/// Explicit type that is a super-type of [`SubType`].
///
/// See [`SubType`] for a more detailed discussion.
pub type SuperType = fn(&'static i32) -> &'static i32;

/// Explicit type that is a sub-type of [`SuperType`].
///
/// This type can be used to test and verify variance of other types, by simply
/// using [`SuperType`] and [`SubType`] as type parameter, and verifying the
/// resulting type exhibits the expected sub-type relationship.
///
/// For instance, [`CovariantType<SubType>`] is guaranteed to be a sub-type of
/// [`CovariantType<SuperType>`].
pub type SubType = for<'a> fn(&'a i32) -> &'a i32;

/// Explicit zero-sized type that is covariant over its type parameter.
///
/// This type is a 1-ZST that is covariant over its type parameter `T`. This
/// can be used in tests that verify variance of other types.
///
/// ## Examples
///
/// The following example shows how `sub_type` can be coerced into
/// `super_type`, since it is covariant over its type parameter and thus a
/// sub-type.
///
/// ```rust
/// # use osi::test::*;
/// let sub_type: CovariantType<SubType> = Default::default();
/// let super_type: CovariantType<SuperType> = sub_type;
/// ```
#[repr(C, packed)]
pub struct CovariantType<T: ?Sized> {
    _fn: [*const T; 0],
}

/// Explicit zero-sized type that is contracovariant over its type parameter.
///
/// This type is a 1-ZST that is contracovariant over its type parameter `T`.
/// This can be used in tests that verify variance of other types.
///
/// ## Examples
///
/// The following example shows how `sub_type` can be coerced into
/// `super_type`, even though their type parameters expose the inverse
/// sub-type relationship.
///
/// ```rust
/// # use osi::test::*;
/// let sub_type: ContravariantType<SuperType> = Default::default();
/// let super_type: ContravariantType<SubType> = sub_type;
/// ```
#[repr(C, packed)]
pub struct ContravariantType<T: ?Sized> {
    _fn: [fn(T); 0],
}

/// Explicit zero-sized type that is invariant over its type parameter.
///
/// This type is a 1-ZST that is invariant over its type parameter `T`.
/// This can be used in tests that verify variance of other types.
///
/// ## Examples
///
/// The following example shows how `sub_type` cannot be coerced into
/// `super_type`, even though their type parameters expose a sub-type
/// relationship.
///
/// ```rust,edition2021,compile_fail
/// # use osi::test::*;
/// let sub_type: InvariantType<SubType> = Default::default();
/// let super_type: InvariantType<SuperType> = sub_type;
/// ```
#[repr(C, packed)]
pub struct InvariantType<T: ?Sized> {
    _fn: [*mut T; 0],
}

impl<T: ?Sized> CovariantType<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            _fn: [],
        }
    }
}

impl<T: ?Sized> core::default::Default for CovariantType<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> ContravariantType<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            _fn: [],
        }
    }
}

impl<T: ?Sized> core::default::Default for ContravariantType<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> InvariantType<T> {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            _fn: [],
        }
    }
}

impl<T: ?Sized> core::default::Default for InvariantType<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Verify that the different subtyping utilities expose the promised
    // subtype relationships.
    #[test]
    fn subtyping_basic() {
        let _cov_sub: CovariantType<SubType> = Default::default();
        let _cov_super: CovariantType<SuperType> = Default::default();
        let _cov_super: CovariantType<SuperType> = CovariantType::<SubType>::new();

        let _ctv_sub: ContravariantType<SuperType> = Default::default();
        let _ctv_super: ContravariantType<SubType> = Default::default();
        let _ctv_super: ContravariantType<SubType> = ContravariantType::<SuperType>::new();

        let _inv_sub: InvariantType<SubType> = Default::default();
        let _inv_super: InvariantType<SuperType> = Default::default();

        assert_eq!(align_of::<CovariantType<SubType>>(), 1);
        assert_eq!(size_of::<CovariantType<SubType>>(), 0);
        assert_eq!(align_of::<CovariantType<SuperType>>(), 1);
        assert_eq!(size_of::<CovariantType<SuperType>>(), 0);

        assert_eq!(align_of::<ContravariantType<SubType>>(), 1);
        assert_eq!(size_of::<ContravariantType<SubType>>(), 0);
        assert_eq!(align_of::<ContravariantType<SuperType>>(), 1);
        assert_eq!(size_of::<ContravariantType<SuperType>>(), 0);

        assert_eq!(align_of::<InvariantType<SubType>>(), 1);
        assert_eq!(size_of::<InvariantType<SubType>>(), 0);
        assert_eq!(align_of::<InvariantType<SuperType>>(), 1);
        assert_eq!(size_of::<InvariantType<SuperType>>(), 0);
    }

    // Verify that conflicting traits can be implemented on `CovariantType<T>`,
    // even for types that have a subtype relationship.
    //
    // This is currently caught by `coherence_leak_check`, and there is
    // discussion to prevent such conflicting impls. However, the discussion
    // is stale (last update in 2001), and it mostly evolves around
    // `for<'a> fn(&'a T)`, which exposes universal lifetimes. Our `SuperType`
    // and `SubType` utilities look very much unrelated to universal lifetimes,
    // and are clearly distinct types. So we just silence
    // `coherence_leak_check` for now, given that the impls work as we would
    // expect.
    #[allow(coherence_leak_check)]
    #[test]
    fn subtyping_impl_covariant() {
        struct Test<T: ?Sized> {
            _t: CovariantType<T>,
            value: u8,
        }

        impl core::default::Default for Test<SuperType> {
            fn default() -> Self {
                Self {
                    _t: Default::default(),
                    value: 1,
                }
            }
        }

        impl core::default::Default for Test<SubType> {
            fn default() -> Self {
                Self {
                    _t: Default::default(),
                    value: 2,
                }
            }
        }

        // Create distinct super- and sub-type instances.
        let test_sub: Test<SubType> = <Test::<SubType> as core::default::Default>::default();
        let test_super: Test<SuperType> = <Test::<SuperType> as core::default::Default>::default();

        // Verify subtype relationship.
        let _p_super: &Test<SuperType> = &test_super;
        let _p_super_sub: &Test<SuperType> = &test_sub;

        // Verify that the correct traits were called.
        assert_eq!(test_super.value, 1);
        assert_eq!(test_sub.value, 2);
    }
}
