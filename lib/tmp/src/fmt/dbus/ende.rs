//! # D-Bus Encoders and Decoders

use core::ops::ControlFlow as Flow;

use crate::fmt::dbus;
use crate::io;

#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Format {
    DVarBe,
    DVarLe,
    Json,
}

pub enum Enc<'sig, 'write> {
    DVar(dbus::dvar::Enc<'sig, 'write>),
}

pub enum Dec<'sig, 'write> {
    DVar(dbus::dvar::Dec<'sig, 'write>),
}

impl<'sig, 'write> Enc<'sig, 'write> {
    pub fn new(
        sig: &'sig dbus::Sig,
        format: Format,
        write: &'write mut dyn io::map::Write,
    ) -> Self {
        match format {
            Format::DVarBe => Self::DVar(
                dbus::dvar::Enc::new_be(sig, write),
            ),
            Format::DVarLe => Self::DVar(
                dbus::dvar::Enc::new_le(sig, write),
            ),
            _ => core::unreachable!(),
        }
    }

    pub fn inline(
        &mut self,
        sig: &dbus::Sig,
        dec: &mut Dec,
    ) -> Flow<Option<dbus::Error>> {
        let mut cur = sig.cursor();

        loop {
            if dec.more() {
                match cur.element().unwrap() {
                    _ => core::unreachable!(),
                }
            } else if let Some(idx) = cur.idx_up() {
                dec.close()?;
                cur.move_to(idx);
            } else {
                break;
            }
        }

        Flow::Continue(())
    }
}

impl<'sig, 'read> Dec<'sig, 'read> {
    pub fn new(
        sig: &'sig dbus::Sig,
        format: Format,
        read: &'read mut dyn io::map::Read,
    ) -> Self {
        match format {
            Format::DVarBe => Self::DVar(
                dbus::dvar::Dec::new_be(sig, read),
            ),
            Format::DVarLe => Self::DVar(
                dbus::dvar::Dec::new_le(sig, read),
            ),
            _ => core::unreachable!(),
        }
    }

    pub fn more(&self) -> bool {
        match *self {
            Self::DVar(ref v) => v.more(),
        }
    }

    pub fn close(&mut self) -> Flow<Option<dbus::Error>> {
        match *self {
            Self::DVar(ref mut v) => v.close()?,
        };

        Flow::Continue(())
    }
}
