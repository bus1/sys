//! # Data and Type Comparisons
//!
//! A selection of utilities for data and type comparisons.

/// Transparent wrapper that provides identity comparison.
///
/// Any type that implements [`core::ops::Deref`] can be transparently wrapped
/// in `Identity` to ensure comparisons are now performed for identity rather
/// than value.
// Unsized values (i.e. fat pointers) could be supported as well, but will
// show weird behavior for dyn-pointers until rustc is fixed
// ([#46139](https://github.com/rust-lang/rust/issues/46139).
#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct Identity<T>(pub T);

impl<T: core::ops::Deref> core::cmp::Eq for Identity<T> {
}

impl<T: core::ops::Deref> core::hash::Hash for Identity<T> {
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        (&raw const *self.0).hash(state)
    }
}

impl<T: core::ops::Deref> core::cmp::Ord for Identity<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&raw const *self.0).cmp(&&raw const *other.0)
    }
}

impl<T: core::ops::Deref> core::cmp::PartialEq for Identity<T> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(&raw const *self.0, &raw const *other.0)
    }
}

impl<T: core::ops::Deref> core::cmp::PartialOrd for Identity<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let v0 = (71, 71);
        let v1 = (71, 71);
        let v0r0 = &v0;
        let v0r1 = &v0;
        let v1r0 = &v1;
        let v1r1 = &v1;

        assert_eq!(v0r0, v0r0);
        assert_eq!(v0r0, v0r1);
        assert_eq!(v0r0, v1r0);
        assert_eq!(v0r0, v1r1);

        assert_eq!(Identity(v0r0), Identity(v0r0));
        assert_eq!(Identity(v0r0), Identity(v0r1));
        assert_ne!(Identity(v0r0), Identity(v1r0));
        assert_ne!(Identity(v0r0), Identity(v1r1));

        assert!(!(v0r0 < v0r0));
        assert!(!(v0r0 < v0r1));
        assert!(!(v0r0 < v1r0));
        assert!(!(v0r0 < v1r1));
        assert!(!(v0r0 > v0r0));
        assert!(!(v0r0 > v0r1));
        assert!(!(v0r0 > v1r0));
        assert!(!(v0r0 > v1r1));

        if &raw const v0 < &raw const v1 {
            assert!(Identity(v0r0) <= Identity(v0r0));
            assert!(Identity(v0r0) <= Identity(v0r1));
            assert!(Identity(v0r0) < Identity(v1r0));
            assert!(Identity(v0r0) < Identity(v1r1));
        } else {
            assert!(Identity(v0r0) >= Identity(v0r0));
            assert!(Identity(v0r0) >= Identity(v0r1));
            assert!(Identity(v0r0) > Identity(v1r0));
            assert!(Identity(v0r0) > Identity(v1r1));
        }
    }
}
