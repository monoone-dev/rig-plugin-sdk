use std::io::{self, BufReader};
use std::process::ExitCode;

use rig_plugin_sdk::{Answer, PluginInfo, TOOLS_ARGUMENT, Tool, serve, serve_tools};

fn main() -> ExitCode {
    let info = PluginInfo {
        id: "echo".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    };
    let served = if std::env::args().nth(1).as_deref() == Some(TOOLS_ARGUMENT) {
        let tools = [Tool {
            name: "echo".to_owned(),
            description: "Says the text back.".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"]
            }),
        }];
        serve_tools(
            BufReader::new(io::stdin()),
            io::stdout(),
            &info,
            &tools,
            |name, arguments| match name {
                "echo" => arguments["text"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "echo needs text".to_owned()),
                _ => Err(format!("There is no tool called {name}.")),
            },
        )
    } else {
        serve(
            BufReader::new(io::stdin()),
            io::stdout(),
            &info,
            |method, params| match method {
                "echo" => Answer::Done(params),
                _ => Answer::Unknown,
            },
        )
    };
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
