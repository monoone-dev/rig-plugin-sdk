use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL: u32 = 1;

pub const MOST_MESSAGE_BYTES: usize = 1 << 20;

pub const PARSE_ERROR: i64 = -32_700;
pub const INVALID_REQUEST: i64 = -32_600;
pub const METHOD_NOT_FOUND: i64 = -32_601;
pub const FAILED: i64 = -32_000;
pub const UNSUPPORTED_PROTOCOL: i64 = -32_002;
pub const TOO_LARGE: i64 = -32_003;

pub mod method {
    pub const INITIALIZE: &str = "rigone/initialize";
    pub const SHUTDOWN: &str = "rigone/shutdown";
    pub const SETTINGS_DESCRIBE: &str = "rigone/settings/describe";
    pub const SETTINGS_APPLY: &str = "rigone/settings/apply";
    pub const TRIGGER_ITEMS: &str = "rigone/trigger/items";
    pub const LOG: &str = "rigone/log";
    pub const NOTIFY: &str = "rigone/notify";
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Granted {
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub secrets: Vec<String>,
    #[serde(default)]
    pub data: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol: u32,
    pub rig_one_version: String,
    pub language: String,
    #[serde(default)]
    pub granted: Granted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol: u32,
    pub plugin: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Done(Value),
    Unknown,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Request {
    pub(crate) jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<Value>,
    pub(crate) method: String,
    #[serde(default)]
    pub(crate) params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ErrorObject {
    pub(crate) code: i64,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Response {
    pub(crate) jsonrpc: String,
    pub(crate) id: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<ErrorObject>,
}

#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("the plugin refused the call ({code}): {message}")]
    Refused { code: i64, message: String },
    #[error("the plugin closed its output before it answered")]
    Closed,
    #[error("the plugin answered with something that is not a reply: {0}")]
    Malformed(String),
    #[error("the plugin could not be reached: {0}")]
    Io(#[from] std::io::Error),
}
