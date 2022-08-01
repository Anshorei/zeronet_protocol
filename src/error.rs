use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid JSON: `{0}`")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Invalid MessagePack: `{0}`")]
    InvalidMessagePack(#[from] rmp_serde::decode::Error),
    #[error("Could not encode MessagePack: `{0}`")]
    EncodeRMPError(#[from] rmp_serde::encode::Error),
    #[error("Error connecting to peer")]
    ConnectionFailure,
    #[error("Connection is closed")]
    ConnectionClosed,
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
    fn from(_: std::sync::mpsc::SendError<T>) -> Error {
        Error::ChannelSendError
    }
}

impl From<decentnet_protocol::error::Error> for Error {
    fn from(error: decentnet_protocol::error::Error) -> Error {
        Error::Other(error.to_string())
    }
}

impl From<decentnet_protocol::address::ParseError> for Error {
    fn from(error: decentnet_protocol::address::ParseError) -> Error {
        Error::Other(error.to_string())
    }
}

impl From<decentnet_protocol::address::AddressError> for Error {
    fn from(error: decentnet_protocol::address::AddressError) -> Error {
        Error::Other(error.to_string())
    }
}
