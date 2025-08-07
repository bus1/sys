//! # Never type
//!
//! This module provides a single item called `Never`, which corresponds to the
//! unstable never-type `!` of the rust compiler.

mod private {
    pub trait FnNullary {
        type Return;
    }

    impl<R> FnNullary for fn() -> R {
        type Return = R;
    }
}

/// An uninhabited type that cannot be instantiated.
///
/// This type corresponds to the unstable
/// [never-type `!`](https://doc.rust-lang.org/std/primitive.never.html) of
/// Rust, but circumvents the unstable nature of it. It is up to the user to
/// decide whether they want to make use of this type.
pub type Never = <fn() -> ! as private::FnNullary>::Return;

#[cfg(test)]
mod test {
    use super::*;

    // Basic test to verify `Never` coerces to other types.
    #[test]
    fn basic() {
        let r: Result<u32, Never> = Ok(71);

        let v = match r {
            Ok(v) => v,
            Err(v) => v,
        };

        assert_eq!(v, 71);
    }
}
