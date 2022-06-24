use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::BridgeBuilder;

#[tokio::test(flavor = "multi_thread")]
async fn must_read() {
    let buf = Cursor::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let (fut, mut io) = BridgeBuilder::new(buf).bridge_read().build();
    let agent_handler = tokio::spawn(fut);

    let consumer = tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 4];
        io.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, &[1, 2, 3, 4]);
        let mut buf = vec![];
        io.read_to_end(&mut buf).unwrap();
        assert_eq!(&buf, &[5, 6, 7, 8, 9, 10]);
    });

    consumer.await.unwrap();
    // Ensure channel is closed.
    agent_handler.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn must_read_write_seek() {
    let buf = Cursor::new(vec![]);

    let (fut, mut io) = BridgeBuilder::new(buf)
        .bridge_read()
        .bridge_write()
        .bridge_seek()
        .build();
    let agent_handler = tokio::spawn(fut);

    let consumer = tokio::task::spawn_blocking(move || {
        io.write_all(&[1, 2, 1, 2]).unwrap();
        io.write_all(&[5, 6, 7, 8, 9, 10]).unwrap();
        io.seek(SeekFrom::Start(2)).unwrap();
        io.write_all(&[3, 4]).unwrap();
        io.seek(SeekFrom::Current(-3)).unwrap();

        let mut buf = vec![];
        io.read_to_end(&mut buf).unwrap();
        assert_eq!(&buf, &[2, 3, 4, 5, 6, 7, 8, 9, 10]);
    });

    consumer.await.unwrap();
    // Ensure channel is closed.
    agent_handler.await.unwrap();
}
