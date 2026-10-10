//! Plugins compiled into this build and the runtimes for script plugins.
//! First-party plugin crates live under `plugins/`; add one line per plugin
//! to [`bundled`].

use jetthing_ui::{PluginFactory, PluginSet};
use std::rc::Rc;

pub fn bundled() -> Vec<PluginFactory> {
    #[allow(unused_mut)]
    let mut plugins: Vec<PluginFactory> = Vec::new();
    #[cfg(feature = "example-plugins")]
    plugins.push(Rc::new(|| {
        Box::new(jetthing_plugin_hello::HelloPlugin::new())
    }));
    plugins
}

pub fn plugin_set() -> PluginSet {
    PluginSet {
        bundled: bundled(),
        loaders: vec![Rc::new(jetthing_plugin_lua::LuaLoader)],
    }
}
