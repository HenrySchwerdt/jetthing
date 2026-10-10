//! Client for the public JetBrains Marketplace REST API
//! (`https://plugins.jetbrains.com/api`). Only read endpoints are used and
//! plugin files are downloaded only on an explicit user action.

use crate::MarketplaceError;
use serde::Deserialize;
use std::io::Read;
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "https://plugins.jetbrains.com";
/// Largest plugin file Jetthing downloads.
pub const MAX_DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    #[serde(default)]
    pub plugins: Vec<SearchPlugin>,
    #[serde(default)]
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VendorRef {
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "publicName")]
    pub public_name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

impl VendorRef {
    /// Display name (`publicName` when the API provides one).
    pub fn display(&self) -> &str {
        self.public_name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or(&self.name)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchPlugin {
    pub id: u64,
    #[serde(default)]
    pub xml_id: String,
    pub name: String,
    #[serde(default)]
    pub preview: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub pricing_model: Option<String>,
    #[serde(default)]
    pub vendor: Option<VendorRef>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Urls {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub source_code_url: Option<String>,
    #[serde(default)]
    pub license_url: Option<String>,
    #[serde(default)]
    pub doc_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginDetails {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub xml_id: String,
    /// HTML description.
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub vendor: Option<VendorRef>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub urls: Urls,
}

impl PluginDetails {
    pub fn page_url(&self, base: &str) -> String {
        format!("{}{}", base.trim_end_matches('/'), self.link)
    }

    /// Homepage: the plugin URL, its source, or its Marketplace page.
    pub fn homepage(&self, base: &str) -> String {
        [&self.urls.url, &self.urls.source_code_url]
            .into_iter()
            .flatten()
            .find(|url| !url.is_empty())
            .cloned()
            .unwrap_or_else(|| self.page_url(base))
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PluginUpdate {
    pub id: u64,
    pub version: String,
    /// Path below `/files/`.
    pub file: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub until: Option<String>,
}

/// Largest page size the search endpoint accepts.
pub const MAX_PAGE: usize = 20;

/// Network error with the server's message for HTTP errors.
fn http_error(url: &str, error: ureq::Error) -> MarketplaceError {
    match error {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            let message = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|json| json.get("message").and_then(|m| m.as_str()).map(str::to_owned))
                .unwrap_or_else(|| body.chars().take(200).collect());
            MarketplaceError::InvalidResponse(format!("{url}: HTTP {code}: {message}"))
        }
        ureq::Error::Transport(transport) => MarketplaceError::Network(transport.to_string()),
    }
}

/// Blocking client; call it off the UI thread.
#[derive(Debug, Clone)]
pub struct MarketplaceClient {
    base: String,
    agent: ureq::Agent,
}

impl Default for MarketplaceClient {
    fn default() -> Self {
        Self::new(
            std::env::var("JETTHING_MARKETPLACE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
        )
    }
}

fn query_escape(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

impl MarketplaceClient {
    pub fn new(base: impl Into<String>) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(15))
            .timeout_read(Duration::from_secs(60))
            .user_agent(concat!("Jetthing/", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            base: base.into().trim_end_matches('/').to_owned(),
            agent,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, MarketplaceError> {
        let url = format!("{}{path}", self.base);
        let response = self.agent.get(&url).call().map_err(|error| http_error(&url, error))?;
        response
            .into_json()
            .map_err(|error| MarketplaceError::InvalidResponse(format!("{url}: {error}")))
    }

    /// Searches plugins; an empty query lists the most downloaded ones.
    pub fn search(
        &self,
        query: &str,
        max: usize,
        offset: usize,
    ) -> Result<SearchPage, MarketplaceError> {
        let max = max.clamp(1, MAX_PAGE);
        let mut path = format!("/api/searchPlugins?max={max}&offset={offset}");
        if query.trim().is_empty() {
            path.push_str("&orderBy=downloads");
        } else {
            path.push_str("&search=");
            path.push_str(&query_escape(query.trim()));
        }
        self.get_json(&path)
    }

    pub fn details(&self, id: u64) -> Result<PluginDetails, MarketplaceError> {
        self.get_json(&format!("/api/plugins/{id}"))
    }

    pub fn latest_update(&self, id: u64) -> Result<PluginUpdate, MarketplaceError> {
        let updates: Vec<PluginUpdate> =
            self.get_json(&format!("/api/plugins/{id}/updates?size=1"))?;
        updates.into_iter().next().ok_or_else(|| {
            MarketplaceError::InvalidResponse(format!("plugin {id} has no published versions"))
        })
    }

    /// Downloads a plugin file (bounded by [`MAX_DOWNLOAD_BYTES`]).
    pub fn download(&self, update: &PluginUpdate) -> Result<Vec<u8>, MarketplaceError> {
        if update.size > MAX_DOWNLOAD_BYTES {
            return Err(MarketplaceError::TooLarge(update.size));
        }
        if update.file.contains("..") || update.file.starts_with('/') {
            return Err(MarketplaceError::InvalidResponse(format!(
                "unexpected file path {}",
                update.file
            )));
        }
        let url = format!("{}/files/{}", self.base, update.file.replace(' ', "%20"));
        let response = self.agent.get(&url).call().map_err(|error| http_error(&url, error))?;
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(MAX_DOWNLOAD_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| MarketplaceError::Network(format!("{url}: {error}")))?;
        if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
            return Err(MarketplaceError::TooLarge(bytes.len() as u64));
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recorded_responses() {
        let page: SearchPage =
            serde_json::from_str(include_str!("../tests/fixtures/search.json")).unwrap();
        assert!(page.total > 0);
        assert_eq!(page.plugins[0].name, "One Dark Theme");
        assert!(page.plugins[0].tags.iter().any(|tag| tag == "Theme"));
        let details: PluginDetails =
            serde_json::from_str(include_str!("../tests/fixtures/details.json")).unwrap();
        assert_eq!(details.xml_id, "com.markskelton.one-dark-theme");
        assert_eq!(details.vendor.as_ref().unwrap().display(), "Mark Skelton");
        assert!(details.homepage(DEFAULT_BASE_URL).contains("github.com"));
        let updates: Vec<PluginUpdate> =
            serde_json::from_str(include_str!("../tests/fixtures/updates.json")).unwrap();
        assert_eq!(updates[0].version, "6.2.5");
        assert!(updates[0].file.ends_with(".zip"));
    }

    #[test]
    fn escapes_queries() {
        assert_eq!(query_escape("one dark&x=1"), "one%20dark%26x%3D1");
    }
}
