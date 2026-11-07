use std::io::{self, BufRead, Write};

use serde::Serialize;
use serde_json::Value;

use super::logic::{Incoming, answered, incoming, initialized, request, response};
use super::model::{
    Answer, CallError, ErrorObject, MOST_MESSAGE_BYTES, PluginInfo, Response, TOO_LARGE, method,
};

enum Line {
    Fits(Vec<u8>),
    TooLong,
}

fn next_line<R: BufRead>(reader: &mut R) -> io::Result<Option<Line>> {
    let mut kept = Vec::new();
    let mut too_long = false;
    let mut read_any = false;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            let line = if too_long {
                Line::TooLong
            } else {
                Line::Fits(kept)
            };
            return Ok(read_any.then_some(line));
        }
        read_any = true;
        let end = available.iter().position(|byte| *byte == b'\n');
        let chunk = &available[..end.unwrap_or(available.len())];
        if !too_long {
            if kept.len() + chunk.len() > MOST_MESSAGE_BYTES {
                too_long = true;
                kept.clear();
            } else {
                kept.extend_from_slice(chunk);
            }
        }
        let used = end.map_or(available.len(), |at| at + 1);
        reader.consume(used);
        if end.is_some() {
            return Ok(Some(if too_long {
                Line::TooLong
            } else {
                Line::Fits(kept)
            }));
        }
    }
}

fn send<W: Write>(writer: &mut W, message: &impl Serialize) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(message).map_err(io::Error::other)?;
    bytes.push(b'\n');
    writer.write_all(&bytes)?;
    writer.flush()
}

pub fn serve<R, W, F>(
    mut reader: R,
    mut writer: W,
    info: &PluginInfo,
    mut answer: F,
) -> io::Result<()>
where
    R: BufRead,
    W: Write,
    F: FnMut(&str, Value) -> Answer,
{
    while let Some(line) = next_line(&mut reader)? {
        let bytes = match line {
            Line::TooLong => {
                let refusal = ErrorObject {
                    code: TOO_LARGE,
                    message: format!("A message may be at most {MOST_MESSAGE_BYTES} bytes."),
                };
                send(&mut writer, &response(Value::Null, Err(refusal)))?;
                continue;
            }
            Line::Fits(bytes) => bytes,
        };
        if bytes.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        match incoming(&bytes) {
            Incoming::Note => {}
            Incoming::Broken { id, code, message } => {
                send(
                    &mut writer,
                    &response(id, Err(ErrorObject { code, message })),
                )?;
            }
            Incoming::Call { id, method, params } => {
                if method == method::SHUTDOWN {
                    send(&mut writer, &response(id, Ok(Value::Null)))?;
                    return Ok(());
                }
                let outcome = if method == method::INITIALIZE {
                    initialized(info, params)
                } else {
                    answered(answer(&method, params), &method)
                };
                send(&mut writer, &response(id, outcome))?;
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Session<R, W> {
    reader: R,
    writer: W,
    next_id: u64,
}

impl<R: BufRead, W: Write> Session<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            next_id: 1,
        }
    }

    pub fn call(&mut self, method: &str, params: Value) -> Result<Value, CallError> {
        let id = self.next_id;
        self.next_id += 1;
        send(&mut self.writer, &request(id, method, params))?;
        loop {
            let bytes = match next_line(&mut self.reader)? {
                None => return Err(CallError::Closed),
                Some(Line::TooLong) => {
                    return Err(CallError::Malformed(format!(
                        "a reply longer than {MOST_MESSAGE_BYTES} bytes"
                    )));
                }
                Some(Line::Fits(bytes)) => bytes,
            };
            let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
                return Err(CallError::Malformed(
                    String::from_utf8_lossy(&bytes).into_owned(),
                ));
            };
            if value.get("method").is_some() {
                continue;
            }
            let reply: Response = serde_json::from_value(value)
                .map_err(|error| CallError::Malformed(error.to_string()))?;
            if reply.id != id {
                continue;
            }
            return match reply.error {
                Some(error) => Err(CallError::Refused {
                    code: error.code,
                    message: error.message,
                }),
                None => Ok(reply.result.unwrap_or(Value::Null)),
            };
        }
    }
}
