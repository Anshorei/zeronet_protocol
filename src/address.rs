use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write, Bytes};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use base64::{encode, decode};
use koibumi_base32 as base32;
use std::convert::TryInto;

#[derive(Debug)]
pub struct ParseError {
	text: String,
}

impl ParseError {
	fn text(text: &str) -> ParseError {
		ParseError {
			text: text.to_string(),
		}
	}
}

impl From<std::num::ParseIntError> for ParseError {
	fn from(err: std::num::ParseIntError) -> ParseError {
		ParseError::text(&format!("Error parsing int: {:?}", err))
	}
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Address {
	IPV4([u8; 4], u16),
	IPV6([u8; 16], u16),
	OnionV2(String, u16),
	OnionV3(String, u16),
	I2PB32(String, u16),
	Loki(String, u16),
}

impl From<SocketAddr> for Address {
	fn from(address: SocketAddr) -> Address {
		match address {
			SocketAddr::V4(ip) => Address::IPV4(ip.ip().octets(), ip.port()),
			SocketAddr::V6(ip) => Address::IPV6(ip.ip().octets(), ip.port()),
		}
	}
}

// impl From<T: ToSocketAddrs>()
// pub fn from<T: ToSocketAddrs>(address: T) -> Address {

// }

impl Address {
	/// Create an address by parsing a string
	/// ```
	/// use zeronet_protocol::Address;
	///
	/// let address = Address::parse("127.0.0.1:8001").unwrap();
	/// assert!(address.is_clearnet());
	/// ```
	pub fn parse<S: Into<String>>(address: S) -> Result<Address, ParseError> {
		let address: String = address.into();
		let parts: Vec<&str> = address.split(":").collect();
		if let Some(address) = parts[0].strip_suffix(".onion") {
			// TODO: handle onion v3 addresses
			// TODO: hash address
			if let Some(port) = parts.get(1) {
				let port = port.to_string().parse::<u16>()?;
				return Ok(Address::OnionV2(address.to_string(), port));
			}
		} else if let Some(address) = parts[0].strip_suffix(".i2p") {
			if let Some(port) = parts.get(1) {
				let port = port.to_string().parse::<u16>()?;
				return Ok(Address::I2PB32(address.to_string(), port));
			}
		}
		let parts: Vec<&str> = address.split(":").collect();
		if parts.len() > 2 {
			// TODO: Implement IPV6 parsing
		} else if let Some(address) = parts.first() {
			let bytes: Vec<Result<u8, _>> = address
				.to_string()
				.split(".")
				.map(|byte| byte.to_string().parse::<u8>())
				.collect();
			let mut address = [0u8; 4];
			if bytes.len() != 4 {
				return Err(ParseError::text("Wrong length for IPV4 address"));
			}
			for (i, byte) in bytes.into_iter().enumerate() {
				address[i] = byte?
			}
			if let Some(port) = parts.get(1) {
				let port = port.to_string().parse::<u16>()?;
				return Ok(Address::IPV4(address, port));
			}
		}

		Err(ParseError::text("Unrecognized address format"))
	}

	/// unpack
	/// ```
	/// use zeronet_protocol::Address;
	///
	/// let bytes = vec![127, 0, 0, 1, 225, 16];
	/// let address = Address::unpack(&bytes).unwrap();
	/// assert_eq!(address.to_string(), "127.0.0.1:4321".to_string());
	/// ```
	/// TODO: test unpack IPV6 and OnionV2
	pub fn unpack(bytes: &Vec<u8>) -> Result<Address, Error> {
		match bytes.len() {
			6 => {
				let mut array = [0u8; 4];
				array.copy_from_slice(&bytes[..4]);
				Ok(Address::IPV4(array, u16::from_le_bytes(bytes[4..6].try_into().unwrap())))
			},
			18 => {
				let mut array = [0u8; 16];
				array.copy_from_slice(&bytes[..16]);
				Ok(Address::IPV6(array, u16::from_le_bytes(bytes[16..18].try_into().unwrap())))
			}
			12 => {
				let port = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
				let mut array = [0u8; 10];
				array.copy_from_slice(&bytes[..10]);
				let address = base32::encode(&array)?;
				Ok(Address::OnionV2(address, port))
			},
			// 42 => // TODO: Onion V3
			_ => Err(Error::empty()),
		}
	}

	/// pack
	/// ```
	/// use zeronet_protocol::Address;
	///
	/// let address = Address::parse("127.0.0.1:4321").unwrap();
	/// let packed = address.pack();
	///
	/// assert_eq!(packed, [127, 0, 0, 1, 225, 16]);
	///
	/// let address = Address::parse("ytcnzluhaxidtbf4.onion:4321").unwrap();
	/// let packed = address.pack();
	/// let unpacked = Address::unpack(&packed).unwrap();
	///
	/// assert_eq!(packed, [196, 196, 220, 174, 135, 5, 208, 57, 132, 188, 225, 16]);
	/// assert_eq!(unpacked.to_string(), "ytcnzluhaxidtbf4.onion:4321".to_string());
	/// ```
	/// TODO: test IPV6 and Onion
	pub fn pack(&self) -> Vec<u8> {
		match self {
			Address::IPV4(address, port) => {
				let mut bytes = address.to_vec();
				bytes.append(&mut port.to_le_bytes().to_vec());
				bytes
			},
			Address::IPV6(address, port) => {
				let mut bytes = address.to_vec();
				bytes.append(&mut port.to_le_bytes().to_vec());
				bytes
			},
			Address::OnionV2(address, port) => {
				let address = address.to_lowercase();
				let mut bytes = base32::decode(address).unwrap();
				bytes.append(&mut port.to_le_bytes().to_vec());
				bytes
			},
			_ => vec![],
		}
	}

	/// To string
	/// ```
	/// use zeronet_protocol::Address;
	///
	/// let address = Address::parse("127.0.0.1:4321").unwrap();
	/// assert_eq!(address.to_string(), "127.0.0.1:4321".to_string());
	/// ```
	pub fn to_string(&self) -> String {
		match self {
			Address::IPV4(address, port) => format!(
				"{}.{}.{}.{}:{}",
				address[0], address[1], address[2], address[3], port
			),
			Address::OnionV2(address, port) => format!(
				"{}.onion:{}", address, port
			),
			_ => "not implemented".to_string(),
		}
	}
	pub fn get_pair(&self) -> Result<(Box<dyn Read + Send>, Box<dyn Write + Send>), Error> {
		match self {
			Address::IPV4(_, _) => {
				let socket = TcpStream::connect(self.to_string())?;
				return Ok((Box::new(socket.try_clone()?), Box::new(socket)));
			}
			_ => Err(Error::empty()),
		}
	}

	/// Change the port of the address.
	/// ```
	/// use zeronet_protocol::Address;
	///
	/// let address = Address::parse("127.0.0.1:4321").unwrap();
	/// let address = address.with_port(1234);
	/// assert_eq!(address.to_string(), "127.0.0.1:1234".to_string());
	/// ```
	pub fn with_port(&self, port: u16) -> Address {
		match self {
			Address::IPV4(ip, _) => Address::IPV4(*ip, port),
			Address::IPV6(ip, _) => Address::IPV6(*ip, port),
			Address::OnionV2(addr, _) => Address::OnionV2(addr.to_string(), port),
			Address::OnionV3(addr, _) => Address::OnionV3(addr.to_string(), port),
			Address::I2PB32(addr, _) => Address::I2PB32(addr.to_string(), port),
			Address::Loki(addr, _) => Address::Loki(addr.to_string(), port),
		}
	}
	pub fn get_port(&self) -> u16 {
		match self {
			Address::IPV4(_, port) => *port,
			Address::IPV6(_, port) => *port,
			Address::OnionV2(_, port) => *port,
			Address::OnionV3(_, port) => *port,
			Address::I2PB32(_, port) => *port,
			Address::Loki(_, port) => *port,
		}
	}
	pub fn is_clearnet(&self) -> bool {
		match self {
			Address::IPV4(_, _) | Address::IPV6(_, _) => true,
			_ => false,
		}
	}
	pub fn is_onion(&self) -> bool {
		match self {
			Address::OnionV2(_, _) | Address::OnionV3(_, _) => true,
			_ => false,
		}
	}
	pub fn is_i2p(&self) -> bool {
		match self {
			Address::I2PB32(_, _) => true,
			_ => false,
		}
	}
	pub fn is_loki(&self) -> bool {
		match self {
			Address::Loki(_, _) => true,
			_ => false,
		}
	}
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
	use super::*;
	use serde_bytes::ByteBuf;

	#[test]
	fn test_bytevec_vs_bytebuf() {
		// This test is just here so that a change in how bytes are serialized
		// won't go unnoticed, particularly as that could mean the can be
		// simplified.
		let address = Address::parse("127.0.0.1:8001").unwrap();
		let bytes = address.pack();
		let serialized_bytes = rmp_serde::to_vec(&bytes).unwrap();

		let byte_buf = ByteBuf::from(bytes.clone());
		let serialized_bytebuf = rmp_serde::to_vec(&byte_buf).unwrap();
		assert_ne!(
			serialized_bytes, serialized_bytebuf,
			"ByteBuf is serialized differently from bytes"
		);

		let serialized_bytebuf_json =
			rmp_serde::to_vec(&serde_json::to_value(&bytes).unwrap()).unwrap();
		assert_ne!(
			serialized_bytebuf, serialized_bytebuf_json,
			"ByteBuf is serialized differently from JSON Value equivalent"
		);

		assert_eq!(
			serialized_bytes, serialized_bytebuf_json,
			"Bytes and JSON equivalent are serialized the same"
		);
	}
}
