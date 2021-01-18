use crate::address::AddressError;
use crate::address::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Invalid JSON: `{0}`")]
  InvalidJson(#[from] serde_json::Error),
  #[error("Invalid MessagePack: `{0}`")]
  InvalidMessagePack(#[from] rmp_serde::decode::Error),
  #[error("Could not encode MessagePack: `{0}`")]
  EncodeRMPError(#[from] rmp_serde::encode::Error),
  #[error("I/O Error: `{0}`")]
  Io(#[from] std::io::Error),
  #[error("Error connecting to peer")]
  ConnectionFailure,
  #[error("Connection is closed")]
  ConnectionClosed,
  #[error("Error parsing address: `{0}`")]
  ParseError(#[from] ParseError),
  #[error("Error doing something with address: `{0}`")]
  AddressError(#[from] AddressError),
  #[error("Error decoding base64 `{0}`")]
  Base64Decode(#[from] base64::DecodeError),
  #[error("Error sending over mpsc channel")]
  ChannelSendError,
  #[error("Error receiving over mpsc channel: `{0}`")]
  ChannelRecvError(#[from] std::sync::mpsc::RecvError),

  #[error("Unexpectedly received a response")]
  UnexpectedResponse,
  #[error("Unexpectedly received a request")]
  UnexpectedRequest,
  #[error("Missing request id")]
  MissingReqId,

  #[error("This shouldn't even exist")]
  Other(String),
}

impl Error {
  pub fn text(message: &str) -> Error {
    Error::Other(message.to_string())
  }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
  fn from(err: std::sync::mpsc::SendError<T>) -> Error {
    Error::ChannelSendError
  }
}
