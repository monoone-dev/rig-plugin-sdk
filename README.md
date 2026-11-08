# rig-plugin-sdk

Write plugins for RigOne in Rust. A plugin can be a native program (MonoOne's own plugins) or a
WebAssembly component (every other publisher). Both speak the same protocol to RigOne.

## A WebAssembly plugin in four commands

```bash
rustup target add wasm32-wasip2
rigone plugin new hello-desk
rigone plugin build hello-desk
rigone plugin keygen ~/.rigone/publisher.key
rigone plugin sign hello-desk/dist/hello-desk-0.1.0-wasm.rigplugin --key ~/.rigone/publisher.key
```

`new` makes a project with `Cargo.toml`, `manifest.json` and `src/lib.rs`. `build` compiles it to
a component, puts it next to its manifest in `dist/<id>/` and packs both into
`dist/<id>-<version>-wasm.rigplugin`. `sign` prints the text MonoOne checks and your signature of
it. Keep the key file private; RigOne never overwrites one.

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
