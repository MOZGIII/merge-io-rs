//! Merge two separate [`AsyncRead`](futures::io::AsyncRead) and
//! [`AsyncWrite`](futures::io::AsyncWrite) objects into a single I/O stream.
//!
//! # Examples
//!
//! ```
//! # #![feature(async_await)]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # futures::executor::block_on(async {
//! use merge_io::MergeIO;
//! use std::io::Cursor;
//! use futures::{AsyncReadExt, AsyncWriteExt};
//!
//! // Prepare `reader` to read data from...
//! let reader = Cursor::new(vec![1, 2, 3, 4]);
//!
//! // ... and `writer` to read data to.
//! let writer: Vec<u8> = vec![];
//!
//! // Merge `reader` and `writer` into a single I/O stream.
//! let mut stream = MergeIO::new(reader, writer);
//!
//! // Read data from stream.
//! let mut read_buf = Vec::<u8>::with_capacity(1024);
//! stream.read_to_end(&mut read_buf).await?;
//!
//! // We got what was in the `reader`!
//! assert_eq!(&read_buf, &[1, 2, 3, 4]);
//!
//! // Write data to stream.
//! stream.write_all(&[10, 20, 30, 40]).await?;
//!
//! // `writer` now contains what we wrote!
//! assert_eq!(stream.writer(), &[10, 20, 30, 40]);
//!
//! # Ok(())
//! # })
//! # }
//! ```

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

use futures::io::Initializer;
use futures::prelude::*;
use std::io::{IoSlice, IoSliceMut, Result};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Merged I/O, delegates reads and writes to the provided
/// [`AsyncRead`](futures::io::AsyncRead) (`R`) and
/// [`AsyncWrite`](futures::io::AsyncWrite) (`W`).
#[derive(Debug)]
pub struct MergeIO<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    reader: R,
    writer: W,
}

impl<R, W> MergeIO<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// Creates new [`MergeIO`](crate::MergeIO), that reads to `reader` and
    /// writes to `writer`.
    pub fn new(reader: R, writer: W) -> Self {
        MergeIO { reader, writer }
    }

    /// Provides access to `reader`.
    pub fn reader(&self) -> &R {
        &self.reader
    }

    /// Provides access to `writer`.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Provides `mut` access to `reader`.
    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Provides `mut` access to `writer`.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Deconstructs `MergeIO` into the `reader` and `writer`.
    pub fn into_inner(self) -> (R, W) {
        (self.reader, self.writer)
    }
}

impl<R, W> AsyncRead for MergeIO<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        self.reader.initializer()
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        AsyncRead::poll_read(Pin::new(&mut self.get_mut().reader), cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        AsyncRead::poll_read_vectored(Pin::new(&mut self.get_mut().reader), cx, bufs)
    }
}

impl<R, W> AsyncWrite for MergeIO<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        AsyncWrite::poll_write(Pin::new(&mut self.get_mut().writer), cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        AsyncWrite::poll_write_vectored(Pin::new(&mut self.get_mut().writer), cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.get_mut().writer), cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        AsyncWrite::poll_close(Pin::new(&mut self.get_mut().writer), cx)
    }
}
