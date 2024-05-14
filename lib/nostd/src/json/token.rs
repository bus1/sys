//! # JSON Tokenizer
//!
//! The tokenizer takes an input stream of valid Unicode Scalar Values and
//! produces a linear stream of JSON tokens. No context sensitive parsing is
//! involved, but only simple token conversions.

use alloc::borrow::Cow;
use core::ops::ControlFlow;

/// Flags control the runtime behavior of a tokenizer. They can enable
/// non-standard behavior, improve diagnostics, or otherwise affect the
/// tokenizer.
///
/// An empty flag-set (default) will provide standard-compatible behavior with
/// conservative runtime behavior. The individual flags are designed so they
/// either opt into non-standard behavior or lessen other conservative
/// restrictions.
pub type Flag = u32;

/// Flag to allow leading zeroes in number values.
///
/// When set, the JSON tokenizer allows leading zeroes in JSON Number Values.
/// These leading zeroes will have no effect on the resulting number value.
/// Multiple consecutive zeroes are allowed.
pub const FLAG_ALLOW_LEADING_ZERO: Flag = 0x00000001;

/// Flag to allow plus sign in number values.
///
/// When set, the JSON tokenizer allows leading plus signs in JSON Number
/// Values. These have no effect on the resulting number value.
pub const FLAG_ALLOW_PLUS_SIGN: Flag = 0x00000002;

/// The tokenizer status describes the state of a tokenizer at a given point in
/// time. It is automatically yielded after every operation that advances a
/// tokenizer.
#[derive(Clone, Copy, Debug, Default)]
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Status {
    /// No token is currently parsed.
    #[default]
    Done,
    /// A token is still being parsed and requires more data.
    Busy,
}

/// This enum denotes the mathematical sign of JSON number values.
#[derive(Clone, Copy, Debug, Default)]
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Sign {
    /// Plus sign used for positive numbers.
    #[default]
    Plus,
    /// Minus sign used for negative numbers.
    Minus,
}

/// Enumeration of all possible error conditions.
#[derive(Clone, Debug)]
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error<'tk> {
    /// Given character is either not a valid character for JSON tokens, or
    /// only allowed inside of other tokens.
    CharacterStray(char),
    /// Given keyword is not valid in JSON.
    KeywordUnknown(Cow<'tk, str>),
    /// Number value was incomplete.
    NumberIncomplete,
    /// String value was incomplete.
    StringIncomplete,
    /// Specified unescaped character is not valid in a string.
    StringCharacterInvalid(char),
    /// Specified escaped character is not a valid string escape code.
    StringEscapeInvalid(char),
    /// String escape sequence is incomplete.
    StringEscapeIncomplete,
    /// Unpaired lead or trail surrogates are not valid in strings.
    StringSurrogateUnpaired,
    /// Comments are not supported by JSON.
    Comment(Cow<'tk, str>),
}

/// Enumeration of all possible tokens that can be yielded by the tokenizer.
///
/// Note that tokens can reference temporary values of the tokenizer (tied to
/// the lifetime `'tk`). Use `Token::own()` to duplicate these values and
/// provide an unrestricted copy of the token.
#[derive(Clone, Debug)]
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Token<'tk> {
    /// JSON colon token
    Colon,
    /// JSON comma token
    Comma,
    /// JSON open-array token
    ArrayOpen,
    /// JSON close-array token
    ArrayClose,
    /// JSON open-object token
    ObjectOpen,
    /// JSON close-object token
    ObjectClose,
    /// JSON null keyword
    Null,
    /// JSON true keyword
    True,
    /// JSON false keyword
    False,
    /// Block of continuous whitespace
    Whitespace {
        raw: Cow<'tk, str>,
    },
    /// JSON number value
    Number {
        raw: Cow<'tk, str>,
        integer: Cow<'tk, [u8]>,
        fraction: Cow<'tk, [u8]>,
        exponent: Cow<'tk, [u8]>,
        integer_sign: Sign,
        exponent_sign: Sign,
    },
    /// JSON string value
    String {
        raw: Cow<'tk, str>,
        chars: Cow<'tk, str>,
    },
}

/// Trait abstraction to report tokens, errors, and other events to the caller.
/// An implementation must be provided to a tokenizer to use for reporting any
/// events during tokenization.
///
/// XXX: There should be some context object to allow passing additional data
///      like span information.
pub trait Report<R> {
    /// Report tokenizer errors.
    fn report_error(
        &mut self,
        error: Error<'_>,
    ) -> ControlFlow<R>;

    /// Report finalized token.
    fn report_token(
        &mut self,
        token: Token<'_>,
    ) -> ControlFlow<R>;
}

// The internal state of the tokenizer. `State::None` is used when the
// tokenizer finished a token and has no associated state. Otherwise, the
// state identifies the current token and possible metadata.
#[derive(Clone, Debug, Default)]
enum State {
    #[default]
    None,
    Slash,
    Keyword,
    Whitespace,
    NumberIntegerNone(Sign),
    NumberIntegerSome(Sign, usize),
    NumberIntegerZero(Sign),
    NumberFractionNone(Sign, usize),
    NumberFractionSome(Sign, usize, usize),
    NumberExponentNone(Sign, usize, usize),
    NumberExponentSign(Sign, usize, usize, Sign),
    NumberExponentSome(Sign, usize, usize, Sign, usize),
    String,
    StringEscape,
    StringUnicode(u8, u32),
    StringSurrogate(u32),
    StringSurrogateEscape(u32),
    StringSurrogateUnicode(u32, u8, u32),
    CommentLine,
}

/// This is a streaming-capable tokenizer for JSON data. It takes an input
/// stream of Unicode Scalar Values and produces a stream of JSON tokens.
///
/// A single engine can be used to tokenize any number of JSON values. Once
/// a value has been fully tokenized, the engine is immediately ready to
/// parse the next token.
///
/// Internal buffers will grow to hold a given token in its entirety. It
/// is the responsibility of the caller to ensure size-limits on the input
/// data. No data is retained after a token is finalized, except internal
/// buffers for cache optimization (unless they exceed an internal
/// threshold).
#[derive(Clone, Debug, Default)]
pub struct Tokenizer {
    flags: Flag,
    state: State,
    acc: alloc::string::String,
    acc_str: alloc::string::String,
    acc_num: alloc::vec::Vec<u8>,
}

impl<'tk> Error<'tk> {
    /// Create owned version of an error.
    ///
    /// For performance reasons, errors reference internal data of the
    /// tokenizer. This duplicates all this referenced data and returns an
    /// equivalent error but without the lifetime restriction.
    pub fn own(self) -> Error<'static> {
        match self {
            Error::CharacterStray(v0) => Error::CharacterStray(v0),
            Error::KeywordUnknown(v0) => Error::KeywordUnknown(Cow::from(v0.into_owned())),
            Error::NumberIncomplete => Error::NumberIncomplete,
            Error::StringIncomplete => Error::StringIncomplete,
            Error::StringCharacterInvalid(v0) => Error::StringCharacterInvalid(v0),
            Error::StringEscapeInvalid(v0) => Error::StringEscapeInvalid(v0),
            Error::StringEscapeIncomplete => Error::StringEscapeIncomplete,
            Error::StringSurrogateUnpaired => Error::StringSurrogateUnpaired,
            Error::Comment(v0) => Error::Comment(Cow::from(v0.into_owned())),
        }
    }
}

impl<'tk> Token<'tk> {
    /// Create owned version of a token.
    ///
    /// For performance reasons, tokens reference internal data of the
    /// tokenizer. This duplicates all this referenced data and returns an
    /// equivalent token but without the lifetime restriction.
    pub fn own(self) -> Token<'static> {
        match self {
            Token::Colon => Token::Colon,
            Token::Comma => Token::Comma,
            Token::ArrayOpen => Token::ArrayOpen,
            Token::ArrayClose => Token::ArrayClose,
            Token::ObjectOpen => Token::ObjectOpen,
            Token::ObjectClose => Token::ObjectClose,
            Token::Null => Token::Null,
            Token::True => Token::True,
            Token::False => Token::False,
            Token::Whitespace { raw } => Token::Whitespace {
                raw: Cow::from(raw.into_owned()),
            },
            Token::Number {
                raw, integer, fraction, exponent, integer_sign, exponent_sign
            } => Token::Number {
                raw: Cow::from(raw.into_owned()),
                integer: Cow::from(integer.into_owned()),
                fraction: Cow::from(fraction.into_owned()),
                exponent: Cow::from(exponent.into_owned()),
                integer_sign,
                exponent_sign,
            },
            Token::String{ raw, chars } => Token::String {
                raw: Cow::from(raw.into_owned()),
                chars: Cow::from(chars.into_owned()),
            },
        }
    }
}

impl Tokenizer {
    /// Create a new tokenizer with the default setup.
    ///
    /// The tokenizer can be used to tokenize multiple tokens sequentially. It
    /// is automatically reset after each token.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new tokenizer with the given flags.
    ///
    /// The tokenizer is created via [`Self::new()`], but the default flags are
    /// replaced with the provided flags.
    pub fn with_flags(flags: Flag) -> Self {
        Self {
            flags: flags,
            ..Default::default()
        }
    }

    // Clear current buffers and prepare for the next token. This should be
    // called after a token finalized.
    fn prepare(&mut self) {
        self.acc.clear();
        self.acc.shrink_to(4096);
        self.acc_str.clear();
        self.acc_str.shrink_to(4096);
        self.acc_num.clear();
        self.acc_num.shrink_to(4096);
        self.state = State::None;
    }

    /// Reset tokenizer and clear state.
    ///
    /// Reset the tokenizer to the same state as when it was created. Internal
    /// buffers might remain allocated for performance reasons.
    pub fn reset(&mut self) {
        self.prepare();
    }

    /// Report current status.
    ///
    /// Report the status of the tokenizer. If a token is currently being
    /// processed, this will yield [`Status::Busy`]. Otherwise, it will yield
    /// [`Status::Done`].
    pub fn status(&self) -> Status {
        match self.state {
            State::None => Status::Done,
            _ => Status::Busy,
        }
    }

    fn acc_raw(&mut self, ch: char) {
        self.acc.push(ch);
    }

    fn acc_num(&mut self, ch: char) {
        self.acc_num.push(u8::try_from(ch.to_digit(10).unwrap()).unwrap());
    }

    fn acc_str(&mut self, ch: char) {
        self.acc_str.push(ch);
    }

    fn report_error<R>(
        &self,
        report: &mut dyn Report<R>,
        error: Error<'_>,
    ) -> ControlFlow<R> {
        report.report_error(error)?;
        ControlFlow::Continue(())
    }

    fn report_token<R>(
        &mut self,
        report: &mut dyn Report<R>,
        token: Token<'_>,
    ) -> ControlFlow<R> {
        report.report_token(token)?;
        ControlFlow::Continue(())
    }

    fn report_keyword<R>(
        &mut self,
        report: &mut dyn Report<R>,
    ) -> ControlFlow<R> {
        match self.acc.as_str() {
            "null" => self.report_token(report, Token::Null),
            "true" => self.report_token(report, Token::True),
            "false" => self.report_token(report, Token::False),
            _ => self.report_error(report, Error::KeywordUnknown(Cow::from(&self.acc))),
        }
    }

    fn report_whitespace<R>(
        &mut self,
        report: &mut dyn Report<R>,
    ) -> ControlFlow<R> {
        report.report_token(
            Token::Whitespace {
                raw: Cow::from(&self.acc),
            },
        )?;
        ControlFlow::Continue(())
    }

    fn report_number<R>(
        &mut self,
        report: &mut dyn Report<R>,
        meta: (Sign, usize, usize, Sign, usize),
    ) -> ControlFlow<R> {
        report.report_token(
            Token::Number {
                raw: Cow::from(&self.acc),
                integer: Cow::from(&self.acc_num[.. meta.1]),
                fraction: Cow::from(&self.acc_num[meta.1 .. (meta.1 + meta.2)]),
                exponent: Cow::from(&self.acc_num[(meta.1 + meta.2) ..]),
                integer_sign: meta.0,
                exponent_sign: meta.3,
            },
        )?;
        ControlFlow::Continue(())
    }

    fn report_string<R>(
        &mut self,
        report: &mut dyn Report<R>,
    ) -> ControlFlow<R> {
        report.report_token(
            Token::String {
                raw: Cow::from(&self.acc),
                chars: Cow::from(&self.acc_str),
            },
        )?;
        ControlFlow::Continue(())
    }

    fn advance_misc<R>(
        &mut self,
        report: &mut dyn Report<R>,
        ch: Option<char>,
    ) -> ControlFlow<R, Option<char>> {
        let rem = match self.state {
            // Slashes can start line-comments, multi-line comments, as well
            // as be part of normal keywords. None of this is supported by
            // JSON, but we try to be a bit clever to get better diagnostics.
            State::Slash => match ch {
                Some(v @ '/') => {
                    self.acc_raw(v);
                    self.state = State::CommentLine;
                    None
                },
                Some(v) => {
                    self.state = State::Keyword;
                    Some(v)
                },
                None => {
                    self.report_keyword(report)?;
                    self.prepare();
                    None
                },
            },

            // Keywords are handled below.
            State::Keyword => ch,

            // Merge as much whitespace into a single whitespace token as
            // possible. Once the first non-whitespace token is found, signal
            // the whitespace token and return the next character as unhandled.
            State::Whitespace => match ch {
                Some(v @ ' ') | Some(v @ '\n') | Some(v @ '\r') | Some(v @ '\t') => {
                    self.acc_raw(v);
                    None
                },
                Some(v) if v.is_whitespace() => {
                    self.report_error(report, Error::CharacterStray(v))?;
                    // Discard the character, but treat it as whitespace token.
                    None
                },
                v => {
                    self.report_whitespace(report)?;
                    self.prepare();
                    v
                },
            },

            // Line comments can start with '#' or '//' and are simply ignored
            // until the next new-line character. JSON does not support
            // comments, but we parse them for better diagnostics.
            State::CommentLine => match ch {
                Some(v @ '\n') => {
                    self.report_error(report, Error::Comment(Cow::from(&self.acc)))?;
                    self.prepare();
                    Some(v)
                },
                Some(v) => {
                    self.acc_raw(v);
                    None
                },
                None => {
                    self.report_error(report, Error::Comment(Cow::from(&self.acc)))?;
                    self.prepare();
                    None
                },
            },

            _ => core::unreachable!(),
        };

        // Anything that cannot be turned into a valid non-keyword token is
        // collected into a keyword token (including valid keywords). This
        // reduces the verbosity of diagnostics significantly. Anything that
        // can start non-keyword tokens will terminate a keyword, except for
        // digit runs.
        if matches!(self.state, State::Keyword) {
            match rem {
                // Valid: starts token
                Some(
                    ':' | ',' | '[' | ']' | '{' | '}' // basic token
                    | ' ' | '\n' | '\r' | '\t' // white-space
                    | '-' | '"' // numbers/strings
                ) => {
                    self.report_keyword(report)?;
                    self.prepare();
                    ControlFlow::Continue(rem)
                },

                // Invalid: starts token
                Some(
                    | '#' | '/' // comments
                    | '+' // numbers
                    | '=' // assignment token
                ) => {
                    self.report_keyword(report)?;
                    self.prepare();
                    ControlFlow::Continue(rem)
                },

                // Invalid: white-space
                Some(v) if v.is_whitespace() => {
                    self.report_keyword(report)?;
                    self.prepare();
                    ControlFlow::Continue(rem)
                },

                Some(v) => {
                    self.acc_raw(v);
                    ControlFlow::Continue(None)
                },

                None => {
                    self.report_keyword(report)?;
                    self.prepare();
                    ControlFlow::Continue(None)
                },
            }
        } else {
            ControlFlow::Continue(rem)
        }
    }

    fn advance_number<R>(
        &mut self,
        report: &mut dyn Report<R>,
        ch: Option<char>,
    ) -> ControlFlow<R, Option<char>> {
        // Parsing numbers is just a matter of parsing the components one
        // after another, where some components are optional. As usual, we
        // keep the unmodified number in the accumulator. However, we also
        // push all digits into a separate accumulator and remember how
        // many digits each component occupies. This allows much simpler
        // number conversions later on.
        match self.state {
            State::NumberIntegerNone(sign_int) => match ch {
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    if v == '0' {
                        self.state = State::NumberIntegerZero(sign_int);
                    } else {
                        self.state = State::NumberIntegerSome(sign_int, 1);
                    }
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_error(report, Error::NumberIncomplete)?;
                    self.report_number(
                        report,
                        (sign_int, 0, 0, Sign::Plus, 0)
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberIntegerSome(sign_int, n_int) => match ch {
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberIntegerSome(sign_int, n_int + 1);
                    ControlFlow::Continue(None)
                },
                Some(v @ '.') => {
                    self.acc_raw(v);
                    self.state = State::NumberFractionNone(sign_int, n_int);
                    ControlFlow::Continue(None)
                },
                Some(v @ 'e' | v @ 'E') => {
                    self.acc_raw(v);
                    self.state = State::NumberExponentNone(sign_int, n_int, 0);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_number(
                        report,
                        (sign_int, n_int, 0, Sign::Plus, 0)
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberIntegerZero(sign_int) => match ch {
                Some(v @ '.') => {
                    self.acc_raw(v);
                    self.state = State::NumberFractionNone(sign_int, 1);
                    ControlFlow::Continue(None)
                },
                Some(v @ 'e' | v @ 'E') => {
                    self.acc_raw(v);
                    self.state = State::NumberExponentNone(sign_int, 1, 0);
                    ControlFlow::Continue(None)
                },
                Some(v @ '0'..='9') => {
                    if (self.flags & FLAG_ALLOW_LEADING_ZERO) != FLAG_ALLOW_LEADING_ZERO {
                        self.report_error(report, Error::CharacterStray(v))?;
                    }
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberIntegerSome(sign_int, 2);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_number(
                        report,
                        (sign_int, 1, 0, Sign::Plus, 0),
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberFractionNone(sign_int, n_int) => match ch {
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberFractionSome(sign_int, n_int, 1);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_error(report, Error::NumberIncomplete)?;
                    self.report_number(
                        report,
                        (sign_int, n_int, 0, Sign::Plus, 0),
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberFractionSome(sign_int, n_int, n_frac) => match ch {
                Some(v @ 'e' | v @ 'E') => {
                    self.acc_raw(v);
                    self.state = State::NumberExponentNone(sign_int, n_int, n_frac);
                    ControlFlow::Continue(None)
                },
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberFractionSome(sign_int, n_int, n_frac + 1);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_number(
                        report,
                        (sign_int, n_int, n_frac, Sign::Plus, 0),
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberExponentNone(sign_int, n_int, n_frac) => match ch {
                Some(v @ '+') => {
                    self.acc_raw(v);
                    self.state = State::NumberExponentSign(sign_int, n_int, n_frac, Sign::Plus);
                    ControlFlow::Continue(None)
                },
                Some(v @ '-') => {
                    self.acc_raw(v);
                    self.state = State::NumberExponentSign(sign_int, n_int, n_frac, Sign::Minus);
                    ControlFlow::Continue(None)
                },
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberExponentSome(sign_int, n_int, n_frac, Sign::Plus, 1);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_error(report, Error::NumberIncomplete)?;
                    self.report_number(
                        report,
                        (sign_int, n_int, n_frac, Sign::Plus, 0),
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberExponentSign(sign_int, n_int, n_frac, sign_exp) => match ch {
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberExponentSome(sign_int, n_int, n_frac, sign_exp, 1);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_error(report, Error::NumberIncomplete)?;
                    self.report_number(
                        report,
                        (sign_int, n_int, n_frac, sign_exp, 0),
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            State::NumberExponentSome(sign_int, n_int, n_frac, sign_exp, n_exp) => match ch {
                Some(v @ '0'..='9') => {
                    self.acc_raw(v);
                    self.acc_num(v);
                    self.state = State::NumberExponentSome(sign_int, n_int, n_frac, sign_exp, n_exp + 1);
                    ControlFlow::Continue(None)
                },
                v => {
                    self.report_number(
                        report,
                        (sign_int, n_int, n_frac, sign_exp, n_exp),
                    )?;
                    self.prepare();
                    ControlFlow::Continue(v)
                },
            },

            _ => core::unreachable!(),
        }
    }

    fn advance_string<R>(
        &mut self,
        report: &mut dyn Report<R>,
        ch: Option<char>,
    ) -> ControlFlow<R, Option<char>> {
        // Strings must be terminated with a quote. Therefore, we can handle
        // `None` early for all string states.
        let Some(ch_value) = ch else {
            self.report_error(report, Error::StringIncomplete)?;
            self.prepare();
            return ControlFlow::Continue(ch);
        };

        // Parsing a string is just a matter of pushing characters into the
        // accumulator and tracking escape-sequences. Unicode-escapes make up
        // most of the complexity, since we must track surrogate pairs to avoid
        // strings with non-paired surrogate escapes.
        // First try advancing any escape parser. If that fails, or if no
        // escape sequence is active, we parse it as normal string character.
        let rem = match self.state {
            State::StringEscape => match ch_value {
                v @ '"' | v @ '\\' | v @ '/' | v @ 'b' | v @ 'f' | v @ 'n' | v @ 'r' | v @ 't' => {
                    self.acc_raw(v);
                    self.acc_str(match v {
                        'b' => '\u{0008}',
                        'f' => '\u{000c}',
                        'n' => '\u{000a}',
                        'r' => '\u{000d}',
                        't' => '\u{0009}',
                        v => v,
                    });
                    self.state = State::String;
                    None
                },
                v @ 'u' => {
                    self.acc_raw(v);
                    self.state = State::StringUnicode(0, 0);
                    None
                },
                v => {
                    self.report_error(report, Error::StringEscapeInvalid(v))?;
                    self.acc_raw(v);
                    self.acc_str(v);
                    self.state = State::String;
                    None
                },
            },

            // A unicode escape sequence always uses the form `\uXXXX`. No
            // shorter version is allowed. The `StringUnicode` state
            // remembers the number of digits parsed, as well as the
            // current value.
            // If a unicode escape encodes a lead-surrogate, it must be
            // followed immediately by an escape that encodes a
            // trail-surrogate. In this case the `StringSurrogate` state is
            // entered.
            // If the Unicode Code Point is not a valid Unicode Scalar
            // Value, nor a valid surrogate pair, it is rejected as
            // invalid.
            State::StringUnicode(num, value) => match ch_value {
                v @ '0'..='9' | v @ 'a'..='f' | v @ 'A'..='F' => {
                    let value = (value << 4) | v.to_digit(16).unwrap();
                    self.acc_raw(v);
                    if num < 3 {
                        // Increase the number of parsed digits by one and
                        // continue parsing until we got 4 total.
                        self.state = State::StringUnicode(num + 1, value);
                    } else if value >= 0xd800 && value <= 0xdbff {
                        // Got a lead-surrogate. It must be followed by a
                        // trail-surrogate immediately.
                        self.state = State::StringSurrogate(value);
                    } else if value >= 0xdc00 && value <= 0xdfff {
                        // Got an unpaired trail-surrogate. This is not
                        // allowed, so reject it straight away.
                        self.report_error(report, Error::StringSurrogateUnpaired)?;
                        self.state = State::String;
                    } else {
                        // Got a valid Unicode Scalar Value.
                        let v = char::from_u32(value).unwrap();
                        self.acc_str(v);
                        self.state = State::String;
                    }
                    None
                },
                v => {
                    self.report_error(report, Error::StringEscapeIncomplete)?;
                    self.state = State::String;
                    Some(v)
                },
            },

            State::StringSurrogate(lead) => match ch_value {
                v @ '\\' => {
                    self.acc_raw(v);
                    self.state = State::StringSurrogateEscape(lead);
                    None
                },
                v => {
                    self.report_error(report, Error::StringSurrogateUnpaired)?;
                    self.state = State::String;
                    Some(v)
                },
            },

            State::StringSurrogateEscape(lead) => match ch_value {
                v @ 'u' => {
                    self.acc_raw(v);
                    self.state = State::StringSurrogateUnicode(lead, 0, 0);
                    None
                },
                v @ '"' | v @ '\\' | v @ '/' | v @ 'b' | v @ 'f' | v @ 'n' | v @ 'r' | v @ 't' => {
                    self.report_error(report, Error::StringSurrogateUnpaired)?;
                    self.acc_raw(v);
                    self.acc_str(match v {
                        'b' => '\u{0008}',
                        'f' => '\u{000c}',
                        'n' => '\u{000a}',
                        'r' => '\u{000d}',
                        't' => '\u{0009}',
                        v => v,
                    });
                    self.state = State::String;
                    None
                },
                v => {
                    self.report_error(report, Error::StringSurrogateUnpaired)?;
                    self.report_error(report, Error::StringEscapeInvalid(v))?;
                    self.acc_raw(v);
                    self.acc_str(v);
                    self.state = State::String;
                    None
                },
            },

            State::StringSurrogateUnicode(lead, num, trail) => match ch_value {
                v @ '0'..='9' | v @ 'a'..='f' | v @ 'A'..='F' => {
                    let trail = (trail << 4) | v.to_digit(16).unwrap();
                    self.acc_raw(v);
                    if num < 3 {
                        // Increase the number of parsed digits by one and
                        // continue parsing until we got 4 total.
                        self.state = State::StringSurrogateUnicode(lead, num + 1, trail);
                    } else if trail >= 0xd800 && trail <= 0xdbff {
                        // This is another lead-surrogate, but we expected
                        // a trail-surrogate. Reject it.
                        self.report_error(report, Error::StringSurrogateUnpaired)?;
                        self.state = State::String;
                    } else if trail >= 0xdc00 && trail <= 0xdfff {
                        // This is a trail-surrogate following a
                        // lead-surrogate. This finalizes the surrogate
                        // pair and the string can continue normally.
                        self.acc_str(
                            char::from_u32(0x10000 + ((lead - 0xd800) << 10) + (trail - 0xdc00))
                                .unwrap(),
                        );
                        self.state = State::String;
                    } else {
                        // We expected a trail-surrogate, but got a
                        // Unicode Scalar Value. Reject this.
                        let _ = char::from_u32(trail).unwrap();
                        self.report_error(report, Error::StringSurrogateUnpaired)?;
                        self.state = State::String;
                    }
                    None
                },
                v => {
                    self.report_error(report, Error::StringEscapeIncomplete)?;
                    self.state = State::String;
                    Some(v)
                },
            },

            _ => Some(ch_value),
        };

        // If the character was handled, return to the caller, otherwise...
        let Some(ch_value) = rem else {
            return ControlFlow::Continue(None);
        };

        // ...treat it as normal string character.
        assert!(matches!(self.state, State::String));
        match ch_value {
            '"' => {
                self.report_string(report)?;
                self.prepare();
                ControlFlow::Continue(None)
            },
            v @ '\\' => {
                self.acc_raw(v);
                self.state = State::StringEscape;
                ControlFlow::Continue(None)
            },
            v @ '\x00'..='\x1f' => {
                self.report_error(report, Error::StringCharacterInvalid(v))?;
                self.acc_raw(v);
                self.acc_str(v);
                ControlFlow::Continue(None)
            },
            v @ '\x20'..='\x21'
            // '\x22' is '"'
            | v @ '\x23'..='\x5b'
            // '\x5c' is '\\'
            | v @ '\x5d'..='\u{d7ff}'
            // '\u{d800}'..='\u{dfff}' are surrogates
            | v @ '\u{e000}'..='\u{10ffff}' => {
                self.acc_raw(v);
                self.acc_str(v);
                ControlFlow::Continue(None)
            },
        }
    }

    fn advance<R>(
        &mut self,
        report: &mut dyn Report<R>,
        ch: Option<char>,
    ) -> ControlFlow<R> {
        // First try to push the next character into the current token handler.
        // If either no token is currently parsed, or if the token cannot
        // consume the character, the sub-handlers will finalize their token
        // and return the character as unhandled.
        let rem = match self.state {
            State::None => {
                ControlFlow::Continue(ch)
            },

            State::Slash
            | State::Keyword
            | State::Whitespace
            | State::CommentLine => {
                self.advance_misc(report, ch)
            },

            State::NumberIntegerNone(_)
            | State::NumberIntegerSome(_, _)
            | State::NumberIntegerZero(_)
            | State::NumberFractionNone(_, _)
            | State::NumberFractionSome(_, _, _)
            | State::NumberExponentNone(_, _, _)
            | State::NumberExponentSign(_, _, _, _)
            | State::NumberExponentSome(_, _, _, _, _) => {
                self.advance_number(report, ch)
            },

            State::String
            | State::StringEscape
            | State::StringUnicode(_, _)
            | State::StringSurrogate(_)
            | State::StringSurrogateEscape(_)
            | State::StringSurrogateUnicode(_, _, _) => {
                self.advance_string(report, ch)
            },
        };

        let v = match rem {
            // A handler yielded a break value. Propagate it.
            ControlFlow::Break(v) => {
                return ControlFlow::Break(v);
            },

            // The character was successfully parsed, or signaled end-of-input.
            // No need to start a new token, but we can simply return to the
            // caller.
            ControlFlow::Continue(None) => {
                return ControlFlow::Continue(());
            },

            // Either no previous token was handled, or this character
            // finalized it. Either way, the character starts a new token.
            ControlFlow::Continue(Some(v)) => {
                v
            },
        };

        match v {
            ':' => {
                self.report_token(report, Token::Colon)?;
            },
            ',' => {
                self.report_token(report, Token::Comma)?;
            },
            '[' => {
                self.report_token(report, Token::ArrayOpen)?;
            },
            ']' => {
                self.report_token(report, Token::ArrayClose)?;
            },
            '{' => {
                self.report_token(report, Token::ObjectOpen)?;
            },
            '}' => {
                self.report_token(report, Token::ObjectClose)?;
            },
            'a'..='z' | 'A'..='Z' => {
                self.acc_raw(v);
                self.state = State::Keyword;
            },
            ' ' | '\n' | '\r' | '\t' => {
                self.acc_raw(v);
                self.state = State::Whitespace;
            },
            '-' => {
                self.acc_raw(v);
                self.state = State::NumberIntegerNone(Sign::Minus);
            },
            '0'..='9' => {
                self.acc_raw(v);
                self.acc_num(v);
                if v == '0' {
                    self.state = State::NumberIntegerZero(Sign::Plus);
                } else {
                    self.state = State::NumberIntegerSome(Sign::Plus, 1);
                }
            },
            '"' => {
                self.state = State::String;
            },

            /*
             * Improved Diagnostics
             *
             * All following handlers are purely added for improved
             * diagnostics. They do not parse valid JSON, but try to coalesce
             * invalid data into reasonable tokens to avoid excessive error
             * reporting. Most of the invalid data is simply turned into a
             * consecutive identifier, which is then reported as invalid once
             * fully parsed.
             */

            '#' => {
                self.state = State::CommentLine;
            },
            '+' => {
                // A leading plus-sign is not allowed for JSON Number
                // Values, yet it is very reasonable to support it. If allowed
                // explicitly, simply parse it as start of a number. Otherwise,
                // treat it as stray character to avoid double-faults. If it is
                // followed by a valid number, it has no effect, anyway.
                if (self.flags & FLAG_ALLOW_PLUS_SIGN) == FLAG_ALLOW_PLUS_SIGN {
                    self.acc_raw(v);
                    self.state = State::NumberIntegerNone(Sign::Plus);
                } else {
                    self.report_error(report, Error::CharacterStray(v))?;
                }
            },
            '/' => {
                // Slashes are not allowed, but are often used to start
                // comments or combine expressions in other languages.
                // Hence, try to be clever and do the same, so we get
                // improved diagnostics.
                self.acc_raw(v);
                self.state = State::Slash;
            },
            '=' => {
                // Raise errors about equal signs, but then treat them as
                // colons, as they usually serve similar purposes.
                self.report_error(report, Error::CharacterStray(v))?;
                self.report_token(report, Token::Colon)?;
            },
            v if v.is_whitespace() => {
                // Raise errors about unsupported whitespace characters,
                // but treat them as whitespace token to properly separate
                // other tokens. The character itself is discarded, though.
                self.report_error(report, Error::CharacterStray(v))?;
                self.state = State::Whitespace;
            },
            '\'' | '(' | ')' | '`' => {
                // We could match these in pairs for improved diagnotics, but
                // so far we simply coalesce them into keywords and it seems
                // to produce useful errors.
                self.acc_raw(v);
                self.state = State::Keyword;
            },
            '!' | '$' | '%' | '&' | '*' | '.' | '<' | '>'
            | '?' | '@' | '\\' | '^' | '_' | '|' | '~' => {
                // All these are invalid punctuation marks, so we simply treat
                // them as part of a keyword and coalesce them.
                self.acc_raw(v);
                self.state = State::Keyword;
            },
            v if (
                v.is_ascii_punctuation()
                || v.is_control()
                || v.is_alphanumeric()
            ) => {
                // Explicitly treat any other punctuation, control character,
                // or alpha-numeric character as part of a keyword, so they get
                // coalesced into a single token.
                self.acc_raw(v);
                self.state = State::Keyword;
            },
            v => {
                // Simply treat everything else as a keyword. It will coalesce
                // multiple invalid characters and reduce diagnostic noise.
                self.acc_raw(v);
                self.state = State::Keyword;
            },
        }

        ControlFlow::Continue(())
    }

    /// Push a single character into the tokenizer and process it. This
    /// will advance the tokenizer state machine and report successfully
    /// parsed tokens to the specified report handler.
    ///
    /// The tokenizer will always continue parsing and is always
    /// ready for more input. Errors are reported as special tokens and
    /// the tokenizer will try its best to recover and proceed. This
    /// allows reporting multiple errors in a single run. It is up to
    /// the caller to decide whether to ultimately reject the input or
    /// use the best-effort result of the tokenizer.
    ///
    /// Whenever a token is successfully parsed, the specified report handler
    /// is invoked with the token as parameter. If the report handler returns
    /// [`ControlFlow::Break`], the tokenizer will reset its
    /// internal state and propagate the value the caller immediately.
    /// If the report handler returns [`ControlFlow::Continue`],
    /// the tokenizer will continue its operation as normal.
    ///
    /// Some JSON Tokens are open-ended, meaning that the tokenizer needs
    /// to know about the end of the input. Pushing `None` into the tokenizer
    /// will be interpreted as `End-of-Input` and finalize or cancel the final
    /// token.
    pub fn push<R>(
        &mut self,
        report: &mut dyn Report<R>,
        ch: Option<char>,
    ) -> ControlFlow<R, Status> {
        if let ControlFlow::Break(v) = self.advance(report, ch) {
            // A break will propagate through the entire chain back to the
            // caller. Ensure we leave the tokenizer in a predictable state,
            // since there is no way to recover from this.
            self.prepare();
            ControlFlow::Break(v)
        } else {
            ControlFlow::Continue(self.status())
        }
    }

    /// Push an entire string into the tokenizer and process it. This is
    /// equivalent to iterating over the characters and pushing them into
    /// the tokenizer individually via [`Self::push()`].
    ///
    /// This will **not** push a final [`Option::None`] into the tokenizer.
    /// Hence, this function can be used to stream multiple strings into a
    /// single tokenizer. See [`Self::parse_str()`] for alternatives.
    pub fn push_str<R>(
        &mut self,
        report: &mut dyn Report<R>,
        data: &str,
    ) -> ControlFlow<R, Status> {
        for ch in data.chars() {
            self.push(report, Some(ch))?;
        }
        ControlFlow::Continue(self.status())
    }

    /// Push the entire string into the tokenizer, followed by an
    /// End-Of-Input marker.
    ///
    /// Note that this does not clear the engine before pushing the string
    /// into it. Hence, make sure to call this on a clean engine, unless
    /// it is meant to be pushed on top of the previous input.
    ///
    /// This will finalize the input and thus always reset the tokenizer
    /// before returning. Moreover, the tokenizer will always report a
    /// status of [`Status::Done`] when finished.
    pub fn parse_str<R>(
        &mut self,
        report: &mut dyn Report<R>,
        data: &str,
    ) -> ControlFlow<R, Status> {
        for ch in data.chars() {
            self.push(report, Some(ch))?;
        }
        self.push(report, None)?;
        ControlFlow::Continue(Status::Done)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    enum Tk {
        E(Error<'static>),
        T(Token<'static>),
    }

    impl Report<()> for alloc::vec::Vec<Tk> {
        fn report_error(
            &mut self,
            error: Error<'_>,
        ) -> ControlFlow<()> {
            self.push(Tk::E(error.own()));
            ControlFlow::Continue(())
        }

        fn report_token(
            &mut self,
            token: Token<'_>,
        ) -> ControlFlow<()> {
            self.push(Tk::T(token.own()));
            ControlFlow::Continue(())
        }
    }

    fn tk(from: &str) -> (ControlFlow<(), Status>, alloc::vec::Vec<Tk>) {
        let mut acc = alloc::vec::Vec::new();
        let r = Tokenizer::new().parse_str(&mut acc, from);
        (r, acc)
    }

    // Basic tokenizer test used to quickly verify the most basic functionality
    // of the tokenizer, without looking too much into the individual details.
    #[test]
    fn basic() {
        assert_eq!(
            tk("null"),
            (
                ControlFlow::Continue(Status::Done),
                alloc::vec![
                    Tk::T(Token::Null),
                ],
            ),
        );
    }

    // List of known tokenization results, easily extendable.
    #[test]
    fn known() {
        let set = alloc::vec![
            // Empty token stream
            ("", (ControlFlow::Continue(Status::Done), alloc::vec![])),

            // All tokens in their most basic form
            (":", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::Colon)])),
            (",", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::Comma)])),
            ("[", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::ArrayOpen)])),
            ("]", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::ArrayClose)])),
            ("{", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::ObjectOpen)])),
            ("}", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::ObjectClose)])),
            ("null", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::Null)])),
            ("true", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::True)])),
            ("false", (ControlFlow::Continue(Status::Done), alloc::vec![Tk::T(Token::False)])),
            (" \n\r\t", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Whitespace { raw: Cow::from(" \n\r\t") }),
            ])),
            ("0", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Number {
                    raw: Cow::from("0"),
                    integer: Cow::from(&[0]),
                    fraction: Cow::from(&[]),
                    exponent: Cow::from(&[]),
                    integer_sign: Sign::Plus,
                    exponent_sign: Sign::Plus,
                }),
            ])),
            (r#""foobar""#, (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::String { raw: Cow::from("foobar"), chars: Cow::from("foobar") }),
            ])),

            // Number values
            ("-0", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Number {
                    raw: Cow::from("-0"),
                    integer: Cow::from(&[0]),
                    fraction: Cow::from(&[]),
                    exponent: Cow::from(&[]),
                    integer_sign: Sign::Minus,
                    exponent_sign: Sign::Plus,
                }),
            ])),
            ("12.34e-56", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Number {
                    raw: Cow::from("12.34e-56"),
                    integer: Cow::from(&[1, 2]),
                    fraction: Cow::from(&[3, 4]),
                    exponent: Cow::from(&[5, 6]),
                    integer_sign: Sign::Plus,
                    exponent_sign: Sign::Minus,
                }),
            ])),
            ("-0e100", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Number {
                    raw: Cow::from("-0e100"),
                    integer: Cow::from(&[0]),
                    fraction: Cow::from(&[]),
                    exponent: Cow::from(&[1, 0, 0]),
                    integer_sign: Sign::Minus,
                    exponent_sign: Sign::Plus,
                }),
            ])),
            ("0.12345678901234567890123456789012345678901234567890123456789", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Number {
                    raw: Cow::from("0.12345678901234567890123456789012345678901234567890123456789"),
                    integer: Cow::from(&[0]),
                    fraction: Cow::from(&[
                        1, 2, 3, 4, 5, 6, 7, 8,
                        9, 0, 1, 2, 3, 4, 5, 6,
                        7, 8, 9, 0, 1, 2, 3, 4,
                        5, 6, 7, 8, 9, 0, 1, 2,
                        3, 4, 5, 6, 7, 8, 9, 0,
                        1, 2, 3, 4, 5, 6, 7, 8,
                        9, 0, 1, 2, 3, 4, 5, 6,
                        7, 8, 9,
                    ]),
                    exponent: Cow::from(&[]),
                    integer_sign: Sign::Plus,
                    exponent_sign: Sign::Plus,
                }),
            ])),
            ("0.0000", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::Number {
                    raw: Cow::from("0.0000"),
                    integer: Cow::from(&[0]),
                    fraction: Cow::from(&[0, 0, 0, 0]),
                    exponent: Cow::from(&[]),
                    integer_sign: Sign::Plus,
                    exponent_sign: Sign::Plus,
                }),
            ])),

            // String values
            (r#""foo\nbar""#, (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::String { raw: Cow::from("foo\\nbar"), chars: Cow::from("foo\nbar") }),
            ])),
            (r#""\u0020""#, (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::String { raw: Cow::from("\\u0020"), chars: Cow::from(" ") }),
            ])),
            (r#""\uD834\udd1e""#, (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::String { raw: Cow::from("\\uD834\\udd1e"), chars: Cow::from("\u{1d11e}") }),
            ])),
            (r#""\"\\\/\b\f\n\r\t""#, (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::T(Token::String { raw: Cow::from(r#"\"\\\/\b\f\n\r\t"#), chars: Cow::from("\"\\/\x08\x0c\x0a\x0d\x09") }),
            ])),

            // Simple errors
            ("+", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::CharacterStray('+')),
            ])),
            ("foobar", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::KeywordUnknown(Cow::from("foobar"))),
            ])),
            ("0e", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::NumberIncomplete),
                Tk::T(Token::Number {
                    raw: Cow::from("0e"),
                    integer: Cow::from(&[0]),
                    fraction: Cow::from(&[]),
                    exponent: Cow::from(&[]),
                    integer_sign: Sign::Plus,
                    exponent_sign: Sign::Plus,
                }),
            ])),
            ("\"", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::StringIncomplete),
            ])),
            ("\"\x00\"", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::StringCharacterInvalid('\x00')),
                Tk::T(Token::String {
                    raw: Cow::from("\0"),
                    chars: Cow::from("\0"),
                }),
            ])),
            ("\"\\ \"", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::StringEscapeInvalid(' ')),
                Tk::T(Token::String {
                    raw: Cow::from("\\ "),
                    chars: Cow::from(" "),
                }),
            ])),
            ("\"\\u \"", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::StringEscapeIncomplete),
                Tk::T(Token::String {
                    raw: Cow::from("\\u "),
                    chars: Cow::from(" "),
                }),
            ])),
            ("\"\\ud834\"", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::StringSurrogateUnpaired),
                Tk::T(Token::String {
                    raw: Cow::from("\\ud834"),
                    chars: Cow::from(""),
                }),
            ])),
            ("# foobar", (ControlFlow::Continue(Status::Done), alloc::vec![
                Tk::E(Error::Comment(Cow::from(" foobar"))),
            ])),

            // Token streams
            (
                r#" "foo\"bar":-0.1e-10 "#,
                (
                    ControlFlow::Continue(Status::Done),
                    alloc::vec![
                        Tk::T(Token::Whitespace { raw: Cow::from(" ") }),
                        Tk::T(Token::String {
                            raw: Cow::from(r#"foo\"bar"#),
                            chars: Cow::from("foo\"bar"),
                        }),
                        Tk::T(Token::Colon),
                        Tk::T(Token::Number {
                            raw: Cow::from(r#"-0.1e-10"#),
                            integer: Cow::from(&[0]),
                            fraction: Cow::from(&[1]),
                            exponent: Cow::from(&[1, 0]),
                            integer_sign: Sign::Minus,
                            exponent_sign: Sign::Minus,
                        }),
                        Tk::T(Token::Whitespace { raw: Cow::from(" ") }),
                    ],
                ),
            ),
        ];

        for (from, to) in set {
            assert_eq!(tk(from), to);
        }
    }
}
