use futures::future::BoxFuture;
use std::cmp;
use std::convert::TryInto;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use virtual_mio::InlineWaker;

use crate::codefs::client::CodeFSClient;
use virtual_fs::{AsyncRead, AsyncSeek, AsyncWrite, FileSystem, Metadata, VirtualFile};

#[derive(Debug)]
pub struct CodeFSVirtualFile {
    fs: CodeFSClient,

    metadata: Metadata,

    path: PathBuf,

    buffer: Vec<u8>,

    cursor: usize,
}

impl CodeFSVirtualFile {
    pub fn new(fs: CodeFSClient, metadata: Metadata, path: PathBuf, buffer: Vec<u8>) -> Self {
        Self {
            fs,
            metadata,
            buffer,
            path,
            cursor: 0usize,
        }
    }
}

const DEFAULT_BUF_SIZE_HINT: usize = 8 * 1024;

impl CodeFSVirtualFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let max_to_read = cmp::min(self.buffer.len() - self.cursor, buf.len());
        let data_to_copy = &self.buffer[self.cursor..][..max_to_read];

        // SAFETY: `buf[..max_to_read]` and `data_to_copy` have the same size, due to
        // how `max_to_read` is computed.
        buf[..max_to_read].copy_from_slice(data_to_copy);

        self.cursor += max_to_read;

        Ok(max_to_read)
    }
}

impl VirtualFile for CodeFSVirtualFile {
    fn last_accessed(&self) -> u64 {
        self.metadata.accessed
    }

    fn last_modified(&self) -> u64 {
        self.metadata.modified
    }

    fn created_time(&self) -> u64 {
        self.metadata.created
    }

    fn size(&self) -> u64 {
        self.metadata.len
    }

    fn set_len(&mut self, new_size: u64) -> virtual_fs::Result<()> {
        self.buffer.resize(new_size.try_into().unwrap(), 0);
        Ok(())
    }

    fn unlink(&mut self) -> BoxFuture<'static, virtual_fs::Result<()>> {
        self.fs.remove_file(&self.path);
        Box::pin(async { Ok(()) })
    }

    fn poll_read_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(0)) // TODO
    }

    fn poll_write_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(DEFAULT_BUF_SIZE_HINT)) // TODO
    }
}

impl AsyncWrite for CodeFSVirtualFile {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.cursor {
            // The cursor is at the end of the buffer: happy path!
            position if position == self.buffer.len() => {
                self.buffer.extend_from_slice(buf);
            }

            // The cursor is at the beginning of the buffer (and the
            // buffer is not empty, otherwise it would have been
            // caught by the previous arm): almost a happy path!
            0 => {
                let mut new_buffer = Vec::with_capacity(self.buffer.len() + buf.len());
                new_buffer.extend_from_slice(buf);
                new_buffer.append(&mut self.buffer);

                self.buffer = new_buffer;
            }

            // The cursor is somewhere in the buffer: not the happy path.
            position => {
                self.buffer.reserve_exact(buf.len());

                let mut remainder = self.buffer.split_off(position);
                self.buffer.extend_from_slice(buf);
                self.buffer.append(&mut remainder);
            }
        }

        self.cursor += buf.len();

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        InlineWaker::block_on(self.fs.write_all(&self.path, &self.buffer));
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.poll_flush(cx)
    }
}

impl AsyncRead for CodeFSVirtualFile {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let read = unsafe { self.read(std::mem::transmute(buf.unfilled_mut())) };
        if let Ok(read) = &read {
            unsafe { buf.assume_init(*read) };
            buf.advance(*read);
        }
        Poll::Ready(read.map(|_| ()))
    }
}

impl AsyncSeek for CodeFSVirtualFile {
    fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
        let to_err = |_| io::ErrorKind::InvalidInput;

        // Calculate the next cursor.
        let next_cursor: i64 = match position {
            // Calculate from the beginning, so `0 + offset`.
            io::SeekFrom::Start(offset) => offset.try_into().map_err(to_err)?,

            // Calculate from the end, so `buffer.len() + offset`.
            io::SeekFrom::End(offset) => {
                TryInto::<i64>::try_into(self.buffer.len()).map_err(to_err)? + offset
            }

            // Calculate from the current cursor, so `cursor + offset`.
            io::SeekFrom::Current(offset) => {
                TryInto::<i64>::try_into(self.cursor).map_err(to_err)? + offset
            }
        };

        // It's an error to seek before byte 0.
        if next_cursor < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seeking before the byte 0",
            ));
        }

        // In this implementation, it's an error to seek beyond the
        // end of the buffer.
        self.cursor = cmp::min(self.buffer.len(), next_cursor.try_into().map_err(to_err)?);

        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        let to_err = |_| io::ErrorKind::InvalidInput;

        Poll::Ready(Ok(self.cursor.try_into().map_err(to_err)?))
    }
}
