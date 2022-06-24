use std::future::Future;
use std::marker::PhantomData;

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};
use tokio::sync::mpsc;

use crate::agent::{Agent, NoopAgent, ReadAgent, SeekAgent, WriteAgent};
use crate::utils::Fls;
use crate::{IOBridge, Req, Resp, Tru};

/// Builder for [`IOBridge`](crate::IOBridge) wrapper object and corresponding bridge future.
pub struct BridgeBuilder<A, T, R, W, S> {
    io: T,
    _marker: PhantomData<(A, R, W, S)>,
}

impl<T> BridgeBuilder<NoopAgent, T, Fls, Fls, Fls> {
    /// Create a new builder with the given async io object.
    pub const fn new(io: T) -> Self {
        Self {
            io,
            _marker: PhantomData,
        }
    }
}

impl<A, T, W, S> BridgeBuilder<A, T, Fls, W, S>
where
    T: AsyncRead + Unpin + Send,
{
    /// Enable [`Read`](std::io::Read) trait bridging.
    pub fn bridge_read(self) -> BridgeBuilder<ReadAgent<T, A>, T, Tru, W, S> {
        BridgeBuilder {
            io: self.io,
            _marker: Default::default(),
        }
    }
}

impl<A, T, R, S> BridgeBuilder<A, T, R, Fls, S>
where
    T: AsyncWrite + Unpin + Send,
{
    /// Enable [`Write`](std::io::Write) trait bridging.
    pub fn bridge_write(self) -> BridgeBuilder<WriteAgent<T, A>, T, R, Tru, S> {
        BridgeBuilder {
            io: self.io,
            _marker: Default::default(),
        }
    }
}

impl<A, T, R, W> BridgeBuilder<A, T, R, W, Fls>
where
    T: AsyncSeek + Unpin + Send,
{
    /// Enable [`Seek`](std::io::Seek) trait bridging.
    pub fn bridge_seek(self) -> BridgeBuilder<SeekAgent<T, A>, T, R, W, Tru> {
        BridgeBuilder {
            io: self.io,
            _marker: Default::default(),
        }
    }
}

impl<A, T, R, W, S> BridgeBuilder<A, T, R, W, S>
where
    Self: Send,
    A: Agent<T>,
    T: Send,
{
    /// Build the bridge future and sync io object.
    ///
    /// # Notice
    /// Please spawn the bridge future before using the sync io object.
    ///
    /// The sync io object can only be used in a blocking thread.
    pub fn build(mut self) -> (impl Future<Output = ()> + Send, IOBridge<R, W, S>) {
        let (req_tx, mut req_rx) = mpsc::channel::<Req>(10);
        let (resp_tx, resp_rx) = mpsc::channel::<Resp>(10);

        (
            async move {
                while let Some(req) = req_rx.recv().await {
                    A::invoke(req, &resp_tx, &mut self.io).await;
                }
            },
            IOBridge {
                rx: resp_rx,
                tx: req_tx,
                _marker: Default::default(),
            },
        )
    }
}
