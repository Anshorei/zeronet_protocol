use koibumi_base32 as base32;
use std::{
  io::{Read, Write},
  convert::TryInto,
  net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    TcpStream,
  },
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
  #[error("Error parsing int: `{0}`")]
  ParseIntError(#[from] std::num::ParseIntError),
  #[error("Address `{address}` has wrong length ({length}) for {expected}")]
  WrongLength {
    address:  String,
    length:   usize,
    expected: String,
  },
  #[error("Unrecognized address format")]
  UnrecognizedAddressFormat,
  #[error("Address is missing port")]
  MissingPort
}

#[derive(Debug, Error)]
pub enum AddressError {
  #[error("Error unpacking address")]
  UnpackError,
  #[error("Unexpected number of bytes {0}")]
  InvalidBytearray(usize),
  #[error("Error creating tcp stream read-write pair")]
  TcpStreamError,
  #[error("I/O Error")]
  IoError(#[from] std::io::Error),
  #[error("Address is of an invalid type")]
  InvalidAddressType,
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

impl TryInto<SocketAddr> for Address {
  type Error = AddressError;

  fn try_into(self) -> Result<SocketAddr, Self::Error> {
    match self {
      Address::IPV4(ip, port) => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), port)),
      Address::IPV6(ip, port) => Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(ip)), port)),
      _ => Err(AddressError::InvalidAddressType),
    }
  }
}

impl TryInto<SocketAddr> for &Address {
  type Error = AddressError;

  fn try_into(self) -> Result<SocketAddr, Self::Error> {
    match self {
      Address::IPV4(ip, port) => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip.clone())), *port)),
      Address::IPV6(ip, port) => Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(ip.clone())), *port)),
      _ => Err(AddressError::InvalidAddressType),
    }
  }
}

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
    if let Ok(socket_address) = address.parse::<SocketAddr>() {
      return Ok(Address::from(socket_address));
    }
    let parts: Vec<&str> = address.split(":").collect();
    let port = parts
      .get(1)
      .map(|port| port.to_string().parse::<u16>())
      .ok_or(ParseError::MissingPort)??;

    if let Some(address) = parts[0].strip_suffix(".onion") {
      return match address.len() {
        16 => Ok(Address::OnionV2(address.to_string(), port)),
        56 => Ok(Address::OnionV3(address.to_string(), port)),
        l => Err(ParseError::WrongLength{
          address: address.to_string(),
          length: l,
          expected: "16 or 56".to_string(),
        })
      }
    } else if let Some(address) = parts[0].strip_suffix(".b32.i2p") {
      return Ok(Address::I2PB32(address.to_string(), port));
    } else if let Some(address) = parts[0].strip_suffix(".loki") {
      return Ok(Address::Loki(address.to_string(), port));
    }

    Err(ParseError::UnrecognizedAddressFormat)
  }

  /// unpack
  /// ```
  /// use zeronet_protocol::Address;
  ///
  /// let bytes = vec![127, 0, 0, 1, 225, 16];
  /// let address = Address::unpack(&bytes).unwrap();
  /// assert_eq!(address.to_string(), "127.0.0.1:4321".to_string());
  ///
  /// let bytes =  vec![196, 196, 220, 174, 135, 5, 208, 57, 132, 188, 225, 16];
  /// let address =  Address::unpack(&bytes).unwrap();
  /// assert_eq!(address.to_string(), "ytcnzluhaxidtbf4.onion:4321".to_string());
  /// // TODO: test IPV6
  /// ```
  pub fn unpack(bytes: &Vec<u8>) -> Result<Address, AddressError> {
    match bytes.len() {
      6 => {
        let mut array = [0u8; 4];
        array.copy_from_slice(&bytes[..4]);
        Ok(Address::IPV4(
          array,
          u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
        ))
      }
      18 => {
        let mut array = [0u8; 16];
        array.copy_from_slice(&bytes[..16]);
        Ok(Address::IPV6(
          array,
          u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
        ))
      }
      12 => {
        let port = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
        let mut array = [0u8; 10];
        array.copy_from_slice(&bytes[..10]);
        let address = base32::encode(&array);
        Ok(Address::OnionV2(address, port))
      }
      34 => {
        let port = u16::from_le_bytes(bytes[32..34].try_into().unwrap());
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes[..32]);
        let address = base32::encode(&array);
        Ok(Address::I2PB32(address, port))
      }
      37 => {
        let port = u16::from_le_bytes(bytes[35..37].try_into().unwrap());
        let mut array = [0u8; 35];
        array.copy_from_slice(&bytes[..35]);
        let address = base32::encode(&array);
        Ok(Address::OnionV3(address, port))
      }
      l => Err(AddressError::InvalidBytearray(l)),
    }
  }

  /// pack
  /// ```
  /// use zeronet_protocol::Address;
  ///
  /// let address = Address::parse("127.0.0.1:4321").expect("could not parse address");
  /// let packed = address.pack();
  ///
  /// assert_eq!(packed, [127, 0, 0, 1, 225, 16]);
  ///
  /// let address_string = "[1001:2002:3003:4004:5005:6006:7007:8008]:4321".to_string();
  /// let address = Address::parse(&address_string).expect("could not parse address");
  /// let packed = address.pack();
  /// let unpacked = Address::unpack(&packed).expect("could not unpack address");
  ///
  /// assert_eq!(packed, [16, 1, 32, 2, 48, 3, 64, 4, 80, 5, 96, 6, 112, 7, 128, 8, 225, 16]);
  /// assert_eq!(unpacked.to_string(), address_string);
  ///
  /// let address_string = "[2001:db8::ff00:42:8329]:4321".to_string();
  /// let address = Address::parse(&address_string).expect("could not parse address");
  /// let packed = address.pack();
  /// let unpacked = Address::unpack(&packed).expect("could not unpack address");
  ///
  /// assert_eq!(packed, [32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 255, 0, 0, 66, 131, 41, 225, 16]);
  /// assert_eq!(unpacked.to_string(), address_string);
  ///
  /// let address_string = "ytcnzluhaxidtbf4.onion:4321".to_string();
  /// let address = Address::parse(&address_string).expect("could not parse address");
  /// let packed = address.pack();
  /// let unpacked = Address::unpack(&packed).expect("could not unpack address");
  ///
  /// assert_eq!(packed, [196, 196, 220, 174, 135, 5, 208, 57, 132, 188, 225, 16]);
  /// assert_eq!(unpacked.to_string(), address_string);
  ///
  /// let address_string = "trackd5xiih3z7xyvvkyz2n65lehqziayjpxzsau3mwccwlelxrdrgid.onion:4321".to_string();
  /// let address = Address::parse(&address_string).expect("could not parse address");
  /// let packed = address.pack();
  /// let unpacked = Address::unpack(&packed).expect("could not unpack address");
  ///
  /// assert_eq!(packed, [156, 64, 37, 15, 183, 66, 15, 188, 254, 248, 173, 85, 140, 233, 190, 234, 200, 120, 101, 0, 194, 95, 124, 200, 20, 219, 44, 33, 89, 100, 93, 226, 56, 153, 3, 225, 16]);
  /// assert_eq!(unpacked.to_string(), address_string);
  ///
  /// let address_string = "udhdrtrcetjm5sxzskjyr5ztpeszydbh4dpl3pl4utgqqw2v4jna.b32.i2p:4321".to_string();
  /// let address = Address::parse(&address_string).expect("could not parse address");
  /// let packed = address.pack();
  /// let unpacked = Address::unpack(&packed).expect("could not unpack address");
  ///
  /// assert_eq!(packed, [160, 206, 56, 206, 34, 36, 210, 206, 202, 249, 146, 147, 136, 247, 51, 121, 37, 156, 12, 39, 224, 222, 189, 189, 124, 164, 205, 8, 91, 85, 226, 90, 225, 16]);
  /// assert_eq!(unpacked.to_string(), address_string);
  ///
  /// // TODO: add support for loki addresses
  /// // let address_string = "dw68y1xhptqbhcm5s8aaaip6dbopykagig5q5u1za4c7pzxto77y.loki:4321".to_string();
  /// // let address = Address::parse(&address_string).expect("could not parse address");
  /// // let packed = address.pack();
  /// // let unpacked = Address::unpack(&packed).expect("could not unpack address");
  ///
  /// // assert_eq!(packed, [160, 206, 56, 206, 34, 36, 210, 206, 202, 249, 146, 147, 136, 247, 51, 121, 37, 156, 12, 39, 224, 222, 189, 189, 124, 164, 205, 8, 91, 85, 226, 90, 225, 16]);
  /// // assert_eq!(unpacked.to_string(), address_string);
  /// ```
  /// TODO: OnionV3, I2P
  pub fn pack(&self) -> Vec<u8> {
    match self {
      Address::IPV4(address, port) => {
        let mut bytes = address.to_vec();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      Address::IPV6(address, port) => {
        let mut bytes = address.to_vec();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      Address::OnionV2(address, port) => {
        let address = address.to_lowercase();
        let mut bytes = base32::decode(address).unwrap();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      Address::OnionV3(address, port) => {
        let address = address.to_lowercase();
        let mut bytes = base32::decode(address).unwrap();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      Address::I2PB32(address, port) => {
        let address = address.to_lowercase();
        let mut bytes = base32::decode(address).unwrap();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      _ => vec![],
      // Address::Loki(address, port) => {
      //   let address = address.to_lowercase();
      //   let mut bytes = base32::decode(address).unwrap();
      //   bytes.append(&mut port.to_le_bytes().to_vec());
      //   bytes
      // }
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
      Address::IPV4(_, _) => {
        let socket_addr: SocketAddr = self.try_into().unwrap();
        socket_addr.to_string()
      },
      Address::IPV6(_, _) => {
        let socket_addr: SocketAddr = self.try_into().unwrap();
        socket_addr.to_string()
      },
      Address::OnionV2(address, port) => format!("{}.onion:{}", address, port),
      Address::OnionV3(address, port) => format!("{}.onion:{}", address, port),
      Address::I2PB32(address, port) => format!("{}.b32.i2p:{}", address, port),
      Address::Loki(address, port) => format!("{}.loki:{}", address, port),
    }
  }
  pub fn get_pair(&self) -> Result<(Box<dyn Read + Send>, Box<dyn Write + Send>), AddressError> {
    match self {
      Address::IPV4(_, _) => {
        let socket = TcpStream::connect(self.to_string())?;
        return Ok((Box::new(socket.try_clone()?), Box::new(socket)));
      }
      _ => Err(AddressError::TcpStreamError),
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
#[cfg_attr(tarpaulin, ignore)]
mod tests {
  use super::*;
  use serde_bytes::ByteBuf;

  #[test]
  fn test_bytevec_vs_bytebuf() {
    // This test is just here so that a change in how bytes are serialized
    // won't go unnoticed, particularly as that could mean they can be
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

impl std::fmt::Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let address_type = match self {
      Address::IPV4(_, _) => "ipv4",
      Address::IPV6(_, _) => "ipv6",
      Address::OnionV2(_, _) => "onionv2",
      Address::OnionV3(_, _) => "onionv3",
      Address::I2PB32(_, _) => "i2pb32",
      Address::Loki(_, _) => "loki",
    };
    write!(f, "{} [{}]", address_type, self.to_string())
  }
}
