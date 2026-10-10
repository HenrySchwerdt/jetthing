//! Plugins compiled into this build. First-party plugin crates live under
//! `plugins/`; add one line per plugin here.

use jetthing_ui::PluginFactory;
use std::rc::Rc;

pub fn bundled() -> Vec<PluginFactory> {
    #[allow(unused_mut)]
    let mut plugins: Vec<PluginFactory> =
        vec![Rc::new(|| Box::new(mongo_raabit::MongoRaabitPlugin::new()))];
    #[cfg(feature = "example-plugins")]
    plugins.push(Rc::new(|| {
        Box::new(jetthing_plugin_hello::HelloPlugin::new())
    }));
    plugins
}
