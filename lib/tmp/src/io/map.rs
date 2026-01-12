//! Mapped I/O Utilities
//!
//! This module provides utilities to work with memory-mapped data. The
//! [`Read`] and [`Write`] abstractions allows consumers and producers to
//! be written independently without requiring knowledge about each other.

use core::mem::MaybeUninit as Uninit;
use core::ops::ControlFlow as Flow;

#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// The offset calculation exceeds the limits of the implementation.
    Overflow,
    /// The data limits were exceeded (for reads it indicates an access past
    /// the end, for writes it indicates a limit of available or reserved
    /// memory).
    Exceeded,
}

/// `Read` allows chunked access to logically linear data.
pub trait Read {
    fn map(&self, idx: usize, len: usize) -> Flow<Option<Error>, &[u8]>;

    fn read_uninit(
        &self,
        idx: &mut usize,
        mut data: &mut [Uninit<u8>],
    ) -> Flow<Option<Error>> {
        while data.len() > 0 {
            let map = self.map(*idx, data.len())?;
            assert!(map.len() > 0);
            let n = core::cmp::min(map.len(), data.len());

            {
                // SAFETY: `Uninit<T>` is `repr(transparent)` and has no
                //     additional invariants on its own if read-only.
                let map_u = unsafe {
                    core::mem::transmute::<&[u8], &[Uninit<u8>]>(map)
                };
                data[..n].copy_from_slice(&map_u[..n]);
            }

            *idx += n;
            data = &mut data[n..];
        }
        Flow::Continue(())
    }

    fn read(
        &self,
        idx: &mut usize,
        data: &mut [u8],
    ) -> Flow<Option<Error>> {
        // SAFETY: `Uninit<T>` is `repr(transparent)` and `read_uninit()` will
        //     (re-)initialize the entire array properly.
        let data_u = unsafe {
            core::mem::transmute::<&mut [u8], &mut [Uninit<u8>]>(data)
        };
        self.read_uninit(idx, data_u)
    }
}

/// `Write` allows chunked mutable access to logically linear data.
pub trait Write {
    /// Commit data
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the first `len` bytes of the map have been
    /// initialized via [`Self::map()`] or one of its derivatives.
    unsafe fn commit(&mut self, len: usize);

    fn map(&mut self, idx: usize, len: usize) -> Flow<Option<Error>, &mut [Uninit<u8>]>;

    fn write<'data>(
        &mut self,
        idx: &mut usize,
        data: &'data [u8],
    ) -> Flow<Option<Error>> {
        // SAFETY: `Uninit<T>` is `repr(transparent)` and allows down-casts.
        let mut data_u = unsafe {
            core::mem::transmute::<&'data [u8], &'data [Uninit<u8>]>(data)
        };

        while data_u.len() > 0 {
            let map = self.map(*idx, data_u.len())?;
            assert!(map.len() > 0);
            let n = core::cmp::min(map.len(), data_u.len());
            map[..n].copy_from_slice(&data_u[..n]);
            *idx += n;
            data_u = &data_u[n..];
        }
        Flow::Continue(())
    }

    fn write_iter(
        &mut self,
        idx: &mut usize,
        data: &mut dyn ExactSizeIterator<Item = u8>,
    ) -> Flow<Option<Error>> {
        loop {
            // We require `ExactSizeIterator`, since iterator hints are not
            // reliable bounds. We thus avoid aggressive overallocation if the
            // `Self`-implementation has no upper bounds (which it is
            // definitely not required to).
            let mut n = data.size_hint().0;
            if n == 0 {
                assert!(data.next().is_none());
                return Flow::Continue(());
            }

            let map = self.map(*idx, n)?;
            assert!(map.len() > 0);
            n = core::cmp::min(n, map.len());

            while n > 0 {
                map[map.len().strict_sub(n)].write(data.next().unwrap());
                *idx += 1;
                n -= 1;
            }
        }
    }

    fn fill(
        &mut self,
        idx: &mut usize,
        mut len: usize,
        data: u8,
    ) -> Flow<Option<Error>> {
        let data_u = Uninit::new(data);
        while len > 0 {
            let map = self.map(*idx, len)?;
            assert!(map.len() > 0);
            let n = core::cmp::min(map.len(), len);
            map[..n].fill(data_u);
            *idx += n;
            len -= n;
        }
        Flow::Continue(())
    }

    fn zero(&mut self, idx: &mut usize, len: usize) -> Flow<Option<Error>> {
        self.fill(idx, len, 0)
    }

    fn align_exp2(&mut self, idx: &mut usize, exp: u8) -> Flow<Option<Error>> {
        match idx.checked_next_multiple_of((1 << exp) as usize) {
            None => Flow::Break(Some(Error::Overflow)),
            Some(v) => self.zero(idx, v.strict_sub(*idx)),
        }
    }
}

impl Read for [u8] {
    fn map(&self, idx: usize, len: usize) -> Flow<Option<Error>, &[u8]> {
        let Some(end) = idx.checked_add(len) else {
            return Flow::Break(Some(Error::Overflow));
        };
        if end > self.len() {
            return Flow::Break(Some(Error::Exceeded));
        }

        Flow::Continue(&self[idx..end])
    }
}

impl Write for alloc::vec::Vec<u8> {
    unsafe fn commit(&mut self, len: usize) {
        // SAFETY: Propagated to caller.
        unsafe {
            self.set_len(self.len().strict_add(len));
        }
    }

    fn map(
        &mut self,
        idx: usize,
        len: usize,
    ) -> Flow<Option<Error>, &mut [Uninit<u8>]> {
        let Some(end) = idx.checked_add(len) else {
            return Flow::Break(Some(Error::Overflow));
        };
        if end > self.len() {
            self.reserve(end - self.len());
        }

        let ptr: *mut u8 = self.as_mut_ptr();
        let ptr_u: *mut Uninit<u8> = ptr as _;
        let cap: usize = self.capacity();
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr_u, cap) };

        Flow::Continue(&mut slice[idx..end])
    }
}
