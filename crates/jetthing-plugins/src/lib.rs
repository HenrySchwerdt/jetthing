use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use thiserror::Error;

pub const PLUGIN_API_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin manifest IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid plugin manifest: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("plugin {0} requires unsupported API version {1}")]
    UnsupportedApi(String, u32),
    #[error("plugin id must use reverse-domain syntax: {0}")]
    InvalidId(String),
}

impl PluginManifest {
    pub fn parse(input: &str) -> Result<Self, PluginError> {
        let manifest: Self = toml::from_str(input)?;
        if manifest.api_version != PLUGIN_API_VERSION {
            return Err(PluginError::UnsupportedApi(
                manifest.id,
                manifest.api_version,
            ));
        }
        if !manifest.id.contains('.')
            || manifest
                .id
                .chars()
                .any(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')))
        {
            return Err(PluginError::InvalidId(manifest.id));
        }
        Ok(manifest)
    }
}

pub fn discover(root: &Path) -> Vec<Result<PluginManifest, PluginError>> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path().join("plugin.toml");
            path.is_file().then(|| {
                fs::read_to_string(path)
                    .map_err(PluginError::from)
                    .and_then(|s| PluginManifest::parse(&s))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_manifest_contract() {
        let manifest = PluginManifest::parse("id='dev.jetthing.rust'\nname='Rust'\nversion='0.1.0'\napi_version=1\ncapabilities=['language']\n").unwrap();
        assert_eq!(manifest.capabilities, ["language"]);
        assert!(
            PluginManifest::parse("id='bad'\nname='Bad'\nversion='1'\napi_version=1\n").is_err()
        );
    }
}
