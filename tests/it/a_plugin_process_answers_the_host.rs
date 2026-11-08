use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use rig_plugin_sdk::{
    CallError, Granted, InitializeParams, InitializeResult, METHOD_NOT_FOUND, MOST_MESSAGE_BYTES,
    PARSE_ERROR, PROTOCOL, Session, TOO_LARGE, UNSUPPORTED_PROTOCOL, method,
};
use serde_json::{Value, json};

struct Plugin {
    child: Child,
    session: Session<BufReader<ChildStdout>, ChildStdin>,
}

fn start() -> Result<Plugin, Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rig-plugin-echo"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.take().ok_or("the plugin has no stdin")?;
    let stdout = child.stdout.take().ok_or("the plugin has no stdout")?;
    Ok(Plugin {
        child,
        session: Session::new(BufReader::new(stdout), stdin),
    })
}

fn hello(protocol: u32) -> InitializeParams {
    InitializeParams {
        protocol,
        rig_one_version: "1.15.4".to_owned(),
        language: "en".to_owned(),
        granted: Granted::default(),
    }
}

fn code(error: &CallError) -> Option<i64> {
    match error {
        CallError::Refused { code, .. } => Some(*code),
        _ => None,
    }
}

#[test]
fn the_plugin_says_who_it_is_answers_and_leaves_when_asked() -> Result<(), Box<dyn Error>> {
    let mut plugin = start()?;

    let answer = plugin
        .session
        .call(method::INITIALIZE, serde_json::to_value(hello(PROTOCOL))?)?;
    let who: InitializeResult = serde_json::from_value(answer)?;
    assert_eq!(who.protocol, PROTOCOL);
    assert_eq!(who.plugin, "echo");

    let echoed = plugin.session.call("echo", json!({ "say": "hi" }))?;
    assert_eq!(echoed, json!({ "say": "hi" }));

    let unknown = plugin.session.call("rigone/future-thing", Value::Null);
    assert_eq!(
        unknown.as_ref().err().and_then(code),
        Some(METHOD_NOT_FOUND)
    );
    assert_eq!(
        plugin.session.call("echo", json!(2))?,
        json!(2),
        "an unknown method must not end the plugin"
    );

    plugin.session.call(method::SHUTDOWN, Value::Null)?;
    let status = plugin.child.wait()?;
    assert!(status.success(), "{status}");
    Ok(())
}

#[test]
fn the_plugin_refuses_a_protocol_it_does_not_speak() -> Result<(), Box<dyn Error>> {
    let mut plugin = start()?;

    let refused = plugin.session.call(
        method::INITIALIZE,
        serde_json::to_value(hello(PROTOCOL + 1))?,
    );

    assert_eq!(
        refused.as_ref().err().and_then(code),
        Some(UNSUPPORTED_PROTOCOL)
    );
    plugin.child.kill()?;
    plugin.child.wait()?;
    Ok(())
}

#[test]
fn a_broken_or_huge_line_gets_an_error_and_the_plugin_keeps_going() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rig-plugin-echo"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("the plugin has no stdin")?;
    let mut stdout = BufReader::new(child.stdout.take().ok_or("the plugin has no stdout")?);
    let mut read = || -> Result<Value, Box<dyn Error>> {
        let mut line = String::new();
        stdout.read_line(&mut line)?;
        Ok(serde_json::from_str(&line)?)
    };

    stdin.write_all(b"{ this is not json\n")?;
    stdin.flush()?;
    assert_eq!(read()?["error"]["code"], json!(PARSE_ERROR));

    let huge = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"echo\",\"params\":\"{}\"}}\n",
        "x".repeat(MOST_MESSAGE_BYTES)
    );
    stdin.write_all(huge.as_bytes())?;
    stdin.flush()?;
    assert_eq!(read()?["error"]["code"], json!(TOO_LARGE));

    stdin.write_all(
        b"{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"echo\",\"params\":\"still here\"}\n",
    )?;
    stdin.flush()?;
    let reply = read()?;
    assert_eq!(reply["id"], json!(7));
    assert_eq!(reply["result"], json!("still here"));

    drop(stdin);
    let status = child.wait()?;
    assert!(
        status.success(),
        "closing stdin ends the plugin cleanly: {status}"
    );
    Ok(())
}
