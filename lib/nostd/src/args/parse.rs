//! # Program Argument Parser
//!
//! Parse raw program arguments into argument reports, following a caller
//! provided argument layout. The layout provides exclusive access to the
//! underlying argument reports, which are invoked by the parser.

use crate::{args, compat};

#[derive(Clone, Copy)]
struct CursorPosition {
    pub index: usize,
    pub level: usize,
}

struct Cursor<'this, 'schema, 'args, R> {
    parser: &'this mut dyn args::ParserReport<'args, R>,
    schema: &'this mut args::Schema<'schema, 'args, R>,
    cursor: Option<CursorPosition>,
}

struct Parser<'this, 'schema, 'args, R> {
    _this: core::marker::PhantomData<&'this mut args::Schema<'schema, 'args, R>>,
    commands_finalized: bool,
    flags_finalized: bool,
}

impl<'this, 'schema, 'args, R> Cursor<'this, 'schema, 'args, R> {
    fn with(
        parser: &'this mut dyn args::ParserReport<'args, R>,
        schema: &'this mut args::Schema<'schema, 'args, R>,
    ) -> Self {
        // Create a cursor pointing to the first element in the list, but with
        // a level of 0. If the first element is a root element, this will
        // match it nicely. Otherwise, the level of 0 will ensure that the path
        // of the first element does not matter and we just use it as anchor.
        let cursor = (!schema.commands().is_empty()).then_some(
            CursorPosition { index: 0, level: 0, }
        );

        Self {
            parser: parser,
            schema: schema,
            cursor: cursor,
        }
    }

    fn key(&self) -> &[&'schema str] {
        // The key of the cursor position is the key of the pointed element,
        // but reduced to the prefix we have entered so far.
        match self.cursor {
            None => &[],
            Some(v) => &self.schema.command_at(v.index).path[0..v.level],
        }
    }

    fn command(&self) -> Option<usize> {
        // The command of the cursor position is the pointed element, but only
        // if we have entered its path entirely. Otherwise, we point to a
        // non-existing intermediate element, which has no entry.
        self.cursor.and_then(|v| {
            (self.schema.command_at(v.index).path.len() <= v.level)
                .then_some(v.index)
        })
    }

    fn command_or_up(&self) -> Option<usize> {
        // Return the same as `command()`, but fall back to its upper element
        // if there is none.
        self.cursor.and_then(|v| {
            (self.schema.command_at(v.index).path.len() <= v.level)
                .then_some(v.index)
                .or_else(|| self.schema.commands().up_from(v.index))
        })
    }

    fn report_error(
        &mut self,
        error: args::Error<'args>,
    ) -> core::ops::ControlFlow<R> {
        let mut context = args::ParserContext::new();
        self.parser.report_error(&mut context, error)
    }

    fn report_parameter_for(
        &mut self,
        idx_command: usize,
        o_parameter: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        let mut context = args::CommandContext::with(self.parser);
        self.schema.command_mut_at(idx_command).report.report_parameter(&mut context, o_parameter)
    }

    fn report_parameter(
        &mut self,
        o_parameter: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        if let Some(v) = self.command() {
            self.report_parameter_for(v, o_parameter)
        } else if let Some(parameter) = o_parameter {
            self.report_error(args::Error::ParameterUnexpected {
                parameter: parameter,
            })
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }

    fn report_set_for(
        &mut self,
        (idx_command, idx_flag): (usize, usize),
        flag_arg: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        let at = self.command_or_up().unwrap_or(idx_command);
        let command = self.schema.command_mut_at(idx_command);
        let flag = command.flags.flag_mut_at(idx_flag);
        let mut context = args::FlagContext::with(
            self.parser,
            command.report,
            at,
            flag_arg,
        );
        flag.report.report_set(&mut context)
    }

    fn report_toggle_for(
        &mut self,
        (idx_command, idx_flag): (usize, usize),
        flag_arg: &'args compat::OsStr,
        value: bool,
    ) -> core::ops::ControlFlow<R> {
        let at = self.command_or_up().unwrap_or(idx_command);
        let command = self.schema.command_mut_at(idx_command);
        let flag = command.flags.flag_mut_at(idx_flag);
        let mut context = args::FlagContext::with(
            self.parser,
            command.report,
            at,
            flag_arg,
        );
        flag.report.report_toggle(&mut context, value)
    }

    fn report_parse_for(
        &mut self,
        (idx_command, idx_flag): (usize, usize),
        flag_arg: &'args compat::OsStr,
        value: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        let at = self.command_or_up().unwrap_or(idx_command);
        let command = self.schema.command_mut_at(idx_command);
        let flag = command.flags.flag_mut_at(idx_flag);
        let mut context = args::FlagContext::with(
            self.parser,
            command.report,
            at,
            flag_arg,
        );
        flag.report.report_parse(&mut context, value)
    }

    fn compare_prefix(
        element: &[&str],
        prefix: (&[&str], &str),
    ) -> core::cmp::Ordering {
        // With this comparator we want to search for an element with a
        // matching prefix. Elements are sorted lexicographically, so we
        // perform normal lexicographic comparisons, but shortcut if the prefix
        // matches. This works fine with a binary-search, as long as this
        // comparator is only used for lookups, not sorting.
        //
        // Note that the prefix is given with a separate final key element to
        // avoid allocations.
        let n_subprefix = prefix.0.len();
        let n_element = element.len();
        let n = core::cmp::min(n_subprefix, n_element);

        // Sub-slicing allows the compiler to omit index-checks in
        // the loop, since all involved ranges are equal.
        let sub_prefix = &prefix.0[..n];
        let sub_element = &element[..n];

        for i in 0..n {
            match sub_element[i].cmp(sub_prefix[i]) {
                core::cmp::Ordering::Equal => (),
                v => return v,
            }
        }

        if n_element > n_subprefix {
            return element[n_subprefix].cmp(prefix.1);
        }

        n_element.cmp(&(n_subprefix + 1))
    }

    fn enter(&mut self, name: &str) -> bool {
        // Append a path-element to the current cursor position. This will look
        // through the command-list and see whether any element has the
        // extended path as prefix. If not, this will return `false` and retain
        // the cursor position. Otherwise, it will advance the cursor to the
        // element that matches the new path, if it exists, otherwise the
        // lowest element with it as prefix. In both cases it will return
        // `true`.

        let key = (self.key(), name);
        let level = key.0.len() + 1;

        // Find _an_ element that has a matching prefix with the new key. Since
        // this is a binary-search, it does not necessarily find the first.
        //
        // If the command-nodes are extended to contain the size of their
        // respective sub-trees, we could limit the searches to the sub-tree of
        // possible candidates. However, the full search is suitably fast with
        // logarithmic behavior and thus we rather minimize the node sizes.
        // This can be adjusted in the future, if lookups need to be faster.
        let mut cand = match self.schema.commands().search_by(
            |v| Self::compare_prefix(v.path, key)
        ) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // Back up for as long as preceding elements have a matching prefix to
        // find the first element with a matching prefix.
        while
            cand > 0 && Self::compare_prefix(
                self.schema.command_at(cand - 1).path,
                key,
            ).is_eq()
        {
            cand -= 1;
        }

        self.cursor = Some(CursorPosition {
            index: cand,
            level: level,
        });

        true
    }

    fn find_flag(
        &self,
        name: &str,
    ) -> Option<(usize, usize)> {
        let mut o_idx = self.cursor.as_ref().map(|v| v.index);

        while let Some(idx) = o_idx {
            if let Ok(v) = self.schema.command_at(idx)
                .flags()
                .search_by(|v| v.name.cmp(name))
            {
                return Some((idx, v));
            }

            o_idx = self.schema.commands().up_from(idx);
        }

        None
    }
}

impl<'this, 'schema, 'args, R> Parser<'this, 'schema, 'args, R> {
    fn new() -> Self {
        Self {
            _this: Default::default(),
            commands_finalized: false,
            flags_finalized: false,
        }
    }

    fn parse_short(
        &mut self,
        cursor: &mut Cursor<'this, 'schema, 'args, R>,
        short_str: &'args compat::OsStr,
    ) -> core::ops::ControlFlow<R> {
        // Our configuration does not allow specifying short options, so none
        // of these can ever match. Hence, treat them all as invalid for now
        // and signal an error. Then ignore the argument and continue.
        cursor.report_error(args::Error::ShortsUnknown {
            shorts: short_str,
        })
    }

    fn parse_long(
        &mut self,
        cursor: &mut Cursor<'this, 'schema, 'args, R>,
        arguments: &mut dyn Iterator<Item = &'args compat::OsStr>,
        flag_str: &'args str,
        value_opt: Option<&'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        let (idx, flag_toggled) = match cursor.find_flag(flag_str) {
            Some(v) => (v, None),
            None => match flag_str.strip_prefix("no-") {
                None => {
                    return cursor.report_error(args::Error::FlagUnknown {
                        flag: flag_str.into(),
                    });
                },
                Some(stripped) => match cursor.find_flag(stripped) {
                    None => {
                        return cursor.report_error(args::Error::FlagUnknown {
                            flag: flag_str.into(),
                        });
                    },
                    Some(v) => (v, Some(stripped)),
                }
            },
        };

        let flag_mode = cursor.schema.flag_at(idx).mode;

        match (flag_mode, flag_toggled, value_opt) {
            (args::FlagMode::Set, Some(_), _)
            | (args::FlagMode::Parse, Some(_), _) => {
                // Flag only exists without `no-*` prefix, but this flag cannot
                // be toggled. Hence, signal an error and ignore the argument.
                cursor.report_error(args::Error::FlagUnexpectedToggle {
                    flag: flag_str.into(),
                })?;
            },
            (args::FlagMode::Set, None, Some(v)) => {
                // Flag is nullary but a value was assigned inline. Signal an
                // error and ignore the argument.
                cursor.report_error(args::Error::FlagUnexpectedValue {
                    flag: flag_str.into(),
                    value: v,
                })?;
            },
            (args::FlagMode::Toggle, _, Some(v)) => {
                // Flag is nullary but a value was assigned inline. Signal an
                // error and ignore the argument.
                cursor.report_error(args::Error::FlagUnexpectedValue {
                    flag: flag_str.into(),
                    value: v,
                })?;
            },
            (args::FlagMode::Set, None, None) => {
                // Correct use of settable-flag.
                cursor.report_set_for(idx, flag_str.into())?;
            },
            (args::FlagMode::Toggle, t, None) => {
                // Correct use of toggle-flag.
                cursor.report_toggle_for(idx, flag_str.into(), t.is_none())?;
            },
            (args::FlagMode::Parse, None, None) => {
                // Flag requires a value, so fetch it.
                match arguments.next() {
                    None => {
                        cursor.report_error(args::Error::FlagNoValue {
                            flag: flag_str.into(),
                        })?;
                    },
                    Some(v) => {
                        cursor.report_parse_for(idx, flag_str.into(), v)?;
                    },
                }
            },
            (args::FlagMode::Parse, None, Some(v)) => {
                // Flag requires a value that was passed inline.
                cursor.report_parse_for(idx, flag_str.into(), v)?;
            },
        }

        core::ops::ControlFlow::Continue(())
    }

    fn parse_command(
        &mut self,
        cursor: &mut Cursor<'this, 'schema, 'args, R>,
        arg_os: &'args compat::OsStr,
        arg_str_opt: Option<&'args str>,
    ) -> core::ops::ControlFlow<R> {
        let entered = match arg_str_opt {
            None => false,
            Some(v) => cursor.enter(v),
        };

        if !entered {
            // The argument does not represent a valid sub-command to enter.
            // This ends the sub-command chain and treats the argument as
            // parameter.
            self.commands_finalized = true;
            cursor.report_parameter(Some(arg_os))?;
        }

        core::ops::ControlFlow::Continue(())
    }

    fn parse_cursor(
        &mut self,
        cursor: &mut Cursor<'this, 'schema, 'args, R>,
        arguments: &mut dyn Iterator<Item = &'args compat::OsStr>,
    ) -> core::ops::ControlFlow<R> {
        loop {
            let arg_os = match arguments.next() {
                None => break,
                Some(v) => v,
            };

            // If all parsing is finalized, shortcut everything.
            if self.commands_finalized && self.flags_finalized {
                cursor.report_parameter(Some(arg_os))?;
                continue;
            }

            // Get the UTF-8 prefix of the argument. Anything we can parse must
            // be valid UTF-8, but some of it might be trailed by arbitrary OS
            // data (e.g., `--path=./some/path` can contain trailing non-UTF-8
            // data). This performs a UTF-8 check on all arguments, but avoids
            // any allocation. Hence, you can parse large data chunks as
            // arguments without incurring anything more expensive than a UTF-8
            // check. For anything bigger than this use `--` or a side-channel.
            let arg_bytes = arg_os.as_encoded_bytes();
            let (arg_front, arg_tail) = match core::str::from_utf8(arg_bytes) {
                Ok(v) => (v, false),
                Err(e) => unsafe {
                    // SAFETY: `Utf8Error::valid_up_to()` points exactly at the
                    //         first byte past a valid UTF-8 section, so we can
                    //         safely cast it to a `str` unchecked.
                    let v = &arg_bytes[..e.valid_up_to()];
                    (core::str::from_utf8_unchecked(v), true)
                },
            };

            if !self.flags_finalized {
                // See whether this argument starts with `--` and thus
                // specifies a flag. This can be one of: `--`, `--flag`, or
                // `--flag=value`. So first decode the argument into flag
                // and value, then handle the distinct cases.
                if let Some(arg_front_dd) = arg_front.strip_prefix("--") {
                    let (flag, unknown, value) = match arg_front_dd.split_once('=') {
                        None => (arg_front_dd, arg_tail, None),
                        Some((before, _)) => {
                            let v = unsafe {
                                // SAFETY: We split off a well-defined UTF-8
                                //         sequence, which is allowed for
                                //         `std::ffi::OsStr`.
                                compat::OsStr::from_encoded_bytes_unchecked(
                                    &arg_bytes[2+before.len()+1..],
                                )
                            };
                            (before, false, Some(v))
                        },
                    };

                    match (flag, unknown, value) {
                        (_, true, _) => {
                            // We have invalid UTF-8 as part of the flag name
                            // (i.e., before any possible `=`). This cannot
                            // match any flag we know.
                            cursor.report_error(args::Error::FlagUnknown {
                                flag: arg_os,
                            })?;
                        },

                        ("", false, None) => {
                            // We got an empty flag. This ends all parsing and
                            // treats all remaining arguments as parameters.
                            self.commands_finalized = true;
                            self.flags_finalized = true;
                        },

                        (_, false, _) => {
                            // We got a complete flag with or without value.
                            // Look up the flag and pass the value along, if
                            // required.
                            self.parse_long(cursor, arguments, flag, value)?;
                        },
                    }

                    // Argument was parsed as flag.
                    continue;
                }

                // See whether the argument specifies short flags. Multiple
                // ones might be combined into a single argument. Note that a
                // single dash without following flags has no special meaning
                // and we do not handle it here.
                if arg_bytes.len() >= 2 && arg_bytes[0] == b'-' {
                    self.parse_short(cursor, arg_os)?;

                    // Argument was parsed as flag.
                    continue;
                }
            }

            if !self.commands_finalized {
                // This argument is either a sub-command or a parameter of the
                // current command. Sub-commands take preference, everything
                // else is treated as command parameter.
                self.parse_command(
                    cursor,
                    arg_os,
                    (!arg_tail).then_some(arg_front),
                )?;

                // Argument was parsed as command or parameter.
                continue;
            }

            // Argument was not parsed, report it as parameter.
            cursor.report_parameter(Some(arg_os))?;
        }

        // Report End-of-Arguments to the active command
        cursor.report_parameter(None)
    }
}

pub fn parse<'this, 'args, R>(
    report: &'this mut dyn args::ParserReport<'args, R>,
    schema: &'this mut args::Schema<'_, 'args, R>,
    arguments: &mut dyn Iterator<Item = &'args compat::OsStr>,
) -> Result<usize, Option<R>> {
    let mut parser = Parser::new();
    let mut cursor = Cursor::with(report, schema);

    if let core::ops::ControlFlow::Break(r) = parser.parse_cursor(
        &mut cursor,
        arguments,
    ) {
        return Err(Some(r));
    }

    cursor.command().ok_or(None)
}
