//! JSON Tokens
//!
//! The main type of this module is [`Dec`], a JSON tokenizer. It takes a
//! data-stream and turns it into a token-stream.
//!
//! ## Deviations
//!
//! This implementation deviates from standards in the following ways:
//!
//!   - Mandatory Utf-8: While the JSON standard does not enforce any transport
//!     level encoding, this implementation requires Utf-8. Other encodings
//!     are not supported, yet gracefully handled via the error tokens.
//!
//!   - Unicode NonCharacters: I-JSON rejects NonCharacters in data
//!     encodings and string escapes (RFC 7493:2.1). This contradicts the
//!     Unicode standard (Corrigendum #9: Clarification About Noncharacters).
//!     This implementation explicitly allows NonCharacters and follows the
//!     Unicode standard.
//!     This restriction is specific to I-JSON, not JSON.

// XXX: The following improvements are planned for this implementation:
//   - Provide span-information for items, especially errors.
//   - Implement optional JSON5 support.
//   - Add better handling of non-JSON syntax to improve error reporting.
//   - `Token` should either be smaller or passed by reference. The current
//     model of returning it causes unnecessary copies for simple tokens.

use alloc::vec;
use core::ops::ControlFlow as Flow;

use crate::io;

/// This type enumerates all errors that can be raised by the tokenizer.
///
/// Errors are raised inline as error-tokens, rather than separately. The
/// tokenizer can always recover from errors and continue parsing. However,
/// the resulting data will likely be corrupted and should be used only for
/// diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<'data> {
    /// When completing the parser, there is buffered data remaining. If this
    /// is intentional, the error can be ignored.
    BufferRemaining,
    /// Buffered data changed while parsing. The provided `io::stream::Read`
    /// implementation might have flushed buffers. Usually this indicates a
    /// programming error, but might be intentional given a specific buffer
    /// implementation.
    BufferChanged,
    /// Item with non-UTF8 data
    ItemNonUtf8 {
        /// Raw byte data from the input buffer
        raw: &'data [u8],
        /// Error when running `core::str::from_utf8()` on `raw`
        error: core::str::Utf8Error,
    },
    /// Item that is not a known token in a JSON stream
    ItemUnknown {
        /// Raw byte data from the input buffer
        raw: &'data [u8],
        /// Raw byte data but parsed as a UTF-8 string
        str: &'data str,
    },
    /// Data ended with an incomplete number
    NumberIncomplete,
    /// Number starts with a plus sign
    NumberLeadingPlus,
    /// Number has multiple consecutive signs
    NumberMultipleSigns,
    /// Number has an empty integer, fraction, or exponent
    NumberRangeEmpty,
    /// Data ended with an incomplete string
    StringIncomplete,
    /// String contains non-UTF8 data
    StringNonUtf8 {
        /// Raw byte data from the input buffer, with unmodified escape
        /// sequences and including surrounding quotation marks.
        raw: &'data [u8],
        /// String content without quotation marks and with escape sequences
        /// converted to their real representation.
        str: &'data [u8],
        /// Error when running `core::str::from_utf8()` on `str`.
        error: core::str::Utf8Error,
    },
    /// Unescaped character that must be escaped
    StringUnescaped {
        code: u8,
    },
    /// Unknown single-character escape sequence
    StringEscapeUnknown {
        code: u8,
    },
    /// Syntactically invalid escape sequence (eq., unicode escape not followed
    /// by 4 hex-characters.
    StringEscapeInvalid,
    /// Unicode lead surrogate escape sequence without following trail
    /// surrogate escape sequence.
    StringEscapeUnpairedLeadSurrogate {
        lead: u32,
    },
    /// Unicode trail surrogate escape sequence without leading lead surrogate
    /// escape sequence.
    StringEscapeUnpairedTrailSurrogate {
        trail: u32,
    },
}

/// This type encodes the sign of a number.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Sign {
    /// The plus sign (`+`)
    #[default]
    Plus,
    /// The minus sign (`-`)
    Minus,
}

/// This type encodes the parameters of a number.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Number<'data> {
    /// Raw data as provided in the source data
    pub raw: &'data [u8],
    /// Sign of the integer part
    pub sign: Sign,
    /// Digits of the integer part
    pub integer: &'data str,
    /// Digits of the fraction part, if any
    pub fraction: Option<&'data str>,
    /// Sign and digits of the exponent, if any
    pub exponent: Option<(Sign, &'data str)>,
}

/// This type enumerates all possible tokens that can be raised by the
/// tokenizer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token<'data> {
    /// Non-fatal error during tokenization
    Error(Error<'data>),
    /// Non-significant whitespace
    Whitespace {
        /// Raw data of the whitespace sequence as provided in the data stream.
        raw: &'data [u8],
        /// Same as `raw` but provided as string.
        str: &'data str,
    },
    /// A colon character (`:`)
    Colon,
    /// A comma character (`,`)
    Comma,
    /// An opening array character (`[`)
    ArrayOpen,
    /// A closing array character (`[`)
    ArrayClose,
    /// An opening object character (`{`)
    ObjectOpen,
    /// A closing object character (`}`)
    ObjectClose,
    /// Null token (`null`)
    Null,
    /// False token (`false`)
    False,
    /// True token (`true`)
    True,
    /// A number value
    Number(Number<'data>),
    /// A String value
    String {
        /// Raw data of the string, including surrounding quotation marks and
        /// unmodified escape sequences.
        raw: &'data [u8],
        /// The parsed string content without quotation marks and with all
        /// escape sequences resolved.
        str: &'data str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    None,
    Whitespace,
    Item,
    Number {
        sign: Sign,
        integer: core::ops::Range<usize>,
        fraction: core::ops::Range<usize>,
        exponent_sign: Sign,
        exponent: core::ops::Range<usize>,
    },
    NumberDone {
        sign: Sign,
        integer: core::ops::Range<usize>,
        fraction: core::ops::Range<usize>,
        exponent_sign: Sign,
        exponent: core::ops::Range<usize>,
    },
    String {
        idx_start: usize,
    },
    StringEscape,
    StringEscapeUnicode {
        idx_start: usize,
    },
    StringEscapeUnicodeTrail {
        idx_start: usize,
        lead: u32,
    },
}

struct DecInner {
    state: State,
    idx: usize,
    done: Option<usize>,
    acc_str: vec::Vec<u8>,
}

/// This implements a streaming-capable JSON tokenizer.
pub struct Dec<'read> {
    inner: DecInner,
    read: &'read mut dyn io::stream::Read,
}

fn unicode_from_hex(v: [char; 4]) -> u32 {
    let mut code = 0;
    code = (code << 4) | v[0].to_digit(16).unwrap();
    code = (code << 4) | v[1].to_digit(16).unwrap();
    code = (code << 4) | v[2].to_digit(16).unwrap();
    code = (code << 4) | v[3].to_digit(16).unwrap();
    code
}

impl DecInner {
    fn new() -> Self {
        Self {
            state: State::None,
            idx: 0,
            done: None,
            acc_str: vec::Vec::new(),
        }
    }

    fn raise_error<'data>(&mut self, error: Error<'data>) -> Token<'data> {
        Token::Error(error)
    }

    fn raise_other<'data>(&mut self, token: Token<'data>) -> Token<'data> {
        token
    }

    fn raise_whitespace<'data>(&mut self, map: &'data [u8]) -> Token<'data> {
        Token::Whitespace {
            raw: map,
            str: core::str::from_utf8(map).unwrap(),
        }
    }

    fn raise_item<'data>(&mut self, map: &'data [u8]) -> Token<'data> {
        match map {
            b"null" => return self.raise_other(Token::Null),
            b"false" => return self.raise_other(Token::False),
            b"true" => return self.raise_other(Token::True),
            _ => {},
        }

        match core::str::from_utf8(map) {
            Ok(v) => Token::Error(Error::ItemUnknown { raw: map, str: v }),
            Err(v) => Token::Error(Error::ItemNonUtf8 { raw: map, error: v }),
        }
    }

    fn raise_number<'data>(
        &'data mut self,
        map: &'data [u8],
        sign: Sign,
        integer: core::ops::Range<usize>,
        fraction: core::ops::Range<usize>,
        exponent_sign: Sign,
        exponent: core::ops::Range<usize>,
    ) -> Token<'data> {
        Token::Number(Number {
            raw: map,
            sign: sign,
            integer: core::str::from_utf8(&map[integer.clone()]).unwrap(),
            fraction: if fraction.start != 0 {
                Some(core::str::from_utf8(&map[fraction.clone()]).unwrap())
            } else { None },
            exponent: if exponent.start != 0 {
                Some((
                    exponent_sign,
                    core::str::from_utf8(&map[exponent.clone()]).unwrap(),
                ))
            } else { None },
        })
    }

    fn number_done<'data>(&'data mut self, raw: &'data [u8]) -> Token<'data> {
        let &State::Number {
            sign, ref integer, ref fraction, exponent_sign, ref exponent
        } = &self.state else { core::unreachable!(); };

        if
            integer.start >= integer.end
            || (fraction.start != 0 && fraction.start >= fraction.end)
            || (exponent.start != 0 && exponent.start >= exponent.end)
        {
            self.state = State::NumberDone {
                sign: sign,
                integer: integer.clone(),
                fraction: fraction.clone(),
                exponent_sign: exponent_sign,
                exponent: exponent.clone(),
            };
            self.raise_error(Error::NumberRangeEmpty)
        } else {
            self.done = Some(self.idx);
            self.raise_number(raw, sign, integer.clone(), fraction.clone(), exponent_sign, exponent.clone())
        }
    }

    fn raise_string<'data>(&'data mut self, map: &'data [u8]) -> Token<'data> {
        let data = if self.acc_str.len() > 0 {
            &*self.acc_str
        } else {
            map
        };
        let data = &data[1..data.len()-1];

        match core::str::from_utf8(data) {
            Ok(v) => Token::String { raw: map, str: v },
            Err(v) => Token::Error(Error::StringNonUtf8 { raw: map, str: data, error: v }),
        }
    }

    fn string_raw(&mut self, raw: &[u8]) {
        if self.acc_str.len() > 0 {
            self.acc_str.extend_from_slice(raw);
        }
    }

    fn string_byte(&mut self, raw: &[u8], v: u8) {
        if self.acc_str.len() == 0 {
            self.acc_str.extend_from_slice(raw);
        }
        self.acc_str.push(v);
    }

    fn string_char(&mut self, raw: &[u8], v: char) {
        if self.acc_str.len() == 0 {
            self.acc_str.extend_from_slice(raw);
        }
        self.acc_str.extend_from_slice(v.encode_utf8(&mut [0; 4]).as_bytes());
    }
}

impl<'read> Dec<'read> {
    /// Create a new tokenizer for the given stream.
    pub fn with(
        read: &'read mut dyn io::stream::Read,
    ) -> Self {
        Self {
            inner: DecInner::new(),
            read: read,
        }
    }

    fn advance_none(&mut self) -> Flow<io::stream::More, Option<Token<'_>>> {
        let token = match self.read.map(1, Some(1))?[0] {
            b' ' | b'\n' | b'\r' | b'\t' => {
                self.inner.state = State::Whitespace;
                self.inner.idx = 1;
                None
            },
            b':' => { self.inner.done = Some(1); Some(self.inner.raise_other(Token::Colon)) },
            b',' => { self.inner.done = Some(1); Some(self.inner.raise_other(Token::Comma)) },
            b'[' => { self.inner.done = Some(1); Some(self.inner.raise_other(Token::ArrayOpen)) },
            b']' => { self.inner.done = Some(1); Some(self.inner.raise_other(Token::ArrayClose)) },
            b'{' => { self.inner.done = Some(1); Some(self.inner.raise_other(Token::ObjectOpen)) },
            b'}' => { self.inner.done = Some(1); Some(self.inner.raise_other(Token::ObjectClose)) },
            b'"' => {
                self.inner.state = State::String { idx_start: 0 };
                self.inner.idx = 1;
                None
            },
            v @ b'+' | v @ b'-' => {
                self.inner.state = State::Number {
                    sign: if v == b'-' { Sign::Minus } else { Sign::Plus },
                    integer: 1..1,
                    fraction: 0..0,
                    exponent_sign: Sign::Plus,
                    exponent: 0..0,
                };
                self.inner.idx = 1;
                if v == b'+' {
                    Some(self.inner.raise_error(Error::NumberLeadingPlus))
                } else {
                    None
                }
            },
            b'0'..=b'9' => {
                self.inner.state = State::Number {
                    sign: Sign::Plus,
                    integer: 0..1,
                    fraction: 0..0,
                    exponent_sign: Sign::Plus,
                    exponent: 0..0,
                };
                self.inner.idx = 1;
                None
            },
            _ => {
                self.inner.state = State::Item;
                self.inner.idx = 1;
                None
            },
        };

        Flow::Continue(token)
    }

    fn advance_whitespace(&mut self) -> Flow<io::stream::More, Option<Token<'_>>> {
        let map = self.read.map_while(
            &mut self.inner.idx,
            None,
            |_, v| matches!(v, b' ' | b'\n' | b'\r' | b'\t'),
        )?;
        self.inner.done = Some(self.inner.idx);
        Flow::Continue(Some(self.inner.raise_whitespace(&map[..self.inner.idx])))
    }

    fn advance_item(&mut self) -> Flow<io::stream::More, Option<Token<'_>>> {
        let map = self.read.map_while(
            &mut self.inner.idx,
            None,
            |_, v| matches!(
                v,
                b'+' | b'-' | b'.' | b'0'..=b'9'
                | b'a'..=b'z' | b'A'..=b'Z',
            ),
        )?;
        self.inner.done = Some(self.inner.idx);
        Flow::Continue(Some(self.inner.raise_item(&map[..self.inner.idx])))
    }

    fn advance_number(&mut self) -> Flow<io::stream::More, Option<Token<'_>>> {
        if let &State::NumberDone {
            sign,
            ref integer,
            ref fraction,
            exponent_sign,
            ref exponent,
        } = &self.inner.state {
            let map = self.read.map(self.inner.idx, None)?;
            self.inner.done = Some(self.inner.idx);
            return Flow::Continue(Some(self.inner.raise_number(
                &map[..self.inner.idx], sign, integer.clone(), fraction.clone(), exponent_sign, exponent.clone(),
            )));
        };

        let &mut State::Number {
            ref mut sign,
            ref mut integer,
            ref mut fraction,
            ref mut exponent_sign,
            ref mut exponent,
        } = &mut self.inner.state else { core::unreachable!(); };

        let token = {
            let map = self.read.map_while(
                &mut self.inner.idx,
                None,
                |_, v| v >= b'0' && v <= b'9',
            )?;

            let range = if exponent.start != 0 {
                *exponent = exponent.start..self.inner.idx;
                &*exponent
            } else if fraction.start != 0 {
                *fraction = fraction.start..self.inner.idx;
                &*fraction
            } else {
                *integer = integer.start..self.inner.idx;
                &*integer
            };

            match map[self.inner.idx] {
                v @ b'+' | v @ b'-' => {
                    let sign_v = if v == b'+' { Sign::Plus } else { Sign::Minus };

                    if range == integer && integer.start >= integer.end {
                        // We already got an integer sign, reject multiple
                        // leading signs.
                        self.inner.idx = self.inner.idx.strict_add(1);
                        *sign = sign_v;
                        *integer = self.inner.idx..self.inner.idx;
                        if self.inner.idx <= 2 {
                            Some(self.inner.raise_error(Error::NumberMultipleSigns))
                        } else {
                            // Suppress further errors of the same type.
                            None
                        }
                    } else if range == exponent && exponent.start >= exponent.end {
                        self.inner.idx = self.inner.idx.strict_add(1);
                        *exponent_sign = sign_v;
                        *exponent = self.inner.idx..self.inner.idx;
                        if
                            self.inner.idx != fraction.end.strict_add(2)
                            && self.inner.idx != integer.end.strict_add(2)
                        {
                            Some(self.inner.raise_error(Error::NumberMultipleSigns))
                        } else {
                            None
                        }
                    } else {
                        // Signs after integer, fraction, or exponent
                        // start a new number (even though a JSON parser
                        // would not allow consecutive numbers).
                        Some(self.inner.number_done(&map[..self.inner.idx]))
                    }
                },

                b'.' => {
                    if range == integer {
                        self.inner.idx = self.inner.idx.strict_add(1);
                        *fraction = self.inner.idx..self.inner.idx;
                        None
                    } else {
                        Some(self.inner.number_done(&map[..self.inner.idx]))
                    }
                },

                b'e' | b'E' => {
                    if range != exponent {
                        self.inner.idx = self.inner.idx.strict_add(1);
                        *exponent = self.inner.idx..self.inner.idx;
                        None
                    } else {
                        // e/E anywhere but after the integer or fraction range
                        // start a new number.
                        Some(self.inner.number_done(&map[..self.inner.idx]))
                    }
                },

                _ => {
                    Some(self.inner.number_done(&map[..self.inner.idx]))
                },
            }
        };

        Flow::Continue(token)
    }

    fn advance_string(&mut self) -> Flow<io::stream::More, Option<Token<'_>>> {
        let token = match self.inner.state {
            State::String { idx_start } => {
                let map = self.read.map_while(
                    &mut self.inner.idx,
                    None,
                    |_, v| !matches!(v, b'"' | b'\\' | 0x00..0x1f),
                )?;

                match map[self.inner.idx] {
                    b'"' => {
                        self.inner.idx = self.inner.idx.strict_add(1);
                        self.inner.string_raw(&map[idx_start..self.inner.idx]);
                        self.inner.done = Some(self.inner.idx);
                        Some(self.inner.raise_string(&map[..self.inner.idx]))
                    },
                    b'\\' => {
                        self.inner.string_raw(&map[idx_start..self.inner.idx]);
                        self.inner.idx = self.inner.idx.strict_add(1);
                        self.inner.state = State::StringEscape;
                        None
                    },
                    v @ 0x00..0x1f => {
                        self.inner.idx = self.inner.idx.strict_add(1);
                        self.inner.string_raw(&map[idx_start..self.inner.idx]);
                        Some(self.inner.raise_error(Error::StringUnescaped {
                            code: v,
                        }))
                    },
                    _ => core::unreachable!(),
                }
            },

            State::StringEscape => {
                let idx1 = self.inner.idx.strict_add(1);
                let map = self.read.map(idx1, None)?;
                let esc = map[self.inner.idx];
                self.inner.idx = idx1;

                match esc {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                        let v = match esc {
                            b'b' => b'\x08',
                            b'f' => b'\x0c',
                            b'n' => b'\x0a',
                            b'r' => b'\x0d',
                            b't' => b'\x09',
                            v => v,
                        };
                        self.inner.string_byte(&map[..self.inner.idx.strict_sub(2)], v);
                        self.inner.state = State::String { idx_start: self.inner.idx };
                        None
                    },
                    b'u' => {
                        self.inner.state = State::StringEscapeUnicode {
                            idx_start: self.inner.idx,
                        };
                        None
                    },
                    v => {
                        self.inner.string_byte(&map[..self.inner.idx.strict_sub(2)], v);
                        self.inner.state = State::String { idx_start: self.inner.idx };
                        Some(self.inner.raise_error(Error::StringEscapeUnknown {
                            code: v,
                        }))
                    },
                }
            },

            State::StringEscapeUnicode { idx_start } => {
                let max = idx_start.strict_add(4);
                #[allow(clippy::manual_is_ascii_check)]
                let map = self.read.map_while(
                    &mut self.inner.idx,
                    Some(max),
                    |_, v| matches!(v, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'),
                )?;

                if self.inner.idx != idx_start.strict_add(4) {
                    self.inner.state = State::String { idx_start: self.inner.idx };
                    self.inner.string_char(
                        &map[..idx_start.strict_sub(2)],
                        char::REPLACEMENT_CHARACTER,
                    );
                    Some(self.inner.raise_error(Error::StringEscapeInvalid))
                } else {
                    let code = unicode_from_hex([
                        char::from_u32(map[idx_start + 0] as u32).unwrap(),
                        char::from_u32(map[idx_start + 1] as u32).unwrap(),
                        char::from_u32(map[idx_start + 2] as u32).unwrap(),
                        char::from_u32(map[idx_start + 3] as u32).unwrap(),
                    ]);
                    if code >= 0xd800 && code <= 0xdbff {
                        // Lead Surrogate
                        self.inner.state = State::StringEscapeUnicodeTrail {
                            idx_start: self.inner.idx,
                            lead: code,
                        };
                        None
                    } else if code >= 0xdc00 && code <= 0xdfff {
                        // Trail Surrogate
                        self.inner.string_char(
                            &map[..self.inner.idx.strict_sub(6)],
                            char::REPLACEMENT_CHARACTER,
                        );
                        self.inner.state = State::String { idx_start: self.inner.idx };
                        Some(self.inner.raise_error(
                            Error::StringEscapeUnpairedTrailSurrogate { trail: code },
                        ))
                    } else {
                        self.inner.string_char(
                            &map[..self.inner.idx.strict_sub(6)],
                            char::from_u32(code).unwrap(),
                        );
                        self.inner.state = State::String { idx_start: self.inner.idx };
                        None
                    }
                }
            },

            State::StringEscapeUnicodeTrail { idx_start, lead } => {
                let max = idx_start.strict_add(6);
                let map = self.read.map_while(
                    &mut self.inner.idx,
                    Some(max),
                    |idx, v| matches!(
                        (idx.strict_sub(idx_start), v),
                        (0, b'\\') | (1, b'u')
                        | (
                            2..=5,
                            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'
                        ),
                    ),
                )?;

                if self.inner.idx != idx_start.strict_add(6) {
                    self.inner.state = State::String { idx_start: idx_start };
                    self.inner.idx = idx_start;
                    self.inner.string_char(
                        &map[..idx_start.strict_sub(6)],
                        char::REPLACEMENT_CHARACTER,
                    );
                    Some(self.inner.raise_error(
                        Error::StringEscapeUnpairedLeadSurrogate { lead: lead },
                    ))
                } else {
                    let code = unicode_from_hex([
                        char::from_u32(map[idx_start + 2] as u32).unwrap(),
                        char::from_u32(map[idx_start + 3] as u32).unwrap(),
                        char::from_u32(map[idx_start + 4] as u32).unwrap(),
                        char::from_u32(map[idx_start + 5] as u32).unwrap(),
                    ]);
                    if code >= 0xd800 && code <= 0xdbff {
                        // This is a lead surrogate following a lead surrogate.
                        // Reject the previous lead surrogate as unpaired and
                        // start over with this lead surrogate.
                        self.inner.string_char(
                            &map[..self.inner.idx.strict_sub(12)],
                            char::REPLACEMENT_CHARACTER,
                        );
                        self.inner.state = State::StringEscapeUnicodeTrail {
                            idx_start: self.inner.idx,
                            lead: code,
                        };
                        Some(self.inner.raise_error(
                            Error::StringEscapeUnpairedLeadSurrogate { lead: lead },
                        ))
                    } else if code >= 0xdc00 && code <= 0xdfff {
                        // This is a trail surrogate following a lead
                        // surrogate, thus a valid surrogate pair.
                        let full = 0x10000 + ((lead - 0xd800) << 10) + (code - 0xdc00);
                        self.inner.string_char(
                            &map[..self.inner.idx.strict_sub(12)],
                            char::from_u32(full).unwrap(),
                        );
                        self.inner.state = State::String { idx_start: self.inner.idx };
                        None
                    } else {
                        // This is not a surrogate, so reject the previous
                        // lead surrogate but keep this codepoint.
                        self.inner.string_char(
                            &map[..self.inner.idx.strict_sub(12)],
                            char::REPLACEMENT_CHARACTER,
                        );
                        self.inner.string_char(
                            &map[..self.inner.idx.strict_sub(6)],
                            char::from_u32(code).unwrap(),
                        );
                        self.inner.state = State::String { idx_start: self.inner.idx };
                        Some(self.inner.raise_error(
                            Error::StringEscapeUnpairedLeadSurrogate { lead: lead },
                        ))
                    }
                }
            },

            _ => core::unreachable!(),
        };

        Flow::Continue(token)
    }

    fn clear_done(&mut self) {
        if let Some(len) = self.inner.done.take() {
            self.inner.state = State::None;
            self.inner.acc_str.clear();
            self.inner.acc_str.shrink_to(4096);
            self.read.advance(len);
        }
    }

    fn complete_inner(&mut self) -> Option<Token<'_>> {
        self.clear_done();

        let token = if self.inner.state == State::None {
            let Flow::Break(_) = self.read.map(1, None) else {
                return Some(self.inner.raise_error(Error::BufferRemaining));
            };

            None
        } else {
            let Flow::Continue(map) = self.read.map(self.inner.idx, None) else {
                self.inner.done = Some(self.inner.idx);
                return Some(self.inner.raise_error(Error::BufferChanged));
            };

            match self.inner.state {
                State::None => None,
                State::Whitespace => Some(self.inner.raise_whitespace(map)),
                State::Item => Some(self.inner.raise_item(map)),
                State::Number { .. }
                | State::NumberDone { .. } => Some(self.inner.number_done(map)),
                State::String { .. }
                | State::StringEscape
                | State::StringEscapeUnicode { .. }
                | State::StringEscapeUnicodeTrail { .. } => {
                    Some(self.inner.raise_error(Error::StringIncomplete))
                },
            }
        };

        token
    }

    fn advance_inner(&mut self) -> Flow<io::stream::More, Option<Token<'_>>> {
        self.clear_done();

        let token = match self.inner.state {
            State::None => self.advance_none()?,
            State::Whitespace => self.advance_whitespace()?,
            State::Item => self.advance_item()?,
            State::Number { .. }
            | State::NumberDone { .. } => self.advance_number()?,
            State::String { .. }
            | State::StringEscape
            | State::StringEscapeUnicode { .. }
            | State::StringEscapeUnicodeTrail { .. } => self.advance_string()?,
        };

        Flow::Continue(token)
    }

    fn advance<'this>(&'this mut self) -> Flow<io::stream::More, Token<'this>> {
        loop {
            let Some(v) = self.advance_inner()? else {
                continue;
            };

            // Without Polonius, the lifetime check here fails due to mutable
            // reborrows in the loop. Override it manually, relying on
            // correctness of Polonius.
            let fixed = {
                osi::cfg::cond! {
                    (polonius) { v },
                    {
                        // SAFETY: Workaround for NLL, unneeded with Polonius.
                        unsafe {
                            core::mem::transmute::<Token<'_>, Token<'this>>(v)
                        }
                    },
                }
            };

            return Flow::Continue(fixed);
        }
    }

    /// Retrieve the next token from the stream.
    ///
    /// If the tokenizer needs more data to parse the next token, then
    /// `io::stream::More` is returned with the required extents. Otherwise,
    /// the next token is returned.
    ///
    /// This should usually be called in a loop until it returns
    /// `Flow::Break(_)`.
    pub fn pop(&mut self) -> Flow<io::stream::More, Token<'_>> {
        self.advance()
    }

    /// Finalize the stream.
    ///
    /// Mark the end of the stream and finalize any open tokens. If no more
    /// tokens are left, this will return `None`.
    ///
    /// Usually, it is enough to call this once. However, due to error tokens
    /// being reported inline, you should call this in a loop until it returns
    /// `None` to ensure you retrieve all error tokens.
    pub fn complete(&mut self) -> Option<Token<'_>> {
        self.complete_inner()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokens_valid() {
        let mut buf: &[u8] = br#" [ "foobar", null ]"#;
        let mut dec = Dec::with(&mut buf);

        assert_eq!(dec.pop(), Flow::Continue(Token::Whitespace { raw: b" ", str: " "}));
        assert_eq!(dec.pop(), Flow::Continue(Token::ArrayOpen));
        assert_eq!(dec.pop(), Flow::Continue(Token::Whitespace { raw: b" ", str: " "}));
        assert_eq!(dec.pop(), Flow::Continue(Token::String { raw: br#""foobar""#, str: "foobar" }));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));
        assert_eq!(dec.pop(), Flow::Continue(Token::Whitespace { raw: b" ", str: " "}));
        assert_eq!(dec.pop(), Flow::Continue(Token::Null));
        assert_eq!(dec.pop(), Flow::Continue(Token::Whitespace { raw: b" ", str: " "}));
        assert_eq!(dec.pop(), Flow::Continue(Token::ArrayClose));
        assert_eq!(dec.complete(), None);
    }

    #[test]
    fn number_valid() {
        let raw = b"\
            0,\
            1,\
            -0.1e+5,\
        ";
        let mut buf: &[u8] = raw;
        let mut dec = Dec::with(&mut buf);

        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"0",
            sign: Sign::Plus,
            integer: "0",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"1",
            sign: Sign::Plus,
            integer: "1",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-0.1e+5",
            sign: Sign::Minus,
            integer: "0",
            fraction: Some("1"),
            exponent: Some((Sign::Plus, "5")),
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));
        assert_eq!(dec.complete(), None);
    }

    #[test]
    fn number_leading_signs() {
        let raw = b"\
            +71,\
            -71,\
            -+71,\
            --71,\
            ++71,\
            --++71,\
        ";
        let mut buf: &[u8] = raw;
        let mut dec = Dec::with(&mut buf);

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberLeadingPlus,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"+71",
            sign: Sign::Plus,
            integer: "71",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-71",
            sign: Sign::Minus,
            integer: "71",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberMultipleSigns,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-+71",
            sign: Sign::Plus,
            integer: "71",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberMultipleSigns,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"--71",
            sign: Sign::Minus,
            integer: "71",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberLeadingPlus,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberMultipleSigns,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"++71",
            sign: Sign::Plus,
            integer: "71",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberMultipleSigns,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"--++71",
            sign: Sign::Plus,
            integer: "71",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.complete(), None);
    }

    #[test]
    fn number_empty_ranges() {
        let raw = b"\
            -,\
            -0.,\
            -0.0e,\
            -0.0e+,\
            -.0e+0,\
            -0.0e+0,\
        ";
        let mut buf: &[u8] = raw;
        let mut dec = Dec::with(&mut buf);

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberRangeEmpty,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-",
            sign: Sign::Minus,
            integer: "",
            fraction: None,
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberRangeEmpty,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-0.",
            sign: Sign::Minus,
            integer: "0",
            fraction: Some(""),
            exponent: None,
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberRangeEmpty,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-0.0e",
            sign: Sign::Minus,
            integer: "0",
            fraction: Some("0"),
            exponent: Some((Sign::Plus, "")),
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberRangeEmpty,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-0.0e+",
            sign: Sign::Minus,
            integer: "0",
            fraction: Some("0"),
            exponent: Some((Sign::Plus, "")),
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Error(
            Error::NumberRangeEmpty,
        )));
        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-.0e+0",
            sign: Sign::Minus,
            integer: "",
            fraction: Some("0"),
            exponent: Some((Sign::Plus, "0")),
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.pop(), Flow::Continue(Token::Number(Number {
            raw: b"-0.0e+0",
            sign: Sign::Minus,
            integer: "0",
            fraction: Some("0"),
            exponent: Some((Sign::Plus, "0")),
        })));
        assert_eq!(dec.pop(), Flow::Continue(Token::Comma));

        assert_eq!(dec.complete(), None);
    }

    // Ensure that strings without escape characters are provided directly from
    // the input buffer without any copying involved. Verify this by comparing
    // the raw pointers of the returned data.
    #[test]
    fn string_noncopy() {
        let raw = br#""foobar""#;
        let mut buf: &[u8] = raw;
        let mut dec = Dec::with(&mut buf);
        let token = dec.pop().continue_value().unwrap();
        let Token::String { raw: token_raw, str: token_str } = token else {
            panic!();
        };
        assert_eq!(token_raw, raw);
        assert_eq!(token_str.as_bytes(), &raw[1..7]);
        assert!(core::ptr::eq(token_raw, raw));
        assert!(core::ptr::eq(token_str.as_bytes(), &raw[1..7]));
        assert_eq!(dec.complete(), None);
    }

    // Ensure that strings are correctly copied over into the accumulator for
    // every possible escape sequence.
    #[test]
    fn string_copy() {
        // Valid single-character escape
        {
            let raw = br#""foo\nbar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo\nbar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid single-character escape
        {
            let raw = br#""foo\zbar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeUnknown { code: b'z' },
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foozbar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Valid single unicode escape
        {
            let raw = br#""foo\u1234bar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo\u{1234}bar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid single unicode escape
        {
            let raw = br#""foo\u01Zbar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeInvalid,
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo�Zbar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid lone trail surrogate
        {
            let raw = br#""foo\udc00bar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeUnpairedTrailSurrogate { trail: 0xdc00 },
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo�bar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Valid paired surrogate
        {
            let raw = br#""foo\ud800\udc00bar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo\u{10000}bar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid lone lead surrogate followed by no escape
        {
            let raw = br#""foo\ud800bar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeUnpairedLeadSurrogate { lead: 0xd800 },
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo�bar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid lone lead surrogate followed by single character escape
        {
            let raw = br#""foo\ud800\nbar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeUnpairedLeadSurrogate { lead: 0xd800 },
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo�\nbar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid lead surrogate followed by another lead
        {
            let raw = br#""foo\ud800\ud800\udc00bar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeUnpairedLeadSurrogate { lead: 0xd800 },
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo�\u{10000}bar",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Invalid lead surrogate followed by a non-surrogate unicode escape.
        {
            let raw = br#""foo\ud800\u1234bar""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);
            assert_eq!(dec.pop(), Flow::Continue(Token::Error(
                Error::StringEscapeUnpairedLeadSurrogate { lead: 0xd800 },
            )));
            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "foo�\u{1234}bar",
            }));
            assert_eq!(dec.complete(), None);
        }
    }

    // Test different escape sequences and their combinations.
    #[test]
    fn string_esc() {
        // All valid single-character escapes
        {
            let raw = br#""\b\f\n\r\t\"\\\/""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);

            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "\x08\x0c\x0a\x0d\x09\"\\/",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Valid non-surrogate unicode escapes
        {
            let raw = br#""\u0000\u1234\uffff""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);

            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "\u{0000}\u{1234}\u{ffff}",
            }));
            assert_eq!(dec.complete(), None);
        }

        // Valid surrogate-pair unicode escapes
        {
            let raw = br#""\ud834\udd1e""#;
            let mut buf: &[u8] = raw;
            let mut dec = Dec::with(&mut buf);

            assert_eq!(dec.pop(), Flow::Continue(Token::String {
                raw: raw,
                str: "\u{1d11e}",
            }));
            assert_eq!(dec.complete(), None);
        }
    }
}
