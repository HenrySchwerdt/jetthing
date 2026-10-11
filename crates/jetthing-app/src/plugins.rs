//! Plugins compiled into this build and the runtimes for script plugins.
//! First-party plugin crates live under `plugins/`; add one line per plugin
//! to [`bundled`].

use jetthing_ui::{PluginFactory, PluginSet};
use std::rc::Rc;

#[allow(clippy::vec_init_then_push)] // pushes depend on enabled features
pub fn bundled() -> Vec<PluginFactory> {
    #[allow(unused_mut)]
    let mut plugins: Vec<PluginFactory> = Vec::new();
    #[cfg(feature = "mongo-raabit")]
    plugins.push(Rc::new(|| Box::new(mongo_raabit::MongoRaabitPlugin::new())));
    #[cfg(feature = "example-plugins")]
    plugins.push(Rc::new(|| {
        Box::new(jetthing_plugin_hello::HelloPlugin::new())
    }));
    plugins
}

pub fn plugin_set() -> PluginSet {
    let mut loaders: Vec<Rc<dyn jetthing_ui::PluginLoader>> =
        vec![Rc::new(jetthing_plugin_lua::LuaLoader)];
    match jetthing_plugin_wasm::WasmLoader::new() {
        Ok(loader) => loaders.push(Rc::new(loader)),
        // WASM plugins then show as unsupported with this reason in the log.
        Err(error) => eprintln!("jetthing: WASM plugin runtime unavailable: {error}"),
    }
    PluginSet {
        bundled: bundled(),
        loaders,
    }
}
