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
	///	use zeronet_protocol::{ZeroConnection, ZeroMessage, Address};
	///
	/// fn handle_connection(stream: TcpStream) {
	/// 	let address = Address::from(stream.peer_addr().unwrap());
	///		let mut connection = ZeroConnection::new(address, Box::new(stream.try_clone().unwrap()), Box::new(stream)).unwrap();
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
	/// 		if let Ok(stream) = stream {
	/// 			handle_connection(stream)
	/// 		}
	/// 	}
	/// }
	/// ```
	pub connection: Connection<ZeroMessage>,
	pub next_req_id: usize,
	pub address: Address,
}

impl ZeroConnection {
	/// Creates a new ZeroConnection from a given reader and writer
	pub fn new(
		address: Address,
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
			address,
		};

		Ok(conn)
	}

	/// Creates a new ZeroConnection from a given address
	pub fn from_address(address: Address) -> Result<ZeroConnection, Error> {
		let (reader, writer) = address.get_pair().unwrap();
		ZeroConnection::new(address, reader, writer)
	}

	/// Connect to an ip and port and perform the handshake,
	/// then return the ZeroConnection.
	pub fn connect(address: String) -> impl Future<Output = Result<ZeroConnection, Error>> {
		return async {
			let address = Address::parse(address)?;
			let mut connection = ZeroConnection::from_address(address).unwrap();

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

	/// Returns a future that will read from the internal reader
	/// and attempt to decode valid ZeroMessages.
	/// The future returns the first Request that gets decoded.
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

	/// Respond to a request.
	/// The `body` variable is flattened into the ZeroMessage,
	/// therefore it should be an object, a map or a pair.
	pub fn respond<T: DeserializeOwned + Serialize>(
		&mut self,
		to: usize,
		body: T,
	) -> impl Future<Output = Result<(), Error>> {
		let message = ZeroMessage::response(to, body);
		self.connection.send(message)
	}

	/// Returns a future that will send a request with
	/// a new `req_id` and then read from internal reader
	/// and attempt to decode valid ZeroMessages.
	/// The future returns the first Response that
	/// has the corresponding `to` field.
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

	/// Get the req_id of the last request
	pub fn last_req_id(&self) -> usize {
		self.next_req_id - 1
	}

	fn req_id(&mut self) -> usize {
		self.next_req_id += 1;
		self.next_req_id - 1
	}
}
