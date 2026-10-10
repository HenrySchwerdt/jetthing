//! Bridges toolchain provisioning and the language server registry: managed
//! installs start without `PATH` changes, and missing-server notices name the
//! component a "Set up" action installs.

use jetthing_lsp::{ProgramResolver, ResolvedProgram, ServerSpec};
use jetthing_toolchains::{LocateSource, Located, Os, Toolchains};
use std::path::{Path, PathBuf};

/// The toolchain component providing a language server, by server id.
pub fn component_for_server(server: &str) -> Option<&'static str> {
    match server {
        "rust-analyzer" => Some("rust-analyzer"),
        "jdtls" => Some("jdtls"),
        "taplo" => Some("taplo"),
        _ => None,
    }
}

/// [`ProgramResolver`] over a snapshot of the toolchain settings and pins.
pub struct ToolchainServerResolver {
    toolchains: Toolchains,
}

impl ToolchainServerResolver {
    pub fn new(toolchains: Toolchains) -> Self {
        Self { toolchains }
    }

    fn environment(&self, components: &[&str]) -> Vec<(String, String)> {
        self.toolchains
            .environment(components)
            .into_iter()
            .map(|(key, value)| (key, value.to_string_lossy().into_owned()))
            .collect()
    }

    /// Eclipse JDT LS from a managed install: run the Equinox launcher on the
    /// resolved JDK directly (its `bin/jdtls` launcher needs Python).
    fn jdtls(&self, located: &Located) -> Option<ResolvedProgram> {
        let home = &located.home;
        let Some(launcher) = equinox_launcher(&home.join("plugins")) else {
            // A launcher script configured by the user; run it as is.
            return Some(ResolvedProgram {
                program: located.executable.clone(),
                args: Vec::new(),
                env: self.environment(&["jdk"]),
            });
        };
        let java = self.toolchains.locate("jdk")?;
        // jdtls needs Java 21+; known-older managed JDKs cannot run it.
        if java
            .version
            .as_ref()
            .and_then(|v| v.as_str().split(['.', '+']).next()?.parse::<u32>().ok())
            .is_some_and(|major| major < 21)
        {
            return None;
        }
        let platform = self.toolchains.platform();
        let os = match platform.os {
            Os::Linux => "config_linux",
            Os::Macos => "config_mac",
            Os::Windows => "config_win",
        };
        let arm = format!("{os}_arm");
        let config =
            if platform.arch == jetthing_toolchains::Arch::Aarch64 && home.join(&arm).is_dir() {
                home.join(arm)
            } else {
                home.join(os)
            };
        let args = [
            "-Declipse.application=org.eclipse.jdt.ls.core.id1",
            "-Dosgi.bundles.defaultStartLevel=4",
            "-Declipse.product=org.eclipse.jdt.ls.core.product",
            "-Dosgi.checkConfiguration=true",
            &format!("-Dosgi.sharedConfiguration.area={}", config.display()),
            "-Dosgi.sharedConfiguration.area.readOnly=true",
            "-Dosgi.configuration.cascaded=true",
            "-Xms1G",
            "--add-modules=ALL-SYSTEM",
            "--add-opens",
            "java.base/java.util=ALL-UNNAMED",
            "--add-opens",
            "java.base/java.lang=ALL-UNNAMED",
            "-jar",
            &launcher.display().to_string(),
        ]
        .map(str::to_owned)
        .to_vec();
        Some(ResolvedProgram {
            program: java.executable,
            args,
            env: self.environment(&["jdk"]),
        })
    }
}

/// `plugins/org.eclipse.equinox.launcher_<version>.jar` (not the per-OS fragments).
fn equinox_launcher(plugins: &Path) -> Option<PathBuf> {
    let mut jars: Vec<PathBuf> = std::fs::read_dir(plugins)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| {
                    name == "org.eclipse.equinox.launcher.jar"
                        || (name.starts_with("org.eclipse.equinox.launcher_")
                            && name.ends_with(".jar"))
                })
        })
        .collect();
    jars.sort();
    jars.pop()
}

impl ProgramResolver for ToolchainServerResolver {
    fn resolve(&self, spec: &ServerSpec) -> Option<ResolvedProgram> {
        let component = component_for_server(spec.id)?;
        let located = self.toolchains.locate(component)?;
        // PATH installs are found by the registry's own candidates.
        if located.source == LocateSource::Path {
            return None;
        }
        if spec.id == "jdtls" {
            return self.jdtls(&located);
        }
        let name = located
            .executable
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        let args = spec
            .candidates
            .iter()
            .find(|candidate| candidate.program == name)
            .map(|candidate| candidate.args.iter().map(|arg| (*arg).to_owned()).collect())
            .unwrap_or_default();
        Some(ResolvedProgram {
            program: located.executable,
            args,
            env: self.environment(&[component]),
        })
    }

    fn setup_component(&self, spec: &ServerSpec) -> Option<String> {
        component_for_server(spec.id)
            .filter(|component| self.toolchains.catalog().get(component).is_some())
            .map(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_lsp::ServerRegistry;
    use jetthing_toolchains::{
        Catalog, ComponentKind, ComponentSpec, Platform, StaticProvider, ToolchainSettings,
        ToolchainStore, catalog::ReleaseIndex,
    };
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn provider(spec: ComponentSpec) -> Arc<StaticProvider> {
        Arc::new(StaticProvider {
            spec,
            index: ReleaseIndex::default(),
        })
    }

    #[test]
    fn jdtls_runs_the_equinox_launcher_on_the_resolved_jdk() {
        let dir = std::env::temp_dir().join(format!("jetthing-tc-lsp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let jdtls = dir.join("jdtls");
        std::fs::create_dir_all(jdtls.join("plugins")).unwrap();
        std::fs::create_dir_all(jdtls.join("bin")).unwrap();
        std::fs::create_dir_all(jdtls.join("config_linux")).unwrap();
        std::fs::write(jdtls.join("bin/jdtls"), "").unwrap();
        for jar in [
            "org.eclipse.equinox.launcher_1.8.0.v1.jar",
            "org.eclipse.equinox.launcher.gtk.linux.x86_64_1.2.0.jar",
        ] {
            std::fs::write(jdtls.join("plugins").join(jar), "").unwrap();
        }
        let jdk = dir.join("jdk");
        std::fs::create_dir_all(jdk.join("bin")).unwrap();
        std::fs::write(jdk.join("bin/java"), "").unwrap();

        let mut catalog = Catalog::new();
        catalog.register(provider(jetthing_toolchains::providers::jdtls::spec()));
        catalog.register(provider(jetthing_toolchains::providers::jdk::spec()));
        catalog.register(provider(ComponentSpec::new(
            "taplo",
            "Taplo",
            ComponentKind::LanguageServer,
            "taplo",
        )));
        let mut toolchains = Toolchains::new(catalog, ToolchainStore::new(dir.join("store")))
            .with_platform(Platform::parse("linux", "x86_64").unwrap())
            .with_environment(Default::default(), BTreeMap::new());
        let settings = ToolchainSettings::parse(
            &format!(
                "[components.jdtls]\npath = {:?}\n[components.jdk]\npath = {:?}\n",
                jdtls.display().to_string(),
                jdk.display().to_string()
            ),
            Path::new("toolchains.toml"),
        )
        .unwrap();
        toolchains.apply_project(None, settings);
        let resolver = ToolchainServerResolver::new(toolchains);
        let registry = ServerRegistry::default();

        let spec = registry.get("jdtls").unwrap().spec();
        let resolved = resolver.resolve(spec).unwrap();
        assert_eq!(resolved.program, jdk.join("bin/java"));
        assert!(resolved.args.contains(&format!(
            "-Dosgi.sharedConfiguration.area={}",
            jdtls.join("config_linux").display()
        )));
        assert_eq!(
            resolved.args.last().unwrap(),
            &jdtls
                .join("plugins/org.eclipse.equinox.launcher_1.8.0.v1.jar")
                .display()
                .to_string()
        );
        assert!(
            resolved
                .env
                .iter()
                .any(|(key, value)| key == "JAVA_HOME" && Path::new(value) == jdk)
        );
        assert_eq!(resolver.setup_component(spec).as_deref(), Some("jdtls"));

        // Not installed: nothing resolved, but the notice can still offer setup.
        let taplo = registry.get("taplo").unwrap().spec();
        assert!(resolver.resolve(taplo).is_none());
        assert_eq!(resolver.setup_component(taplo).as_deref(), Some("taplo"));
        let kotlin = registry.get("kotlin").unwrap().spec();
        assert_eq!(resolver.setup_component(kotlin), None);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
