//! # Workarounds for Rust without Polonius
//!
//! This module provides helpers to expose features of Polonius, even if built
//! without Polonius. Polonius is a borrow checker for Rust, which replaces
//! NLL.
//!
//! This module expects `cfg(polonius)` to be set for Polonius builds. In this
//! case, the provided utilities will likely be a no-op or identity function.
//! If unset, workarounds for NLL are provided.

#[doc(hidden)]
#[macro_export]
macro_rules!
    crate_polonius_coerce_unsafe
{ ($from:ty, $to:ty, $value:expr) => {
    {
        $crate::marker::phantom_unsafe();
        let from: $from = $value;
        let to: $to = {
            $crate::cfg::cond! {
                (polonius) { from },
                { core::mem::transmute::<$from, $to>(from) },
            }
        };
        to
    }
}}

/// Perform a value coercion that relies on Polonius.
///
/// This takes the following operands:
///
/// 1) Source type
/// 2) Target type
/// 3) Value expression
///
/// This function will coerce the value expression from the source type to
/// the target type. If `cfg(polonius)` is unset, this will use a transmute
/// instead of a coercion.
///
/// ## Safety
///
/// This macro is only safe if one of the following is true:
///
/// 1) `cfg(polonius)` is set for this compilation.
/// 2) The same code is separately verified to compile cleanly with
///    `cfg(polonius)` set.
///
/// That is, this macro exposes features that pass Polonius, but not NLL. It is
/// up to the caller to ensure code is suitably compile-tested with Polonius.
#[doc(inline)]
pub use crate_polonius_coerce_unsafe as coerce_unsafe;

#[cfg(test)]
mod test {
    use super::*;

    // A very basic use of `ceroce_unsafe()` that performs a coercion that is
    // safe with both NLL and Polonius. So this is a no-op.
    #[test]
    fn coerce_basic() {
        let mut v: u16 = 0x1f1f;
        let from: &mut u16 = &mut v;
        let to: &u16 = unsafe { coerce_unsafe!(&mut u16, &u16, from) };
        assert_eq!(*to, 0x1f1f);
    }

    // A common setup where a loop needs to borrow from a mutable reference,
    // but only if the return value is actually dependent on the lifetime. That
    // is, a lifetime is "retrospectively" reborrowed to a shorter lifetime if
    // the function returned a value that is not dependent on the lifetime.
    #[test]
    fn coerce_loop_return() {
        fn get<'a>(v: &'a mut (u16,)) -> Option<&'a u16> {
            if v.0 == 0 {
                Some(&v.0)
            } else {
                v.0 -= 1;
                None
            }
        }

        fn loop_get<'a>(v: &'a mut (u16,)) -> &'a u16 {
            loop {
                // `get()` should only get `'a` if it returns `Some(_)`. If it
                // returns `None`, we pretend it got a shorter reborrowed
                // lifetime.
                let Some(r) = get(v) else {
                    continue;
                };

                return unsafe { coerce_unsafe!(&'_ u16, &'a u16, r) };
            }
        }

        let mut v = (0x1fu16,);
        let r = loop_get(&mut v);
        assert_eq!(*r, 0);
        assert_eq!(v.0, 0);
    }
}
