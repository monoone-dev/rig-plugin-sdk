use std::error::Error;
use std::io::BufReader;
use std::process::{Command, Stdio};

use rig_plugin_sdk::Session;
use serde_json::json;

#[test]
fn an_agent_lists_the_plugin_tools_and_calls_one() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rig-plugin-echo"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.take().ok_or("the plugin has no stdin")?;
    let stdout = child.stdout.take().ok_or("the plugin has no stdout")?;
    let mut agent = Session::new(BufReader::new(stdout), stdin);

    let hello = agent.call(
        "initialize",
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "claude", "version": "2" } }),
    )?;
    assert_eq!(hello["serverInfo"]["name"], "echo");
    assert!(hello["capabilities"]["tools"].is_object());

    let listed = agent.call("tools/list", json!({}))?;
    assert_eq!(listed["tools"][0]["name"], "echo");
    assert_eq!(listed["tools"][0]["inputSchema"]["type"], "object");

    let said = agent.call(
        "tools/call",
        json!({ "name": "echo", "arguments": { "text": "hi there" } }),
    )?;
    assert_eq!(said["content"][0]["text"], "hi there");
    assert_eq!(said["isError"], false);

    let refused = agent.call("tools/call", json!({ "name": "nope", "arguments": {} }))?;
    assert_eq!(refused["isError"], true);

    drop(agent);
    child.wait()?;
    Ok(())
}
