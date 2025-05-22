//! # Configuration Flag Utilities
//!
//! This module provides utilities to help dealing with configuration flags as
//! usually used in `#[cfg()]` or [`core::cfg!()`].

#[doc(hidden)]
#[macro_export]
macro_rules! crate_cfg_cond {
    // Helper to expand the argument in the current context.
    (@expand(
        $($v:tt)*
    )) => { $($v)* };

    // Termination of the recursive macro.
    (@internal(
        ($(($acc:meta),)*),
    )) => {};

    // Recursive macro that expands each block prefixed with a cfg-condition
    // plus a negated condition of all preceding items. It recurses by adding
    // the condition to the accumulator and proceeding with the next item.
    (@internal(
        ($(($acc:meta),)*),
        (($($cond:meta)?), ($($v:tt)*)),
        $($rem:tt,)*
    )) => {
        #[cfg(all(
            $($cond,)?
            not(any($($acc,)*)),
        ))]
        $crate::crate_cfg_cond! { @expand($($v)*) }

        $crate::crate_cfg_cond! {
            @internal(
                ($(($acc),)* $(($cond),)?),
                $($rem,)*
            )
        }
    };

    // Take a list of condition+block, plus optionally a terminating
    // block and produce the related cfg-conditions.
    (
        $(
            ($cond:meta) { $($v:tt)* },
        )+
        $(
            { $($else:tt)* },
        )?
    ) => {
        $crate::crate_cfg_cond! {
            @internal(
                (),
                $(
                    (($cond), ($($v)*)),
                )+
                $(
                    ((), ($($else)*)),
                )?
            )
        }
    };
}

/// Turn a list of cascading conditions with associated blocks into a list of
/// blocks with corresponding `#[cfg(...)]` conditions.
///
/// This macro allows writing statements similar in style to
/// `if-{}-else-if-{}-[...]-else-{}` statements with `cfg(...)` conditions,
/// and then turning them into the correct coalesced `#[cfg(...)]` blocks.
///
/// Note that this macro cannot produce expressions, and is not valid in
/// expression context. Use [`core::cfg!()`] from the standard library for that
/// purpose.
#[doc(inline)]
pub use crate_cfg_cond as cond;

#[cfg(test)]
mod test {
    use super::*;

    cond! {
        (test) {
            const V0: bool = true;
        },
    }

    cond! {
        (test) {
            const V1: bool = true;
        },
        {
            const V1: bool = false;
        },
    }

    cond! {
        (not(test)) {
            const V2: bool = false;
        },
        {
            const V2: bool = true;
        },
    }

    cond! {
        (not(test)) {
            const V3: bool = false;
        },
        (test) {
            const V3: bool = true;
        },
        (any(not(test), not(test))) {
            const V3: bool = false;
        },
    }

    cond! {
        (not(test)) {
            const V4: bool = false;
        },
        (any(not(test), not(test))) {
            const V4: bool = false;
        },
        (any(test, not(test))) {
            const V4: bool = true;
        },
    }

    cond! {
        (not(test)) {
            const V5: bool = false;
        },
        (any(test, not(test))) {
            const V5: bool = true;
        },
        (not(test)) {
            const V5: bool = false;
        },
        {
            const V5: bool = false;
        },
    }

    cond! {
        (not(test)) {
            const V6: bool = false;
        },
        (any(not(test), not(test))) {
            const V6: bool = false;
        },
        (any(not(test))) {
            const V6: bool = false;
        },
        {
            const V6: bool = true;
        },
    }

    #[test]
    fn basic_cond() {
        assert!(V0);
        assert!(V1);
        assert!(V2);
        assert!(V3);
        assert!(V4);
        assert!(V5);
        assert!(V6);
    }
}
