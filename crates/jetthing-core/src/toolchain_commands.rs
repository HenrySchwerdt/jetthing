//! Toolchain provisioning commands (`toolchain.*`): list, locate, install with
//! progress, cancel, uninstall, "use existing path" and project pins.
//!
//! Installs only start from `toolchain.install`, i.e. from an explicit user
//! action (Set up / Install buttons, palette, CLI); nothing downloads silently.

use crate::{IdeState, spec, string_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_toolchains::service::{JobId, ToolchainEvent, ToolchainService};
use jetthing_toolchains::{
    Catalog, ToolVersion, ToolchainSettings, ToolchainStore, Toolchains, VersionRequest,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Provisioning state owned by the IDE.
pub struct ToolchainState {
    pub toolchains: Toolchains,
    pub service: ToolchainService,
    /// `toolchains.toml` that `toolchain.usePath` writes (user level).
    pub user_settings: Option<PathBuf>,
    /// Problem loading `toolchains.toml`, shown instead of failing project open.
    pub settings_error: Option<String>,
}

impl Default for ToolchainState {
    fn default() -> Self {
        let store = ToolchainStore::new(jetthing_toolchains::default_root());
        store.clean();
        Self::new(
            Toolchains::new(Catalog::new(), store),
            ToolchainSettings::user_file(),
        )
    }
}

impl ToolchainState {
    pub fn new(toolchains: Toolchains, user_settings: Option<PathBuf>) -> Self {
        Self {
            toolchains,
            service: ToolchainService::default(),
            user_settings,
            settings_error: None,
        }
    }

    /// Reloads user and project `toolchains.toml` and the project's pins.
    pub fn reload(&mut self, project_root: Option<&Path>) {
        let project_file = project_root.map(ToolchainSettings::project_file);
        let settings =
            ToolchainSettings::load_from(self.user_settings.as_deref(), project_file.as_deref());
        self.settings_error = settings.as_ref().err().map(ToString::to_string);
        self.toolchains
            .apply_project(project_root, settings.unwrap_or_default());
    }

    /// Status-bar text and fraction for the first running install.
    pub fn progress(&self) -> Option<(String, Option<f32>)> {
        let job = self.service.jobs().first()?;
        let name = self
            .toolchains
            .catalog()
            .get(&job.component)
            .map_or(job.component.clone(), |p| p.spec().name.clone());
        let mut text = job.describe(&name);
        let more = self.service.jobs().len() - 1;
        if more > 0 {
            text.push_str(&format!(" (+{more})"));
        }
        Some((text, job.fraction()))
    }

    fn component_name(&self, id: &str) -> String {
        self.toolchains
            .catalog()
            .get(id)
            .map_or(id.to_owned(), |p| p.spec().name.clone())
    }
}

impl IdeState {
    pub(crate) fn project_root(&self) -> Option<PathBuf> {
        self.project.as_ref().map(|p| p.root().to_owned())
    }

    /// Applies finished installs to the status line. Call from the UI loop.
    pub fn poll_toolchains(&mut self) {
        let events = self.toolchains.service.poll();
        self.apply_toolchain_events(&events);
    }

    fn apply_toolchain_events(&mut self, events: &[ToolchainEvent]) {
        for event in events {
            if let ToolchainEvent::Finished {
                component, result, ..
            } = event
            {
                let name = self.toolchains.component_name(component);
                self.status = match result {
                    Ok(installed) => format!("{name} {} installed", installed.version),
                    Err(error) => format!("{name} setup failed: {error}"),
                };
            }
        }
    }

    /// Starts installing `component`; returns the job id.
    pub fn install_toolchain(
        &mut self,
        component: &str,
        request: Option<VersionRequest>,
    ) -> Result<JobId, String> {
        if self
            .toolchains
            .toolchains
            .catalog()
            .get(component)
            .is_none()
        {
            return Err(format!("unknown toolchain component `{component}`"));
        }
        let job = self
            .toolchains
            .service
            .install(&self.toolchains.toolchains, component, request);
        self.status = format!("Setting up {}…", self.toolchains.component_name(component));
        Ok(job)
    }
}

fn located_json(located: &jetthing_toolchains::Located) -> Value {
    json!({
        "path": located.executable,
        "home": located.home,
        "version": located.version,
        "source": located.source,
    })
}

fn component_param(state: &IdeState, params: &Value) -> Result<String, CommandError> {
    let id = string_param(params, "component")?;
    if state.toolchains.toolchains.catalog().get(&id).is_none() {
        return Err(CommandError::InvalidParams(format!(
            "unknown toolchain component `{id}`"
        )));
    }
    Ok(id)
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec("toolchain.list", "Toolchains", "Toolchains", None, "{}"),
            |_| true,
            |state: &mut IdeState, _| {
                let toolchains = &state.toolchains.toolchains;
                let components: Vec<Value> = toolchains
                    .catalog()
                    .specs()
                    .filter_map(|spec| toolchains.status(&spec.id))
                    .map(|status| {
                        json!({
                            "id": status.spec.id,
                            "name": status.spec.name,
                            "kind": status.spec.kind,
                            "requires": status.spec.requires,
                            "located": status.located.as_ref().map(located_json),
                            "installed": status.installed.iter().map(|i| &i.version).collect::<Vec<_>>(),
                            "pin": status.pin,
                        })
                    })
                    .collect();
                Ok(json!({
                    "root": toolchains.store().root(),
                    "platform": toolchains.platform().key(),
                    "components": components,
                    "settingsError": state.toolchains.settings_error,
                }))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.locate",
                "Locate Toolchain Component",
                "Toolchains",
                None,
                r#"{"component":"string"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let id = component_param(state, &params)?;
                Ok(state
                    .toolchains
                    .toolchains
                    .locate(&id)
                    .as_ref()
                    .map_or(Value::Null, located_json))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.missing",
                "Missing Toolchain Components",
                "Toolchains",
                None,
                r#"{"components":["string"]}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let ids = crate::string_array_param(&params, "components")?;
                let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
                Ok(json!(state.toolchains.toolchains.missing(&ids)))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.install",
                "Set Up Toolchain Component",
                "Toolchains",
                None,
                r#"{"component":"string","version":"string?","wait":"bool?"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let id = component_param(state, &params)?;
                let request = params
                    .get("version")
                    .and_then(Value::as_str)
                    .map(VersionRequest::parse);
                let job = state
                    .install_toolchain(&id, request)
                    .map_err(CommandError::Failed)?;
                if params.get("wait").and_then(Value::as_bool) != Some(true) {
                    return Ok(json!({"job": job}));
                }
                // Headless use (CLI): block until the install finishes.
                let (events, result) = state
                    .toolchains
                    .service
                    .wait(job, Duration::from_secs(60 * 60));
                state.apply_toolchain_events(&events);
                match result {
                    Some(Ok(installed)) => Ok(json!({
                        "job": job,
                        "component": installed.component,
                        "version": installed.version,
                        "path": installed.path,
                    })),
                    Some(Err(error)) => Err(CommandError::Failed(error)),
                    None => Err(CommandError::Failed("install timed out".into())),
                }
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.cancel",
                "Cancel Toolchain Setup",
                "Toolchains",
                None,
                r#"{"job":"number?"}"#,
            ),
            |state: &IdeState| state.toolchains.service.is_busy(),
            |state: &mut IdeState, params| {
                match params.get("job").and_then(Value::as_u64) {
                    Some(job) => {
                        if !state.toolchains.service.cancel(job) {
                            return Err(CommandError::InvalidParams(format!(
                                "no running setup job {job}"
                            )));
                        }
                    }
                    None => state.toolchains.service.cancel_all(),
                }
                state.status = "Cancelling toolchain setup…".into();
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.jobs",
                "Toolchain Setup Jobs",
                "Toolchains",
                None,
                "{}",
            ),
            |_| true,
            |state: &mut IdeState, _| {
                let jobs: Vec<Value> = state
                    .toolchains
                    .service
                    .jobs()
                    .iter()
                    .map(|job| {
                        json!({
                            "job": job.id,
                            "component": job.component,
                            "version": job.version,
                            "text": job.describe(&state.toolchains.component_name(&job.component)),
                            "fraction": job.fraction(),
                        })
                    })
                    .collect();
                Ok(json!(jobs))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.uninstall",
                "Uninstall Toolchain Component",
                "Toolchains",
                None,
                r#"{"component":"string","version":"string"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let id = string_param(&params, "component")?;
                let version = ToolVersion::new(string_param(&params, "version")?);
                if state
                    .toolchains
                    .service
                    .jobs()
                    .iter()
                    .any(|job| job.component == id)
                {
                    return Err(CommandError::Failed(format!(
                        "{id} is being set up; cancel the setup first"
                    )));
                }
                state
                    .toolchains
                    .toolchains
                    .store()
                    .uninstall(&id, &version)
                    .map_err(|error| CommandError::Failed(error.to_string()))?;
                state.status = format!(
                    "Uninstalled {} {version}",
                    state.toolchains.component_name(&id)
                );
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.usePath",
                "Use Existing Toolchain Installation",
                "Toolchains",
                None,
                r#"{"component":"string","path":"string?"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let id = component_param(state, &params)?;
                let path = params
                    .get("path")
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                if let Some(path) = &path {
                    let toolchains = &state.toolchains.toolchains;
                    let spec = toolchains
                        .catalog()
                        .get(&id)
                        .map(|p| p.spec().clone())
                        .ok_or_else(|| CommandError::InvalidParams(id.clone()))?;
                    let usable = path.is_file()
                        || jetthing_toolchains::store::find_executable(
                            path,
                            &spec,
                            &toolchains.platform(),
                        )
                        .is_some();
                    if !usable {
                        return Err(CommandError::InvalidParams(format!(
                            "no {} executable ({}) found at {}",
                            spec.name,
                            spec.primary_executable(),
                            path.display()
                        )));
                    }
                }
                let file = state.toolchains.user_settings.clone().ok_or_else(|| {
                    CommandError::Failed("no user configuration directory".into())
                })?;
                ToolchainSettings::update_file(&file, &id, |component| {
                    component.path = path.clone();
                })
                .map_err(|error| CommandError::Failed(error.to_string()))?;
                let root = state.project_root();
                state.toolchains.reload(root.as_deref());
                let located = state.toolchains.toolchains.locate(&id);
                Ok(located.as_ref().map_or(Value::Null, located_json))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolchain.pins",
                "Project Toolchain Pins",
                "Toolchains",
                None,
                "{}",
            ),
            |_| true,
            |state: &mut IdeState, _| Ok(json!(state.toolchains.toolchains.pins().pins)),
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in_commands;
    use jetthing_toolchains::catalog::{IndexedRelease, ReleaseIndex};
    use jetthing_toolchains::download::sha256_hex;
    use jetthing_toolchains::{
        ArchiveKind, Artifact, Checksum, ComponentKind, ComponentSpec, Platform, StaticProvider,
    };
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::sync::Arc;

    fn archive(payload: &[u8]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, "demo-1.0/bin/demo", payload)
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    /// Serves `body` to every connection on a local port.
    fn serve(body: Vec<u8>) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/demo-1.0.tar.gz", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        url
    }

    fn state_with_demo(dir: &Path, url: String, sha256: String) -> IdeState {
        let platform = Platform::parse("linux", "x86_64").unwrap();
        let mut catalog = Catalog::new();
        catalog.register(Arc::new(StaticProvider {
            spec: ComponentSpec::new("demo", "Demo Tool", ComponentKind::BuildTool, "demo"),
            index: ReleaseIndex {
                releases: vec![IndexedRelease {
                    version: ToolVersion::new("1.0"),
                    channels: vec![],
                    artifacts: BTreeMap::from([(
                        platform.key(),
                        Artifact {
                            url,
                            checksum: Checksum::Sha256(sha256),
                            archive: ArchiveKind::TarGz,
                            size: None,
                        },
                    )]),
                    license: None,
                }],
            },
        }));
        let toolchains = Toolchains::new(catalog, ToolchainStore::new(dir.join("toolchains")))
            .with_platform(platform)
            .with_environment(Default::default(), BTreeMap::new());
        IdeState {
            toolchains: ToolchainState::new(toolchains, Some(dir.join("user/toolchains.toml"))),
            ..IdeState::default()
        }
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jetthing-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn install_locate_uninstall_round_trip_through_commands() {
        let dir = temp("toolchain-commands");
        let body = archive(b"#!/bin/sh\n");
        let sha = sha256_hex(&body);
        let mut state = state_with_demo(&dir, serve(body), sha);
        let commands = built_in_commands();

        let missing = commands
            .execute(
                "toolchain.missing",
                &mut state,
                json!({"components": ["demo"]}),
            )
            .unwrap();
        assert_eq!(missing, json!(["demo"]));
        assert_eq!(
            commands
                .execute("toolchain.locate", &mut state, json!({"component": "demo"}))
                .unwrap(),
            Value::Null
        );
        assert!(!commands.is_enabled("toolchain.cancel", &state));

        let result = commands
            .execute(
                "toolchain.install",
                &mut state,
                json!({"component": "demo", "wait": true}),
            )
            .unwrap();
        assert_eq!(result["version"], "1.0");
        assert_eq!(state.status, "Demo Tool 1.0 installed");

        let located = commands
            .execute("toolchain.locate", &mut state, json!({"component": "demo"}))
            .unwrap();
        assert_eq!(located["source"]["kind"], "managed");
        let list = commands
            .execute("toolchain.list", &mut state, json!({}))
            .unwrap();
        assert_eq!(list["components"][0]["installed"], json!(["1.0"]));

        commands
            .execute(
                "toolchain.uninstall",
                &mut state,
                json!({"component": "demo", "version": "1.0"}),
            )
            .unwrap();
        assert!(!dir.join("toolchains/demo/1.0").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_installs_and_bad_input_return_structured_errors() {
        let dir = temp("toolchain-errors");
        let body = archive(b"x");
        let mut state = state_with_demo(&dir, serve(body), "00".repeat(32));
        let commands = built_in_commands();
        let error = commands
            .execute(
                "toolchain.install",
                &mut state,
                json!({"component": "demo", "wait": true}),
            )
            .unwrap_err();
        assert!(error.to_string().contains("checksum mismatch"), "{error}");
        assert!(state.status.contains("setup failed"));
        let error = commands
            .execute(
                "toolchain.install",
                &mut state,
                json!({"component": "nope"}),
            )
            .unwrap_err();
        assert!(matches!(error, CommandError::InvalidParams(_)));
        let error = commands
            .execute(
                "toolchain.usePath",
                &mut state,
                json!({"component": "demo", "path": dir.join("missing")}),
            )
            .unwrap_err();
        assert!(matches!(error, CommandError::InvalidParams(_)), "{error}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn use_path_persists_and_project_pins_are_reported() {
        let dir = temp("toolchain-usepath");
        let mut state = state_with_demo(&dir, "http://127.0.0.1:9/x".into(), "00".repeat(32));
        let commands = built_in_commands();
        let home = dir.join("opt/demo");
        std::fs::create_dir_all(home.join("bin")).unwrap();
        std::fs::write(home.join("bin/demo"), "").unwrap();
        let located = commands
            .execute(
                "toolchain.usePath",
                &mut state,
                json!({"component": "demo", "path": home}),
            )
            .unwrap();
        assert_eq!(located["source"]["kind"], "configured");
        assert_eq!(located["home"], json!(home));
        let saved = std::fs::read_to_string(dir.join("user/toolchains.toml")).unwrap();
        assert!(saved.contains("[components.demo]"), "{saved}");
        commands
            .execute(
                "toolchain.usePath",
                &mut state,
                json!({"component": "demo"}),
            )
            .unwrap();
        assert!(state.toolchains.toolchains.locate("demo").is_none());

        let project = dir.join("project");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join(".bazelversion"), "7.3.1\n").unwrap();
        commands
            .execute("project.open", &mut state, json!({"path": project}))
            .unwrap();
        let pins = commands
            .execute("toolchain.pins", &mut state, json!({}))
            .unwrap();
        assert_eq!(pins[0]["component"], "bazel");
        assert_eq!(pins[0]["request"]["value"], "7.3.1");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
