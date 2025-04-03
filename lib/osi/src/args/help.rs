//! # Help Information for Program Arguments
//!
//! This module provides an optional way to generate help information for
//! custom program argument layouts, using the information provided by the
//! argument reports.

use crate::args;

/// Writer trait to format help information to an output stream.
///
/// When help information is formatted via the `Help` type, the information is
/// written via a trait object of `Write`. This allows dynamic styling and
/// other advanced formatting of the help information.
///
/// NB: Several callbacks provide width information about an entire section.
///     This allows aligning entries of a section. However, these widths are
///     given as unicode character counts, rather than glyph clusters, or
///     terminal cell counts.
///     This will likely not lead to issues, given that width are only
///     calculated for flag and command names, which are recommended to be
///     ASCII-only.
///     This might be adjusted in the future when reliable width information
///     can be provided.
pub trait Write<E> {
    /// Format plain multi-line information provided by the caller. This can be
    /// used to write introductory comments, or provide sections that have pure
    /// and unstructured text.
    fn write_info(
        &mut self,
        info: &str,
    ) -> core::ops::ControlFlow<E>;

    /// Write a section header starting a new section of the help information.
    fn write_section(
        &mut self,
        section: &str,
    ) -> core::ops::ControlFlow<E>;

    /// Write usage information for the command. `entry` is the entrypoint that
    /// was used to invoke the command, while `path` describes the command
    /// chain to the chosen command (empty for the root level).
    fn write_usage(
        &mut self,
        entry: &str,
        path: &[&str],
    ) -> core::ops::ControlFlow<E>;

    /// Write information on a single flag. `width` contains the maximum width
    /// (in `chars`) of all `flag` strings used in this section. It can be used
    /// to align content suitably. `flag`, `mode`, and `info` are taken
    /// verbatim from the flag layout.
    fn write_flag(
        &mut self,
        flag: &str,
        mode: args::FlagMode,
        info: Option<&str>,
        width: usize,
    ) -> core::ops::ControlFlow<E>;

    /// Write information on a single command. `width` contains the maximum
    /// width (in `chars`) of all `command` strings used in this section. It
    /// can be used to align content suitably. `command` and `info` are taken
    /// verbatim from the command layout.
    fn write_command(
        &mut self,
        command: &str,
        info: Option<&str>,
        width: usize,
    ) -> core::ops::ControlFlow<E>;
}

/// Help flag implementation for program arguments. This represents a `--help`
/// flag and remembers whether it was set or not. Additionally, it can be used
/// to render help information, even if not requested on the command-line.
///
/// The intermediate `Flag` object must be used as report for the argument
/// layout (see `Help::flag()`). The `Help` object cannot be used directly,
/// since this would mutable borrow it and prevent access to the argument
/// layout for help information. Instead, the `Flag` intermediate is used to
/// hide the interior mutability of `Help`.
#[derive(Clone, Debug)]
pub struct Help<'this> {
    entry: &'this str,
    info: &'this str,
    index: core::cell::Cell<Option<usize>>,
}

/// Flag report for `Help`. Can be created via `Help::flag()` and represents
/// the layout report for the `Help` object.
pub struct Flag<'this, 'help> {
    help: &'this Help<'help>,
}

impl<'this> Help<'this> {
    /// Create a new help flag implementation with the specified information.
    ///
    /// `entry` specifies the entry-point of the program, usually the program
    /// name. It is usually prepended to usage information.
    ///
    /// `info` represents free-form text prepended to help-information.
    pub fn with(
        entry: &'this str,
        info: &'this str,
    ) -> Self {
        Self {
            entry: entry,
            info: info,
            index: core::cell::Cell::new(None),
        }
    }

    /// Create a flag report for use in an argument layout. The returned flag
    /// implements `args::FlagReport` and can be used with `args::Flag`.
    ///
    /// Multiple independet flags can be created for the same shared `Help`
    /// object. They will share the underlying storage and override each other.
    pub fn flag(&self) -> Flag {
        Flag {
            help: self,
        }
    }

    fn write<E, R>(
        &self,
        w: &mut dyn Write<E>,
        schema: &args::Schema<R>,
        idx_command: usize,
    ) -> core::ops::ControlFlow<E> {
        let command = schema.command_at(idx_command);
        let path = command.path();

        // Write general information
        w.write_info(self.info)?;

        // Write usage section
        w.write_section("Usage")?;
        w.write_usage(self.entry, path)?;

        // Write flag section
        {
            let mut o_width = None;

            for flag in command.flags_iter() {
                o_width = Some(usize::max(
                    o_width.unwrap_or(0),
                    flag.name.chars().count(),
                ));
            }

            if let Some(width) = o_width {
                w.write_section("Flags")?;

                for flag in command.flags_iter() {
                    w.write_flag(
                        flag.name,
                        flag.mode,
                        flag.help_short,
                        width,
                    )?;
                }
            }
        }

        // Write command section
        {
            let mut o_width = None;

            let iter = schema.commands()
                .iter_from(idx_command + 1)
                .map_while(|v| {
                    (
                        v.path.len() > path.len()
                        && v.path[..path.len()].eq(path)
                    ).then_some(v)
                })
                .filter(|v| {
                    v.path.len() == path.len() + 1
                });

            for cmd in iter.clone() {
                o_width = Some(usize::max(
                    o_width.unwrap_or(0),
                    cmd.path[path.len()].chars().count(),
                ));
            }

            if let Some(width) = o_width {
                w.write_section("Commands")?;

                for cmd in iter {
                    w.write_command(
                        cmd.path[path.len()],
                        cmd.help_short,
                        width,
                    )?;
                }
            }
        }

        core::ops::ControlFlow::Continue(())
    }

    /// Render help information if it was requested via a flag.
    pub fn help<E, R>(
        &self,
        w: &mut dyn Write<E>,
        schema: &args::Schema<R>,
    ) -> Result<bool, E> {
        let Some(idx_command) = self.index.get() else {
            return Ok(false);
        };

        match self.write(w, schema, idx_command) {
            core::ops::ControlFlow::Continue(()) => Ok(true),
            core::ops::ControlFlow::Break(v) => Err(v),
        }
    }

    /// Render help information if it was requested via a flag.
    pub fn try_help<E, R>(
        &self,
        w: &mut dyn Write<E>,
        schema: &args::Schema<R>,
    ) -> Result<bool, E> {
        let Some(idx_command) = self.index.get() else {
            return Ok(false);
        };

        match self.write(w, schema, idx_command) {
            core::ops::ControlFlow::Continue(()) => Ok(true),
            core::ops::ControlFlow::Break(v) => Err(v),
        }
    }
}

impl<'this, 'help, 'args, R> args::FlagReport<'args, R> for Flag<'this, 'help> {
    fn report_set(
        &mut self,
        context: &mut args::FlagContext<'_, 'args, R>,
    ) -> core::ops::ControlFlow<R> {
        self.help.index.set(Some(context.command_current()));
        core::ops::ControlFlow::Continue(())
    }

    fn report_toggle(
        &mut self,
        context: &mut args::FlagContext<'_, 'args, R>,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        if value {
            self.help.index.set(Some(context.command_current()));
        } else {
            self.help.index.set(None);
        }

        core::ops::ControlFlow::Continue(())
    }
}
