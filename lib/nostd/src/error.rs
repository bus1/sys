//! # Error Handling
//!
//! This module provides utilities around error handling.

use alloc::boxed::Box;

/// An object to represent errors that were not caught, but have to be
/// propagated. Any kind of error information can be folded into this
/// type and then propagated in a uniform manner.
///
/// XXX: When `std::error::Error` becomes available in `core`, we can switch to
///      it unconditionally. This is tracked in upstream as `error_in_core`.
pub enum Uncaught {
    Any(Box<dyn core::any::Any>),
    Debug(Box<dyn core::fmt::Debug>),
    Display(Box<dyn core::fmt::Display>),

    #[cfg(feature = "std")]
    Error(Box<dyn std::error::Error>),
    #[cfg(not(feature = "std"))]
    Error(Box<dyn core::fmt::Display>),

    StaticAny(&'static dyn core::any::Any),
    StaticDebug(&'static dyn core::fmt::Debug),
    StaticDisplay(&'static dyn core::fmt::Display),

    #[cfg(feature = "std")]
    StaticError(&'static dyn std::error::Error),
    #[cfg(not(feature = "std"))]
    StaticError(&'static dyn core::fmt::Display),
}

impl core::fmt::Debug for Uncaught {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        match self {
            Uncaught::Any(_) => write!(fmt, "Uncaught::Any()"),
            Uncaught::Debug(v) => write!(fmt, "Uncaught::Debug({:?})", v),
            Uncaught::Display(v) => write!(fmt, "Uncaught::Display({})", v),

            #[cfg(feature = "std")]
            Uncaught::Error(v) => write!(fmt, "Uncaught::Error({:?})", v),
            #[cfg(not(feature = "std"))]
            Uncaught::Error(v) => write!(fmt, "Uncaught::Error({})", v),

            Uncaught::StaticAny(_) => write!(fmt, "Uncaught::StaticAny()"),
            Uncaught::StaticDebug(v) => write!(fmt, "Uncaught::StaticDebug({:?})", v),
            Uncaught::StaticDisplay(v) => write!(fmt, "Uncaught::StaticDisplay({})", v),

            #[cfg(feature = "std")]
            Uncaught::StaticError(v) => write!(fmt, "Uncaught::StaticError({:?})", v),
            #[cfg(not(feature = "std"))]
            Uncaught::StaticError(v) => write!(fmt, "Uncaught::StaticError({})", v),
        }
    }
}

impl core::fmt::Display for Uncaught {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        match self {
            Uncaught::Any(_) => write!(fmt, "Uncaught(Any)"),
            Uncaught::Debug(v) => write!(fmt, "Uncaught(Debug): {:?}", v),
            Uncaught::Display(v) => write!(fmt, "Uncaught(Display): {}", v),
            Uncaught::Error(v) => write!(fmt, "Uncaught(Error): {}", v),

            Uncaught::StaticAny(_) => write!(fmt, "Uncaught(StaticAny)"),
            Uncaught::StaticDebug(v) => write!(fmt, "Uncaught(StaticDebug): {:?}", v),
            Uncaught::StaticDisplay(v) => write!(fmt, "Uncaught(StaticDisplay): {}", v),
            Uncaught::StaticError(v) => write!(fmt, "Uncaught(StaticError): {}", v),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Uncaught {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Uncaught::Any(_) => None,
            Uncaught::Debug(_) => None,
            Uncaught::Display(_) => None,
            Uncaught::Error(v) => v.source(),
            Uncaught::StaticAny(_) => None,
            Uncaught::StaticDebug(_) => None,
            Uncaught::StaticDisplay(_) => None,
            Uncaught::StaticError(v) => v.source(),
        }
    }
}

impl Uncaught {
    /// Fold anything into an uncaught error, exposing nothing of the
    /// underlying element.
    pub fn fold_any(v: Box<dyn core::any::Any>) -> Self {
        Self::Any(v)
    }

    /// Fold anything into an uncaught error, exposing nothing of the
    /// underlying element.
    pub fn fold_static_any(v: &'static dyn core::any::Any) -> Self {
        Self::StaticAny(v)
    }

    /// Box anything into an uncaught error, exposing nothing of the
    /// underlying element.
    pub fn box_any<T>(v: T) -> Self
    where
        T: core::any::Any + 'static,
    {
        Self::fold_any(Box::new(v))
    }

    /// Fold any debuggable into an uncaught error, exposing only the
    /// debug value.
    pub fn fold_debug(v: Box<dyn core::fmt::Debug>) -> Self {
        Self::Debug(v)
    }

    /// Fold any debuggable into an uncaught error, exposing only the
    /// debug value.
    pub fn fold_static_debug(v: &'static dyn core::fmt::Debug) -> Self {
        Self::StaticDebug(v)
    }

    /// Box any debuggable into an uncaught error, exposing only the
    /// debug value.
    pub fn box_debug<T>(v: T) -> Self
    where
        T: core::fmt::Debug + 'static,
    {
        Self::fold_debug(Box::new(v))
    }

    /// Fold any displayable into an uncaught error, exposing only the
    /// display value.
    pub fn fold_display(v: Box<dyn core::fmt::Display>) -> Self {
        Self::Display(v)
    }

    /// Fold any displayable into an uncaught error, exposing only the
    /// display value.
    pub fn fold_static_display(v: &'static dyn core::fmt::Display) -> Self {
        Self::StaticDisplay(v)
    }

    /// Box any displayable into an uncaught error, exposing only the
    /// display value.
    pub fn box_display<T>(v: T) -> Self
    where
        T: core::fmt::Display + 'static,
    {
        Self::fold_display(Box::new(v))
    }

    /// Take any error and fold it into an uncaught error, exposing the
    /// full `Error` trait.
    #[cfg(feature = "std")]
    pub fn fold_error(v: Box<dyn std::error::Error>) -> Self {
        Self::Error(v)
    }

    /// Take any fallback error and fold it into an uncaught error, exposing
    /// the full `Error` trait.
    ///
    /// This function is exposed if no `std` is used, and thus serves as
    /// fallback when `std::error::Error` is not available.
    #[cfg(not(feature = "std"))]
    pub fn fold_error(v: Box<dyn core::fmt::Display>) -> Self {
        Self::Error(v)
    }

    /// Take any error and fold it into an uncaught error, exposing the
    /// full `Error` trait.
    #[cfg(feature = "std")]
    pub fn fold_static_error(v: &'static dyn std::error::Error) -> Self {
        Self::StaticError(v)
    }

    /// Take any fallback error and fold it into an uncaught error, exposing
    /// the full `Error` trait.
    ///
    /// This function is exposed if no `std` is used, and thus serves as
    /// fallback when `std::error::Error` is not available.
    #[cfg(not(feature = "std"))]
    pub fn fold_static_error(v: &'static dyn core::fmt::Display) -> Self {
        Self::StaticError(v)
    }

    /// Take any error and box it into an uncaught error, exposing the
    /// full `Error` trait.
    #[cfg(feature = "std")]
    pub fn box_error<T>(v: T) -> Self
    where
        T: std::error::Error + 'static,
    {
        Self::fold_error(Box::new(v))
    }

    /// Take any fallback error and box it into an uncaught error, exposing the
    /// full `Error` trait.
    ///
    /// This function is exposed if no `std` is used, and thus serves as
    /// fallback when `std::error::Error` is not available.
    #[cfg(not(feature = "std"))]
    pub fn box_error<T>(v: T) -> Self
    where
        T: core::fmt::Display + 'static,
    {
        Self::fold_error(Box::new(v))
    }
}

#[cfg(test)]
mod tests {
    use std::format;
    use super::*;

    // Test basic operations of `Uncaught`, including its boxing and
    // folding constructors, as well as the formatting traits.
    #[test]
    fn uncaught_basic() {
        let e = Uncaught::box_any(0);
        assert_eq!(format!("{:?}", e), "Uncaught::Any()");
        assert_eq!(format!("{}", e), "Uncaught(Any)");

        let e = Uncaught::box_debug(0);
        assert_eq!(format!("{:?}", e), "Uncaught::Debug(0)");
        assert_eq!(format!("{}", e), "Uncaught(Debug): 0");

        let e = Uncaught::box_display(0);
        assert_eq!(format!("{:?}", e), "Uncaught::Display(0)");
        assert_eq!(format!("{}", e), "Uncaught(Display): 0");

        #[cfg(feature = "std")]
        {
            let e = Uncaught::box_error(std::io::Error::other("foobar"));
            assert_eq!(format!("{:?}", e), "Uncaught::Error(Custom { kind: Other, error: \"foobar\" })");
            assert_eq!(format!("{}", e), "Uncaught(Error): foobar");
        }
        #[cfg(not(feature = "std"))]
        {
            let e = Uncaught::box_error("foobar");
            assert_eq!(format!("{:?}", e), "Uncaught::Error(foobar)");
            assert_eq!(format!("{}", e), "Uncaught(Error): foobar");
        }
    }
}
