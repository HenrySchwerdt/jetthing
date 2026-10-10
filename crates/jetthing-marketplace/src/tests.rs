use super::*;
use jetthing_plugin_api::AssetKind;
use std::io::Write;

fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

const THEME_XML: &str = r#"<idea-plugin>
  <id>com.example.dusk-theme</id>
  <name>Dusk Theme</name>
  <version>1.4.0</version>
  <vendor url="https://example.com">Example</vendor>
  <description><![CDATA[<p>A calm <b>dark</b> theme.</p><ul><li>Two variants</li></ul>]]></description>
  <depends>com.intellij.modules.platform</depends>
  <extensions defaultExtensionNs="com.intellij">
    <themeProvider id="1" path="/dusk.theme.json"/>
    <bundledColorScheme path="/extra_scheme"/>
  </extensions>
</idea-plugin>"#;

fn theme_plugin() -> Vec<u8> {
    let jar = zip(&[
        ("META-INF/plugin.xml", THEME_XML.as_bytes()),
        ("META-INF/pluginIcon.svg", b"<svg/>"),
        (
            "dusk.theme.json",
            br#"{"name": "Dusk", "dark": true, "editorScheme": "/dusk.xml"}"#,
        ),
        (
            "dusk.xml",
            br#"<scheme name="Dusk Editor" version="142" parent_scheme="Darcula"></scheme>"#,
        ),
    ]);
    zip(&[("dusk-theme/lib/dusk-theme-1.4.0.jar", &jar)])
}

fn details(xml_id: &str) -> PluginDetails {
    serde_json::from_value(serde_json::json!({
        "id": 42, "name": "Dusk Theme", "xmlId": xml_id, "link": "/plugin/42-dusk",
        "description": "<p>A calm theme.</p>",
        "vendor": {"name": "uuid", "publicName": "Example"},
        "urls": {"sourceCodeUrl": "https://example.com/src", "licenseUrl": "https://example.com/LICENSE"},
        "tags": [{"name": "Theme"}]
    }))
    .unwrap()
}

#[test]
fn theme_plugins_convert_into_declarative_jetthing_plugins() {
    let inspection = inspect(&theme_plugin()).unwrap();
    assert_eq!(inspection.plugin.id, "com.example.dusk-theme");
    assert_eq!(inspection.plugin.version, "1.4.0");
    assert_eq!(inspection.class_count, 0);
    let Verdict::Convertible(assets) = inspection.verdict() else {
        panic!("convertible: {:?}", inspection.verdict());
    };
    // The declared scheme file is missing, so only the theme and the scheme it references count.
    assert_eq!(
        assets
            .iter()
            .map(|a| (a.kind, a.name.as_str()))
            .collect::<Vec<_>>(),
        [
            (AssetKind::Theme, "Dusk"),
            (AssetKind::ColorScheme, "Dusk Editor")
        ]
    );
    let converted = convert(
        &inspection,
        &details("com.example.dusk-theme"),
        "1.4.0",
        api::DEFAULT_BASE_URL,
    )
    .unwrap();
    let manifest = &converted.manifest;
    assert_eq!(manifest.id, "com.example.dusk-theme");
    assert_eq!(manifest.vendor, "Example");
    assert_eq!(
        manifest.homepage.as_deref(),
        Some("https://example.com/src")
    );
    assert!(manifest.description.contains("A calm theme."));
    assert!(
        manifest
            .description
            .contains("plugins.jetbrains.com/plugin/42-dusk")
    );
    assert!(
        manifest
            .description
            .contains("License: https://example.com/LICENSE")
    );
    assert_eq!(manifest.assets.len(), 2);
    assert_eq!(manifest.assets[0].path, "assets/themes/dusk.theme.json");
    // The archive is a valid .jtplugin whose manifest round-trips.
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(converted.archive)).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("jetthing-plugin.toml").unwrap(),
        &mut text,
    )
    .unwrap();
    assert_eq!(
        &jetthing_plugin_api::PluginManifest::parse(&text).unwrap(),
        manifest
    );
    assert!(archive.by_name("assets/schemes/dusk.xml").is_ok());
}

#[test]
fn code_plugins_are_unsupported_with_a_reason() {
    let xml = r#"<idea-plugin><id>com.example.tool</id><name>Tool</name>
      <extensions defaultExtensionNs="com.intellij">
        <toolWindow id="Tool" factoryClass="com.example.Factory"/>
        <platform.lsp.serverSupportProvider implementation="com.example.Lsp"/>
      </extensions>
      <actions><action id="x" class="com.example.Action"/></actions>
    </idea-plugin>"#;
    let jar = zip(&[
        ("META-INF/plugin.xml", xml.as_bytes()),
        ("com/example/Factory.class", b"\xca\xfe"),
    ]);
    // A bare jar download works too.
    let inspection = inspect(&jar).unwrap();
    let Verdict::Unsupported(reason) = inspection.verdict() else {
        panic!()
    };
    assert!(
        reason.starts_with("Requires IntelliJ Platform (JVM) — not supported"),
        "{reason}"
    );
    assert!(reason.contains("1 compiled JVM classes"), "{reason}");
    assert!(reason.contains("toolWindow"), "{reason}");
    assert!(reason.contains("language server"), "{reason}");
    assert!(matches!(
        convert(
            &inspection,
            &details("com.example.tool"),
            "1",
            api::DEFAULT_BASE_URL
        ),
        Err(MarketplaceError::Unsupported(_))
    ));
}

#[test]
fn resource_plugins_without_usable_assets_and_broken_archives_are_explained() {
    let jar = zip(&[(
        "META-INF/plugin.xml",
        b"<idea-plugin><id>a.b</id><name>Docs</name></idea-plugin>",
    )]);
    let Verdict::Unsupported(reason) = inspect(&jar).unwrap().verdict() else {
        panic!()
    };
    assert!(reason.contains("Nothing Jetthing can use"), "{reason}");
    assert!(
        inspect(b"not a zip")
            .unwrap_err()
            .to_string()
            .contains("not a zip")
    );
    let no_xml = zip(&[("x/lib/a.jar", &zip(&[("readme.txt", b"hi")]))]);
    assert!(
        inspect(&no_xml)
            .unwrap_err()
            .to_string()
            .contains("plugin.xml")
    );
}

#[test]
fn textmate_grammars_are_found() {
    let jar = zip(&[
        (
            "META-INF/plugin.xml",
            b"<idea-plugin><id>a.tm</id><name>TM</name></idea-plugin>",
        ),
        ("bundles/foo/syntaxes/foo.tmLanguage.json", b"{}"),
    ]);
    let Verdict::Convertible(assets) = inspect(&jar).unwrap().verdict() else {
        panic!()
    };
    assert_eq!(assets[0].kind, AssetKind::TextmateBundle);
}

#[test]
fn ids_text_and_hints() {
    assert_eq!(
        convert::plugin_id("com.markskelton.one-dark-theme", 1),
        "com.markskelton.one-dark-theme"
    );
    assert_eq!(
        convert::plugin_id("IdeaVIM", 164),
        "com.jetbrains.marketplace.IdeaVIM"
    );
    assert_eq!(
        convert::plugin_id("", 7),
        "com.jetbrains.marketplace.plugin-7"
    );
    assert_eq!(
        convert::html_to_text("<p>One &amp; two</p><ul><li>a</li><li>b</li></ul>"),
        "One & two\n• a\n• b"
    );
    assert_eq!(
        tag_hint(&["Theme".into()]),
        Some("Likely installable: UI theme")
    );
    assert_eq!(tag_hint(&["Tools Integration".into()]), None);
}
