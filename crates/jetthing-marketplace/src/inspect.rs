//! Inspection of a downloaded IntelliJ plugin: reads `META-INF/plugin.xml`
//! from the plugin jar(s) and decides whether Jetthing can use it without
//! the IntelliJ Platform (JVM).

use crate::MarketplaceError;
use jetthing_plugin_api::AssetKind;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::BTreeMap;
use std::io::{Cursor, Read};

const MAX_ENTRY_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ENTRIES: usize = 50_000;

/// What `plugin.xml` declares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginXml {
    pub id: String,
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub description: String,
    /// Resource assets declared by extension points: (kind, path in jar).
    pub declared: Vec<(AssetKind, String)>,
    /// Extension points and other elements that need JVM code.
    pub code_elements: Vec<String>,
}

/// A downloaded plugin, flattened: all non-class resources of its jars.
#[derive(Debug, Clone, Default)]
pub struct Inspection {
    pub plugin: PluginXml,
    pub class_count: usize,
    /// Resource path (without leading `/`) → bytes.
    pub resources: BTreeMap<String, Vec<u8>>,
}

/// A usable asset found in the plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundAsset {
    pub kind: AssetKind,
    /// Resource path inside the plugin jar.
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Data-only plugin; these assets can be installed.
    Convertible(Vec<FoundAsset>),
    /// Needs the IntelliJ Platform; the reason is shown to the user.
    Unsupported(String),
}

/// Resource-only extension points (`com.intellij` namespace) and the
/// attribute naming their file.
fn resource_extension(name: &str) -> Option<(AssetKind, &'static str)> {
    Some(match name {
        "themeProvider" => (AssetKind::Theme, "path"),
        "bundledColorScheme" => (AssetKind::ColorScheme, "path"),
        "bundledKeymap" => (AssetKind::Keymap, "file"),
        "defaultLiveTemplates" => (AssetKind::LiveTemplates, "file"),
        _ => return None,
    })
}

/// Elements under `<extensions>` that carry no behaviour.
fn harmless_extension(name: &str) -> bool {
    matches!(name, "themeMetadataProvider" | "iconMapper" | "pluginIcon")
}

fn read_zip(bytes: &[u8]) -> Result<zip::ZipArchive<Cursor<&[u8]>>, MarketplaceError> {
    zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| MarketplaceError::InvalidPlugin(format!("not a zip/jar archive: {error}")))
}

fn read_entry(file: &mut zip::read::ZipFile<'_>) -> Result<Vec<u8>, MarketplaceError> {
    let mut bytes = Vec::new();
    file.take(MAX_ENTRY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
    if bytes.len() as u64 > MAX_ENTRY_BYTES {
        return Err(MarketplaceError::InvalidPlugin(format!(
            "{} is too large",
            file.name()
        )));
    }
    Ok(bytes)
}

/// Adds the entries of a jar to `inspection`; returns its plugin.xml, if any.
fn scan_jar(
    bytes: &[u8],
    inspection: &mut Inspection,
) -> Result<Option<Vec<u8>>, MarketplaceError> {
    let mut jar = read_zip(bytes)?;
    if jar.len() > MAX_ENTRIES {
        return Err(MarketplaceError::InvalidPlugin(
            "archive has too many entries".into(),
        ));
    }
    let mut plugin_xml = None;
    for index in 0..jar.len() {
        let mut file = jar
            .by_index(index)
            .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
        if !file.is_file() {
            continue;
        }
        let Some(path) = file
            .enclosed_name()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
        else {
            continue;
        };
        if path.ends_with(".class") {
            inspection.class_count += 1;
            continue;
        }
        let data = read_entry(&mut file)?;
        if path == "META-INF/plugin.xml" {
            plugin_xml = Some(data.clone());
        }
        inspection.resources.insert(path, data);
    }
    Ok(plugin_xml)
}

/// Inspects a plugin download: a `.zip` with `<dir>/lib/*.jar`, or a `.jar`.
pub fn inspect(bytes: &[u8]) -> Result<Inspection, MarketplaceError> {
    let mut inspection = Inspection::default();
    let mut archive = read_zip(bytes)?;
    let is_jar = archive
        .file_names()
        .any(|name| name == "META-INF/plugin.xml");
    let mut plugin_xml = None;
    if is_jar {
        plugin_xml = scan_jar(bytes, &mut inspection)?;
    } else {
        if archive.len() > MAX_ENTRIES {
            return Err(MarketplaceError::InvalidPlugin(
                "archive has too many entries".into(),
            ));
        }
        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .map_err(|error| MarketplaceError::InvalidPlugin(error.to_string()))?;
            let name = file.name().to_owned();
            if !file.is_file() || !name.contains("/lib/") {
                continue;
            }
            if name.ends_with(".jar") {
                let jar = read_entry(&mut file)?;
                if let Some(xml) = scan_jar(&jar, &mut inspection)? {
                    plugin_xml.get_or_insert(xml);
                }
            } else if name.ends_with(".so") || name.ends_with(".dll") || name.ends_with(".dylib") {
                // Native libraries are code too.
                inspection.class_count += 1;
            }
        }
    }
    let xml = plugin_xml.ok_or_else(|| {
        MarketplaceError::InvalidPlugin("no META-INF/plugin.xml found in the plugin".into())
    })?;
    inspection.plugin = parse_plugin_xml(&String::from_utf8_lossy(&xml))?;
    Ok(inspection)
}

/// Records what `element` (child of `parent`) declares; returns the field
/// whose text content follows, if any.
fn record_element(
    plugin: &mut PluginXml,
    parent: Option<&str>,
    element: &quick_xml::events::BytesStart<'_>,
) -> Option<String> {
    let name = String::from_utf8_lossy(element.local_name().as_ref()).into_owned();
    let attribute = |key: &str| {
        element
            .attributes()
            .flatten()
            .find(|attribute| attribute.key.local_name().as_ref() == key.as_bytes())
            .and_then(|attribute| {
                attribute
                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                    .ok()
            })
            .map(|value| value.into_owned())
    };
    match (parent, name.as_str()) {
        (Some("extensions"), _) => {
            if let Some((kind, key)) = resource_extension(&name) {
                if let Some(path) = attribute(key) {
                    plugin
                        .declared
                        .push((kind, path.trim_start_matches('/').to_owned()));
                }
            } else if !harmless_extension(&name) {
                plugin.code_elements.push(name);
            }
            None
        }
        (Some("actions"), _) => {
            plugin.code_elements.push(format!("action `{name}`"));
            None
        }
        (
            Some("idea-plugin"),
            "applicationListeners"
            | "projectListeners"
            | "application-components"
            | "project-components"
            | "module-components",
        ) => {
            plugin.code_elements.push(name);
            None
        }
        (Some("idea-plugin"), "id" | "name" | "version" | "vendor" | "description") => Some(name),
        _ => None,
    }
}

fn set_text(plugin: &mut PluginXml, field: &str, value: String) {
    let target = match field {
        "id" => &mut plugin.id,
        "name" => &mut plugin.name,
        "version" => &mut plugin.version,
        "vendor" => &mut plugin.vendor,
        "description" => &mut plugin.description,
        _ => return,
    };
    target.push_str(&value);
}

/// Parses the parts of `plugin.xml` Jetthing needs.
pub fn parse_plugin_xml(xml: &str) -> Result<PluginXml, MarketplaceError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut plugin = PluginXml::default();
    let mut stack: Vec<String> = Vec::new();
    let mut text_target: Option<String> = None;
    loop {
        let event = reader
            .read_event()
            .map_err(|error| MarketplaceError::InvalidPlugin(format!("plugin.xml: {error}")))?;
        match event {
            Event::Start(element) => {
                text_target =
                    record_element(&mut plugin, stack.last().map(String::as_str), &element);
                stack.push(String::from_utf8_lossy(element.local_name().as_ref()).into_owned());
            }
            Event::Empty(element) => {
                record_element(&mut plugin, stack.last().map(String::as_str), &element);
            }
            Event::End(_) => {
                stack.pop();
                text_target = None;
            }
            Event::Text(text) => {
                if let Some(target) = &text_target {
                    let value = text.decode().map(|t| t.into_owned()).unwrap_or_default();
                    set_text(&mut plugin, target, value);
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(target) = &text_target {
                    let bytes: &[u8] = &reference;
                    let value = match bytes {
                        b"amp" => "&",
                        b"lt" => "<",
                        b"gt" => ">",
                        b"quot" => "\"",
                        b"apos" => "'",
                        _ => "",
                    };
                    set_text(&mut plugin, target, value.to_owned());
                }
            }
            Event::CData(data) => {
                if let Some(target) = &text_target {
                    set_text(
                        &mut plugin,
                        target,
                        String::from_utf8_lossy(&data).into_owned(),
                    );
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    for field in [
        &mut plugin.id,
        &mut plugin.name,
        &mut plugin.version,
        &mut plugin.vendor,
        &mut plugin.description,
    ] {
        *field = field.trim().to_owned();
    }
    if plugin.id.is_empty() {
        plugin.id = plugin.name.clone();
    }
    Ok(plugin)
}

/// `name` attribute of a scheme XML (`<scheme name="…">`).
fn scheme_name(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let mut reader = Reader::from_str(&text);
    loop {
        match reader.read_event().ok()? {
            Event::Start(element) | Event::Empty(element) => {
                return element
                    .attributes()
                    .flatten()
                    .find(|a| a.key.as_ref() == b"name")
                    .and_then(|a| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok())
                    .map(|v| v.into_owned());
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

fn file_stem(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    file.split('.').next().unwrap_or(file).to_owned()
}

impl Inspection {
    /// Usable assets: declared ones, editor schemes referenced by themes, and
    /// TextMate grammars.
    pub fn assets(&self) -> Vec<FoundAsset> {
        let mut assets: Vec<FoundAsset> = Vec::new();
        let push = |assets: &mut Vec<FoundAsset>, kind: AssetKind, path: String, name: String| {
            if !assets.iter().any(|a| a.path == path) {
                assets.push(FoundAsset { kind, path, name });
            }
        };
        for (kind, path) in &self.plugin.declared {
            let Some(bytes) = self.resources.get(path) else {
                continue;
            };
            let name = match kind {
                AssetKind::Theme => serde_json::from_slice::<serde_json::Value>(bytes)
                    .ok()
                    .and_then(|json| json.get("name").and_then(|n| n.as_str()).map(str::to_owned)),
                AssetKind::ColorScheme | AssetKind::Keymap => scheme_name(bytes),
                _ => None,
            }
            .unwrap_or_else(|| file_stem(path));
            push(&mut assets, *kind, path.clone(), name);
            // A theme names its editor color scheme in "editorScheme".
            if *kind == AssetKind::Theme
                && let Some(scheme) = serde_json::from_slice::<serde_json::Value>(bytes)
                    .ok()
                    .and_then(|json| {
                        json.get("editorScheme")
                            .and_then(|s| s.as_str())
                            .map(str::to_owned)
                    })
            {
                let scheme = scheme.trim_start_matches('/').to_owned();
                if let Some(bytes) = self.resources.get(&scheme) {
                    let name = scheme_name(bytes).unwrap_or_else(|| file_stem(&scheme));
                    push(&mut assets, AssetKind::ColorScheme, scheme, name);
                }
            }
        }
        for path in self.resources.keys() {
            let lower = path.to_ascii_lowercase();
            if lower.ends_with(".tmlanguage") || lower.ends_with(".tmlanguage.json") {
                push(
                    &mut assets,
                    AssetKind::TextmateBundle,
                    path.clone(),
                    file_stem(path),
                );
            }
        }
        assets
    }

    pub fn verdict(&self) -> Verdict {
        if self.class_count > 0 {
            let mut reason = format!(
                "Requires IntelliJ Platform (JVM) — not supported: the plugin contains {} compiled JVM classes",
                self.class_count
            );
            if !self.plugin.code_elements.is_empty() {
                let mut unique: Vec<&str> = Vec::new();
                for element in &self.plugin.code_elements {
                    if !unique.contains(&element.as_str()) {
                        unique.push(element);
                    }
                }
                let shown: Vec<&str> = unique.into_iter().take(4).collect();
                reason.push_str(&format!(" and uses {}", shown.join(", ")));
            }
            if self
                .plugin
                .code_elements
                .iter()
                .any(|element| element.contains("lsp") || element.contains("Lsp"))
            {
                reason.push_str(". It integrates a language server through JVM code; configure that server in Jetthing's language server settings instead");
            }
            return Verdict::Unsupported(reason);
        }
        if !self.plugin.code_elements.is_empty() {
            return Verdict::Unsupported(format!(
                "Requires IntelliJ Platform (JVM) — not supported: it uses {}",
                self.plugin.code_elements.join(", ")
            ));
        }
        let assets = self.assets();
        if assets.is_empty() {
            return Verdict::Unsupported(
                "Nothing Jetthing can use: no themes, color schemes, keymaps, live templates or TextMate grammars".into(),
            );
        }
        Verdict::Convertible(assets)
    }
}
