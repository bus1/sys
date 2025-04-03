//! # Program Argument Layout
//!
//! This module allows creating layouts for program arguments, which can
//! then be used to parse program arguments for.
//!
//! Program arguments consist of:
//!
//! - **Commands**: Commands are the high-level functionality that is to be
//!   invoked. A single command is selected per invocation, and available
//!   commands form a tree. A command can be selected by specifying the path
//!   to the command:
//!
//!   > ./program root sub [FLAGS..] [PARAMETERS..]
//!
//!   This would select a command called `sub` which lives under the
//!   root-level command called `root`.
//!
//!   Every command can have associated flags, and the flags of a command can
//!   be specified after the command-selector, or after any sub-command
//!   selector (sub-command flags take precedence on conflict).
//!
//!   Any unknown sub-command will terminate the command-selector and be
//!   treated as parameter. All parameters are passed to the selected
//!   sub-command.
//!
//!   As part of the layout, all commands provide their full path. The tree
//!   layout is compiled internally and not exposed as part of the
//!   configuration. It is valid to define sub-commands without defining a
//!   matchin root-level command.
//!
//!   For instance, a layout with commands registered for the following paths
//!   would parse the listed invocations:
//!
//!   > []
//!   > ["foo"]
//!   > ["foo", "A"]
//!   > ["foo", "B"]
//!   >
//!   > ./program [FLAGS..]
//!   > ./program foo [FLAGS..]
//!   > ./program foo A [FLAGS..]
//!   > ./program foo B [FLAGS..]
//!
//!   Omitting one of the configured commands would lead to the matching
//!   invocation to be treated as error.
//!
//! - **Flags**: The behavior of a program can be controlled via flags. They
//!   allow structured arguments to a selected command. A specific flag can be
//!   passed via the double-colon syntax:
//!
//!   > ./program --flag
//!
//!   Flags that can be toggled can additionally be prefixed with `no-`:
//!
//!   > ./program --no-flag
//!
//!   Lastly, flags that take values can be specified in two ways:
//!
//!   > ./program --flag value
//!   > ./program --flag=value
//!
//!   Depending on the flag, following flags might override or extend previous
//!   values. In the case of bare flags that can only be set, specifying a
//!   flag multiple times has no additional affect. Toggleable flags will
//!   override previous definitions. Flags that take a value depend on the
//!   underlying parser. Some override previous values, while others collect
//!   all flag values in a list. Additionally, using the `no-` previous can be
//!   used by such collectors to clear the list and start over.
//!
//!   Flags are always associated with a command and can only be specified when
//!   the given command is part of the selected command path. A flag `bar` that
//!   is configured for the root-level command `foo` can be specified in the
//!   following positions:
//!
//!   > ./program foo --bar
//!   > ./program foo --bar sub
//!   > ./program foo sub --bar
//!   > ./program foo sub parameters --bar
//!   > ./program foo sub --bar parameters
//!
//! - **Parameters**: All arguments to a program that do not parse will be
//!   treated as parameters to the selected command. The first parameter
//!   terminates the command-selection, but flags can still be provided.
//!
//!   Note that an empty flag `--` terminates all argument parsing and
//!   following arguments will be treated as parameters unconditionally. This
//!   is also the recommended way to pass any parameters not under control of
//!   the caller, since it will prevent parameters starting with dashes to be
//!   interpreted as arguments.
//!
//! The layout module allows assembling a static representation of valid
//! program arguments. The actual processing of parameters, flags, and the
//! selected command is propagated to the provided report objects. The layout
//! merely defines the structure of the arguments.
//!
//! To parse program arguments into report objects, the layout must be
//! available for mutable access (to allow writing the report object). For mere
//! introspection of the layout immutable access is sufficient (e.g., to
//! compile help information).
//!
//! Unfortunately, Rust lacks suitable tools to easily transform between trees
//! of mutable references to trees of immutable references. Therefore, a lot of
//! the accessors return indices instead of references. Those indices are
//! guaranteed to be valid and stable for a given layout, even if mutable.

use crate::args;

/// Enumeration of possible modes for a flag. This defines flag behavior that
/// affects the argument parser. All other properties of the flag are deferred
/// to the report.
#[derive(Clone, Copy, Debug)]
pub enum FlagMode {
    /// Flag can be set only (unary mode). It takes no arguments and has to be
    /// present in the argument list verbatim.
    Set,
    /// Flag can be toggled (boolean mode). It takes no arguments but can be
    /// present with a `no-` prefix for inversion.
    Toggle,
    /// Flag takes a value (parser mode). The following argument is taken
    /// verbatim as value for this flag.
    Parse,
}

/// Defining properties of an individual flag apart from its location in the
/// layout hierarchy. Open-coded to allow borrowing multiple fields
/// simultaneously.
pub struct Flag<'schema, 'args, R> {
    pub(super) name: &'schema str,
    pub(super) mode: FlagMode,
    pub(super) report: &'schema mut dyn args::FlagReport<'args, R>,
    pub(super) help_short: Option<&'schema str>,
}

/// Fixed-size array of flag definitions, compiled for faster lookups. This is
/// effectively an array of `Flag` values, but hidden to prevent modifications
/// and to guarantee internal invariants.
///
/// The array is guaranteed to be ordered based on depth-first pre-order tree
/// traversal. That is, a node is directly followed by all its sub-nodes. Each
/// level is ordered based on their lexicographic order.
pub struct FlagSet<'schema, 'args, const N: usize, R> {
    inner: [FlagDef<'schema, 'args, R>; N],
}

/// Variable-sized reference to a `FlagSet`. This is effectively a slice of
/// `Flag` values, provided by a `FlagSet`.
#[repr(transparent)]
pub struct FlagSetRef<'schema, 'args, R> {
    inner: [FlagDef<'schema, 'args, R>],
}

/// An iterator over the flags in a `FlagSet`.
pub struct FlagSetIter<'this, 'schema, 'args, R>(core::slice::Iter<'this, FlagDef<'schema, 'args, R>>);

/// Defining properties of an individual command apart from its location in the
/// layout hierarchy. Open-coded to allow borrowing multiple fields
/// simultaneously.
pub struct Command<'schema, 'args, R> {
    pub(super) path: &'schema [&'schema str],
    pub(super) report: &'schema mut dyn args::CommandReport<'args, R>,
    pub(super) flags: &'schema mut FlagSetRef<'schema, 'args, R>,
    pub(super) help_short: Option<&'schema str>,
}

/// Fixed-size array of command definitions, compiled for faster lookups. This
/// is effectively an array of `Command` values, but hidden from access to
/// prevent modifications and to guarantee internal invariants.
pub struct CommandSet<'schema, 'args, const N: usize, R> {
    inner: [CommandDef<'schema, 'args, R>; N],
}

/// Variable-sized reference to a `CommandSet`. This is effectively a slice of
/// `Command` values provided by a `CommandSet`.
#[repr(transparent)]
pub struct CommandSetRef<'schema, 'args, R> {
    inner: [CommandDef<'schema, 'args, R>],
}

/// An iterator over the commands in a `CommandSet`.
pub struct CommandSetIter<'this, 'schema, 'args, R>(core::slice::Iter<'this, CommandDef<'schema, 'args, R>>);

/// Top-level definition of a layout of program arguments. This defines the
/// full layout of arguments to a program.
///
/// The `'schema` lifetime is used to reference any keys used as identifiers
/// for sub-elements of the schema. These keys must live at least as long as
/// the schema to ensure it can be traversed and introspected.
///
/// The `'args` lifetime is forwarded to the report objects, and not used for
/// any other purposes.
pub struct Schema<'schema, 'args, R> {
    commands: &'schema mut CommandSetRef<'schema, 'args, R>,
}

// Internal information on flag definitions.
pub(super) struct FlagDef<'schema, 'args, R> {
    info: Flag<'schema, 'args, R>,
}

// Internal information on command definitions.
pub(super) struct CommandDef<'schema, 'args, R> {
    info: Command<'schema, 'args, R>,
    up: Option<usize>,
}

impl<'schema, 'args, R> Flag<'schema, 'args, R> {
    /// Create a new flag with the provided information.
    pub fn with(
        name: &'schema str,
        mode: FlagMode,
        report: &'schema mut dyn args::FlagReport<'args, R>,
        help_short: Option<&'schema str>,
    ) -> Self {
        Self {
            name: name,
            mode: mode,
            report: report,
            help_short: help_short,
        }
    }

    /// Yield the name of the flag.
    pub fn name(&self) -> &'schema str {
        self.name
    }

    /// Yield the mode of the flag.
    pub fn mode(&self) -> FlagMode {
        self.mode
    }

    /// Yield the report object of the flag.
    pub fn report_mut(&mut self) -> &mut dyn args::FlagReport<'args, R> {
        self.report
    }

    /// Yield the short-help of the flag.
    pub fn help_short(&self) -> Option<&'schema str> {
        self.help_short
    }
}

impl<'schema, 'args, R> FlagDef<'schema, 'args, R> {
    fn with(flag: Flag<'schema, 'args, R>) -> Self {
        Self {
            info: flag,
        }
    }
}

impl<'schema, 'args, const N: usize, R> FlagSet<'schema, 'args, N, R> {
    fn compile(&mut self) {
        // Sort all flags by their name. We need this during lookups to ensure
        // a stable binary-search.
        self.inner.sort_unstable_by(|lhs, rhs| {
            lhs.info.name.cmp(rhs.info.name)
        });
    }

    /// Create a new array of flags from the provided flag information. This
    /// will compile the array for faster lookups.
    pub fn with(flags: [Flag<'schema, 'args, R>; N]) -> Self {
        let mut this = Self {
            inner: flags.map(|flag| FlagDef::with(flag)),
        };

        this.compile();
        this
    }

    /// Cast the reference of a sized array of flags to a fat-reference of an
    /// unsized slice of flags.
    pub fn as_ref(&self) -> &FlagSetRef<'schema, 'args, R> {
        let inner: &[FlagDef<'schema, 'args, R>] = &self.inner;

        // SAFETY: `*SetRef` is a transparent wrapper around `inner` with
        //         the same invariants.
        unsafe {
            core::mem::transmute(inner)
        }
    }

    /// Cast the mutable reference of a sized array of flags to a mutable
    /// fat-reference of an unsized slice of flags.
    pub fn as_mut(&mut self) -> &mut FlagSetRef<'schema, 'args, R> {
        let inner: &mut [FlagDef<'schema, 'args, R>] = &mut self.inner;

        // SAFETY: `*SetRef` is a transparent wrapper around `inner` with
        //         the same invariants.
        unsafe {
            core::mem::transmute(inner)
        }
    }
}

impl<'schema, 'args, R> FlagSetRef<'schema, 'args, R> {
    /// Yield the flag at the specified position.
    pub fn flag_at(
        &self,
        at: usize,
    ) -> &Flag<'schema, 'args, R> {
        &self.inner[at].info
    }

    /// Yield the mutable flag at the specified position.
    pub fn flag_mut_at(
        &mut self,
        at: usize,
    ) -> &mut Flag<'schema, 'args, R> {
        &mut self.inner[at].info
    }

    /// Yield an iterator over all flags.
    pub fn iter(&self) -> FlagSetIter<'_, 'schema, 'args, R> {
        FlagSetIter(self.inner.iter())
    }

    /// Yield the length of the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Perform a binary search on the inner array of flags, using
    /// `binary_search_by` of the standard library.
    pub fn search_by<'this, F>(&'this self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&'this Flag<'schema, 'args, R>) -> core::cmp::Ordering,
    {
        self.inner.binary_search_by(|v| f(&v.info))
    }
}

impl<'schema, 'args, R> Command<'schema, 'args, R> {
    /// Create a new command with the provided information.
    pub fn with(
        path: &'schema [&'schema str],
        report: &'schema mut dyn args::CommandReport<'args, R>,
        flags: &'schema mut FlagSetRef<'schema, 'args, R>,
        help_short: Option<&'schema str>,
    ) -> Self {
        Self {
            path: path,
            report: report,
            flags: flags,
            help_short: help_short,
        }
    }

    /// Yield the path of this command.
    pub fn path(&self) -> &'schema [&'schema str] {
        self.path
    }

    /// Yield the report object of this command.
    pub fn report_mut(&mut self) -> &mut dyn args::CommandReport<'args, R> {
        self.report
    }

    /// Yield the flags of this command.
    pub fn flags(&self) -> &FlagSetRef<'schema, 'args, R> {
        self.flags
    }

    /// Yield the mutable flags of this command.
    pub fn flags_mut(&mut self) -> &mut FlagSetRef<'schema, 'args, R> {
        self.flags
    }

    /// Yield the short-help of the command.
    pub fn help_short(&self) -> Option<&'schema str> {
        self.help_short
    }

    /// Yield a slice-iterator over all flags.
    pub fn flags_iter(&self) -> FlagSetIter<'_, 'schema, 'args, R> {
        self.flags.iter()
    }

    /// Yield the flag at the specified position.
    pub fn flag_at(
        &self,
        at: usize,
    ) -> &Flag<'schema, 'args, R> {
        self.flags.flag_at(at)
    }

    /// Yield the mutable flag at the specified position.
    pub fn flag_mut_at(
        &mut self,
        at: usize,
    ) -> &mut Flag<'schema, 'args, R> {
        self.flags.flag_mut_at(at)
    }
}

impl<'schema, 'args, R> CommandDef<'schema, 'args, R> {
    fn with(command: Command<'schema, 'args, R>) -> Self {
        Self {
            info: command,
            up: None,
        }
    }
}

impl<'schema, 'args, const N: usize, R> CommandSet<'schema, 'args, N, R> {
    fn compile(&mut self) {
        // Sort all commands by their lexicographic order. We need this during
        // lookups to ensure a stable binary-search.
        self.inner.sort_unstable_by(|lhs, rhs| {
            lhs.info.path.cmp(rhs.info.path)
        });

        // Now iterate the sorted commands and build backpointers from each
        // entry to its logically preceding element. This effectively builds a
        // tree that allows fast traversals without backtracking.
        //
        // NB: This computation is linear, despite its nested loop. The nested
        //     loop will never visit an element twice, except if it terminates.
        let cmds = &mut self.inner;
        for i in 0..cmds.len() {
            assert_eq!(cmds[i].up, None);

            let mut o_up = match i {
                0 => None,
                _ => Some(i - 1),
            };

            while let Some(up) = o_up {
                if cmds[i].info.path.starts_with(cmds[up].info.path) {
                    cmds[i].up = Some(up);
                    break;
                }
                o_up = cmds[up].up;
            }
        }
    }

    /// Create a new array of commands from the provided command information.
    /// This will compile the array for faster lookups.
    pub fn with(commands: [Command<'schema, 'args, R>; N]) -> Self {
        let mut this = Self {
            inner: commands.map(|cmd| CommandDef::with(cmd)),
        };

        this.compile();
        this
    }

    /// Cast the reference of a sized array of commands to a fat-reference of
    /// an unsized slice of commands.
    pub fn as_ref(&self) -> &CommandSetRef<'schema, 'args, R> {
        let inner: &[CommandDef<'schema, 'args, R>] = &self.inner;

        // SAFETY: `*SetRef` is a transparent wrapper around `inner` with
        //         the same invariants.
        unsafe {
            core::mem::transmute(inner)
        }
    }

    /// Cast the mutable reference of a sized array of commands to a
    /// mutable fat-reference of an unsized slice of commands.
    pub fn as_mut(&mut self) -> &mut CommandSetRef<'schema, 'args, R> {
        let inner: &mut [CommandDef<'schema, 'args, R>] = &mut self.inner;

        // SAFETY: `*SetRef` is a transparent wrapper around `inner` with
        //         the same invariants.
        unsafe {
            core::mem::transmute(inner)
        }
    }
}

impl<'schema, 'args, R> CommandSetRef<'schema, 'args, R> {
    /// Yield the command at the specified position.
    pub fn command_at(
        &self,
        at: usize,
    ) -> &Command<'schema, 'args, R> {
        &self.inner[at].info
    }

    /// Yield the mutable command at the specified position.
    pub fn command_mut_at(
        &mut self,
        at: usize,
    ) -> &mut Command<'schema, 'args, R> {
        &mut self.inner[at].info
    }

    /// Yield an iterator over all commands.
    pub fn iter(&self) -> CommandSetIter<'_, 'schema, 'args, R> {
        CommandSetIter(self.inner.iter())
    }

    /// Yield an iterator over all commands starting at the given index.
    pub fn iter_from(
        &self,
        from: usize,
    ) -> CommandSetIter<'_, 'schema, 'args, R> {
        CommandSetIter(self.inner[from..].iter())
    }

    /// Yield the length of the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Perform a binary search on the inner array of commands, using
    /// `binary_search_by` of the standard library.
    pub fn search_by<'a, F>(&'a self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Command<'schema, 'args, R>) -> core::cmp::Ordering,
    {
        self.inner.binary_search_by(|v| f(&v.info))
    }

    /// Yield the command on the next layer up of the command at the specified
    /// position. Yield `None` if no such layer up is defined.
    ///
    /// NB: This can skip layers if they are not defined.
    pub fn up_from(&self, from: usize) -> Option<usize> {
        self.inner[from].up
    }
}

impl<'schema, 'args, R> Schema<'schema, 'args, R> {
    /// Create a new schema with the given set of commands.
    pub fn with(
        commands: &'schema mut CommandSetRef<'schema, 'args, R>,
    ) -> Self {
        Self {
            commands: commands,
        }
    }

    /// Yield the commands of this schema.
    pub fn commands(&self) -> &CommandSetRef<'schema, 'args, R> {
        self.commands
    }

    /// Yield the mutable commands of this schema.
    pub fn commands_mut(&mut self) -> &mut CommandSetRef<'schema, 'args, R> {
        self.commands
    }

    /// Yield a slice-iterator over all commands.
    pub fn commands_iter(&self) -> CommandSetIter<'_, 'schema, 'args, R> {
        self.commands.iter()
    }

    /// Yield the command at the specified position.
    pub fn command_at(
        &self,
        at: usize,
    ) -> &Command<'schema, 'args, R> {
        self.commands.command_at(at)
    }

    /// Yield the mutable command at the specified position.
    pub fn command_mut_at(
        &mut self,
        at: usize,
    ) -> &mut Command<'schema, 'args, R> {
        self.commands.command_mut_at(at)
    }

    /// Yield the flag at the specified position.
    pub fn flag_at(
        &self,
        (at_command, at_flag): (usize, usize),
    ) -> &Flag<'schema, 'args, R> {
        self.command_at(at_command).flag_at(at_flag)
    }

    /// Yield the mutable flag at the specified position.
    pub fn flag_mut_at(
        &mut self,
        (at_command, at_flag): (usize, usize),
    ) -> &mut Flag<'schema, 'args, R> {
        self.command_mut_at(at_command).flag_mut_at(at_flag)
    }
}

impl<'this, 'schema, 'args, R>
    core::clone::Clone
for
    FlagSetIter<'this, 'schema, 'args, R>
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'this, 'schema, 'args, R>
    core::clone::Clone
for
    CommandSetIter<'this, 'schema, 'args, R>
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'schema, 'args, const N: usize, R>
    core::ops::Deref
for
    FlagSet<'schema, 'args, N, R>
{
    type Target = FlagSetRef<'schema, 'args, R>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'schema, 'args, const N: usize, R>
    core::ops::DerefMut
for
    FlagSet<'schema, 'args, N, R>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'schema, 'args, const N: usize, R>
    core::ops::Deref
for
    CommandSet<'schema, 'args, N, R>
{
    type Target = CommandSetRef<'schema, 'args, R>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'schema, 'args, const N: usize, R>
    core::ops::DerefMut
for
    CommandSet<'schema, 'args, N, R>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'this, 'schema, 'args, R>
    core::iter::Iterator
for
    FlagSetIter<'this, 'schema, 'args, R>
{
    type Item = &'this Flag<'schema, 'args, R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| &v.info)
    }
}

impl<'this, 'schema, 'args, R>
    core::iter::Iterator
for
    CommandSetIter<'this, 'schema, 'args, R>
{
    type Item = &'this Command<'schema, 'args, R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| &v.info)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use crate::compat;
    use super::*;

    #[test]
    fn layout_empty() {
        let mut commands = CommandSet::with([]);
        let schema = Schema::<()>::with(&mut commands);

        assert_eq!(schema.commands.inner.len(), 0);
    }

    #[test]
    fn layout_command() {
        let mut command_a: Option<Vec<&compat::OsStr>> = None;

        let mut command_a_flags = FlagSet::with([]);
        let mut commands = CommandSet::with([
            Command::with(&["A"], &mut command_a, &mut command_a_flags, None),
        ]);
        let schema = Schema::<()>::with(&mut commands);

        assert_eq!(schema.commands.inner.len(), 1);
    }

    #[test]
    fn layout_commands() {
        let mut command_a: Option<Vec<&compat::OsStr>> = None;
        let mut command_b: Option<Vec<&compat::OsStr>> = None;
        let mut command_c: Option<Vec<&compat::OsStr>> = None;

        let mut command_a_flags = FlagSet::with([]);
        let mut command_b_flags = FlagSet::with([]);
        let mut command_c_flags = FlagSet::with([]);
        let mut commands = CommandSet::with([
            Command::with(&["A"], &mut command_a, &mut command_a_flags, None),
            Command::with(&["B"], &mut command_b, &mut command_b_flags, None),
            Command::with(&["C"], &mut command_c, &mut command_c_flags, None),
        ]);
        let schema = Schema::<()>::with(&mut commands);

        assert_eq!(schema.commands.inner.len(), 3);
    }

    #[test]
    fn layout_flag() {
        let mut flag_x: Option<()> = None;
        let mut command_a: Option<Vec<&compat::OsStr>> = None;

        let mut command_a_flags = FlagSet::with([
            Flag::with("x", FlagMode::Set, &mut flag_x, None),
        ]);
        let mut commands = CommandSet::with([
            Command::with(&["A"], &mut command_a, &mut command_a_flags, None),
        ]);
        let schema = Schema::<()>::with(&mut commands);

        assert_eq!(schema.commands.inner.len(), 1);
        assert_eq!(schema.commands.inner[0].info.flags.inner.len(), 1);
    }

    #[test]
    fn layout_flags() {
        let mut flag_x: Option<()> = None;
        let mut flag_y: Option<()> = None;
        let mut flag_z: Option<()> = None;
        let mut command_a: Option<Vec<&compat::OsStr>> = None;

        let mut command_a_flags = FlagSet::with([
            Flag::with("x", FlagMode::Set, &mut flag_x, None),
            Flag::with("y", FlagMode::Set, &mut flag_y, None),
            Flag::with("z", FlagMode::Set, &mut flag_z, None),
        ]);
        let mut commands = CommandSet::with([
            Command::with(&["A"], &mut command_a, &mut command_a_flags, None),
        ]);
        let schema = Schema::<()>::with(&mut commands);

        assert_eq!(schema.commands.inner.len(), 1);
        assert_eq!(schema.commands.inner[0].info.flags.inner.len(), 3);
    }

    #[test]
    fn layout_all() {
        let mut flag_x0: Option<()> = None;
        let mut flag_y0: Option<()> = None;
        let mut flag_y1: Option<()> = None;
        let mut flag_z0: Option<()> = None;
        let mut flag_z1: Option<()> = None;
        let mut flag_z2: Option<()> = None;
        let mut command_a: Option<Vec<&compat::OsStr>> = None;
        let mut command_b: Option<Vec<&compat::OsStr>> = None;
        let mut command_c: Option<Vec<&compat::OsStr>> = None;

        let mut command_a_flags = FlagSet::with([
            Flag::with("x0", FlagMode::Set, &mut flag_x0, None),
        ]);
        let mut command_b_flags = FlagSet::with([
            Flag::with("y0", FlagMode::Set, &mut flag_y0, None),
            Flag::with("y1", FlagMode::Set, &mut flag_y1, None),
        ]);
        let mut command_c_flags = FlagSet::with([
            Flag::with("z0", FlagMode::Set, &mut flag_z0, None),
            Flag::with("z1", FlagMode::Set, &mut flag_z1, None),
            Flag::with("z2", FlagMode::Set, &mut flag_z2, None),
        ]);
        let mut commands = CommandSet::with([
            Command::with(&["A"], &mut command_a, &mut command_a_flags, None),
            Command::with(&["B"], &mut command_b, &mut command_b_flags, None),
            Command::with(&["C"], &mut command_c, &mut command_c_flags, None),
        ]);
        let schema = Schema::<()>::with(&mut commands);

        assert_eq!(schema.commands.inner.len(), 3);
        assert_eq!(schema.commands.inner[0].info.flags.inner.len(), 1);
        assert_eq!(schema.commands.inner[1].info.flags.inner.len(), 2);
        assert_eq!(schema.commands.inner[2].info.flags.inner.len(), 3);
    }
}
