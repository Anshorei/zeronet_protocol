use crate::util::is_default;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Handshake {
  /// The PeerID of the sender
  pub peer_id:         String,
  pub fileserver_port: usize,
  /// Time at which the message was sent
  pub time:            u64,
  #[serde(default, skip_serializing_if = "is_default")]
  pub crypt:           Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub crypt_supported: Vec<String>,
  /// For backwards compatibility with ZeroNet-py < v0.7.0
  #[serde(default, skip_serializing_if = "is_default")]
  pub use_bin_type:    bool,
  #[serde(default, skip_serializing_if = "is_default")]
  pub onion:           Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub protocol:        String,
  #[serde(default, skip_serializing_if = "is_default")]
  pub port_opened:     Option<bool>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub rev:             usize,
  /// The address this handshake is addressed to, including ".onion" or ".b32.i2p"
  #[serde(default, skip_serializing_if = "is_default", rename = "target_ip")]
  pub target_address:  Option<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub version:         String,
}

impl Handshake {
  pub fn new() -> Handshake {
    let now = SystemTime::now();
    Handshake {
      version:         "0.7.6".to_string(),
      rev:             4565,
      protocol:        "v2".to_string(),
      use_bin_type:    true,
      fileserver_port: 0,
      port_opened:     Some(false),
      crypt_supported: vec![],
      time:            now.duration_since(UNIX_EPOCH).unwrap().as_secs(),

      onion:          None,
      crypt:          None,
      target_address: None,
      peer_id:        String::new(),
    }
  }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PingResponse {
  pub body: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Announce {
  pub port:            usize,
  #[serde(default, skip_serializing_if = "is_default")]
  pub add:             Vec<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub need_types:      Vec<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub need_num:        usize,
  #[serde(default, skip_serializing_if = "is_default")]
  pub hashes:          Vec<ByteBuf>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub onions:          Vec<String>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub onion_signs:     Vec<ByteBuf>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub onion_sign_this: String,
  #[serde(default, skip_serializing_if = "is_default")]
  pub delete:          bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AnnounceResponse {
  pub peers: Vec<AnnouncePeers>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AnnouncePeers {
  #[serde(rename = "ipv4", alias = "ip4", skip_serializing_if = "is_default")]
  pub ip_v4:    Vec<ByteBuf>,
  #[serde(rename = "ipv6", skip_serializing_if = "is_default")]
  pub ip_v6:    Vec<ByteBuf>,
  #[serde(rename = "onion", skip_serializing_if = "is_default")]
  pub onion_v2: Vec<ByteBuf>,
  // TODO: use correct length for next two
  #[serde(skip_serializing_if = "is_default")]
  pub onion_v3: Vec<ByteBuf>, // 42 bytes?
  #[serde(skip_serializing_if = "is_default")]
  pub i2p_b32:  Vec<ByteBuf>,
  #[serde(skip_serializing_if = "is_default")]
  pub loki:     Vec<ByteBuf>,
}

impl Debug for AnnouncePeers {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let iterator = self
      .ip_v4
      .iter()
      .chain(self.ip_v6.iter())
      .chain(self.onion_v2.iter())
      .chain(self.onion_v3.iter())
      .chain(self.i2p_b32.iter())
      .chain(self.loki.iter());
    let strings: Vec<String> = iterator
      .map(|ip| crate::PeerAddr::unpack(ip).unwrap().to_string())
      .collect();
    write!(f, "[{}]", strings.join(", "))
  }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetFile {
  pub site:       String,
  pub inner_path: String,
  pub location:   usize,
  #[serde(skip_serializing_if = "is_default")]
  pub file_size:  usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetFileResponse {
  pub body:     ByteBuf,
  pub location: usize,
  pub size:     usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFile {
  pub inner_path: String,
  pub size:       usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFileResponse {
  pub stream_bytes: usize,
}

pub struct Pex {
  pub site:        String,
  pub peers:       Vec<ByteBuf>,
  pub peers_onion: Vec<ByteBuf>,
  pub need:        usize,
}

pub struct PexResponse {
  pub peers:       Vec<ByteBuf>,
  pub peers_onion: Vec<ByteBuf>,
}

pub struct UpdateFile {
  pub site:       String,
  pub inner_path: String,
  pub body:       ByteBuf,
  pub diffs:      Vec<Diff>,
}

pub struct Diff {
  pub opcode: String,
  pub diff:   String,
}

pub struct UpdateFileResponse {
  pub ok: bool,
}

pub struct ListModified {
  pub site:  String,
  pub since: usize,
}

pub struct ListModifiedResponse {
  pub modified_files: HashMap<String, usize>,
}

pub struct GetHashfield {
  pub site: String,
}

pub struct GetHashfieldResponse {
  pub hashfield_raw: ByteBuf,
}

pub struct SetHashfield {
  pub site:          String,
  pub hashfield_raw: ByteBuf,
}

pub struct SetHashfieldResponse {
  pub ok: bool,
}

pub struct FindHashIds {
  pub site:     String,
  pub hash_ids: Vec<usize>,
}

pub struct FindHashIdsResponse {
  pub peers:       HashMap<usize, Vec<ByteBuf>>,
  pub peers_onion: HashMap<usize, Vec<ByteBuf>>,
}

pub struct Checkport {
  pub port: u16,
}

pub struct CheckportResponse {
  pub status:      String,
  pub ip_external: String,
}

pub struct GetPieceFields {
  pub site: String,
}

// TODO: do piecefields properly
pub struct GetPieceFieldsResponse {
  pub piecefields_packed: ByteBuf,
}

// TODO: do piecefields properly
pub struct SetPieceFields {
  pub site:               String,
  pub piecefields_packed: ByteBuf,
}

pub struct SetPieceFieldsResponse {
  pub ok: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Error {
  pub error: String,
}
