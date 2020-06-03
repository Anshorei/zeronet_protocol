use crate::requestable::Requestable;
use crate::util::is_default;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod templates;
pub mod value;

use value::Value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Response {
	pub cmd: String,
	pub to: usize,
	#[serde(flatten)]
	response: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Request {
	pub cmd: String,
	pub req_id: usize,
	#[serde(default, skip_serializing_if = "is_default")]
	params: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(untagged, rename_all = "camelCase")]
pub enum ZeroMessage {
	Response(Response),
	Request(Request),
}

impl ZeroMessage {
	pub fn request<V: DeserializeOwned + Serialize>(
		cmd: &str,
		req_id: usize,
		body: V,
	) -> ZeroMessage {
		let request = Request {
			cmd: cmd.to_string(),
			req_id,
			params: serde_json::from_value(serde_json::to_value(body).unwrap()).unwrap(),
		};
		ZeroMessage::Request(request)
	}
	pub fn response<V: DeserializeOwned + Serialize>(to: usize, body: V) -> ZeroMessage {
		let response = Response {
			cmd: "response".to_string(),
			to,
			response: serde_json::from_value(serde_json::to_value(body).unwrap()).unwrap(),
		};
		ZeroMessage::Response(response)
	}
	pub fn is_response(&self) -> bool {
		match self {
			ZeroMessage::Response(_) => true,
			_ => false,
		}
	}
	pub fn is_request(&self) -> bool {
		!self.is_response()
	}
	pub fn body<V: DeserializeOwned + Serialize>(self) -> V {
		let body = match self {
			ZeroMessage::Response(res) => res.response,
			ZeroMessage::Request(req) => req.params,
		};
		serde_json::from_value(serde_json::to_value(body).unwrap()).unwrap()
	}
}

impl Requestable for ZeroMessage {
	fn req_id(&self) -> Option<usize> {
		match self {
			ZeroMessage::Request(req) => Some(req.req_id),
			_ => None,
		}
	}
	fn to(&self) -> Option<usize> {
		match self {
			ZeroMessage::Response(res) => Some(res.to),
			_ => None,
		}
	}
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
	use super::ZeroMessage;

	fn des(text: &str) -> Result<ZeroMessage, serde_json::error::Error> {
		serde_json::from_str(text)
	}

	fn rmps(msg: &ZeroMessage) -> Vec<u8> {
		let jsoned = serde_json::to_value(&msg).unwrap();
		rmp_serde::to_vec_named(&jsoned).unwrap()

		// rmp_serde::to_vec_named(msg).unwrap()
	}

	fn rmpd(bytes: Vec<u8>) -> ZeroMessage {
		rmp_serde::from_slice(&bytes).unwrap()
	}

	use serde::{Deserialize, Serialize};
	#[derive(Deserialize, Serialize, Debug)]
	struct AnnounceParams {
		hashes: Vec<serde_bytes::ByteBuf>,
		port: usize,
		need_types: Vec<String>,
		delete: bool,
	}

	#[test]
	fn test_announce() {
		let text = r#"
		{
			"cmd": "announce",
			"req_id": 0,
			"params": {
				"hashes": [
					[89, 112, 7, 110, 192, 202, 246, 172, 153, 204, 68, 17, 131, 21, 113, 111, 125, 39, 66, 195, 91, 53, 41, 172, 78, 234, 146, 138, 48, 150, 109, 29],
					[29, 193, 202, 145, 155, 127, 205, 249, 222, 181, 121, 80, 223, 86, 149, 175, 49, 199, 10, 242, 237, 120, 239, 250, 84, 225, 196, 19, 67, 54, 74, 31],
					[154, 94, 94, 135, 80, 65, 245, 232, 228, 170, 254, 51, 215, 25, 155, 238, 32, 182, 95, 83, 131, 168, 192, 125, 22, 53, 43, 147, 91, 235, 29, 146]
				],
				"onion_signs": [],
				"onion_sign_this": "",
				"port": 15441,
				"need_types": ["ipv4"],
				"need_num": 20,
				"add": ["onion", "ipv4"],
				"delete": true
			}
		}"#;
		let msg = des(text).unwrap();
		assert_eq!(msg.is_request(), true);
		assert_eq!(rmpd(rmps(&msg)), msg);

		let bytes = vec![
			131, 163, 99, 109, 100, 168, 97, 110, 110, 111, 117, 110, 99, 101, 166, 112, 97, 114, 97,
			109, 115, 136, 163, 97, 100, 100, 146, 165, 111, 110, 105, 111, 110, 164, 105, 112, 118, 52,
			166, 100, 101, 108, 101, 116, 101, 195, 166, 104, 97, 115, 104, 101, 115, 147, 220, 0, 32,
			89, 112, 7, 110, 204, 192, 204, 202, 204, 246, 204, 172, 204, 153, 204, 204, 68, 17, 204,
			131, 21, 113, 111, 125, 39, 66, 204, 195, 91, 53, 41, 204, 172, 78, 204, 234, 204, 146, 204,
			138, 48, 204, 150, 109, 29, 220, 0, 32, 29, 204, 193, 204, 202, 204, 145, 204, 155, 127, 204,
			205, 204, 249, 204, 222, 204, 181, 121, 80, 204, 223, 86, 204, 149, 204, 175, 49, 204, 199,
			10, 204, 242, 204, 237, 120, 204, 239, 204, 250, 84, 204, 225, 204, 196, 19, 67, 54, 74, 31,
			220, 0, 32, 204, 154, 94, 94, 204, 135, 80, 65, 204, 245, 204, 232, 204, 228, 204, 170, 204,
			254, 51, 204, 215, 25, 204, 155, 204, 238, 32, 204, 182, 95, 83, 204, 131, 204, 168, 204,
			192, 125, 22, 53, 43, 204, 147, 91, 204, 235, 29, 204, 146, 168, 110, 101, 101, 100, 95, 110,
			117, 109, 20, 170, 110, 101, 101, 100, 95, 116, 121, 112, 101, 115, 145, 164, 105, 112, 118,
			52, 175, 111, 110, 105, 111, 110, 95, 115, 105, 103, 110, 95, 116, 104, 105, 115, 160, 171,
			111, 110, 105, 111, 110, 95, 115, 105, 103, 110, 115, 144, 164, 112, 111, 114, 116, 205, 60,
			81, 166, 114, 101, 113, 95, 105, 100, 0,
		];
		assert_eq!(rmpd(bytes), msg);

		let params: AnnounceParams = msg.body();
		assert_eq!(params.port, 15441);
	}

	#[test]
	fn test_announce_msgpack() {}

	#[test]
	fn test_get_file() {
		let text = r#"
		{
			"cmd": "getFile",
			"req_id": 0,
			"params": {}
		}"#;
		let msg = des(text).unwrap();
		assert_eq!(msg.is_request(), true);
		assert_eq!(rmpd(rmps(&msg)), msg);
	}

	#[test]
	fn test_get_file_response() {
		let msg = des(
			r#"
		{
			"cmd": "response",
			"to": 1,
			"body": "content.json content",
			"location": 1132,
			"size": 1132
		}"#,
		);
		assert_eq!(msg.is_ok(), true, "Deserializes response");
	}

	#[test]
	fn test_handshake() {
		let msg = des(
			r#"
		{
			"cmd": "handshake",
			"req_id": 0,
			"params": {
				"crypt": null,
				"crypt_supported": ["tls-rsa"],
				"fileserver_port": 15441,
				"onion": "zp2ynpztyxj2kw7x",
				"protocol": "v2",
				"port_opened": true,
				"peer_id": "-ZN0056-DMK3XX30mOrw",
				"rev": 2122,
				"target_ip": "192.168.1.13",
				"version": "0.5.6"
			}
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_handshake_response() {
		let msg = des(
			r#"
		{
			"protocol": "v2",
			"onion": "boot3rdez4rzn36x",
			"to": 0,
			"crypt": null,
			"cmd": "response",
			"rev": 2092,
			"crypt_supported": [],
			"target_ip": "zp2ynpztyxj2kw7x.onion",
			"version": "0.5.5",
			"fileserver_port": 15441,
			"port_opened": false,
			"peer_id": ""
		 }"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_stream_file() {
		let msg = des(
			r#"
		{
			"cmd": "streamFile",
			"req_id": 1,
			"params": {
				"site": "1ADDR",
				"inner_path": "content.json",
				"size": 1234
			}
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_stream_file_response() {
		let msg = des(
			r#"
		{
			"cmd": "response",
			"to": 1,
			"stream_bytes": 1234
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_ping() {
		let msg = des(
			r#"
		{
			"cmd": "ping",
			"req_id": 0
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_pong() {
		let msg = des(
			r#"
		{
			"cmd": "response",
			"to": 0,
			"body": "Pong!"
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_pex() {
		let msg = des(
			r#"
		{
			"cmd": "pex",
			"req_id": 0,
			"params": {
				"site": "1ADDR"
			}
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}

	#[test]
	fn test_pex_response() {
		let msg = des(
			r#"
		{
			"cmd": "response",
			"to": 0,
			"peers": [],
			"peers_onion": []
		}"#,
		);
		assert_eq!(msg.is_ok(), true);
	}
}
