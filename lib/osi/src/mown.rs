//! # Maybe-Owned Type
//!
//! This module provides the `Mown` type. This is a generic type that
//! represents data that is either owned or borrowed.

/// A *Maybe-Owned* type represents values that are either owned or borrowed.
///
/// The main motivation of the `Mown` type is to generalize whether an object
/// stores borrowed or owned data. When an object stores a reference to
/// borrowed data, the caller must ensure this data lives long enough. This is
/// often cumbersome since Rust lacks support for self-referential data types.
/// But if the object instead stores a [`Mown`] object, the caller can decide
/// whether to pass in borrowed or owned data.
///
/// ## Similarity to Cow
///
/// This type is almost identical to [`Cow`](alloc::borrow::Cow), but does not
/// require [`Clone`](core::clone::Clone) or any kind of support for
/// mutability.
pub enum Mown<'a, B: ?Sized, O = &'a B> {
    Borrowed(&'a B),
    Owned(O),
}

impl<'a, B, O> Mown<'a, B, O>
where
    B: 'a + ?Sized,
{
    /// Create a new borrowed `Mown`.
    pub const fn new_borrowed(v: &'a B) -> Self {
        Self::Borrowed(v)
    }

    /// Create a new owned `Mown`.
    pub const fn new_owned(v: O) -> Self {
        Self::Owned(v)
    }

    /// Check whether the `Mown` is borrowed.
    pub const fn is_borrowed(&self) -> bool {
        match *self {
            Self::Borrowed(_) => true,
            Self::Owned(_) => false,
        }
    }

    /// Check whether the `Mown` is owned.
    pub const fn is_owned(&self) -> bool {
        !self.is_borrowed()
    }
}

impl<'a, B, O> Mown<'a, B, O>
where
    B: ?Sized,
    O: core::borrow::Borrow<B>,
{
    /// Dereference the `Mown` to the borrowed type.
    pub fn deref(&self) -> &B {
        match *self {
            Self::Borrowed(v) => v,
            Self::Owned(ref v) => v.borrow(),
        }
    }
}

impl<'a, B, O, T> core::convert::AsRef<T> for Mown<'a, B, O>
where
    B: ?Sized + core::convert::AsRef<T>,
    O: core::borrow::Borrow<B>,
    T: ?Sized,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<'a, B, O> core::clone::Clone for Mown<'a, B, O>
where
    B: ?Sized,
    O: core::clone::Clone,
{
    fn clone(&self) -> Self {
        match *self {
            Self::Borrowed(v) => Self::Borrowed(v),
            Self::Owned(ref v) => Self::Owned(v.clone()),
        }
    }
}

impl<'a, B, O> core::fmt::Debug for Mown<'a, B, O>
where
    B: ?Sized + core::fmt::Debug,
    O: core::borrow::Borrow<B>,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match *self {
            Self::Borrowed(v) => fmt.debug_tuple("Mown::Borrowed").field(&v).finish(),
            Self::Owned(ref v) => fmt.debug_tuple("Mown::Owned").field(&v.borrow()).finish(),
        }
    }
}

impl<'a, B, O> core::default::Default for Mown<'a, B, O>
where
    B: ?Sized,
    O: core::default::Default,
{
    fn default() -> Self {
        Self::new_owned(O::default())
    }
}

impl<'a, B, O> core::ops::Deref for Mown<'a, B, O>
where
    B: ?Sized,
    O: core::borrow::Borrow<B>,
{
    type Target = B;

    fn deref(&self) -> &B {
        Mown::deref(self)
    }
}

impl<'a, B, O> core::cmp::Eq for Mown<'a, B, O>
where
    B: ?Sized + core::cmp::Eq,
    O: core::borrow::Borrow<B>,
{
}

impl<'a, B, O> core::convert::From<&'a B> for Mown<'a, B, O>
where
    B: ?Sized,
{
    fn from(v: &'a B) -> Self {
        Self::new_borrowed(v)
    }
}

impl<'a, B, O> core::hash::Hash for Mown<'a, B, O>
where
    B: ?Sized + core::hash::Hash,
    O: core::borrow::Borrow<B>,
{
    fn hash<Op>(&self, state: &mut Op)
    where
        Op: core::hash::Hasher,
    {
        (**self).hash(state)
    }
}

impl<'a, B, O> core::cmp::Ord for Mown<'a, B, O>
where
    B: ?Sized + core::cmp::Ord,
    O: core::borrow::Borrow<B>,
{
    fn cmp(&self, v: &Self) -> core::cmp::Ordering {
        (**self).cmp(v)
    }
}

impl<'a, B, O> core::cmp::PartialEq for Mown<'a, B, O>
where
    B: ?Sized + core::cmp::PartialEq,
    O: core::borrow::Borrow<B>,
{
    fn eq(&self, v: &Self) -> bool {
        (**self).eq(v)
    }
}

impl<'a, B, O> core::cmp::PartialOrd for Mown<'a, B, O>
where
    B: ?Sized + core::cmp::PartialOrd,
    O: core::borrow::Borrow<B>,
{
    fn partial_cmp(&self, v: &Self) -> Option<core::cmp::Ordering> {
        (**self).partial_cmp(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::string::String;

    // Verify basic behavior of `Mown`.
    #[test]
    fn basic() {
        // Verify that niches are used by `Mown`. Since `String` simply wraps
        // `Vec`, we know that it has a non-zero-annotation. Thus the `Mown`
        // enum must be able to re-use the nieche bits.
        assert_eq!(
            core::mem::size_of::<Mown<'_, str, String>>(),
            core::mem::size_of::<String>(),
        );

        // Create owned and borrowed variants and compare them.
        let b = Mown::<str, String>::new_borrowed("foobar");
        let o = Mown::<str, String>::new_owned("foobar".into());
        assert_eq!(b, o);
        assert_eq!(&*b, &*o);
        assert_eq!(&*b, "foobar");
        assert_eq!(&*o, "foobar");
        assert!(b.is_borrowed());
        assert!(!o.is_borrowed());
        assert!(!b.is_owned());
        assert!(o.is_owned());
    }
}
