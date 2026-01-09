//! # Streaming I/O Utilities
//!
//! This module provides utilities to work with data streams. The [`Read`] and
//! [`Write`] abstractions allow consumers and producers to be written
//! independently with requiring knowledge about each other.

use core::mem::MaybeUninit as Uninit;
use core::ops::ControlFlow as Flow;

/// `More` describes the data extents needed to serve a request.
///
/// The main use is for [`Read::map()`] and its derivatives to signal how much
/// more data is needed to serve the request.
///
/// The structure describes the absolute requirements, rather than than the
/// relative delta. That is, a minimum of `16` means the buffer should be of
/// size 16, rather than 16 more bytes are needed on top of whatever the buffer
/// currently is.
#[derive(Clone, Copy, Debug, Hash)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct More {
    /// The minimum number of bytes the buffer must have to serve the request.
    pub min: usize,
    /// An optional maximum number of bytes the request can make use of.
    pub max: Option<usize>,
}

/// `Read` allows buffered reads from a data stream.
///
/// This trait is a connection between protocol implementations and transport
/// layers. That is, it allows writing code that reads structured data from a
/// data stream without knowing the transport layer used to stream the data.
///
/// The trait is similar to [`std::io::Read`] but is designed for buffered
/// streams that perform transport layer operations 
///
/// The actual transport layer operations are not part of this trait, but must
/// be handled separately. This trait is just an abstraction for the data
/// buffer.
pub trait Read {
    /// Advance the stream by the specified number of bytes.
    ///
    /// This will irrevocably discard the specified number of bytes of the
    /// underlying stream of data and advance the position.
    ///
    /// The underlying stream will buffer data until this function is called.
    fn advance(&mut self, len: usize);

    /// Map the data of the stream at the current position.
    ///
    /// This will return a linear memory mapping of the data of the stream at
    /// the current position with at least a length of `min`. The maximum
    /// length is a hint and may be ignored by the implementation. A maximum
    /// of `None` is equivalent to a maximum of `Some(usize::MAX)`.
    ///
    /// This function does not advance the position of the underlying stream.
    /// Repeated calls to this function will operate on the same data. Use
    /// [`Self::advance()`] to advance the position of the stream.
    /// Furthermore, this function does not perform any I/O. This function
    /// merely maps the available data buffers or rearranges the data to ensure
    /// it is available as a linear mapping.
    ///
    /// This function cannot fail, except if insufficient data is available. If
    /// the implementation fails for other reasons, it must be recoverable and
    /// handled out of band.
    ///
    /// If the underlying stream does not have sufficient data buffered, this
    /// will return [`ControlFlow::Break()`](core::ops::ControlFlow::Break)
    /// with information on how much data is needed. It is up to the caller to
    /// pass this information to the stream operators to ensure more data is
    /// made available.
    fn map(&self, min: usize, max_hint: Option<usize>) -> Flow<More, &[u8]>;
}

/// `Write` allows buffered writes to a data stream.
///
/// This trait is a connection between protocol implementations and transport
/// layers. That is, it allows writing code that writes structured data to a
/// data stream without knowing the transport layer used to stream the data.
///
/// The trait is similar to [`std::io::Write`] but is designed for buffered
/// streams that perform transport layer operations 
///
/// The actual transport layer operations are not part of this trait, but must
/// be handled separately. This trait is just an abstraction for the data
/// buffer.
pub trait Write {
    /// Commit the specified number of bytes to the stream.
    ///
    /// This will mark the given number of bytes as ready to be written and
    /// advance the position of the stream. No data is actually written to the
    /// transport layer, but merely marked to be ready. Transport layer
    /// operations must be handled separately.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the first `len` bytes of the stream buffer
    /// have been initialized via [`Self::map()`] or one of its derivatives.
    unsafe fn commit(&mut self, len: usize);

    /// Map the data buffer of the stream for writing.
    ///
    /// This will return a linear memory mapping of the data buffer with at
    /// least a length of `min`. The maximum length is a hint and may be
    /// ignored by the implementation. A maximum of `None` is equivalent to a
    /// maximum of `Some(usize::MAX)`.
    ///
    /// The data buffer can be repeatedly written to by the caller. No data is
    /// written to the stream until [`Self::commit()`] is called. Repeated
    /// calls must return the same mapping, unless the data is committed
    /// between those calls. The mapping might be moved between two calls, but
    /// the content of initialized cells must be retained, except if committed
    /// in between.
    ///
    /// This function cannot fail, except if insufficient buffer space is
    /// available. If the implementation fails for other reasons, it must be
    /// recoverable and handled out of band.
    ///
    /// If the underlying stream does not have sufficient data buffers, this
    /// will return [`ControlFlow::Break()`](core::ops::ControlFlow::Break)
    /// with information on how much space is needed. It is up to the caller to
    /// pass this information to the stream operators to ensure more data is
    /// made available.
    fn map(&mut self, min: usize, max_hint: Option<usize>) -> Flow<More, &mut [Uninit<u8>]>;

    /// Map the initialized data buffer of the stream for writing.
    ///
    /// This works like [`Self::map()`] but returns an initialized reference.
    /// This will always truncate the slice to the requested length.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the first `len` bytes of the stream buffer
    /// have been initialized via [`Self::map()`] or one of its derivatives.
    unsafe fn map_unchecked(&mut self, len: usize) -> Flow<More, &mut [u8]> {
        self.map(len, Some(len)).map_continue(|v| {
            // SAFETY: Propagated to caller.
            unsafe {
                core::mem::transmute::<&mut [Uninit<u8>], &mut [u8]>(
                    &mut v[..len],
                )
            }
        })
    }

    /// Commit data directly to the stream.
    ///
    /// This takes a data buffer, writes it to the stream buffers at the
    /// current position, and then commits the data.
    ///
    /// If insufficient buffer space is available to atomically write the
    /// entire data blob, this will forward the return value from
    /// [`Self::map()`].
    fn write(&mut self, data: &[u8]) -> Flow<More> {
        // SAFETY: `Uninit<T>` is `repr(transparent)` and allows down-casts.
        let data_u = unsafe { core::mem::transmute::<&[u8], &[Uninit<u8>]>(data) };
        let map = self.map(data.len(), Some(data.len()))?;
        map[..data.len()].copy_from_slice(data_u);

        // SAFETY: `data.len()` bytes were copied, so they must be initialized.
        unsafe { self.commit(data.len()) };

        Flow::Continue(())
    }
}

/// Map data of the stream for as long as the predicate indicates.
///
/// This extends [`Read::map()`] by mapping buffered data for as long as the
/// provided predicate returns `true`. The predicate will be called for each
/// byte given the index of the byte relative to the current stream position
/// and its value. Once the predicate returns `false`, the map up until this
/// position is returned.
///
/// The behavior otherwise matches [`Read::map()`].
///
/// `n` is the offset relative to the current streaming position where to
/// start running the predicate. `n` is incremented each time the predicate
/// is run and returned `true`. This avoids calling the predicate multiple
/// times on the same values even if the mapping operation is interrupted
/// with [`ControlFlow::Break()`](core::ops::ControlFlow::Break).
///
/// That is, on success, `n` will reflect the relative index (relative to the
/// current stream position) of the first byte that failed the predicate.
/// However, the returned map is guaranteed to be at least 1 byte bigger and
/// thus always includes this byte. The returned map might be arbitrarily big
/// and the caller must truncate it, if required.
pub fn read_map_while<'this, This, Predicate>(
    this: &'this This,
    n: &mut usize,
    max: Option<usize>,
    mut predicate: Predicate,
) -> Flow<More, &'this [u8]>
where
    This: ?Sized + Read,
    Predicate: FnMut(usize, u8) -> bool,
{
    let max_v = max.unwrap_or(usize::MAX);

    loop {
        let n1 = n.strict_add(1);
        let map = this.map(n1, max)?;
        let map = &map[..core::cmp::min(map.len(), max_v)];
        assert!(map.len() >= n1);
        assert!(map.len() <= max_v);

        while *n < map.len() {
            if !predicate(*n, map[*n]) {
                return Flow::Continue(map);
            }
            *n += 1;
        }

        if *n >= max_v {
            return Flow::Continue(map);
        }
    }
}

impl<'this> dyn Read + 'this {
    /// Map data of the stream for as long as the predicate indicates.
    ///
    /// This is an alias for [`read_map_while()`].
    pub fn map_while<Predicate>(
        &self,
        n: &mut usize,
        max: Option<usize>,
        predicate: Predicate,
    ) -> Flow<More, &[u8]>
    where
        Predicate: FnMut(usize, u8) -> bool,
    {
        read_map_while(self, n, max, predicate)
    }
}

impl<'data> Read for &'data [u8] {
    fn advance(&mut self, len: usize) {
        let v = core::mem::take(self);
        *self = &v[len..];
    }

    fn map(&self, min: usize, max: Option<usize>) -> Flow<More, &[u8]> {
        if min <= self.len() {
            Flow::Continue(self)
        } else {
            Flow::Break(More { min: min, max: max })
        }
    }
}

impl Write for alloc::vec::Vec<u8> {
    unsafe fn commit(&mut self, len: usize) {
        // SAFETY: Propagated to caller.
        unsafe {
            self.set_len(self.len().strict_add(len));
        }
    }

    fn map(&mut self, min: usize, max: Option<usize>) -> Flow<More, &mut [Uninit<u8>]> {
        self.reserve(max.unwrap_or(min));
        Flow::Continue(self.spare_capacity_mut())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // A basic test of the `Write` trait and its helpers, using the trivial
    // `Vec`-based implementation.
    #[test]
    fn write_basic() {
        let mut vec = alloc::vec::Vec::new();

        let v = vec.map(0, None).continue_value();
        assert!(v.is_some());

        // Initialize the vector with increasing values.
        let v = vec.map(128, None).continue_value().unwrap();
        assert!(v.len() >= 128);
        for (i, e) in v.iter_mut().enumerate() {
            e.write(i as u8);
        }

        // Verify that a re-map correctly shows the values.
        let v = unsafe { vec.map_unchecked(16).continue_value().unwrap() };
        assert_eq!(v.len(), 16);
        for (i, e) in v.iter().enumerate() {
            assert_eq!(*e, i as u8);
        }

        // Commit the prefix.
        unsafe { vec.commit(16) };

        // Verify that the prefix was stripped and a re-map shows the tail.
        let v = unsafe { vec.map_unchecked(16).continue_value().unwrap() };
        assert_eq!(v.len(), 16);
        for (i, e) in v.iter().enumerate() {
            assert_eq!(*e, (i + 16) as u8);
        }

        // Discard the previous temporary values and rewrite the next 16 values
        // again starting at 0 and immediately commit it.
        let _ = vec.write(
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        ).continue_value().unwrap();

        // Verify that the final committed data is twice the numbers 0-15.
        assert_eq!(vec.len(), 32);
        for (i, e) in vec.iter().enumerate() {
            assert_eq!(*e, (i % 16) as u8);
        }
    }
}
