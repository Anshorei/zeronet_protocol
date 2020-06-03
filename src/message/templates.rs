use serde::{Serialize, Deserialize};
use crate::util::is_default;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Handshake {
  #[serde(default, skip_serializing_if = "is_default")]
  crypt: String,
  #[serde(default, skip_serializing_if = "is_default")]
  crypt_supported: Vec<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  fileserver_port: usize,
  #[serde(default, skip_serializing_if = "is_default")]
  onion: String,
  #[serde(default, skip_serializing_if = "is_default")]
  protocol: String,
  #[serde(default, skip_serializing_if = "is_default")]
  port_opened: bool,
  #[serde(default, skip_serializing_if = "is_default")]
  peer_id: String,
  #[serde(default, skip_serializing_if = "is_default")]
  rev: usize,
  #[serde(default, skip_serializing_if = "is_default")]
  target_ip: String,
  #[serde(default, skip_serializing_if = "is_default")]
  version: String,
}
