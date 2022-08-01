use crate::ZeroConnection;
use decentnet_protocol::{
    message::{RequestType, ZeroMessage},
    templates::Ping,
};
use futures::executor::block_on;
use std::{
    io::{Error, ErrorKind, Read, Result, Write},
    sync::mpsc::{channel, Receiver, Sender},
};

struct ChannelWriter {
    tx: Sender<Vec<u8>>,
    buffer: Option<Vec<u8>>,
}

impl ChannelWriter {
    fn new(tx: Sender<Vec<u8>>) -> ChannelWriter {
        ChannelWriter { tx, buffer: None }
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut buffer = match self.buffer.take() {
            Some(buffer) => buffer,
            None => vec![],
        };
        buffer.append(&mut buf.to_vec());
        self.buffer = Some(buffer);

        // TODO: rmp-serde does not flush the write
        // remove this once this is corrected
        self.flush()?;

        return Ok(buf.len());
    }

    fn flush(&mut self) -> Result<()> {
        if let Some(buffer) = self.buffer.take() {
            self.tx
                .send(buffer)
                .map_err(|_| Error::new(ErrorKind::NotConnected, "Could not send on channel"))?;
        }
        Ok(())
    }
}

struct ChannelReader {
    rx: Receiver<Vec<u8>>,
    buffer: Option<Vec<u8>>,
}

impl ChannelReader {
    fn new(rx: Receiver<Vec<u8>>) -> ChannelReader {
        ChannelReader { rx, buffer: None }
    }
}

impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut buffer = match self.buffer.take() {
            Some(buffer) => buffer,
            None => vec![],
        };
        while let Ok(mut res) = self.rx.try_recv() {
            buffer.append(&mut res);
        }
        if buffer.len() == 0 {
            return Err(Error::from(ErrorKind::Interrupted));
        }
        let length = std::cmp::min(buf.len(), buffer.len());
        let mut iterator = buffer.into_iter();
        for i in 0..length {
            if let Some(byte) = iterator.next() {
                buf[i] = byte;
            }
        }
        self.buffer = Some(iterator.collect());
        Ok(length)
    }
}

fn create_pair() -> (ZeroConnection, ZeroConnection) {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let conn1 = ZeroConnection::new(
        Box::new(ChannelReader::new(rx2)),
        Box::new(ChannelWriter::new(tx1)),
    );
    let conn2 = ZeroConnection::new(
        Box::new(ChannelReader::new(rx1)),
        Box::new(ChannelWriter::new(tx2)),
    );
    (conn1.unwrap(), conn2.unwrap())
}

#[test]
fn test_connection() {
    let (mut server, mut client) = create_pair();
    std::thread::spawn(move || {
        block_on(client.request("ping", RequestType::Ping(Ping()))).unwrap();
    });
    let request = block_on(server.recv());
    assert!(request.is_ok());
}

#[test]
fn multiple_receivers() {
    let (mut server1, mut client) = create_pair();
    let mut server2 = server1.clone();

    std::thread::spawn(move || {
        block_on(client.connection.send(
            ZeroMessage::request("ping", 0, RequestType::Ping(Ping())),
            None,
        ))
        .unwrap();
        block_on(client.connection.send(
            ZeroMessage::request("ping", 1, RequestType::Ping(Ping())),
            None,
        ))
        .unwrap();
        block_on(client.connection.send(
            ZeroMessage::request("ping", 2, RequestType::Ping(Ping())),
            None,
        ))
        .unwrap();
        block_on(client.connection.send(
            ZeroMessage::request("ping", 3, RequestType::Ping(Ping())),
            None,
        ))
        .unwrap();
    });
    std::thread::spawn(move || {
        block_on(server1.recv()).ok().unwrap();
        block_on(server1.recv()).ok().unwrap();
    });
    block_on(server2.recv()).ok().unwrap();
    let result = block_on(server2.recv());
    assert!(result.is_ok());
}

#[test]
fn multiple_clients() {
    let (mut server, mut client1) = create_pair();
    let client2 = client1.clone();

    std::thread::spawn(move || {
        block_on(client1.request("ping", RequestType::Ping(Ping()))).unwrap();
    });
    let result = block_on(server.recv()).ok().unwrap();
    assert!(result.req_id == client2.last_req_id());
}
