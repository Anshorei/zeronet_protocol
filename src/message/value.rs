use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::Number;
use std::collections::HashMap;

/// Value is a custom enum mimicking serde_json::Value
/// but with serde_bytes::ByteBuf added in, this way
/// we can deserialize the parameters correctly.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Value {
  Null,
  Bool(bool),
  Number(Number),
  String(String),
  Bytes(ByteBuf),
  Array(Vec<Value>),
  Object(HashMap<String, Value>),
}

impl Default for Value {
  fn default() -> Self {
    Value::Null
  }
}
