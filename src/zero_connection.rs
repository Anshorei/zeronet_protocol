use crate::address::Address;
use crate::async_connection::Connection;
use crate::async_connection::SharedState;
use crate::error::Error;
use crate::message::{Request, Response, ZeroMessage};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct ZeroConnection {
	/// A ZeroNet Protocol connection
	///
	/// The ZeroNet Protocol is specified at
	/// https://zeronet.io/docs/help_zeronet/network_protocol/
	///
	/// # Examples
	/// ```no_run
	/// use std::net::{TcpStream, TcpListener};
	/// use futures::executor::block_on;
	///	use zeronet_protocol::{ZeroConnection, ZeroMessage};
	///
	/// fn handle_connection(stream: TcpStream) {
	///		let mut connection = ZeroConnection::new(Box::new(stream.try_clone().unwrap()), Box::new(stream)).unwrap();
	///		let request = block_on(connection.recv()).unwrap();
	///
	///		let body = "anything serializable".to_string();
	///		block_on(connection.respond(request.req_id, body));
	/// }
	///
	/// fn main() {
	/// 	let listener = TcpListener::bind("127.0.0.1:8001").unwrap();
	///
	/// 	for stream in listener.incoming() {
	/// 		match stream {
	/// 			Ok(stream) => handle_connection(stream),
	/// 			_ => {},
	/// 		}
	/// 	}
	/// }
	/// ```
	pub connection: Connection<ZeroMessage>,
	pub next_req_id: usize,
}

impl ZeroConnection {
	fn req_id(&mut self) -> usize {
		self.next_req_id += 1;
		self.next_req_id - 1
	}

	/// Get the req_id of the last request
	pub fn last_req_id(&self) -> usize {
		self.next_req_id - 1
	}

	pub fn from_address(address: Address) -> Result<ZeroConnection, Error> {
		let (reader, writer) = address.get_pair().unwrap();
		ZeroConnection::new(reader, writer)
	}

	/// Create a new ZeroConnection from a given reader and writer
	pub fn new(
		reader: Box<dyn Read + Send>,
		writer: Box<dyn Write + Send>,
	) -> Result<ZeroConnection, Error> {
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

	pub fn connect(
		address: String,
		port: usize,
	) -> impl Future<Output = Result<ZeroConnection, Error>> {
		let address = Address::IPV4(address, port);
		let mut connection = ZeroConnection::from_address(address).unwrap();

		return async {
			let body = crate::message::templates::Handshake::default();
			let message = ZeroMessage::request("handshake", connection.req_id(), body);
			let result = connection.connection.request(message).await;
			if result.is_ok() {
				return Ok(connection);
			} else {
				return Err(Error::empty());
			}
		};
	}

	pub fn recv(&mut self) -> impl Future<Output = Result<Request, Error>> {
		let result = self.connection.recv();

		return async {
			match result.await {
				Err(err) => Err(err),
				Ok(ZeroMessage::Response(_)) => Err(Error::empty()),
				Ok(ZeroMessage::Request(req)) => Ok(req),
			}
		};
	}

	pub fn respond<T: DeserializeOwned + Serialize>(
		&mut self,
		to: usize,
		body: T,
	) -> impl Future<Output = Result<(), Error>> {
		let message = ZeroMessage::response(to, body);
		self.connection.send(message)
	}

	pub fn request<T: DeserializeOwned + Serialize>(
		&mut self,
		cmd: &str,
		body: T,
	) -> impl Future<Output = Result<Response, Error>> {
		let message = ZeroMessage::request(cmd, self.req_id(), body);
		let result = self.connection.request(message);

		return async {
			match result.await {
				Err(err) => Err(err),
				Ok(ZeroMessage::Response(res)) => Ok(res),
				Ok(ZeroMessage::Request(_)) => Err(Error::empty()),
			}
		};
	}
}
