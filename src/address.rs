use std::{
  convert::TryInto,
  io::{self, Read, Write},
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, ToSocketAddrs},
  option, vec,
};
use thiserror::Error;

#[cfg(any(feature = "tor", feature = "i2p"))]
use koibumi_base32 as base32;
#[cfg(feature = "i2p")]
use i2p::net::{I2pSocketAddr, ToI2pSocketAddrs};

pub trait ToPeerAddrs {
  /// Returned iterator over peer addresses which this type may correspond
  /// to.
  type Iter: Iterator<Item = PeerAddr>;

  /// Converts this object to an iterator of resolved `PeerAddr`s.
  ///
  /// The returned iterator may not actually yield any values depending on the
  /// outcome of any resolution performed.
  ///
  /// Note that this function may block the current thread while resolution is
  /// performed.
  ///
  /// # Errors
  ///
  /// Any errors encountered during resolution will be returned as an `Err`.
  fn to_peer_addrs(&self) -> io::Result<Self::Iter>;
}

/// to_peer_addr
/// ```
/// use zeronet_protocol::{ToPeerAddrs, PeerAddr};
///
/// let peer_addr = PeerAddr::parse("127.0.0.1:4321").unwrap();
/// assert!(peer_addr.to_peer_addrs().is_ok());
/// assert!(peer_addr.to_peer_addrs().unwrap().len() == 1);
/// ```
impl ToPeerAddrs for PeerAddr {
  type Iter = option::IntoIter<PeerAddr>;
  fn to_peer_addrs(&self) -> io::Result<Self::Iter> {
    Ok(Some(self.clone()).into_iter())
  }
}

///
/// ```
/// use std::net::{SocketAddr, ToSocketAddrs};
/// use zeronet_protocol::ToPeerAddrs;
///
/// let socket_addr = "127.0.0.1:4321".to_socket_addrs().unwrap().next().unwrap();
/// let peer_addrs = ToPeerAddrs::to_peer_addrs(
///   &socket_addr as &dyn ToSocketAddrs<Iter = std::option::IntoIter<SocketAddr>>
/// );
/// assert!(peer_addrs.is_ok());
/// assert!(peer_addrs.unwrap().len() == 1);
/// ```
impl <I: Iterator<Item = SocketAddr>> ToPeerAddrs for dyn ToSocketAddrs<Iter = I> {
  type Iter = vec::IntoIter<PeerAddr>;
  fn to_peer_addrs(&self) -> std::io::Result<Self::Iter> {
    let addrs: Vec<_> = self
      .to_socket_addrs()?
      .map(|addr| match addr {
        SocketAddr::V4(addr) => PeerAddr::IPV4(addr.ip().octets(), addr.port()),
        SocketAddr::V6(addr) => PeerAddr::IPV6(addr.ip().octets(), addr.port()),
      })
      .collect();

    Ok(addrs.into_iter())
  }
}

impl ToPeerAddrs for SocketAddr {
  type Iter = vec::IntoIter<PeerAddr>;
  fn to_peer_addrs(&self) -> io::Result<Self::Iter> {
    ToPeerAddrs::to_peer_addrs(self as &dyn ToSocketAddrs<Iter = option::IntoIter<SocketAddr>>)
  }
}

#[cfg(feature = "i2p")]
///
/// ```
/// use i2p::net::{I2pSocketAddr, ToI2pSocketAddrs};
/// use zeronet_protocol::ToPeerAddrs;
///
/// let socket_addr = "udhdrtrcetjm5sxzskjyr5ztpeszydbh4dpl3pl4utgqqw2v4jna.b32.i2p:4321".to_socket_addrs().unwrap().next().unwrap();
/// let peer_addrs = ToPeerAddrs::to_peer_addrs(
///   &socket_addr as &dyn ToI2pSocketAddrs<Iter = std::option::IntoIter<I2pSocketAddr>>
/// );
/// assert!(peer_addrs.is_ok());
/// assert!(peer_addrs.unwrap().len() == 1);
/// ```
impl <I: Iterator<Item = I2pSocketAddr>> ToPeerAddrs for dyn ToI2pSocketAddrs<Iter = I> {
  type Iter = vec::IntoIter<PeerAddr>;
  fn to_peer_addrs(&self) -> std::io::Result<Self::Iter> {
    let addrs: Vec<_> = self
      .to_socket_addrs()?
      .map(|addr| PeerAddr::I2PB32(addr.dest().string(), addr.port()))
      .collect();

    Ok(addrs.into_iter())
  }
}

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
  MissingPort,
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
pub enum PeerAddr {
  IPV4([u8; 4], u16),
  IPV6([u8; 16], u16),
  #[cfg(feature = "tor")]
  OnionV2(String, u16),
  #[cfg(feature = "tor")]
  OnionV3(String, u16),
  #[cfg(feature = "i2p")]
  I2PB32(String, u16),
  #[cfg(feature = "loki")]
  Loki(String, u16),
}

impl From<SocketAddr> for PeerAddr {
  fn from(address: SocketAddr) -> PeerAddr {
    match address {
      SocketAddr::V4(ip) => PeerAddr::IPV4(ip.ip().octets(), ip.port()),
      SocketAddr::V6(ip) => PeerAddr::IPV6(ip.ip().octets(), ip.port()),
    }
  }
}

impl TryInto<SocketAddr> for PeerAddr {
  type Error = AddressError;

  fn try_into(self) -> Result<SocketAddr, Self::Error> {
    match self {
      PeerAddr::IPV4(ip, port) => Ok(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
        port,
      )),
      PeerAddr::IPV6(ip, port) => Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(ip)), port)),
      #[cfg(feature = "tor")]
      _ => Err(AddressError::InvalidAddressType),
      #[cfg(feature = "i2p")]
      _ => Err(AddressError::InvalidAddressType),
    }
  }
}

impl TryInto<SocketAddr> for &PeerAddr {
  type Error = AddressError;

  fn try_into(self) -> Result<SocketAddr, Self::Error> {
    match self {
      PeerAddr::IPV4(ip, port) => Ok(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from(ip.clone())),
        *port,
      )),
      PeerAddr::IPV6(ip, port) => Ok(SocketAddr::new(
        IpAddr::V6(Ipv6Addr::from(ip.clone())),
        *port,
      )),
      #[cfg(feature = "i2p")]
      _ => Err(AddressError::InvalidAddressType),
      #[cfg(feature = "tor")]
      _ => Err(AddressError::InvalidAddressType),
    }
  }
}

impl PeerAddr {
  /// Create an address by parsing a string
  /// ```
  /// use zeronet_protocol::PeerAddr;
  ///
  /// let address = PeerAddr::parse("127.0.0.1:8001").unwrap();
  /// assert!(address.is_clearnet());
  /// ```
  pub fn parse<S: Into<String>>(address: S) -> Result<PeerAddr, ParseError> {
    let address: String = address.into();
    if let Ok(socket_address) = address.parse::<SocketAddr>() {
      return Ok(PeerAddr::from(socket_address));
    }
    let parts: Vec<&str> = address.split(":").collect();
    let port = parts
      .get(1)
      .map(|port| port.to_string().parse::<u16>())
      .ok_or(ParseError::MissingPort)??;

    #[cfg(feature = "tor")]
    if let Some(address) = parts[0].strip_suffix(".onion") {
      return match address.len() {
        16 => Ok(PeerAddr::OnionV2(address.to_string(), port)),
        56 => Ok(PeerAddr::OnionV3(address.to_string(), port)),
        l => Err(ParseError::WrongLength {
          address:  address.to_string(),
          length:   l,
          expected: "16 or 56".to_string(),
        }),
      };
    }
    #[cfg(feature = "i2p")]
    if let Some(address) = parts[0].strip_suffix(".b32.i2p") {
      return Ok(PeerAddr::I2PB32(address.to_string(), port));
    }
    #[cfg(feature = "loki")]
    if let Some(address) = parts[0].strip_suffix(".loki") {
      return Ok(PeerAddr::Loki(address.to_string(), port));
    }

    Err(ParseError::UnrecognizedAddressFormat)
  }

  /// Unpack the address from bytes
  /// ```
  /// use zeronet_protocol::PeerAddr;
  ///
  /// let bytes = vec![127, 0, 0, 1, 225, 16];
  /// let address = PeerAddr::unpack(&bytes).unwrap();
  /// assert_eq!(address.to_string(), "127.0.0.1:4321".to_string());
  /// ```
  pub fn unpack(bytes: &Vec<u8>) -> Result<PeerAddr, AddressError> {
    match bytes.len() {
      6 => {
        let mut array = [0u8; 4];
        array.copy_from_slice(&bytes[..4]);
        Ok(PeerAddr::IPV4(
          array,
          u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
        ))
      }
      18 => {
        let mut array = [0u8; 16];
        array.copy_from_slice(&bytes[..16]);
        Ok(PeerAddr::IPV6(
          array,
          u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
        ))
      }
      #[cfg(feature = "tor")]
      12 => {
        let port = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
        let mut array = [0u8; 10];
        array.copy_from_slice(&bytes[..10]);
        let address = base32::encode(&array);
        Ok(PeerAddr::OnionV2(address, port))
      }
      #[cfg(feature = "i2p")]
      34 => {
        let port = u16::from_le_bytes(bytes[32..34].try_into().unwrap());
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes[..32]);
        let address = base32::encode(&array);
        Ok(PeerAddr::I2PB32(address, port))
      }
      #[cfg(feature = "tor")]
      37 => {
        let port = u16::from_le_bytes(bytes[35..37].try_into().unwrap());
        let mut array = [0u8; 35];
        array.copy_from_slice(&bytes[..35]);
        let address = base32::encode(&array);
        Ok(PeerAddr::OnionV3(address, port))
      }
      l => Err(AddressError::InvalidBytearray(l)),
    }
  }

  /// Pack the address into bytes
  /// ```
  /// use zeronet_protocol::PeerAddr;
  ///
  /// let address = PeerAddr::parse("127.0.0.1:4321").expect("could not parse address");
  /// let packed = address.pack();
  ///
  /// assert_eq!(packed, [127, 0, 0, 1, 225, 16]);
  /// ```
  pub fn pack(&self) -> Vec<u8> {
    match self {
      PeerAddr::IPV4(address, port) => {
        let mut bytes = address.to_vec();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      PeerAddr::IPV6(address, port) => {
        let mut bytes = address.to_vec();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      #[cfg(feature = "tor")]
      PeerAddr::OnionV2(address, port) => {
        let address = address.to_lowercase();
        let mut bytes = base32::decode(address).unwrap();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      #[cfg(feature = "tor")]
      PeerAddr::OnionV3(address, port) => {
        let address = address.to_lowercase();
        let mut bytes = base32::decode(address).unwrap();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      #[cfg(feature = "i2p")]
      PeerAddr::I2PB32(address, port) => {
        let address = address.to_lowercase();
        let mut bytes = base32::decode(address).unwrap();
        bytes.append(&mut port.to_le_bytes().to_vec());
        bytes
      }
      #[cfg(feature = "loki")]
      PeerAddr::Loki(_address, _port) => {
        unimplemented!()
        //   let address = address.to_lowercase();
        //   let mut bytes = base32::decode(address).unwrap();
        //   bytes.append(&mut port.to_le_bytes().to_vec());
        //   bytes
      }
    }
  }

  /// To string
  /// ```
  /// use zeronet_protocol::PeerAddr;
  ///
  /// let address = PeerAddr::parse("127.0.0.1:4321").unwrap();
  /// assert_eq!(address.to_string(), "127.0.0.1:4321".to_string());
  /// ```
  pub fn to_string(&self) -> String {
    match self {
      PeerAddr::IPV4(_, _) => {
        let socket_addr: SocketAddr = self.try_into().unwrap();
        socket_addr.to_string()
      }
      PeerAddr::IPV6(_, _) => {
        let socket_addr: SocketAddr = self.try_into().unwrap();
        socket_addr.to_string()
      }
      #[cfg(feature = "tor")]
      PeerAddr::OnionV2(address, port) => format!("{}.onion:{}", address, port),
      #[cfg(feature = "tor")]
      PeerAddr::OnionV3(address, port) => format!("{}.onion:{}", address, port),
      #[cfg(feature = "i2p")]
      PeerAddr::I2PB32(address, port) => format!("{}.b32.i2p:{}", address, port),
      #[cfg(feature = "loki")]
      PeerAddr::Loki(address, port) => format!("{}.loki:{}", address, port),
    }
  }
  pub fn get_pair(&self) -> Result<(Box<dyn Read + Send>, Box<dyn Write + Send>), AddressError> {
    match self {
      PeerAddr::IPV4(_, _) => {
        let socket = TcpStream::connect(self.to_string())?;
        return Ok((Box::new(socket.try_clone()?), Box::new(socket)));
      }
      _ => Err(AddressError::TcpStreamError),
    }
  }

  /// Change the port of the address.
  /// ```
  /// use zeronet_protocol::PeerAddr;
  ///
  /// let address = PeerAddr::parse("127.0.0.1:4321").unwrap();
  /// let address = address.with_port(1234);
  /// assert_eq!(address.to_string(), "127.0.0.1:1234".to_string());
  /// ```
  pub fn with_port(&self, port: u16) -> PeerAddr {
    match self {
      PeerAddr::IPV4(ip, _) => PeerAddr::IPV4(*ip, port),
      PeerAddr::IPV6(ip, _) => PeerAddr::IPV6(*ip, port),
      #[cfg(feature = "tor")]
      PeerAddr::OnionV2(addr, _) => PeerAddr::OnionV2(addr.to_string(), port),
      #[cfg(feature = "tor")]
      PeerAddr::OnionV3(addr, _) => PeerAddr::OnionV3(addr.to_string(), port),
      #[cfg(feature = "i2p")]
      PeerAddr::I2PB32(addr, _) => PeerAddr::I2PB32(addr.to_string(), port),
      #[cfg(feature = "loki")]
      PeerAddr::Loki(addr, _) => PeerAddr::Loki(addr.to_string(), port),
    }
  }
  pub fn get_port(&self) -> u16 {
    match self {
      PeerAddr::IPV4(_, port) => *port,
      PeerAddr::IPV6(_, port) => *port,
      #[cfg(feature = "tor")]
      PeerAddr::OnionV2(_, port) => *port,
      #[cfg(feature = "tor")]
      PeerAddr::OnionV3(_, port) => *port,
      #[cfg(feature = "i2p")]
      PeerAddr::I2PB32(_, port) => *port,
      #[cfg(feature = "loki")]
      PeerAddr::Loki(_, port) => *port,
    }
  }
  pub fn is_clearnet(&self) -> bool {
    match self {
      PeerAddr::IPV4(_, _) | PeerAddr::IPV6(_, _) => true,
      #[cfg(any(feature = "tor", feature = "i2p"))]
      _ => false,
    }
  }
  #[cfg(feature = "tor")]
  pub fn is_onion(&self) -> bool {
    match self {
      PeerAddr::OnionV2(_, _) | PeerAddr::OnionV3(_, _) => true,
      _ => false,
    }
  }
  #[cfg(feature = "i2p")]
  pub fn is_i2p(&self) -> bool {
    match self {
      PeerAddr::I2PB32(_, _) => true,
      _ => false,
    }
  }
  #[cfg(feature = "loki")]
  pub fn is_loki(&self) -> bool {
    match self {
      PeerAddr::Loki(_, _) => true,
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
    let address = PeerAddr::parse("127.0.0.1:8001").unwrap();
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

  #[test]
  fn test_pack_ipv4() {
    let address = PeerAddr::parse("127.0.0.1:4321").expect("could not parse address");
    let packed = address.pack();

    assert_eq!(packed, [127, 0, 0, 1, 225, 16]);
  }

  #[test]
  fn test_pack_ipv6() {
    let address_string = "[1001:2002:3003:4004:5005:6006:7007:8008]:4321".to_string();
    let address = PeerAddr::parse(&address_string).expect("could not parse address");
    let packed = address.pack();
    let unpacked = PeerAddr::unpack(&packed).expect("could not unpack address");

    assert_eq!(packed, [16, 1, 32, 2, 48, 3, 64, 4, 80, 5, 96, 6, 112, 7, 128, 8, 225, 16]);
    assert_eq!(unpacked.to_string(), address_string);
  }

  #[test]
  fn test_pack_ipv6_shorthand() {
    let address_string = "[2001:db8::ff00:42:8329]:4321".to_string();
    let address = PeerAddr::parse(&address_string).expect("could not parse address");
    let packed = address.pack();
    let unpacked = PeerAddr::unpack(&packed).expect("could not unpack address");

    assert_eq!(packed, [32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 255, 0, 0, 66, 131, 41, 225, 16]);
    assert_eq!(unpacked.to_string(), address_string);
  }

  #[cfg(feature = "tor")]
  #[test]
  fn test_pack_onionv2() {
    let address_string = "ytcnzluhaxidtbf4.onion:4321".to_string();
    let address = PeerAddr::parse(&address_string).expect("could not parse address");
    let packed = address.pack();
    let unpacked = PeerAddr::unpack(&packed).expect("could not unpack address");

    assert_eq!(packed, [196, 196, 220, 174, 135, 5, 208, 57, 132, 188, 225, 16]);
    assert_eq!(unpacked.to_string(), address_string);
  }

  #[cfg(feature = "tor")]
  #[test]
  fn test_pack_onionv3() {
    let address_string = "trackd5xiih3z7xyvvkyz2n65lehqziayjpxzsau3mwccwlelxrdrgid.onion:4321".to_string();
    let address = PeerAddr::parse(&address_string).expect("could not parse address");
    let packed = address.pack();
    let unpacked = PeerAddr::unpack(&packed).expect("could not unpack address");

    assert_eq!(packed, [156, 64, 37, 15, 183, 66, 15, 188, 254, 248, 173, 85, 140, 233, 190, 234, 200, 120, 101, 0, 194, 95, 124, 200, 20, 219, 44, 33, 89, 100, 93, 226, 56, 153, 3, 225, 16]);
    assert_eq!(unpacked.to_string(), address_string);
  }

  #[cfg(feature = "i2p")]
  #[test]
  fn test_pack_i2pb32() {
    let address_string = "udhdrtrcetjm5sxzskjyr5ztpeszydbh4dpl3pl4utgqqw2v4jna.b32.i2p:4321".to_string();
    let address = PeerAddr::parse(&address_string).expect("could not parse address");
    let packed = address.pack();
    let unpacked = PeerAddr::unpack(&packed).expect("could not unpack address");

    assert_eq!(packed, [160, 206, 56, 206, 34, 36, 210, 206, 202, 249, 146, 147, 136, 247, 51, 121, 37, 156, 12, 39, 224, 222, 189, 189, 124, 164, 205, 8, 91, 85, 226, 90, 225, 16]);
    assert_eq!(unpacked.to_string(), address_string);
  }

  #[cfg(feature = "loki")]
  #[test]
  fn test_pack_loki() {
    let address_string = "dw68y1xhptqbhcm5s8aaaip6dbopykagig5q5u1za4c7pzxto77y.loki:4321".to_string();
    let address = PeerAddr::parse(&address_string).expect("could not parse address");
    let packed = address.pack();
    let unpacked = PeerAddr::unpack(&packed).expect("could not unpack address");

    assert_eq!(packed, [160, 206, 56, 206, 34, 36, 210, 206, 202, 249, 146, 147, 136, 247, 51, 121, 37, 156, 12, 39, 224, 222, 189, 189, 124, 164, 205, 8, 91, 85, 226, 90, 225, 16]);
    assert_eq!(unpacked.to_string(), address_string);
  }
}

impl std::fmt::Display for PeerAddr {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let address_type = match self {
      PeerAddr::IPV4(_, _) => "ipv4",
      PeerAddr::IPV6(_, _) => "ipv6",
      #[cfg(feature = "tor")]
      PeerAddr::OnionV2(_, _) => "onionv2",
      #[cfg(feature = "tor")]
      PeerAddr::OnionV3(_, _) => "onionv3",
      #[cfg(feature = "i2p")]
      PeerAddr::I2PB32(_, _) => "i2pb32",
      #[cfg(feature = "loki")]
      PeerAddr::Loki(_, _) => "loki",
    };
    write!(f, "{} [{}]", address_type, self.to_string())
  }
}
