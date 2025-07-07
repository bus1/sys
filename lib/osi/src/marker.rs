//! # Primitive Meta Traits and Types
//!
//! A selection of meta traits and types to encode the intrinsic properties
//! of user types.

/// Zero-sized type used to mark a type parameter as invariant.
///
/// Types that are both passed as an argument _and_ used as part of the return
/// value from a function are invariant. See [the reference][1] for more
/// information.
///
/// [1]: https://doc.rust-lang.org/stable/reference/subtyping.html#variance
///
/// ## Layout
///
/// Any instance of this type is guaranteed to be a 1-ZST.
///
// MSRV(unknown): This is available upstream as
//                `feature(phantom_variance_markers)` with unclear stability
//                timeline.
#[repr(transparent)]
pub struct PhantomInvariant<T: ?Sized>(
    core::marker::PhantomData<fn(T) -> T>,
);

/// Zero-sized type used to mark a lifetime as invariant.
///
/// Invariant lifetimes must be live for the exact length declared, neither
/// shorter nor longer. See [the reference][1] for more information.
///
/// [1]: https://doc.rust-lang.org/stable/reference/subtyping.html#variance
///
/// ## Layout
///
/// Any instance of this type is guaranteed to be a 1-ZST.
// MSRV(unknown): This is available upstream as
//                `feature(phantom_variance_markers)` with unclear stability
//                timeline.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PhantomInvariantLifetime<'a>(
    PhantomInvariant<&'a ()>,
);

impl<T: ?Sized> PhantomInvariant<T> {
    /// Constructs a new instance of the variance marker.
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T: ?Sized> core::clone::Clone for PhantomInvariant<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> core::marker::Copy for PhantomInvariant<T> {
}

impl<T: ?Sized> core::fmt::Debug for PhantomInvariant<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}<{}>",
            stringify!(PhantomInvariant),
            core::any::type_name::<T>(),
        )
    }
}

impl<T: ?Sized> core::default::Default for PhantomInvariant<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> core::cmp::Eq for PhantomInvariant<T> {
}

impl<T: ?Sized> core::hash::Hash for PhantomInvariant<T> {
    fn hash<Op: core::hash::Hasher>(&self, _: &mut Op) {
    }
}

impl<T: ?Sized> core::cmp::Ord for PhantomInvariant<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<T: ?Sized> core::cmp::PartialEq for PhantomInvariant<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T: ?Sized> core::cmp::PartialOrd for PhantomInvariant<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl PhantomInvariantLifetime<'_> {
    /// Constructs a new instance of the variance marker.
    pub const fn new() -> Self {
        Self(PhantomInvariant::new())
    }
}

impl core::fmt::Debug for PhantomInvariantLifetime<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", stringify!(PhantomInvariantLifetime))
    }
}
