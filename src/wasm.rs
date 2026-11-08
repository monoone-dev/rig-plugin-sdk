use serde_json::{Value, json};

use super::model::{Granted, Tool, TriggerItem};

#[allow(clippy::all, clippy::pedantic)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "plugin",
        pub_export_macro: true,
        export_macro_name: "export_guest",
        default_bindings_module: "rig_plugin_sdk::wasm::bindings",
    });
}

pub use bindings::rigone::plugin::host;

pub trait WasmPlugin {
    fn initialize(_granted: &Granted) -> Result<(), String> {
        Ok(())
    }

    #[must_use]
    fn settings() -> Value {
        json!({ "fields": [] })
    }

    fn apply_settings(_values: Value) -> Result<(), String> {
        Ok(())
    }

    fn trigger_items(_source: &str, _condition: &str) -> Result<Vec<TriggerItem>, String> {
        Ok(Vec::new())
    }

    #[must_use]
    fn tools() -> Vec<Tool> {
        Vec::new()
    }

    fn call_tool(name: &str, _arguments: Value) -> Result<String, String> {
        Err(format!("There is no tool called {name}."))
    }
}

pub fn adapt_initialize<T: WasmPlugin>(granted: &str) -> Result<(), String> {
    T::initialize(&serde_json::from_str(granted).unwrap_or_default())
}

pub fn adapt_settings_describe<T: WasmPlugin>() -> Result<String, String> {
    Ok(T::settings().to_string())
}

pub fn adapt_settings_apply<T: WasmPlugin>(values: &str) -> Result<(), String> {
    T::apply_settings(serde_json::from_str(values).unwrap_or(Value::Null))
}

pub fn adapt_trigger_items<T: WasmPlugin>(source: &str, condition: &str) -> Result<String, String> {
    serde_json::to_string(&T::trigger_items(source, condition)?).map_err(|error| error.to_string())
}

#[must_use]
pub fn adapt_tools_list<T: WasmPlugin>() -> String {
    serde_json::to_string(&T::tools()).unwrap_or_else(|_| "[]".to_owned())
}

pub fn adapt_tools_call<T: WasmPlugin>(name: &str, arguments: &str) -> Result<String, String> {
    T::call_tool(name, serde_json::from_str(arguments).unwrap_or(Value::Null))
}

#[macro_export]
macro_rules! export_plugin {
    ($plugin:ty) => {
        struct RigOneExport;

        impl $crate::wasm::bindings::exports::rigone::plugin::guest::Guest for RigOneExport {
            fn initialize(granted: String) -> Result<(), String> {
                $crate::wasm::adapt_initialize::<$plugin>(&granted)
            }

            fn settings_describe() -> Result<String, String> {
                $crate::wasm::adapt_settings_describe::<$plugin>()
            }

            fn settings_apply(values: String) -> Result<(), String> {
                $crate::wasm::adapt_settings_apply::<$plugin>(&values)
            }

            fn trigger_items(source: String, condition: String) -> Result<String, String> {
                $crate::wasm::adapt_trigger_items::<$plugin>(&source, &condition)
            }

            fn tools_list() -> String {
                $crate::wasm::adapt_tools_list::<$plugin>()
            }

            fn tools_call(name: String, arguments: String) -> Result<String, String> {
                $crate::wasm::adapt_tools_call::<$plugin>(&name, &arguments)
            }
        }

        $crate::wasm::bindings::export_guest!(RigOneExport with_types_in $crate::wasm::bindings);
    };
}
