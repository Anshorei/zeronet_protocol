pub mod address;
pub mod error;
pub mod async_connection;
pub mod message;
pub mod zero_connection;
pub mod requestable;

pub use zero_connection::ZeroConnection;
pub use address::Address;

#[cfg(test)]
mod tests {
	use futures::executor::block_on;
	use crate::requestable::Requestable;
	use super::Address;
	use super::ZeroConnection;

	fn handshake() -> serde_json::Value {
		let text = r#"
			{
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
			}"#;
		let value = serde_json::from_str(text).unwrap();
		value
	}

	fn announce() -> serde_json::Value {
		let text = r#"
			{
				"hashes": [],
				"onions": [],
				"onion_signs": [],
				"onion_sign_this": "",
				"port": 15441,
				"need_types": ["ipv4"],
				"need_num": 20,
				"add": ["onion"]
			}"#;
		let value = serde_json::from_str(text).unwrap();
		value
	}

	#[test]
	fn test_request() {
		let address = Address::IPV4("127.0.0.1".to_string(), 8002);
		let mut conn = ZeroConnection::from_address(address).unwrap();
		let handshake_future = conn.request("handshake", handshake());
		let response = block_on(handshake_future).unwrap();
		assert_eq!(response.to(), Some(conn.last_req_id()));

		let announce_future = conn.request("announce", announce());
		let response = block_on(announce_future).unwrap();
		assert_eq!(response.to(), Some(conn.last_req_id()));
	}
}
