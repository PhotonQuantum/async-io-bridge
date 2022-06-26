# async-io-bridge

## DEPRECATED

PLEASE USE `tokio_util::io::SyncIoBridge` INSTEAD.

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PhotonQuantum/async-io-bridge/Test?style=flat-square)](https://github.com/PhotonQuantum/async-io-bridge/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/async-io-bridge?style=flat-square)](https://crates.io/crates/async-io-bridge)
[![Documentation](https://img.shields.io/docsrs/async-io-bridge?style=flat-square)](https://docs.rs/async-io-bridge)

A compat wrapper around `std::io::{Read, Write, Seek}` that implements `tokio::io::{AsyncRead, AsyncWrite, AsyncSeek}`.

See [tokio-io-compat](https://github.com/PhotonQuantum/tokio-io-compat) if you want to wrap async io objects to provide sync interfaces.

## Notice

Never consume the wrapped io object in the same thread as its original one, or you will get a deadlock.

## Example
```rust
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use async_io_bridge::BridgeBuilder;

let mut async_io = Cursor::new(vec![]); // some async io object
// Bridge all io traits you need.
let (fut, mut sync_io) = BridgeBuilder::new(async_io)
    .bridge_read()
    .bridge_write()
    .bridge_seek()
    .build();

// Spawn the bridge future to provide data to the sync io wrapper.
let bridge_handler = tokio::task::spawn(fut);
// Do anything you want with the sync io wrapper in a separate blocking thread.
let blocking_handler = tokio::task::spawn_blocking(move || {
    sync_io.write_all(&[0, 1, 2, 3, 4]).unwrap();
    sync_io.seek(SeekFrom::Start(2)).unwrap();

    let mut buffer = [0; 1];
    sync_io.read_exact(&mut buffer).unwrap();
    assert_eq!(&buffer, &[2]);
});

blocking_handler.await.unwrap();
bridge_handler.await.unwrap();
```