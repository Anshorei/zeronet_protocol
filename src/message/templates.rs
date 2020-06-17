use crate::util::is_default;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Handshake {
	pub peer_id: String,
	pub fileserver_port: usize,
	pub time: u64,
	#[serde(default, skip_serializing_if = "is_default")]
	pub crypt: Option<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub crypt_supported: Vec<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub use_bin_type: bool,
	#[serde(default, skip_serializing_if = "is_default")]
	pub onion: Option<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub protocol: String,
	#[serde(default, skip_serializing_if = "is_default")]
	pub port_opened: Option<bool>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub rev: usize,
	#[serde(default, skip_serializing_if = "is_default")]
	pub target_ip: Option<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub version: String,
}

impl Handshake {
	pub fn new() -> Handshake {
		let now = SystemTime::now();
		Handshake {
			version: "0.7".to_string(),
			rev: 4486,
			protocol: "v2".to_string(),
			use_bin_type: true,
			fileserver_port: 0,
			port_opened: Some(false),
			crypt_supported: vec![],
			time: now.duration_since(UNIX_EPOCH).unwrap().as_secs(),

			onion: None,
			crypt: None,
			target_ip: None,
			peer_id: String::new(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Announce {
	pub port: usize,
	#[serde(default, skip_serializing_if = "is_default")]
	pub add: Vec<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub need_types: Vec<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub need_num: usize,
	#[serde(default, skip_serializing_if = "is_default")]
	pub hashes: Vec<ByteBuf>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub onions: Vec<String>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub onion_signs: Vec<ByteBuf>,
	#[serde(default, skip_serializing_if = "is_default")]
	pub onion_sign_this: String,
	#[serde(default, skip_serializing_if = "is_default")]
	pub delete: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AnnounceResponse {
	pub peers: Vec<AnnouncePeers>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AnnouncePeers {
	#[serde(rename = "ipv4", alias = "ip4")]
	pub ip_v4: Vec<ByteBuf>,
	#[serde(rename = "ipv6")]
	pub ip_v6: Vec<ByteBuf>,
	#[serde(rename = "onion")]
	pub onion_v2: Vec<ByteBuf>,
	// TODO: use correct length for next two
	pub onion_v3: Vec<ByteBuf>, // 42 bytes?
	pub i2p_b32: Vec<ByteBuf>,
	pub loki: Vec<ByteBuf>,
}

impl Debug for AnnouncePeers {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let iterator = self.ip_v4.iter()
			.chain(self.ip_v6.iter())
			.chain(self.onion_v2.iter())
			.chain(self.onion_v3.iter())
			.chain(self.i2p_b32.iter())
			.chain(self.loki.iter());
		let strings: Vec<String> = iterator.map(|ip| crate::address::Address::unpack(ip).unwrap().to_string()).collect();
		write!(f, "[{}]", strings.join(", "))
	}
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Error {
	pub error: String,
}
