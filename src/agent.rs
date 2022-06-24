use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::{AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::Sender;

use crate::{Req, Resp};

#[async_trait]
pub trait Agent<T>: Send {
    async fn invoke(req: Req, tx: &Sender<Resp>, io: &mut T)
    where
        T: 'async_trait;
}

pub struct NoopAgent;

#[async_trait]
impl<T> Agent<T> for NoopAgent
where
    T: Send,
{
    async fn invoke(_: Req, _: &Sender<Resp>, _: &mut T)
    where
        T: 'async_trait,
    {
    }
}

pub struct ReadAgent<T, A>(PhantomData<(T, A)>);

impl<T, A> Default for ReadAgent<T, A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[async_trait]
impl<T, A> Agent<T> for ReadAgent<T, A>
where
    T: AsyncRead + Unpin + Send,
    A: Agent<T>,
{
    async fn invoke(req: Req, tx: &Sender<Resp>, io: &mut T)
    where
        T: 'async_trait,
    {
        match req {
            Req::Read(buf) => {
                // SAFETY:
                // 1. No one besides us has this pointer.
                // 2. The buffer is alive because the caller thread is blocked since the buffer being sent to this thread.
                let resp = unsafe { io.read(buf.as_mut()).await };
                tx.send(Resp::Read(resp))
                    .await
                    .expect("Failed to send response to sync code");
            }
            _ => A::invoke(req, tx, io).await,
        }
    }
}

pub struct WriteAgent<T, A>(PhantomData<(T, A)>);

impl<T, A> Default for WriteAgent<T, A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[async_trait]
impl<T, A> Agent<T> for WriteAgent<T, A>
where
    T: AsyncWrite + Unpin + Send,
    A: Agent<T>,
{
    async fn invoke(req: Req, tx: &Sender<Resp>, io: &mut T)
    where
        T: 'async_trait,
    {
        match req {
            Req::Write(buf) => {
                // SAFETY:
                // 1. No one besides us has this pointer.
                // 2. The buffer is alive because the caller thread is blocked since the buffer being sent to this thread.
                let resp = unsafe { io.write(buf.as_ref()).await };
                tx.send(Resp::Write(resp))
                    .await
                    .expect("Failed to send response to sync code");
            }
            Req::Flush => {
                let resp = io.flush().await;
                tx.send(Resp::Flush(resp))
                    .await
                    .expect("Failed to send response to sync code");
            }
            _ => A::invoke(req, tx, io).await,
        }
    }
}

pub struct SeekAgent<T, A>(PhantomData<(T, A)>);

impl<T, A> Default for SeekAgent<T, A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[async_trait]
impl<T, A> Agent<T> for SeekAgent<T, A>
where
    T: AsyncSeek + Unpin + Send,
    A: Agent<T>,
{
    async fn invoke(req: Req, tx: &Sender<Resp>, io: &mut T)
    where
        T: 'async_trait,
    {
        match req {
            Req::Seek(pos) => {
                let resp = io.seek(pos).await;
                tx.send(Resp::Seek(resp))
                    .await
                    .expect("Failed to send response to sync code");
            }
            _ => A::invoke(req, tx, io).await,
        }
    }
}
