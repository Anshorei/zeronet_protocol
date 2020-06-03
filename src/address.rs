use crate::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};

pub enum Address {
	IPV4(String, usize),
	IPV6(String, usize),
	Onion(String),
	I2P(String),
	Loki(String),
	Mock(Receiver<String>, Sender<String>),
}

impl Address {
	pub fn get_pair(&self) -> Result<(Box<dyn Read + Send>, Box<dyn Write + Send>), Error> {
		match self {
			Address::IPV4(address, port) => {
				let socket = TcpStream::connect(format!("{}:{}", address, port))?;
				return Ok((Box::new(socket.try_clone()?), Box::new(socket)));
			}
			_ => Err(Error::empty()),
		}
	}
}
