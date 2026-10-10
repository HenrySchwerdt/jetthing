//! JetBrains Marketplace integration: browse and search
//! `plugins.jetbrains.com`, inspect a downloaded plugin, and convert plugins
//! that need no IntelliJ Platform code (UI themes, editor color schemes,
//! keymaps, live templates, TextMate grammars) into declarative Jetthing
//! plugins. Everything else is reported as unsupported, with the reason.

pub mod api;
pub mod convert;
pub mod inspect;

pub use api::{MarketplaceClient, PluginDetails, PluginUpdate, SearchPage, SearchPlugin};
pub use convert::{Converted, convert};
pub use inspect::{FoundAsset, Inspection, Verdict, inspect};

use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MarketplaceError {
    #[error("JetBrains Marketplace is not reachable: {0}")]
    Network(String),
    #[error("unexpected Marketplace response: {0}")]
    InvalidResponse(String),
    #[error("plugin download is too large ({0} bytes)")]
    TooLarge(u64),
    #[error("invalid plugin: {0}")]
    InvalidPlugin(String),
    #[error("{0}")]
    Unsupported(String),
}

/// Guess from Marketplace tags, shown before anything is downloaded.
pub fn tag_hint(tags: &[String]) -> Option<&'static str> {
    let has = |name: &str| tags.iter().any(|tag| tag.eq_ignore_ascii_case(name));
    if has("Theme") {
        Some("Likely installable: UI theme")
    } else if has("Editor Color Schemes") {
        Some("Likely installable: editor color scheme")
    } else if has("Keymap") {
        Some("Likely installable: keymap")
    } else if has("Live Templates") {
        Some("May be installable: live templates")
    } else {
        None
    }
}

/// Downloads the latest version of `id`, inspects it and converts it.
pub fn fetch_and_convert(
    client: &MarketplaceClient,
    id: u64,
) -> Result<Converted, MarketplaceError> {
    let details = client.details(id)?;
    let update = client.latest_update(id)?;
    let bytes = client.download(&update)?;
    let inspection = inspect(&bytes)?;
    convert(&inspection, &details, &update.version, client.base_url())
}

#[cfg(test)]
mod tests;
