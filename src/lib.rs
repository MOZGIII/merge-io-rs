#![warn(missing_debug_implementations, rust_2018_idioms)]

use futures::io::Initializer;
use futures::prelude::*;
use std::io::{IoSlice, IoSliceMut, Result};
use std::pin::Pin;
use std::task::{Context, Poll};

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
    pub fn new(reader: R, writer: W) -> Self {
        MergeIO { reader, writer }
    }

    pub fn reader(&self) -> &R {
        &self.reader
    }

    pub fn writer(&self) -> &W {
        &self.writer
    }

    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

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
