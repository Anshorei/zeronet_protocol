#[derive(Debug, PartialEq, Eq)]
pub struct Error {
	message: String,
}

impl Error {
	pub fn empty() -> Error {
		Error {
			message: String::new(),
		}
	}
	pub fn text(message: &str) -> Error {
		Error {
			message: message.to_string(),
		}
	}
}

impl From<rmp_serde::encode::Error> for Error {
	fn from(err: rmp_serde::encode::Error) -> Error {
		Error {
			message: format!("{:?}", err),
		}
	}
}

impl From<rmp_serde::decode::Error> for Error {
	fn from(_: rmp_serde::decode::Error) -> Error {
		Error::empty()
	}
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
	fn from(_: std::sync::mpsc::SendError<T>) -> Error {
		Error::empty()
	}
}

impl From<std::sync::mpsc::RecvError> for Error {
	fn from(_: std::sync::mpsc::RecvError) -> Error {
		Error::empty()
	}
}

impl From<std::io::Error> for Error {
	fn from(_: std::io::Error) -> Error {
		Error::empty()
	}
}
