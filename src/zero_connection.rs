use crate::message::ZeroMessage;
use crate::async_connection::Connection;
use crate::error::Error;
use crate::async_connection::SharedState;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::address::Address;
use std::io::{Read, Write};

pub struct ZeroConnection {
	pub connection: Connection<ZeroMessage>,
	pub next_req_id: usize,
}

impl ZeroConnection {
	fn req_id(&mut self) -> usize {
		self.next_req_id += 1;
		self.next_req_id-1
	}

	pub fn last_req_id(&self) -> usize {
		self.next_req_id-1
	}

	pub fn from_address(address: Address) -> Result<ZeroConnection, Error> {
		let (reader, writer) = address.get_pair().unwrap();
		ZeroConnection::new(reader, writer)
	}

	// TODO: integrate handshake and return future
	pub fn new(reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>) -> Result<ZeroConnection, Error> {
		let shared_state = SharedState::<ZeroMessage> {
			reader: Arc::new(Mutex::new(reader)),
			writer: Arc::new(Mutex::new(writer)),
			requests: HashMap::new(),
			value: Arc::new(Mutex::new(None)),
			waker: None,
		};
		let conn = Connection {
			shared_state: Arc::new(Mutex::new(shared_state)),
		};
		let conn = ZeroConnection {
			connection: conn,
			next_req_id: 0,
		};

		Ok(conn)
	}

	pub fn recv(&mut self) -> impl Future<Output = Result<ZeroMessage, Error>> {
		self.connection.recv()
	}

	pub fn respond(&mut self, to: usize, body: serde_json::Value) -> Result<(), Error> {
		let message = ZeroMessage::response(to, body);
		self.connection.send(message)
	}

	pub fn request(&mut self, cmd: &str, body: serde_json::Value) -> impl Future<Output = Result<ZeroMessage, Error>> {
		let message = ZeroMessage::request(cmd, self.req_id(), body);
		self.connection.request(message)
	}
}