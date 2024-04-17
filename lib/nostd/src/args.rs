//! # Program Arguments
//!
//! This module implements a basic parser for program arguments, supporting the
//! standard command, flag, and parameter dispatching.

use crate::{compat, error};

pub mod help;
pub mod layout;
pub mod parse;
pub mod report;

pub use layout::{
    FlagMode, Flag, FlagSet, FlagSetRef,
    Command, CommandSet, CommandSetRef,
    Schema,
};
pub use parse::parse;
pub use report::{
    FlagReport, CommandReport, ParserReport,
    FlagContext, CommandContext, ParserContext,
    Shared,
};

/// Enumeration of all possible errors that can be reported by the program
/// argument parser.
#[derive(Debug)]
pub enum Error<'args> {
    /// Uncaught error forwarded from a report.
    Uncaught(error::Uncaught),

    /// The given short-option flags are unknown and cannot be handled. The
    /// flags are provided without the leading dash. Multiple consecutive
    /// flags can be reported in a single error.
    ShortsUnknown {
        shorts: &'args compat::OsStr,
    },

    /// The given flag is unknown and cannot be handled.
    FlagUnknown {
        flag: &'args compat::OsStr,
    },

    /// The given flag is known but was specified with the toggle-prefix `no-`,
    /// which is not valid for this flag.
    FlagUnexpectedToggle {
        flag: &'args compat::OsStr,
    },

    /// The given flag is known but was provided an inline value despite not
    /// taking any values.
    FlagUnexpectedValue {
        flag: &'args compat::OsStr,
        value: &'args compat::OsStr,
    },

    /// The given flag is known but was provided no value despite requiring
    /// one.
    FlagNoValue {
        flag: &'args compat::OsStr,
    },

    /// A flag value was specified as invalid UTF-8, despite the flag requiring
    /// valid UTF-8 values.
    FlagValueNotUtf8 {
        flag: &'args compat::OsStr,
        value: &'args compat::OsStr,
    },

    /// A command parameter was specified but the current command does not take
    /// parameters.
    ParameterUnexpected {
        parameter: &'args compat::OsStr,
    },

    /// A command parameter was specified as invalid UTF-8, despite the given
    /// command requiring valid UTF-8 parameters.
    ParameterNotUtf8 {
        parameter: &'args compat::OsStr,
    },
}
