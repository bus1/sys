//! # D-Bus DVariant Format
//!
//! This implements the wire encoding of the original D-Bus specification
//! (v0.43 and later). It provides encoders and decoders for memory mapped
//! data.
//!
//! ## Deviations
//!
//! The following behavior deviates from the D-Bus Specification v0.43:
//!
//! - The length of the encoded data can be up to `isize::MAX`. That is,
//!   anything that can be mapped into the address space can be decoded and
//!   encoded. No arbitrary limits are enforced.
//! - The nesting depth of compound types is not limited. The length of the
//!   type signature naturally limits the nesting depth. No additional limits
//!   are enforced.
//! - All operations run in O(n) space and time relative to the length of the
//!   encoded data (data length includes the type signature). If the caller
//!   needs stricter limits, they must enforce it manually.
//! - Embedded 0 bytes are supported for strings and objects. They do not get
//!   any special treatment.

use alloc::{sync, vec};
use core::ops::ControlFlow as Flow;

use crate::fmt::dbus;
use crate::io;

#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum Format {
    DVarBe,
    DVarLe,
}

#[derive(Clone)]
struct Level {
    idx: usize,
    from: usize,
    meta: usize,
}

type MownSig<'sig> = osi::mown::Mown<'sig, dbus::Sig, sync::Arc<dbus::Sig>>;
type Cursor<'sig> = dbus::Cursor<'sig, sync::Arc<dbus::Sig>>;

pub struct Enc<'sig, 'write> {
    done: bool,
    format: Format,
    cursor: Cursor<'sig>,
    level: Level,
    stack: vec::Vec<(dbus::Element, Level, Option<Cursor<'sig>>)>,
    write: &'write mut dyn io::map::Write,
    enc: Option<(&'write mut Level, Option<usize>, &'write mut usize)>,
}

pub struct Dec<'sig, 'read> {
    format: Format,
    cursor: Cursor<'sig>,
    level: Level,
    stack: vec::Vec<(dbus::Element, Level, Option<Cursor<'sig>>)>,
    read: &'read mut dyn io::map::Read,
    dec: Option<(&'read mut Level, Option<usize>, &'read mut usize)>,
}

impl core::convert::From<io::map::Error> for dbus::Error {
    fn from(v: io::map::Error) -> Self {
        Self::Io(v)
    }
}

impl Format {
    fn is_be(&self) -> bool {
        match *self {
            Format::DVarBe => true,
            Format::DVarLe => false,
        }
    }

    fn en_u8(&self, v: u8) -> [u8; 1] {
        [v]
    }

    fn en_u16(&self, v: u16) -> [u8; 2] {
        if self.is_be() { v.to_be_bytes() } else { v.to_le_bytes() }
    }

    fn en_u32(&self, v: u32) -> [u8; 4] {
        if self.is_be() { v.to_be_bytes() } else { v.to_le_bytes() }
    }

    fn en_u64(&self, v: u64) -> [u8; 8] {
        if self.is_be() { v.to_be_bytes() } else { v.to_le_bytes() }
    }

    fn de_u8(&self, v: [u8; 1]) -> u8 {
        if self.is_be() { u8::from_be_bytes(v) } else { u8::from_le_bytes(v) }
    }

    fn de_u16(&self, v: [u8; 2]) -> u16 {
        if self.is_be() { u16::from_be_bytes(v) } else { u16::from_le_bytes(v) }
    }

    fn de_u32(&self, v: [u8; 4]) -> u32 {
        if self.is_be() { u32::from_be_bytes(v) } else { u32::from_le_bytes(v) }
    }
}

impl<'sig, 'write> Enc<'sig, 'write> {
    pub fn with(
        sig: MownSig<'sig>,
        format: Format,
        write: &'write mut dyn io::map::Write,
    ) -> Self {
        Self {
            done: false,
            cursor: dbus::Cursor::new(sig),
            format: format,
            level: Level {
                idx: 0,
                from: 0,
                meta: 0,
            },
            stack: vec::Vec::new(),
            write: write,
            enc: None,
        }
    }

    pub fn new_be(
        sig: &'sig dbus::Sig,
        write: &'write mut dyn io::map::Write,
    ) -> Self {
        Self::with(MownSig::new_borrowed(sig), Format::DVarBe, write)
    }

    pub fn new_le(
        sig: &'sig dbus::Sig,
        write: &'write mut dyn io::map::Write,
    ) -> Self {
        Self::with(MownSig::new_borrowed(sig), Format::DVarLe, write)
    }

    pub fn enc(&mut self) -> Result<Enc<'_, '_>, dbus::Error> {
        let up_step = self.cursor.idx_step();
        let (up_sig, up_idx) = self.cursor.raw();

        if let Some(v) = up_sig.at(*up_idx) {
            Ok(Enc {
                done: false,
                cursor: Cursor::new_borrowed(v),
                format: self.format,
                level: Level {
                    idx: self.level.idx,
                    from: self.level.idx,
                    meta: self.level.idx,
                },
                stack: vec::Vec::new(),
                write: self.write,
                enc: Some((&mut self.level, up_step, up_idx)),
            })
        } else {
            Err(dbus::Error::Mismatch)
        }
    }

    pub fn enc_with(&mut self, sig: MownSig) -> Result<Enc<'_, '_>, dbus::Error> {
        let (cursor_sig, cursor_idx) = self.cursor.raw();

        if Some(&*sig) == cursor_sig.at(*cursor_idx) {
            self.enc()
        } else {
            Err(dbus::Error::Mismatch)
        }
    }

    pub fn commit(&mut self) -> Result<(), dbus::Error> {
        if self.cursor.idx_step().is_some() || !self.stack.is_empty() {
            return Err(dbus::Error::Pending);
        }

        if let Some((up_level, up_step, up_idx)) = self.enc.take() {
            up_level.idx = self.level.idx;
            if let Some(v) = up_step {
                *up_idx = v;
            }
            self.done = true;
        } else if !self.done {
            unsafe { self.write.commit(self.level.idx) };
            self.done = true;
        }

        Ok(())
    }

    fn write(
        write: &mut dyn io::map::Write,
        idx: &mut usize,
        data: &[u8],
    ) -> Flow<Option<dbus::Error>> {
        write.write(idx, data).map_break(|v| v.map(|v| v.into()))
    }

    fn write_iter(
        write: &mut dyn io::map::Write,
        idx: &mut usize,
        data: &mut dyn ExactSizeIterator<Item = u8>,
    ) -> Flow<Option<dbus::Error>> {
        write.write_iter(idx, data).map_break(|v| v.map(|v| v.into()))
    }

    fn zero(
        write: &mut dyn io::map::Write,
        idx: &mut usize,
        len: usize,
    ) -> Flow<Option<dbus::Error>> {
        write.zero(idx, len).map_break(|v| v.map(|v| v.into()))
    }

    fn align(
        write: &mut dyn io::map::Write,
        idx: &mut usize,
        exp: u8,
    ) -> Flow<Option<dbus::Error>> {
        write.align_exp2(idx, exp).map_break(|v| v.map(|v| v.into()))
    }

    fn fixed(
        &mut self,
        element: dbus::Element,
        data: &[u8],
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(element) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut idx = self.level.idx;
        Self::align(self.write, &mut idx, element.dvar_alignment_exp())?;
        Self::write(self.write, &mut idx, data)?;
        self.level.idx = idx;
        self.cursor.move_step();

        Flow::Continue(self)
    }

    fn str8(
        &mut self,
        element: dbus::Element,
        data: &str,
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(element) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }
        let Ok(n): Result<u8, _> = data.len().try_into() else {
            return Flow::Break(Some(dbus::Error::DataOverflow));
        };

        let mut idx = self.level.idx;
        Self::align(self.write, &mut idx, dbus::Element::U8.dvar_alignment_exp())?;
        Self::write(self.write, &mut idx, &self.format.en_u8(n))?;
        Self::write(self.write, &mut idx, data.as_bytes())?;
        Self::zero(self.write, &mut idx, 1)?;
        self.level.idx = idx;
        self.cursor.move_step();

        Flow::Continue(self)
    }

    fn str32(
        &mut self,
        element: dbus::Element,
        data: &str,
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(element) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }
        let Ok(n): Result<u32, _> = data.len().try_into() else {
            return Flow::Break(Some(dbus::Error::DataOverflow));
        };

        let mut idx = self.level.idx;
        Self::align(self.write, &mut idx, dbus::Element::U32.dvar_alignment_exp())?;
        Self::write(self.write, &mut idx, &self.format.en_u32(n))?;
        Self::write(self.write, &mut idx, data.as_bytes())?;
        Self::zero(self.write, &mut idx, 1)?;
        self.level.idx = idx;
        self.cursor.move_step();

        Flow::Continue(self)
    }

    pub fn u16(&mut self, data: u16) -> Flow<Option<dbus::Error>, &mut Self> {
        self.fixed(dbus::Element::U16, &self.format.en_u16(data))
    }

    pub fn u32(&mut self, data: u32) -> Flow<Option<dbus::Error>, &mut Self> {
        self.fixed(dbus::Element::U32, &self.format.en_u32(data))
    }

    pub fn u64(&mut self, data: u64) -> Flow<Option<dbus::Error>, &mut Self> {
        self.fixed(dbus::Element::U64, &self.format.en_u64(data))
    }

    pub fn string(&mut self, data: &str) -> Flow<Option<dbus::Error>, &mut Self> {
        self.str32(dbus::Element::String, data)
    }

    pub fn object(&mut self, data: &str) -> Flow<Option<dbus::Error>, &mut Self> {
        self.str32(dbus::Element::Object, data)
    }

    pub fn signature(&mut self, data: &str) -> Flow<Option<dbus::Error>, &mut Self> {
        self.str8(dbus::Element::Signature, data)
    }

    pub fn variant_with(
        &mut self,
        sig: osi::mown::Mown<'sig, dbus::Sig, sync::Arc<dbus::Sig>>,
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(dbus::Element::Variant) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }
        let Ok(n): Result<u8, _> = sig.len().try_into() else {
            return Flow::Break(Some(dbus::Error::DataOverflow));
        };

        let mut level = self.level.clone();
        Self::align(self.write, &mut level.idx, dbus::Element::U8.dvar_alignment_exp())?;
        Self::write(self.write, &mut level.idx, &self.format.en_u8(n))?;
        Self::write_iter(self.write, &mut level.idx, &mut sig.into_iter().map(|v| v.code()))?;
        Self::zero(self.write, &mut level.idx, 1)?;
        level.meta = level.idx;
        level.from = level.idx;

        let mut cursor = Cursor::new(sig);
        core::mem::swap(&mut cursor, &mut self.cursor);
        core::mem::swap(&mut level, &mut self.level);
        self.stack.push((dbus::Element::Variant, level, Some(cursor)));

        Flow::Continue(self)
    }

    pub fn variant(&mut self, sig: &'sig dbus::Sig) -> Flow<Option<dbus::Error>, &mut Self> {
        self.variant_with(osi::mown::Mown::new_borrowed(sig))
    }

    pub fn array(&mut self) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(dbus::Element::Array) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut level = self.level.clone();
        let align = self.cursor.down().unwrap().dvar_alignment_exp();
        Self::align(self.write, &mut level.idx, dbus::Element::U32.dvar_alignment_exp())?;
        level.meta = level.idx;
        Self::write(self.write, &mut level.idx, &self.format.en_u32(0))?;
        Self::align(self.write, &mut level.idx, align)?;
        level.from = level.idx;

        core::mem::swap(&mut level, &mut self.level);
        self.stack.push((dbus::Element::Array, level, None));
        self.cursor.move_down();

        Flow::Continue(self)
    }

    // NB: The related element is usually referred to as `struct`, yet that
    //     is a reserved keyword in Rust, hence this uses `structure`.
    pub fn structure(&mut self) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(dbus::Element::StructOpen) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut level = self.level.clone();
        let align = self.cursor.dvar_alignment_exp().unwrap();
        Self::align(self.write, &mut level.idx, align)?;
        level.meta = level.idx;
        level.from = level.idx;

        core::mem::swap(&mut level, &mut self.level);
        self.stack.push((dbus::Element::StructOpen, level, None));
        self.cursor.move_down();

        Flow::Continue(self)
    }

    pub fn dict(&mut self) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(dbus::Element::DictOpen) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut level = self.level.clone();
        let align = self.cursor.dvar_alignment_exp().unwrap();
        Self::align(self.write, &mut level.idx, align)?;
        level.meta = level.idx;
        level.from = level.idx;

        core::mem::swap(&mut level, &mut self.level);
        self.stack.push((dbus::Element::DictOpen, level, None));
        self.cursor.move_down();

        Flow::Continue(self)
    }

    pub fn close(&mut self) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.idx_step().is_some() {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }
        let Some(
            &mut (up_element, ref mut up_level, ref mut up_cursor)
        ) = self.stack.last_mut() else {
            return Flow::Break(Some(dbus::Error::Mismatch));
        };

        match up_element {
            dbus::Element::Variant => {
                // Nothing to finalize.
            },
            dbus::Element::Array => {
                let n = self.level.idx.strict_sub(self.level.from);
                if n > 0 {
                    let Ok(n): Result<u32, _> = n.try_into() else {
                        return Flow::Break(Some(dbus::Error::DataOverflow));
                    };
                    let mut idx = self.level.meta;
                    Self::write(self.write, &mut idx, &self.format.en_u32(n))?;
                }
            },
            dbus::Element::StructOpen => {
                // Nothing to finalize for structures.
            },
            dbus::Element::DictOpen => {
                // Nothing to finalize for structures.
            },
            _ => core::unreachable!(),
        }

        core::mem::swap(&mut self.level, up_level);
        self.level.idx = up_level.idx;

        if let Some(ref mut v) = up_cursor {
            core::mem::swap(v, &mut self.cursor);
        } else {
            self.cursor.move_up();
        }

        self.cursor.move_step();
        self.stack.pop();

        Flow::Continue(self)
    }
}

impl<'sig, 'read> Dec<'sig, 'read> {
    pub fn with(
        sig: MownSig<'sig>,
        format: Format,
        read: &'read mut dyn io::map::Read,
    ) -> Self {
        Self {
            cursor: dbus::Cursor::new(sig),
            format: format,
            level: Level {
                idx: 0,
                from: 0,
                meta: 0,
            },
            stack: vec::Vec::new(),
            read: read,
            dec: None,
        }
    }

    pub fn new_be(
        sig: &'sig dbus::Sig,
        read: &'read mut dyn io::map::Read,
    ) -> Self {
        Self::with(MownSig::new_borrowed(sig), Format::DVarBe, read)
    }

    pub fn new_le(
        sig: &'sig dbus::Sig,
        read: &'read mut dyn io::map::Read,
    ) -> Self {
        Self::with(MownSig::new_borrowed(sig), Format::DVarLe, read)
    }

    pub fn more(&self) -> bool {
        self.level.idx < self.level.meta
    }

    pub fn dec(&mut self) -> Result<Dec<'_, '_>, dbus::Error> {
        let up_step = self.cursor.idx_step();
        let (up_sig, up_idx) = self.cursor.raw();

        if let Some(v) = up_sig.at(*up_idx) {
            Ok(Dec {
                cursor: Cursor::new_borrowed(v),
                format: self.format,
                level: Level {
                    idx: self.level.idx,
                    from: self.level.idx,
                    meta: self.level.idx,
                },
                stack: vec::Vec::new(),
                read: self.read,
                dec: Some((&mut self.level, up_step, up_idx)),
            })
        } else {
            Err(dbus::Error::Mismatch)
        }
    }

    pub fn dec_with(&mut self, sig: MownSig) -> Result<Dec<'_, '_>, dbus::Error> {
        let (cursor_sig, cursor_idx) = self.cursor.raw();

        if Some(&*sig) == cursor_sig.at(*cursor_idx) {
            self.dec()
        } else {
            Err(dbus::Error::Mismatch)
        }
    }

    pub fn commit(&mut self) -> Result<(), dbus::Error> {
        if self.cursor.idx_step().is_some() || !self.stack.is_empty() {
            return Err(dbus::Error::Pending);
        }

        if let Some((up_level, up_step, up_idx)) = self.dec.take() {
            up_level.idx = self.level.idx;
            if let Some(v) = up_step {
                *up_idx = v;
            }
        }

        Ok(())
    }

    fn read(
        read: &mut dyn io::map::Read,
        idx: &mut usize,
        data: &mut [u8],
    ) -> Flow<Option<dbus::Error>> {
        read.read(idx, data).map_break(|v| v.map(|v| v.into()))
    }

    fn read_uninit(
        read: &mut dyn io::map::Read,
        idx: &mut usize,
        data: &mut [core::mem::MaybeUninit<u8>],
    ) -> Flow<Option<dbus::Error>> {
        read.read_uninit(idx, data).map_break(|v| v.map(|v| v.into()))
    }

    fn align(
        _read: &mut dyn io::map::Read,
        idx: &mut usize,
        exp: u8,
    ) -> Flow<Option<dbus::Error>> {
        match idx.checked_next_multiple_of((1 << exp) as usize) {
            None => Flow::Break(Some(dbus::Error::Io(io::map::Error::Overflow))),
            Some(v) => {
                *idx = v;
                Flow::Continue(())
            },
        }
    }

    fn fixed(
        &mut self,
        element: dbus::Element,
        data: &mut [u8],
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(element) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut idx = self.level.idx;
        Self::align(self.read, &mut idx, element.dvar_alignment_exp())?;
        Self::read(self.read, &mut idx, data)?;
        self.level.idx = idx;
        self.cursor.move_step();

        Flow::Continue(self)
    }

    fn str8(
        &mut self,
        element: dbus::Element,
        data: &mut alloc::string::String,
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(element) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut idx = self.level.idx;

        // Read length byte.
        let mut len_u = [0; _];
        Self::align(self.read, &mut idx, dbus::Element::U8.dvar_alignment_exp())?;
        Self::read(self.read, &mut idx, &mut len_u)?;
        let len = self.format.de_u8(len_u) as usize;

        // Read the string.
        let mut buffer = alloc::vec::Vec::with_capacity(len);
        let buf_p = &mut buffer.spare_capacity_mut()[..len];
        Self::read_uninit(self.read, &mut idx, buf_p)?;
        // SAFETY: `Self::read_uninit()` always initializes the full slice.
        unsafe { buffer.set_len(len) };

        // Validate UTF-8.
        *data = match alloc::string::String::from_utf8(buffer) {
            Ok(v) => v,
            Err(_) => return Flow::Break(Some(dbus::Error::DataNonUtf8)),
        };

        // Skip unused terminating 0.
        idx = idx.strict_add(1);

        self.level.idx = idx;
        self.cursor.move_step();

        Flow::Continue(self)
    }

    fn str32(
        &mut self,
        element: dbus::Element,
        data: &mut alloc::string::String,
    ) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(element) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut idx = self.level.idx;

        // Read length byte.
        let mut len_u = [0; _];
        Self::align(self.read, &mut idx, dbus::Element::U32.dvar_alignment_exp())?;
        Self::read(self.read, &mut idx, &mut len_u)?;
        let len = self.format.de_u32(len_u) as usize;

        // Read the string.
        let mut buffer = alloc::vec::Vec::with_capacity(len);
        let buf_p = &mut buffer.spare_capacity_mut()[..len];
        Self::read_uninit(self.read, &mut idx, buf_p)?;
        // SAFETY: `Self::read_uninit()` always initializes the full slice.
        unsafe { buffer.set_len(len) };

        // Validate UTF-8.
        *data = match alloc::string::String::from_utf8(buffer) {
            Ok(v) => v,
            Err(_) => return Flow::Break(Some(dbus::Error::DataNonUtf8)),
        };

        // Skip unused terminating 0.
        idx = idx.strict_add(1);

        self.level.idx = idx;
        self.cursor.move_step();

        Flow::Continue(self)
    }

    pub fn u16(&mut self, data: &mut u16) -> Flow<Option<dbus::Error>, &mut Self> {
        let mut v = [0; _];
        self.fixed(dbus::Element::U16, &mut v)?;
        *data = self.format.de_u16(v);
        Flow::Continue(self)
    }

    pub fn string(&mut self, data: &mut alloc::string::String) -> Flow<Option<dbus::Error>, &mut Self> {
        self.str32(dbus::Element::String, data)
    }

    pub fn signature(&mut self, data: &mut alloc::string::String) -> Flow<Option<dbus::Error>, &mut Self> {
        self.str8(dbus::Element::Signature, data)
    }

    pub fn array(&mut self) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.element() != Some(dbus::Element::Array) {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }

        let mut level = self.level.clone();
        let align = self.cursor.down().unwrap().dvar_alignment_exp();

        let mut len_u = [0; _];
        Self::align(self.read, &mut level.idx, dbus::Element::U32.dvar_alignment_exp())?;
        Self::read(self.read, &mut level.idx, &mut len_u)?;
        level.meta = self.format.de_u32(len_u) as usize;
        Self::align(self.read, &mut level.idx, align)?;

        core::mem::swap(&mut level, &mut self.level);
        self.stack.push((dbus::Element::Array, level, None));
        self.cursor.move_down();

        Flow::Continue(self)
    }

    pub fn close(&mut self) -> Flow<Option<dbus::Error>, &mut Self> {
        if self.cursor.idx_step().is_some() {
            return Flow::Break(Some(dbus::Error::Mismatch));
        }
        let Some(
            &mut (up_element, ref mut up_level, ref mut up_cursor)
        ) = self.stack.last_mut() else {
            return Flow::Break(Some(dbus::Error::Mismatch));
        };

        match up_element {
            dbus::Element::Variant => {
                // Nothing to finalize.
            },
            dbus::Element::Array => {
                // Nothing to finalize.
            },
            dbus::Element::StructOpen => {
                // Nothing to finalize for structures.
            },
            dbus::Element::DictOpen => {
                // Nothing to finalize for structures.
            },
            _ => core::unreachable!(),
        }

        core::mem::swap(&mut self.level, up_level);
        self.level.idx = up_level.idx;

        if let Some(ref mut v) = up_cursor {
            core::mem::swap(v, &mut self.cursor);
        } else {
            self.cursor.move_up();
        }

        self.cursor.move_step();
        self.stack.pop();

        Flow::Continue(self)
    }
}

#[allow(clippy::octal_escapes)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        {
            let mut buf = vec::Vec::new();
            let mut enc = Enc::new_le(dbus::sig!(b"as"), &mut buf);
            enc.array().continue_value().unwrap()
                .string("foo").continue_value().unwrap()
                .string("bar").continue_value().unwrap()
                .close().continue_value().unwrap()
                .commit().unwrap();

            assert_eq!(buf, b"\
                \x10\0\0\0\
                \x03\0\0\0foo\0\
                \x03\0\0\0bar\0\
            ");
        }

        {
            let mut buf = vec::Vec::new();
            let mut enc = Enc::new_le(dbus::sig!(b"a{sv}"), &mut buf);
            enc.array().continue_value().unwrap()
                .dict().continue_value().unwrap()
                .string("foo").continue_value().unwrap()
                .variant(dbus::sig!(b"s")).continue_value().unwrap()
                .string("bar").continue_value().unwrap()
                .close().continue_value().unwrap()
                .close().continue_value().unwrap()
                .close().continue_value().unwrap()
                .commit().unwrap();

            assert_eq!(buf, b"\
                \x14\0\0\0\
                \0\0\0\0\
                \x03\0\0\0\
                foo\0\
                \x01s\0\0\
                \x03\0\0\0\
                bar\0\
            ");
        }

        {
            let mut buf = vec::Vec::new();
            let mut enc = Enc::new_le(dbus::sig!(b"as"), &mut buf);
            {
                let mut sub0 = enc.enc_with(dbus::sig!(b"as").into()).unwrap();
                sub0.array().continue_value().unwrap();
                {
                    let mut sub1 = sub0.enc().unwrap();
                    sub1.string("000").continue_value().unwrap();
                    sub1.commit().unwrap();
                }
                {
                    let mut sub2 = sub0.enc().unwrap();
                    sub2.string("001").continue_value().unwrap();
                    sub2.commit().unwrap();
                }
                sub0.string("002").continue_value().unwrap()
                    .string("003").continue_value().unwrap();
                sub0.close().continue_value().unwrap();
                sub0.commit().unwrap();
            }
            enc.commit().unwrap();

            assert_eq!(buf, b"\
                \x20\0\0\0\
                \x03\0\0\0000\0\
                \x03\0\0\0001\0\
                \x03\0\0\0002\0\
                \x03\0\0\0003\0\
            ");
        }
    }
}
