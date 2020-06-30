use crate::address::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
	InvalidData(String),
	Io(String),
	Other(String),
	ConnectionFailure,
	ParseError,
	EncodeError,
	DecodeError,
	NotYetImplemented,
}

impl Error {
	pub fn todo() -> Error {
		Error::NotYetImplemented
	}
	pub fn text(message: &str) -> Error {
		Error::Other(message.to_string())
	}
}

impl From<rmp_serde::encode::Error> for Error {
	fn from(err: rmp_serde::encode::Error) -> Error {
		Error::text(&format!("{:?}", err))
	}
}

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Error {
		match err.classify() {
			serde_json::error::Category::Data => Error::InvalidData(err.to_string()),
			_ => Error::text(&format!("{:?}", err)),
		}
	}
}

impl From<rmp_serde::decode::Error> for Error {
	fn from(err: rmp_serde::decode::Error) -> Error {
		match err {
			rmp_serde::decode::Error::InvalidMarkerRead(_) => Error::Io(format!("{:?}", err)),
			_ => Error::text(&format!("{:?}", err)),
		}
	}
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
	fn from(err: std::sync::mpsc::SendError<T>) -> Error {
		Error::text(&format!("{:?}", err))
	}
}

impl From<std::sync::mpsc::RecvError> for Error {
	fn from(err: std::sync::mpsc::RecvError) -> Error {
		Error::text(&format!("{:?}", err))
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Error {
		Error::text(&format!("Io Error: {:?}", err))
	}
}

impl From<ParseError> for Error {
	fn from(_: ParseError) -> Error {
		Error::ParseError
	}
}

impl From<base64::DecodeError> for Error {
	fn from(_: base64::DecodeError) -> Error {
		Error::DecodeError
	}
}

impl From<koibumi_base32::EncodeError> for Error {
	fn from(_: koibumi_base32::EncodeError) -> Error {
		Error::EncodeError
	}
}
