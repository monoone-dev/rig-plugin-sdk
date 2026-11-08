use serde_json::Value;

use super::model::{
    Answer, ErrorObject, FAILED, INVALID_REQUEST, InitializeParams, InitializeResult, MCP_PROTOCOL,
    METHOD_NOT_FOUND, PARSE_ERROR, PROTOCOL, PluginInfo, Request, Response, UNSUPPORTED_PROTOCOL,
};

pub(crate) const JSONRPC: &str = "2.0";

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Incoming {
    Call {
        id: Value,
        method: String,
        params: Value,
    },
    Note,
    Broken {
        id: Value,
        code: i64,
        message: String,
    },
}

pub(crate) fn incoming(line: &[u8]) -> Incoming {
    let Ok(value) = serde_json::from_slice::<Value>(line) else {
        return Incoming::Broken {
            id: Value::Null,
            code: PARSE_ERROR,
            message: "This line is not JSON.".to_owned(),
        };
    };
    let id = value.get("id").cloned().unwrap_or(Value::Null);
    match serde_json::from_value::<Request>(value) {
        Ok(request) if request.jsonrpc == JSONRPC => match request.id {
            Some(id) => Incoming::Call {
                id,
                method: request.method,
                params: request.params,
            },
            None => Incoming::Note,
        },
        _ => Incoming::Broken {
            id,
            code: INVALID_REQUEST,
            message: "This is not a JSON-RPC 2.0 request.".to_owned(),
        },
    }
}

pub(crate) fn initialized(info: &PluginInfo, params: Value) -> Result<Value, ErrorObject> {
    let asked: InitializeParams = serde_json::from_value(params).map_err(|_| ErrorObject {
        code: INVALID_REQUEST,
        message: "rigone/initialize needs protocol, rigOneVersion and language.".to_owned(),
    })?;
    if asked.protocol != PROTOCOL {
        return Err(ErrorObject {
            code: UNSUPPORTED_PROTOCOL,
            message: format!(
                "This plugin speaks protocol {PROTOCOL}, not {}.",
                asked.protocol
            ),
        });
    }
    serde_json::to_value(InitializeResult {
        protocol: PROTOCOL,
        plugin: info.id.clone(),
        version: info.version.clone(),
    })
    .map_err(|error| ErrorObject {
        code: FAILED,
        message: error.to_string(),
    })
}

pub(crate) fn answered(answer: Answer, method: &str) -> Result<Value, ErrorObject> {
    match answer {
        Answer::Done(value) => Ok(value),
        Answer::Unknown => Err(ErrorObject {
            code: METHOD_NOT_FOUND,
            message: format!("This plugin does not know {method}."),
        }),
        Answer::Failed(message) => Err(ErrorObject {
            code: FAILED,
            message,
        }),
    }
}

pub(crate) fn response(id: Value, outcome: Result<Value, ErrorObject>) -> Response {
    let (result, error) = match outcome {
        Ok(value) => (Some(value), None),
        Err(error) => (None, Some(error)),
    };
    Response {
        jsonrpc: JSONRPC.to_owned(),
        id,
        result,
        error,
    }
}

pub(crate) fn request(id: u64, method: &str, params: Value) -> Request {
    Request {
        jsonrpc: JSONRPC.to_owned(),
        id: Some(Value::from(id)),
        method: method.to_owned(),
        params,
    }
}

pub(crate) fn mcp_hello(info: &PluginInfo, params: &Value) -> Value {
    let protocol = params["protocolVersion"].as_str().unwrap_or(MCP_PROTOCOL);
    serde_json::json!({
        "protocolVersion": protocol,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": info.id, "version": info.version }
    })
}

pub(crate) fn tool_said(outcome: Result<String, String>) -> Value {
    let (text, is_error) = match outcome {
        Ok(text) => (text, false),
        Err(text) => (text, true),
    };
    serde_json::json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error
    })
}
