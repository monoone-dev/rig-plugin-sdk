use std::io::{self, BufReader};
use std::process::ExitCode;

use rig_plugin_sdk::{Answer, PluginInfo, serve};

fn main() -> ExitCode {
    let info = PluginInfo {
        id: "echo".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    };
    let served = serve(
        BufReader::new(io::stdin()),
        io::stdout(),
        &info,
        |method, params| match method {
            "echo" => Answer::Done(params),
            _ => Answer::Unknown,
        },
    );
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
