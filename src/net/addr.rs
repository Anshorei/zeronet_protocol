use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnpackError {
  #[error("Wrong number of bytes")]
  ByteLength,
}

pub trait PackableAddr {
  fn pack(&self) -> Vec<u8>;
  fn unpack(bytes: &Vec<u8>) -> Result<Self, UnpackError> where Self: Sized;
}

impl PackableAddr for SocketAddrV4 {
  /// ```
  /// use std::net::{SocketAddrV4, IpAddr, Ipv4Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let socket_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
  /// let bytes = socket_addr.pack();
  /// assert_eq!(bytes, vec![127, 0, 0, 1, 144, 31]);
  /// ```
  fn pack(&self) -> Vec<u8> {
    let mut bytes = self.ip().octets().to_vec();
    bytes.append(&mut self.port().to_le_bytes().to_vec());
    bytes
  }

  /// ```
  /// use std::net::{SocketAddrV4, IpAddr, Ipv4Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let bytes = vec![127, 0, 0, 1, 144, 31];
  /// let socket_addr = SocketAddrV4::unpack(&bytes).unwrap();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.ip(), &Ipv4Addr::new(127, 0, 0, 1));
  /// ```
  fn unpack(bytes: &Vec<u8>) -> Result<Self, UnpackError>
  where Self: Sized {
    if bytes.len() != 6 {
      return Err(UnpackError::ByteLength)
    }

    let mut ip = [0u8; 4];
    ip.copy_from_slice(&bytes[..4]);
    let mut port = [0u8; 2];
    port.copy_from_slice(&bytes[4..6]);

    Ok(SocketAddrV4::new(
      Ipv4Addr::from(ip),
      u16::from_le_bytes(port),
    ))
  }
}

impl PackableAddr for SocketAddrV6 {
  /// ```
  /// use std::net::{SocketAddrV6, IpAddr, Ipv6Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let socket_addr = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
  /// let bytes = socket_addr.pack();
  /// assert_eq!(bytes, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 144, 31]);
  /// ```
  fn pack(&self) -> Vec<u8> {
    let mut bytes = self.ip().octets().to_vec();
    bytes.append(&mut self.port().to_le_bytes().to_vec());
    bytes
  }

  /// ```
  /// use std::net::{SocketAddrV6, IpAddr, Ipv6Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let bytes = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 144, 31];
  /// let socket_addr = SocketAddrV6::unpack(&bytes).unwrap();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.ip(), &Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
  /// ```
  fn unpack(bytes: &Vec<u8>) -> Result<Self, UnpackError>
  where Self: Sized {
    if bytes.len() != 18 {
      return Err(UnpackError::ByteLength)
    }

    let mut ip = [0u8; 16];
    ip.copy_from_slice(&bytes[..16]);
    let mut port = [0u8; 2];
    port.copy_from_slice(&bytes[16..18]);

    Ok(SocketAddrV6::new(
      Ipv6Addr::from(ip),
      u16::from_le_bytes(port),
      0,
      0,
    ))
  }
}

impl PackableAddr for SocketAddr {
  /// ```
  /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
  /// let bytes = socket_addr.pack();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
  /// ```
  ///
  /// ```
  /// use std::net::{SocketAddr, IpAddr, Ipv6Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let socket_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8080);
  /// let bytes = socket_addr.pack();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.ip(), IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
  /// ```
  fn pack(&self) -> Vec<u8> {
    match &self {
      SocketAddr::V4(addr) => addr.pack(),
      SocketAddr::V6(addr) => addr.pack(),
    }
  }

  /// ```
  /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let bytes = vec![127, 0, 0, 1, 144, 31];
  /// let socket_addr = SocketAddr::unpack(&bytes).unwrap();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
  /// ```
  ///
  /// ```
  /// use std::net::{SocketAddr, IpAddr, Ipv6Addr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let bytes = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 144, 31];
  /// let socket_addr = SocketAddr::unpack(&bytes).unwrap();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.ip(), IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
  /// ```
  fn unpack(bytes: &Vec<u8>) -> Result<Self, UnpackError>
  where Self: Sized {
    match bytes.len() {
      6 => Ok(SocketAddr::V4(SocketAddrV4::unpack(bytes)?)),
      18 => Ok(SocketAddr::V6(SocketAddrV6::unpack(bytes)?)),
      _ => Err(UnpackError::ByteLength)
    }
  }
}

#[cfg(feature = "i2p")]
use i2p::net::{I2pSocketAddr, I2pAddr};
#[cfg(feature = "i2p")]
use koibumi_base32 as base32;

#[cfg(feature = "i2p")]
impl PackableAddr for I2pSocketAddr {
  /// ```
  /// use i2p::net::{I2pSocketAddr, I2pAddr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let socket_addr = I2pSocketAddr::new(I2pAddr::new("q7pllaqdxb6ftmt5rd6iov4htnjex46x5kvemek2issd7rhcj5dq.b32.i2p"), 8080);
  /// let bytes = socket_addr.pack();
  /// assert_eq!(bytes, vec![135, 222, 181, 130, 3, 184, 124, 89, 178, 125, 136, 252, 135, 87, 135, 155, 82, 75, 243, 215, 234, 170, 70, 17, 90, 68, 164, 63, 196, 226, 79, 71, 144, 31]);
  /// ```
  fn pack(&self) -> Vec<u8> {
    let dest = self.dest().string();
    let mut bytes = base32::decode(dest.strip_suffix(".b32.i2p").unwrap()).unwrap();
    bytes.append(&mut self.port().to_le_bytes().to_vec());
    bytes
  }

  /// ```
  /// use i2p::net::{I2pSocketAddr, I2pAddr};
  /// use zeronet_protocol::net::PackableAddr;
  ///
  /// let bytes = vec![135, 222, 181, 130, 3, 184, 124, 89, 178, 125, 136, 252, 135, 87, 135, 155, 82, 75, 243, 215, 234, 170, 70, 17, 90, 68, 164, 63, 196, 226, 79, 71, 144, 31];
  /// let socket_addr = I2pSocketAddr::unpack(&bytes).unwrap();
  /// assert_eq!(socket_addr.port(), 8080);
  /// assert_eq!(socket_addr.dest(), I2pAddr::new("q7pllaqdxb6ftmt5rd6iov4htnjex46x5kvemek2issd7rhcj5dq.b32.i2p"));
  /// ```
  fn unpack(bytes: &Vec<u8>) -> Result<Self, UnpackError>
  where Self: Sized {
    if bytes.len() != 34 {
      return Err(UnpackError::ByteLength)
    }

    let mut address = [0u8; 32];
    address.copy_from_slice(&bytes[..32]);
    let mut port = [0u8; 2];
    port.copy_from_slice(&bytes[32..34]);

    Ok(I2pSocketAddr::new(
      I2pAddr::new(&format!("{}.b32.i2p", base32::encode(&address))),
      u16::from_le_bytes(port),
    ))
  }
}

pub enum PeerAddr {
  Ip(SocketAddr),
  #[cfg(feature = "i2p")]
  I2p(I2pSocketAddr),
  // #[cfg(feature = "tor")]
  // Tor(TorSocketAddr),
}

impl PackableAddr for PeerAddr {
  fn pack(&self) -> Vec<u8> {
    match self {
      PeerAddr::Ip(addr) => addr.pack(),
      #[cfg(feature = "i2p")]
      PeerAddr::I2p(addr) => addr.pack(),
      // #[cfg(feature = "tor")]
      // PeerAddr::Tor(addr) => addr.pack(),
    }
  }
  fn unpack(bytes: &Vec<u8>) -> Result<Self, UnpackError> {
    if let Ok(addr) = SocketAddr::unpack(bytes) {
      return Ok(PeerAddr::Ip(addr))
    }
    #[cfg(feature = "i2p")]
    if let Ok(addr) = I2pSocketAddr::unpack(bytes) {
      return Ok(PeerAddr::I2p(addr))
    }
    // #[cfg(feature = "tor")]
    // if let Ok(addr) = TorSocketAddr::unpack(bytes) {
    //   return PeerAddr::Tor(addr)
    // }

    Err(UnpackError::ByteLength)
  }
}
