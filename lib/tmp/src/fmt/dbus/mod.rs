//! # D-Bus Variants
//!
//! XXX

pub mod dvar;
pub mod element;
pub mod ende;
pub mod signature;

pub use element::Element;
pub use signature::{Cursor, Sig, sig};

#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// The underlying I/O operation failed.
    Io(crate::io::map::Error),
    /// The provided type does not match the signature.
    Mismatch,
    /// The signature has not been fully processed.
    Pending,
    /// The passed data overflows the supported length of the encoding.
    DataOverflow,
    /// The data is not valid UTF-8.
    DataNonUtf8,
}
