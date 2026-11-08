# rig-plugin-sdk

Write plugins for RigOne in Rust. A plugin can be a native program (MonoOne's own plugins) or a
WebAssembly component (every other publisher). Both speak the same protocol to RigOne.

## Add it to a plugin

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
rig-plugin-sdk = { version = "0.1", features = ["wasm"] }
```

Build the plugin as a WebAssembly component:

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

Put the `.wasm` file next to a `manifest.json` (`"format": "wasm"`, `"entry": "plugin.wasm"`). The
`rigone plugin` command that makes, builds, packs and signs a plugin in one step is coming to this
repository; until then RigOne's team publishes plugins for you.

## What a plugin implements

```rust
use rig_plugin_sdk::wasm::{WasmPlugin, host};
use rig_plugin_sdk::{Tool, TriggerItem};

struct Plugin;

impl WasmPlugin for Plugin {
    fn trigger_items(source: &str, condition: &str) -> Result<Vec<TriggerItem>, String> {
        Ok(Vec::new())
    }
}

rig_plugin_sdk::export_plugin!(Plugin);
```

Every method has a default, so implement only what your manifest's `provides` names:

| Method | Called when |
|---|---|
| `initialize` | before every call, with the permissions RigOne granted |
| `settings`, `apply_settings` | the plugin page shows and saves settings |
| `trigger_items` | a trigger that uses one of your `triggerSources` checks for new items |
| `tools`, `call_tool` | an agent lists or calls your tools (`agentTools`) |

## What a plugin may use

Only what RigOne gives it, through `host`:

| Call | Limit |
|---|---|
| `host::fetch` | HTTPS only, only to hosts in `permissions.network` |
| `host::secret` | only names in `permissions.secrets` |
| `host::data_read`, `data_write`, `data_list` | only the plugin's own folder, up to 64 MB |
| `host::log`, `host::notify` | a line for the log, a sentence for the person |
| `host::now_ms` | the time |

A plugin cannot read other files, start programs, open network connections itself or read
environment variables. One call may use 128 MB of memory and 10 seconds; after that RigOne stops
it and says so. The interface is `wit/plugin.wit` (`rigone:plugin@1`).

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option. A plugin built with this SDK may use any license.
