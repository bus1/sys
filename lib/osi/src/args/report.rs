//! # Reports for Program Argument Parsing
//!
//! The report module provides traits and hooks to integrate the generic
//! program argument parser into external frameworks.

use crate::{args, compat};

/// Context passed to `FlagReport` interactions. This provides access to a wide
/// range of information at the time of report.
pub struct FlagContext<'this, 'args, R> {
    parser: &'this mut dyn ParserReport<'args, R>,
    command: &'this mut dyn CommandReport<'args, R>,
    command_current: usize,
    flag_arg: &'args compat::OsStr,
}

/// Context passed to `CommandReport` interactions. This provides access to a
/// wide range of information at the time of report.
pub struct CommandContext<'this, 'args, R> {
    parser: &'this mut dyn ParserReport<'args, R>,
}

/// Context passed to `ParserReport` interactions. This provides access to a
/// wide range of information at the time of report.
pub struct ParserContext<'this, 'args, R> {
    _parser: core::marker::PhantomData<&'this mut dyn ParserReport<'args, R>>,
}

/// Report trait used to define how a specific flag is to be handled. This is
/// used by the argument parser to report handling of a specific flag.
pub trait FlagReport<'args, R> {
    /// Report a flag that is set in the program arguments.
    fn report_set(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
    ) -> core::ops::ControlFlow<R> {
        context.report_error(args::Error::FlagNoValue {
            flag: context.flag_arg(),
        })
    }

    /// Report a flag that is toggled in the program arguments.
    fn report_toggle(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        if value {
            self.report_set(context)
        } else {
            context.report_error(args::Error::FlagUnexpectedToggle {
                flag: context.flag_arg(),
            })
        }
    }

    /// Report a flag that is parsed in the program arguments.
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        context.report_error(args::Error::FlagUnexpectedValue {
            flag: context.flag_arg(),
            value: value,
        })
    }
}

/// Report trait used to define how a specific command is to be handled. This
/// is used by the argument parser to report handling of a specific command.
pub trait CommandReport<'args, R> {
    /// Report a parameter that is provided in the program arguments.
    fn report_parameter(
        &mut self,
        context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(v) = value {
            context.report_error(args::Error::ParameterUnexpected {
                parameter: v,
            })
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

/// Report trait used to define how a specific parsing event is to be handled.
/// This is used by the argument parser to report handling of errors and other
/// events during parser.
pub trait ParserReport<'args, R> {
    /// Report an error during program argument parsing.
    fn report_error(
        &mut self,
        context: &mut ParserContext<'_, 'args, R>,
        error: args::Error<'args>,
    ) -> core::ops::ControlFlow<R>;
}

/// Abstraction over shared reports that use interior mutability. This can be
/// used to share an underlying report object across multiple users, protecting
/// the report with a cell that follows interior mutability patterns.
pub struct Shared<'this, Inner> {
    inner: &'this Inner,
}

impl<'this, 'args, R> FlagContext<'this, 'args, R> {
    pub(super) fn with(
        parser: &'this mut dyn ParserReport<'args, R>,
        command: &'this mut dyn CommandReport<'args, R>,
        command_current: usize,
        flag_arg: &'args compat::OsStr,
    ) -> Self {
        Self {
            parser: parser,
            command: command,
            command_current: command_current,
            flag_arg: flag_arg,
        }
    }

    /// Yield the parser report this flag was defined on.
    pub fn parser(&mut self) -> &mut dyn ParserReport<'args, R> {
        self.parser
    }

    /// Yield the command report this flag was defined on.
    pub fn command(&mut self) -> &mut dyn CommandReport<'args, R> {
        self.command
    }

    /// Yield the index of the last command at the time of this report. That
    /// is, the index of the command this flag was used on, rather than the
    /// command this flag was defined on.
    pub fn command_current(&mut self) -> usize {
        self.command_current
    }

    /// Yield the name of the flag as given in the program arguments.
    pub fn flag_arg(&self) -> &'args compat::OsStr {
        self.flag_arg
    }

    /// Report an error via the parser report of this context.
    pub fn report_error(
        &mut self,
        error: args::Error<'args>,
    ) -> core::ops::ControlFlow<R> {
        self.parser.report_error(&mut ParserContext::new(), error)
    }
}

impl<'this, 'args, R> CommandContext<'this, 'args, R> {
    pub(super) fn with(
        parser: &'this mut dyn ParserReport<'args, R>,
    ) -> Self {
        Self {
            parser: parser,
        }
    }

    /// Yield the parser report this flag was defined on.
    pub fn parser(&mut self) -> &mut dyn ParserReport<'args, R> {
        self.parser
    }

    /// Report an error via the parser report of this context.
    pub fn report_error(
        &mut self,
        error: args::Error<'args>,
    ) -> core::ops::ControlFlow<R> {
        self.parser.report_error(&mut ParserContext::new(), error)
    }
}

impl<'this, 'args, R> ParserContext<'this, 'args, R> {
    pub(super) fn new() -> Self {
        Self {
            _parser: Default::default(),
        }
    }
}

impl<'this, Inner> Shared<'this, Inner> {
    /// Create a new shared report for the given report object.
    pub fn with(inner: &'this Inner) -> Self {
        Self {
            inner: inner,
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<()>
{
    fn report_set(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
    ) -> core::ops::ControlFlow<R> {
        *self = Some(());
        core::ops::ControlFlow::Continue(())
    }

    fn report_toggle(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        *self = if value {
            Some(())
        } else {
            None
        };
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    bool
{
    fn report_set(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
    ) -> core::ops::ControlFlow<R> {
        *self = true;
        core::ops::ControlFlow::Continue(())
    }

    fn report_toggle(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        *self = value;
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    &'args compat::OsStr
{
    fn report_parse(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        *self = value;
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<&'args compat::OsStr>
{
    fn report_parse(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        *self = Some(value);
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    &'args str
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            *self = str_value;
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<&'args str>
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            *self = Some(str_value);
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    alloc::string::String
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            *self = str_value.into();
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<alloc::string::String>
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            *self = Some(str_value.into());
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    alloc::vec::Vec<&'args compat::OsStr>
{
    fn report_parse(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        self.push(value);
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<alloc::vec::Vec<&'args compat::OsStr>>
{
    fn report_parse(
        &mut self,
        _context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        self.get_or_insert_with(Default::default).push(value);
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    alloc::vec::Vec<&'args str>
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            self.push(str_value);
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<alloc::vec::Vec<&'args str>>
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            self.get_or_insert_with(Default::default).push(str_value);
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    alloc::vec::Vec<alloc::string::String>
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            self.push(str_value.into());
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    FlagReport<'args, R>
for
    Option<alloc::vec::Vec<alloc::string::String>>
{
    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        if let Ok(str_value) = value.to_str() {
            self.get_or_insert_with(Default::default).push(str_value.into());
            core::ops::ControlFlow::Continue(())
        } else {
            context.report_error(args::Error::FlagValueNotUtf8 {
                flag: context.flag_arg(),
                value: value,
            })
        }
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    ()
{
    fn report_parameter(
        &mut self,
        context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(v) = value {
            context.report_error(args::Error::ParameterUnexpected {
                parameter: v,
            })
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    alloc::vec::Vec<&'args compat::OsStr>
{
    fn report_parameter(
        &mut self,
        _context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(v) = value {
            self.push(v);
        }
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    Option<alloc::vec::Vec<&'args compat::OsStr>>
{
    fn report_parameter(
        &mut self,
        _context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(v) = value {
            self.get_or_insert_with(Default::default).push(v);
        }
        core::ops::ControlFlow::Continue(())
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    alloc::vec::Vec<&'args str>
{
    fn report_parameter(
        &mut self,
        context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(arg_value) = value {
            if let Ok(str_value) = arg_value.to_str() {
                self.push(str_value);
                core::ops::ControlFlow::Continue(())
            } else {
                context.report_error(args::Error::ParameterNotUtf8 {
                    parameter: arg_value,
                })
            }
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    Option<alloc::vec::Vec<&'args str>>
{
    fn report_parameter(
        &mut self,
        context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(arg_value) = value {
            if let Ok(str_value) = arg_value.to_str() {
                self.get_or_insert_with(Default::default).push(str_value);
                core::ops::ControlFlow::Continue(())
            } else {
                context.report_error(args::Error::ParameterNotUtf8 {
                    parameter: arg_value,
                })
            }
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    alloc::vec::Vec<alloc::string::String>
{
    fn report_parameter(
        &mut self,
        context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(arg_value) = value {
            if let Ok(str_value) = arg_value.to_str() {
                self.push(str_value.into());
                core::ops::ControlFlow::Continue(())
            } else {
                context.report_error(args::Error::ParameterNotUtf8 {
                    parameter: arg_value,
                })
            }
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

impl<'args, R>
    CommandReport<'args, R>
for
    Option<alloc::vec::Vec<alloc::string::String>>
{
    fn report_parameter(
        &mut self,
        context: &mut CommandContext<'_, 'args, R>,
        value: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(arg_value) = value {
            if let Ok(str_value) = arg_value.to_str() {
                self.get_or_insert_with(Default::default).push(str_value.into());
                core::ops::ControlFlow::Continue(())
            } else {
                context.report_error(args::Error::ParameterNotUtf8 {
                    parameter: arg_value,
                })
            }
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

impl<'args>
    ParserReport<'args, ()>
for
    alloc::vec::Vec<args::Error<'args>>
{
    fn report_error(
        &mut self,
        _context: &mut ParserContext<'_, 'args, ()>,
        error: args::Error<'args>,
    ) -> core::ops::ControlFlow<()> {
        self.push(error);
        core::ops::ControlFlow::Continue(())
    }
}

impl<'this, 'args, Inner, R>
    FlagReport<'args, R>
for
    Shared<'this, core::cell::Cell<Inner>>
where
    Inner: Copy + FlagReport<'args, R>,
{
    fn report_set(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
    ) -> core::ops::ControlFlow<R> {
        let mut v = self.inner.get();
        v.report_set(context)?;
        self.inner.set(v);
        core::ops::ControlFlow::Continue(())
    }

    fn report_toggle(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        let mut v = self.inner.get();
        v.report_toggle(context, value)?;
        self.inner.set(v);
        core::ops::ControlFlow::Continue(())
    }

    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        let mut v = self.inner.get();
        v.report_parse(context, value)?;
        self.inner.set(v);
        core::ops::ControlFlow::Continue(())
    }
}

impl<'this, 'args, Inner, R>
    FlagReport<'args, R>
for
    Shared<'this, core::cell::RefCell<Inner>>
where
    Inner: FlagReport<'args, R>,
{
    fn report_set(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
    ) -> core::ops::ControlFlow<R> {
        self.inner.borrow_mut().report_set(context)
    }

    fn report_toggle(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        self.inner.borrow_mut().report_toggle(context, value)
    }

    fn report_parse(
        &mut self,
        context: &mut FlagContext<'_, 'args, R>,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        self.inner.borrow_mut().report_parse(context, value)
    }
}
