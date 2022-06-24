//! # async-io-bridge
//!
//! A compat wrapper around `std::io::{Read, Write, Seek}` that implements `tokio::io::{AsyncRead, AsyncWrite, AsyncSeek}`.
//!
//! See [tokio-io-compat](https://github.com/PhotonQuantum/tokio-io-compat) if you want to wrap async io objects to provide sync interfaces.
//!
//! ## Notice
//!
//! Never consume the wrapped io object in the same thread as its original one, or you will get a deadlock.
//!
//! ## Example
//! ```rust
//! use std::io::{Cursor, Read, Seek, SeekFrom, Write};
//! use async_io_bridge::BridgeBuilder;
//!
//! # #[tokio::test]
//! # async fn doc_test() {
//! let mut async_io = Cursor::new(vec![]); // some async io object
//! // Bridge all io traits you need.
//! let (fut, mut sync_io) = BridgeBuilder::new(async_io)
//!     .bridge_read()
//!     .bridge_write()
//!     .bridge_seek()
//!     .build();
//!
//! // Spawn the bridge future to provide data to the sync io wrapper.
//! let bridge_handler = tokio::task::spawn(fut);
//! // Do anything you want with the sync io wrapper in a separate blocking thread.
//! let blocking_handler = tokio::task::spawn_blocking(move || {
//!     sync_io.write_all(&[0, 1, 2, 3, 4]).unwrap();
//!     sync_io.seek(SeekFrom::Start(2)).unwrap();
//!
//!     let mut buffer = [0; 1];
//!     sync_io.read_exact(&mut buffer).unwrap();
//!     assert_eq!(&buffer, &[2]);
//! });
//!
//! blocking_handler.await.unwrap();
//! bridge_handler.await.unwrap();
//! # }
//! ```
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::ptr::NonNull;

use tokio::sync::mpsc::{Receiver, Sender};

pub use builder::BridgeBuilder;

use crate::utils::{Carrier, Tru};

mod agent;
mod builder;
#[cfg(test)]
mod tests;
mod utils;

#[doc(hidden)]
pub enum Req {
    Read(Carrier<[u8]>),
    Write(Carrier<[u8]>),
    Flush,
    Seek(SeekFrom),
}

impl Debug for Req {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Req::Read(_) => write!(f, "Read"),
            Req::Write(_) => write!(f, "Write"),
            Self::Flush => write!(f, "Flush"),
            Self::Seek(_) => write!(f, "Seek"),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Resp {
    Read(io::Result<usize>),
    Write(io::Result<usize>),
    Flush(io::Result<()>),
    Seek(io::Result<u64>),
}

/// The wrapped io object.
pub struct IOBridge<R, W, S> {
    rx: Receiver<Resp>,
    tx: Sender<Req>,
    _marker: PhantomData<(R, W, S)>,
}

impl<W, S> Read for IOBridge<Tru, W, S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.tx
            .try_send(Req::Read(unsafe { Carrier::new(NonNull::from(buf)) }))
            .expect("Failed to send read request");
        if let Resp::Read(res) = self
            .rx
            .blocking_recv()
            .expect("Async side closed unexpectedly")
        {
            res
        } else {
            panic!("Invariant: unexpected op response")
        }
    }
}

impl<R, S> Write for IOBridge<R, Tru, S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx
            .try_send(Req::Write(unsafe { Carrier::new(NonNull::from(buf)) }))
            .expect("Failed to send write request");
        if let Resp::Write(res) = self
            .rx
            .blocking_recv()
            .expect("Async side closed unexpectedly")
        {
            res
        } else {
            panic!("Invariant: unexpected op response")
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tx
            .try_send(Req::Flush)
            .expect("Failed to send flush request");
        if let Resp::Flush(res) = self
            .rx
            .blocking_recv()
            .expect("Async side closed unexpectedly")
        {
            res
        } else {
            panic!("Invariant: unexpected op response")
        }
    }
}

impl<R, W> Seek for IOBridge<R, W, Tru> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.tx
            .try_send(Req::Seek(pos))
            .expect("Failed to send seek request");
        if let Resp::Seek(res) = self
            .rx
            .blocking_recv()
            .expect("Async side closed unexpectedly")
        {
            res
        } else {
            panic!("Invariant: unexpected op response")
        }
    }
}
