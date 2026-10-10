//! Converts a data-only IntelliJ plugin into a declarative Jetthing plugin
//! archive (`.jtplugin`) whose `[[assets]]` list its themes, color schemes,
//! keymaps, live templates and TextMate grammars.

use crate::inspect::{FoundAsset, Inspection, Verdict};
use crate::{MarketplaceError, api::PluginDetails};
use jetthing_plugin_api::{AssetDecl, AssetKind, PluginKind, PluginManifest, valid_id};
use std::io::Write;

/// Jetthing plugin id for a Marketplace plugin id (`xmlId`).
pub fn plugin_id(xml_id: &str, marketplace_id: u64) -> String {
    let cleaned: String = xml_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches(|c| c == '.' || c == '-')
        .to_owned();
    if valid_id(&cleaned) {
        cleaned
    } else if !cleaned.is_empty() && valid_id(&format!("com.jetbrains.marketplace.{cleaned}")) {
        format!("com.jetbrains.marketplace.{cleaned}")
    } else {
        format!("com.jetbrains.marketplace.plugin-{marketplace_id}")
    }
}

/// Plain text from Marketplace HTML descriptions.
pub fn html_to_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if matches!(
                    name.as_str(),
                    "p" | "br" | "br/" | "li" | "div" | "h1" | "h2" | "h3" | "ul" | "ol"
                ) {
                    text.push('\n');
                    if name == "li" && !tag.starts_with('/') {
                        text.push_str("• ");
                    }
                }
            }
            _ if in_tag => tag.push(c),
            _ => text.push(c),
        }
    }
    let decoded = text
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    decoded
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn asset_dir(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Theme => "themes",
        AssetKind::ColorScheme => "schemes",
        AssetKind::Keymap => "keymaps",
        AssetKind::LiveTemplates => "templates",
        AssetKind::TextmateBundle => "textmate",
    }
}

/// Result of a conversion: the manifest and the `.jtplugin` bytes.
pub struct Converted {
    pub manifest: PluginManifest,
    pub archive: Vec<u8>,
    pub assets: Vec<FoundAsset>,
}

/// Builds a `.jtplugin` from an inspected plugin, or explains why not.
pub fn convert(
    inspection: &Inspection,
    details: &PluginDetails,
    version: &str,
    base_url: &str,
) -> Result<Converted, MarketplaceError> {
    let assets = match inspection.verdict() {
        Verdict::Convertible(assets) => assets,
        Verdict::Unsupported(reason) => return Err(MarketplaceError::Unsupported(reason)),
    };
    let xml_id = if details.xml_id.is_empty() {
        &inspection.plugin.id
    } else {
        &details.xml_id
    };
    let id = plugin_id(xml_id, details.id);
    let mut files: Vec<(String, &[u8])> = Vec::new();
    let mut declarations = Vec::new();
    for asset in &assets {
        let Some(bytes) = inspection.resources.get(&asset.path) else {
            continue;
        };
        let file = asset.path.rsplit('/').next().unwrap_or(&asset.path);
        let target = format!("assets/{}/{}", asset_dir(asset.kind), file);
        if files.iter().any(|(path, _)| *path == target) {
            continue;
        }
        declarations.push(AssetDecl {
            kind: asset.kind,
            path: target.clone(),
            name: Some(asset.name.clone()),
        });
        files.push((target, bytes.as_slice()));
    }
    let mut kinds: Vec<&str> = declarations.iter().map(|a| a.kind.label()).collect();
    kinds.dedup();
    let description = {
        let text = html_to_text(if details.description.is_empty() {
            &inspection.plugin.description
        } else {
            &details.description
        });
        let short: String = text.chars().take(2000).collect();
        let license = details
            .urls
            .license_url
            .as_deref()
            .filter(|url| !url.is_empty())
            .map(|url| format!(" License: {url}"))
            .unwrap_or_default();
        format!(
            "{short}\n\nConverted from JetBrains Marketplace plugin {} ({}).{license}",
            details.name,
            details.page_url(base_url)
        )
    };
    let vendor = details
        .vendor
        .as_ref()
        .map(|vendor| vendor.display().to_owned())
        .unwrap_or_else(|| inspection.plugin.vendor.clone());
    let manifest = PluginManifest {
        id,
        name: details.name.clone(),
        version: version.to_owned(),
        api_version: jetthing_plugin_api::API_VERSION,
        description,
        vendor,
        license: None,
        homepage: Some(details.homepage(base_url)),
        kind: PluginKind::Declarative,
        entry: None,
        permissions: Vec::new(),
        capabilities: std::iter::once("jetbrains-marketplace".to_owned())
            .chain(kinds.iter().map(|kind| kind.replace(' ', "-")))
            .collect(),
        dependencies: Vec::new(),
        app: None,
        assets: declarations,
    };
    manifest
        .validate()
        .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
    let toml = toml::to_string(&manifest)
        .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let zip_error =
        |error: zip::result::ZipError| MarketplaceError::InvalidPlugin(error.to_string());
    writer
        .start_file(jetthing_plugin_api::MANIFEST_FILE, options)
        .map_err(zip_error)?;
    writer
        .write_all(toml.as_bytes())
        .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
    for (path, bytes) in files {
        writer.start_file(path, options).map_err(zip_error)?;
        writer
            .write_all(bytes)
            .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
    }
    let archive = writer.finish().map_err(zip_error)?.into_inner();
    Ok(Converted {
        manifest,
        archive,
        assets,
    })
}
