//! # Branded Types
//!
//! Brands are unique identifiers to encode connections between types and
//! enforce these connections in the type system. It is sometimes referred
//! to as _"generativity"_.

/// A trusted and invariant but not necessarily unique brand identified by
/// its lifetime parameter.
///
/// While lifetimes are used as underlying identifiers for brands, they can
/// easily be misused if not properly protected. Therefore, this type provides
/// a protected identifier for liftime brands.
///
/// In particular, this type ensures that:
///
///  1. Lifetime identifiers are invariant over their lifetime. That is, an
///     instance of `Id<'a>` will never subtype `Id<'b>`, unless the lifetimes
///     are identical (i.e., in particular `'a ⊇ 'b` does not imply
///     `Id<'a> ⊇ Id<'b>`, or vice versa).
///  2. Lifetime identifiers cannot be forged. Every instance of this type
///     originates in [`Unique`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Id<'brand> {
    _brand: crate::marker::PhantomInvariantLifetime<'brand>,
}

/// A brand that is uniquely identified by its lifetime parameter.
///
/// This type is a zero-cost, unique identifier, that cannot be forged in
/// safe Rust. Its lifetime argument uniquely identifies it. The only way
/// to create a new instance is [`unique()`].
///
/// This type is a 1-ZST with no runtime overhead, which is invariant over
/// its lifetime argument.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct Unique<'brand> {
    id: Id<'brand>,
}

/// Create a new unique brand for a closure invocation.
///
/// The brand is not returned, but instead the passed closure is invoked and
/// passed a new and unique brand. This brand cannot be moved out of this
/// closure.
pub fn unique<Op, R>(op: Op) -> R
where
    for<'any_brand> Op: FnOnce(Unique<'any_brand>) -> R,
{
    let unique = Unique { id: Id { _brand: Default::default() } };
    op(unique)
}

impl<'brand> core::fmt::Debug for Id<'brand> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        fmt.debug_tuple("Id<#[unique] '_>").finish()
    }
}

impl<'brand> core::convert::From<Unique<'brand>> for Id<'brand> {
    fn from(v: Unique<'brand>) -> Self {
        v.id
    }
}

impl<'brand> core::fmt::Debug for Unique<'brand> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        fmt.debug_struct("Unique").field("id", &self.id).finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unique_basic() {
        let v = unique(|v| {
            assert_eq!(
                std::format!("{:?}", v),
                "Unique { id: Id<#[unique] '_> }",
            );

            let id0: Id<'_> = v.into();
            let id1: Id<'_> = id0.clone(); // clone
            let id2: Id<'_> = id0; // copy
            assert_eq!(id0, id1);
            assert_eq!(id0, id2);

            71
        });
        assert_eq!(v, 71);
    }
}
