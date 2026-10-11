//! Debugger: line breakpoints and Debug Adapter Protocol sessions.

use crate::java_debug::JavaStage;
use crate::{IdeState, ToolWindow, spec, string_param, usize_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_dap::{Adapter, DebugClient, SessionState, Step};
use jetthing_runner::{RunConfiguration, RunTarget, TaskKind, TaskPlan, TaskState};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const BREAKPOINTS_FILE: &str = ".jetthing/breakpoints.json";

/// What to debug once its build has finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingDebug {
    pub name: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
}

/// The editor's view of the debugger: where execution stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPoint {
    /// Project-relative path.
    pub path: PathBuf,
    /// Zero-based line.
    pub line: usize,
}

/// A Java debug session being prepared through jdtls (see `java_debug`).
pub(crate) struct PendingJava {
    pub name: String,
    /// Open Java document whose language server runs the plugin commands.
    pub anchor: PathBuf,
    pub main_class: Option<String>,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub stage: JavaStage,
    pub started: Instant,
}

#[derive(Default)]
pub struct DebuggerState {
    /// Project-relative path → zero-based breakpoint lines.
    pub breakpoints: BTreeMap<PathBuf, BTreeSet<usize>>,
    pub client: Option<DebugClient>,
    pub pending: Option<PendingDebug>,
    pub(crate) java: Option<PendingJava>,
    /// Name of the configuration being debugged.
    pub name: String,
    /// Frame already revealed in the editor (to reveal each stop only once).
    revealed: Option<(i64, i64)>,
    pub error: Option<String>,
}

impl DebuggerState {
    pub(crate) fn reset_revealed(&mut self) {
        self.revealed = None;
    }
}

impl IdeState {
    pub(crate) fn absolute_breakpoints(
        &self,
    ) -> Result<BTreeMap<PathBuf, BTreeSet<usize>>, String> {
        let root = self.project.as_ref().ok_or("no project is open")?.root();
        Ok(self
            .debugger
            .breakpoints
            .iter()
            .map(|(path, lines)| (root.join(path), lines.clone()))
            .collect())
    }

    pub(crate) fn load_breakpoints(&mut self) {
        self.debugger = DebuggerState::default();
        let Some(project) = &self.project else {
            return;
        };
        if let Ok(text) = std::fs::read_to_string(project.root().join(BREAKPOINTS_FILE))
            && let Ok(map) = serde_json::from_str::<BTreeMap<PathBuf, BTreeSet<usize>>>(&text)
        {
            self.debugger.breakpoints = map;
        }
    }

    fn save_breakpoints(&self) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        let path = project.root().join(BREAKPOINTS_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let json = serde_json::to_string_pretty(&self.debugger.breakpoints)
            .map_err(|error| error.to_string())?;
        std::fs::write(path, json).map_err(|error| error.to_string())
    }

    /// Toggles a line breakpoint; returns whether the line now has one.
    pub fn toggle_breakpoint(&mut self, path: &Path, line: usize) -> Result<bool, String> {
        let lines = self
            .debugger
            .breakpoints
            .entry(path.to_owned())
            .or_default();
        let added = lines.insert(line) || {
            lines.remove(&line);
            false
        };
        let lines = lines.clone();
        if lines.is_empty() {
            self.debugger.breakpoints.remove(path);
        }
        if let (Some(client), Some(project)) = (self.debugger.client.as_mut(), &self.project) {
            let absolute = project.root().join(path);
            client.with_session(|session| session.update_breakpoints(absolute, lines));
        }
        self.save_breakpoints()?;
        Ok(added)
    }

    pub fn breakpoint_lines(&self, path: &Path) -> Vec<usize> {
        self.debugger
            .breakpoints
            .get(path)
            .map(|lines| lines.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn is_debugging(&self) -> bool {
        self.debugger
            .client
            .as_ref()
            .is_some_and(|client| !client.session().is_terminated())
            || self.debugger.pending.is_some()
            || self.debugger.java.is_some()
    }

    /// Starts debugging the selected run configuration (or the detected Cargo binary).
    pub fn start_debugging(&mut self) -> Result<(), String> {
        if self.is_debugging() {
            return Err("A debug session is already running; stop it first.".into());
        }
        let root = self
            .project
            .as_ref()
            .ok_or("no project is open")?
            .root()
            .to_owned();
        let config = match self.run_configurations.selected() {
            Some(config) => config.clone(),
            None => default_debug_configuration(&root)?,
        };
        self.debugger.error = None;
        self.debugger.name = config.name.clone();
        match &config.target {
            RunTarget::Cargo {
                command,
                package,
                bin,
                options,
                program_arguments,
            } if command == "run" => {
                let cwd = config.cwd(&root);
                let crate_dir = cwd.clone();
                let binary = bin
                    .clone()
                    .or_else(|| option_value(options, "--bin"))
                    .or_else(|| package.clone())
                    .or_else(|| cargo_package_name(&crate_dir))
                    .ok_or("cannot tell which binary to debug; set `bin` in the configuration")?;
                let example = option_value(options, "--example");
                let mut build_args = vec!["build".to_owned()];
                build_args.extend(
                    options
                        .iter()
                        .filter(|option| *option != "--release")
                        .cloned(),
                );
                if let Some(package) = package {
                    build_args.extend(["-p".into(), package.clone()]);
                }
                if let Some(bin) = bin {
                    build_args.extend(["--bin".into(), bin.clone()]);
                }
                let name = example.clone().unwrap_or(binary);
                let target = cargo_target_dir(&crate_dir, &root);
                let program = match example {
                    Some(_) => target.join("debug/examples").join(&name),
                    None => target.join("debug").join(&name),
                };
                self.debugger.pending = Some(PendingDebug {
                    name: config.name.clone(),
                    program,
                    args: program_arguments.clone(),
                    cwd: cwd.clone(),
                    env: config
                        .env
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                });
                let plan =
                    TaskPlan::new(format!("Build {}", config.name), "cargo", build_args, cwd)
                        .with_kind(TaskKind::Build);
                self.start_task(Ok(plan), "Build", TaskKind::Build)
                    .inspect_err(|_| self.debugger.pending = None)
            }
            RunTarget::Shell { program, arguments } => {
                let cwd = config.cwd(&root);
                let program = if Path::new(program).is_absolute() {
                    PathBuf::from(program)
                } else if program.contains('/') {
                    cwd.join(program)
                } else {
                    jetthing_runner::find_program(program)
                        .ok_or_else(|| format!("`{program}` was not found on PATH"))?
                };
                self.launch_debugger(PendingDebug {
                    name: config.name.clone(),
                    program,
                    args: arguments.clone(),
                    cwd,
                    env: config
                        .env
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                })
            }
            RunTarget::Gradle {
                options,
                program_arguments,
                ..
            } => {
                let main_class = option_value_prefixed(options, "-Pjetthing.mainClass=");
                self.start_java_debugging(&config, &root, main_class, program_arguments.clone())
            }
            other => Err(format!(
                "Debugging {} configurations is not supported yet; native programs (Cargo, shell) can be debugged with lldb-dap or GDB, Java (Gradle) with the java-debug plugin",
                other.type_name()
            )),
        }
    }

    /// Starts a Java debug session through jdtls and the java-debug plugin.
    fn start_java_debugging(
        &mut self,
        config: &RunConfiguration,
        root: &Path,
        main_class: Option<String>,
        args: Vec<String>,
    ) -> Result<(), String> {
        let anchor = self.java_anchor(root)?;
        self.debugger.java = Some(PendingJava {
            name: config.name.clone(),
            anchor,
            main_class,
            args,
            cwd: config.cwd(root),
            env: config
                .env
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            stage: JavaStage::Build,
            started: Instant::now(),
        });
        self.tool_windows.show(ToolWindow::Debug);
        self.status = format!("Starting Java debugger for {}", config.name);
        Ok(())
    }

    /// An open Java document: the active one, else the first source with a `main` method
    /// (opened so the language server knows it).
    fn java_anchor(&mut self, root: &Path) -> Result<PathBuf, String> {
        let is_java = |path: &Path| path.extension().is_some_and(|ext| ext == "java");
        let active = self
            .active_document
            .and_then(|index| self.documents.get(index))
            .map(|document| document.path.clone())
            .filter(|path| is_java(path));
        let relative = match active {
            Some(path) => path,
            None => {
                let found = find_java_main(root)
                    .ok_or("No Java source with a main method was found to debug")?;
                self.open_file(&found)?;
                found
            }
        };
        Ok(root.join(relative))
    }

    /// Starts the adapter for a built program.
    fn launch_debugger(&mut self, pending: PendingDebug) -> Result<(), String> {
        if !pending.program.is_file() {
            return Err(format!(
                "{} does not exist; build it first",
                pending.program.display()
            ));
        }
        let adapter = Adapter::discover_native().map_err(|error| error.to_string())?;
        let launch =
            adapter.launch_request(&pending.program, &pending.args, &pending.cwd, &pending.env);
        let breakpoints = self.absolute_breakpoints()?;
        let client = DebugClient::start(adapter, launch, breakpoints, &pending.cwd)
            .map_err(|error| error.to_string())?;
        self.debugger.name = pending.name.clone();
        self.debugger.client = Some(client);
        self.debugger.revealed = None;
        self.tool_windows.show(ToolWindow::Debug);
        self.status = format!("Debugging {}", pending.name);
        Ok(())
    }

    /// Called when a task finishes: launches a debug session waiting for its build.
    pub(crate) fn continue_pending_debug(&mut self) {
        let Some(pending) = self.debugger.pending.take() else {
            return;
        };
        match self.runner.state() {
            TaskState::Exited(0) => {
                if let Err(error) = self.launch_debugger(pending) {
                    self.debugger.error = Some(error.clone());
                    self.status = error;
                }
            }
            _ => {
                self.status = format!("Build failed; not debugging {}", pending.name);
            }
        }
    }

    /// Pumps the adapter and reveals new execution points in the editor.
    pub fn poll_debug(&mut self) {
        self.poll_java_debug();
        let Some(client) = self.debugger.client.as_mut() else {
            return;
        };
        client.poll();
        let session = client.session();
        if let Some(error) = session.error() {
            self.debugger.error = Some(error.to_owned());
        }
        let reveal = match session.state() {
            SessionState::Stopped { thread_id, .. } => session
                .selected_frame()
                .map(|frame| (*thread_id, frame.clone())),
            _ => None,
        };
        if let Some((thread_id, frame)) = reveal
            && self.debugger.revealed != Some((thread_id, frame.id))
        {
            self.debugger.revealed = Some((thread_id, frame.id));
            self.reveal_frame(&frame);
            self.tool_windows.show(ToolWindow::Debug);
        }
        if !matches!(
            self.debugger.client.as_ref().map(|c| c.session().state()),
            Some(SessionState::Stopped { .. })
        ) {
            self.debugger.revealed = None;
        }
    }

    fn reveal_frame(&mut self, frame: &jetthing_dap::StackFrame) {
        let (Some(path), Some(project)) = (&frame.path, &self.project) else {
            return;
        };
        let Ok(relative) = path.strip_prefix(project.root()).map(Path::to_path_buf) else {
            return;
        };
        if self.open_file(&relative).is_err() {
            return;
        }
        if let Some(index) = self.active_document {
            let document = &mut self.documents[index];
            let line = usize::try_from(frame.line - 1).unwrap_or(0);
            if let Some(start) = document.text.line_to_char(line) {
                document
                    .selections
                    .set_single(jetthing_text::Selection::caret(start));
            }
        }
    }

    /// Where the editor should draw the execution-point marker.
    pub fn execution_point(&self) -> Option<ExecutionPoint> {
        let session = self.debugger.client.as_ref()?.session();
        if !matches!(session.state(), SessionState::Stopped { .. }) {
            return None;
        }
        let frame = session.selected_frame()?;
        let path = frame
            .path
            .as_ref()?
            .strip_prefix(self.project.as_ref()?.root())
            .ok()?
            .to_owned();
        Some(ExecutionPoint {
            path,
            line: usize::try_from(frame.line - 1).ok()?,
        })
    }

    fn with_debug_client(&mut self, action: impl FnOnce(&mut DebugClient)) -> Result<(), String> {
        let client = self
            .debugger
            .client
            .as_mut()
            .filter(|client| !client.session().is_terminated())
            .ok_or("no debug session is running")?;
        action(client);
        Ok(())
    }
}

fn option_value(options: &[String], name: &str) -> Option<String> {
    options
        .iter()
        .position(|option| option == name)
        .and_then(|index| options.get(index + 1))
        .cloned()
}

fn option_value_prefixed(options: &[String], prefix: &str) -> Option<String> {
    options
        .iter()
        .find_map(|option| option.strip_prefix(prefix))
        .map(str::to_owned)
}

/// Project-relative path of a Java file declaring `static void main`, searching
/// `src/` trees (bounded, skipping build output).
fn find_java_main(root: &Path) -> Option<PathBuf> {
    let mut stack = vec![root.to_owned()];
    let mut visited = 0;
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).ok()?.flatten() {
            visited += 1;
            if visited > 20_000 {
                return None;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if !name.starts_with('.') && !matches!(&*name, "build" | "out" | "target") {
                    stack.push(path);
                }
            } else if name.ends_with(".java")
                && std::fs::read_to_string(&path)
                    .is_ok_and(|text| text.contains("static void main"))
            {
                return path.strip_prefix(root).ok().map(Path::to_path_buf);
            }
        }
    }
    None
}

fn cargo_package_name(crate_dir: &Path) -> Option<String> {
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).ok()?;
    let value: toml::Table = toml::from_str(&manifest).ok()?;
    value
        .get("package")?
        .get("name")?
        .as_str()
        .map(str::to_owned)
}

/// `target/` of the workspace that contains `crate_dir` (closest existing, else local).
fn cargo_target_dir(crate_dir: &Path, root: &Path) -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(dir);
    }
    crate_dir
        .ancestors()
        .take_while(|dir| dir.starts_with(root))
        .filter(|dir| dir.join("Cargo.toml").is_file())
        .last()
        .unwrap_or(crate_dir)
        .join("target")
}

fn default_debug_configuration(root: &Path) -> Result<RunConfiguration, String> {
    if root.join("Cargo.toml").is_file() {
        return Ok(RunConfiguration::new(
            "Debug",
            RunTarget::Cargo {
                command: "run".into(),
                package: None,
                bin: None,
                options: Vec::new(),
                program_arguments: Vec::new(),
            },
        ));
    }
    Err("select a run configuration to debug (Cargo or shell configurations are supported)".into())
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec(
                "debug.toggleBreakpoint",
                "Toggle Line Breakpoint",
                "Run",
                Some("⌘F8"),
                r#"{"path":"string?","line":"number?"}"#,
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, params| {
                let path = match params.get("path").and_then(Value::as_str) {
                    Some(path) => PathBuf::from(path),
                    None => state
                        .active_document
                        .and_then(|index| state.documents.get(index))
                        .map(|document| document.path.clone())
                        .ok_or_else(|| CommandError::Disabled("debug.toggleBreakpoint".into()))?,
                };
                let line = match params.get("line") {
                    Some(_) => usize_param(&params, "line")?,
                    None => {
                        let index = state.active_document.ok_or_else(|| {
                            CommandError::Disabled("debug.toggleBreakpoint".into())
                        })?;
                        let document = &state.documents[index];
                        document
                            .text
                            .char_to_line(document.selections.primary().head)
                    }
                };
                let added = state
                    .toggle_breakpoint(&path, line)
                    .map_err(CommandError::Failed)?;
                Ok(json!({"added": added}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("debug.start", "Debug", "Run", Some("⌃D"), "{}"),
            |state: &IdeState| {
                state.project.is_some() && !state.runner.is_running() && !state.is_debugging()
            },
            |state: &mut IdeState, _| {
                state.start_debugging().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    for (id, label, shortcut, step) in [
        (
            "debug.resume",
            "Resume Program",
            Some("⌥⌘R"),
            Some(Step::Continue),
        ),
        ("debug.pause", "Pause Program", None, Some(Step::Pause)),
        ("debug.stepOver", "Step Over", Some("F8"), Some(Step::Next)),
        (
            "debug.stepInto",
            "Step Into",
            Some("F7"),
            Some(Step::StepIn),
        ),
        (
            "debug.stepOut",
            "Step Out",
            Some("⇧F8"),
            Some(Step::StepOut),
        ),
        ("debug.stop", "Stop Debugging", None, None),
    ] {
        registry
            .register(Command::new(
                spec(id, label, "Run", shortcut, "{}"),
                move |state: &IdeState| {
                    let Some(session) = state.debugger.client.as_ref().map(|c| c.session()) else {
                        return false;
                    };
                    match (step, session.state()) {
                        (_, SessionState::Terminated { .. }) => false,
                        (None, _) => true,
                        (Some(Step::Pause), SessionState::Running) => true,
                        (Some(Step::Pause), _) => false,
                        (Some(_), SessionState::Stopped { .. }) => true,
                        _ => false,
                    }
                },
                move |state: &mut IdeState, _| {
                    state
                        .with_debug_client(|client| match step {
                            Some(step) => client.with_session(|session| session.step(step)),
                            None => client.stop(),
                        })
                        .map_err(CommandError::Failed)?;
                    Ok(Value::Null)
                },
            ))
            .expect("unique built-in id");
    }
    registry
        .register(Command::new(
            spec(
                "debug.selectFrame",
                "Select Stack Frame",
                "Run",
                None,
                r#"{"frame":0}"#,
            ),
            |state: &IdeState| state.debugger.client.is_some(),
            |state: &mut IdeState, params| {
                let frame = params["frame"].as_i64().ok_or_else(|| {
                    CommandError::InvalidParams("frame must be an integer".into())
                })?;
                state
                    .with_debug_client(|client| client.with_session(|s| s.select_frame(frame)))
                    .map_err(CommandError::Failed)?;
                let selected = state
                    .debugger
                    .client
                    .as_ref()
                    .and_then(|client| client.session().selected_frame().cloned());
                if let Some(frame) = selected {
                    state.reveal_frame(&frame);
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "debug.loadVariables",
                "Expand Variable",
                "Run",
                None,
                r#"{"reference":0}"#,
            ),
            |state: &IdeState| state.debugger.client.is_some(),
            |state: &mut IdeState, params| {
                let reference = params["reference"].as_i64().ok_or_else(|| {
                    CommandError::InvalidParams("reference must be an integer".into())
                })?;
                state
                    .with_debug_client(|client| {
                        client.with_session(|session| session.request_variables(reference))
                    })
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "debug.evaluate",
                "Evaluate Expression",
                "Run",
                Some("⌥F8"),
                r#"{"expression":"string"}"#,
            ),
            |state: &IdeState| {
                state.debugger.client.as_ref().is_some_and(|client| {
                    matches!(client.session().state(), SessionState::Stopped { .. })
                })
            },
            |state: &mut IdeState, params| {
                let expression = string_param(&params, "expression")?;
                state
                    .with_debug_client(|client| client.with_session(|s| s.evaluate(&expression)))
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

#[cfg(all(test, unix))]
mod tests {
    use crate::{IdeState, built_in_commands};
    use jetthing_dap::SessionState;
    use serde_json::{Value, json};

    #[test]
    fn reads_the_package_name_from_a_manifest() {
        let dir = std::env::temp_dir().join(format!("jt-pkg-name-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(super::cargo_package_name(&dir).as_deref(), Some("demo"));
        std::fs::remove_dir_all(dir).unwrap();
    }
    use std::time::{Duration, Instant};

    #[test]
    fn breakpoints_toggle_and_persist() {
        let root = std::env::temp_dir().join(format!("jetthing-debug-bp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.c"), "int main(void) {\n  return 0;\n}\n").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("editor.openFile", &mut state, json!({"path": "main.c"}))
            .unwrap();
        let added = commands
            .execute("debug.toggleBreakpoint", &mut state, json!({"line": 1}))
            .unwrap();
        assert_eq!(added["added"], true);
        assert_eq!(state.breakpoint_lines(std::path::Path::new("main.c")), [1]);
        let mut reopened = IdeState::default();
        commands
            .execute("project.open", &mut reopened, json!({"path": root}))
            .unwrap();
        assert_eq!(
            reopened.breakpoint_lines(std::path::Path::new("main.c")),
            [1]
        );
        commands
            .execute(
                "debug.toggleBreakpoint",
                &mut reopened,
                json!({"path": "main.c", "line": 1}),
            )
            .unwrap();
        assert!(
            reopened
                .breakpoint_lines(std::path::Path::new("main.c"))
                .is_empty()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn debugs_a_shell_configuration_with_the_native_adapter() {
        let Some(compiler) = ["cc", "gcc", "clang"]
            .into_iter()
            .find(|c| jetthing_runner::find_program(c).is_some())
        else {
            eprintln!("skipping: no C compiler");
            return;
        };
        if jetthing_dap::Adapter::discover_native().is_err() {
            eprintln!("skipping: no lldb-dap or gdb");
            return;
        }
        let root = std::env::temp_dir().join(format!("jetthing-debug-run-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("main.c"),
            "#include <stdio.h>\nint main(void) {\n  int x = 1;\n  printf(\"x=%d\\n\", x);\n  return 0;\n}\n",
        )
        .unwrap();
        assert!(
            std::process::Command::new(compiler)
                .args(["-g", "-O0", "-o", "app", "main.c"])
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute(
                "runConfig.save",
                &mut state,
                json!({"configuration": {"name": "app", "type": "shell", "program": "./app"}}),
            )
            .unwrap();
        commands
            .execute(
                "debug.toggleBreakpoint",
                &mut state,
                json!({"path": "main.c", "line": 3}),
            )
            .unwrap();
        commands
            .execute("debug.start", &mut state, Value::Null)
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        while state.execution_point().is_none() && Instant::now() < deadline {
            state.poll_debug();
            std::thread::sleep(Duration::from_millis(20));
        }
        let point = state.execution_point().expect("stopped at the breakpoint");
        assert_eq!((point.path.to_str(), point.line), (Some("main.c"), 3));
        assert_eq!(
            state.documents[state.active_document.unwrap()].path,
            std::path::Path::new("main.c")
        );
        assert!(commands.is_enabled("debug.stepOver", &state));
        commands
            .execute("debug.resume", &mut state, Value::Null)
            .unwrap();
        while !state
            .debugger
            .client
            .as_ref()
            .unwrap()
            .session()
            .is_terminated()
            && Instant::now() < deadline
        {
            state.poll_debug();
            std::thread::sleep(Duration::from_millis(20));
        }
        let session = state.debugger.client.as_ref().unwrap().session();
        assert!(matches!(session.state(), SessionState::Terminated { .. }));
        assert!(!state.is_debugging());
        std::fs::remove_dir_all(root).unwrap();
    }
}
