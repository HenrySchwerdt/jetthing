//! Debug session state machine.
//!
//! [`Session`] owns no I/O: it consumes adapter messages with [`Session::handle`] and
//! produces outgoing requests through an internal queue that the caller drains with
//! [`Session::take_outgoing`]. This keeps the protocol logic deterministic and testable
//! with scripted adapter messages.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// `initialize` sent, waiting for capabilities.
    Initializing,
    /// `launch` sent; breakpoints are configured once `initialized` arrives.
    Launching,
    Running,
    Stopped {
        thread_id: i64,
        reason: String,
        description: Option<String>,
    },
    Terminated {
        exit_code: Option<i64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    pub id: i64,
    pub name: String,
    pub path: Option<PathBuf>,
    /// One-based line as reported by the adapter.
    pub line: i64,
    pub column: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    /// Non-zero when the variable has children.
    pub reference: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub name: String,
    pub reference: i64,
    pub expensive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadInfo {
    pub id: i64,
    pub name: String,
}

/// A breakpoint as the adapter reports it (verified or not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakpointStatus {
    pub path: PathBuf,
    /// Zero-based line requested by the user.
    pub line: usize,
    pub verified: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputCategory {
    Console,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputLine {
    pub category: OutputCategory,
    pub text: String,
}

/// What to launch: the adapter-specific `launch` arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct LaunchRequest {
    pub adapter_id: String,
    pub arguments: Value,
}

/// Commands that change the run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Continue,
    Next,
    StepIn,
    StepOut,
    Pause,
}

pub struct Session {
    seq: i64,
    state: SessionState,
    launch: LaunchRequest,
    /// Requested breakpoints: path → zero-based lines.
    breakpoints: BTreeMap<PathBuf, BTreeSet<usize>>,
    statuses: Vec<BreakpointStatus>,
    pending: HashMap<i64, Pending>,
    outgoing: Vec<Value>,
    capabilities: Value,
    threads: Vec<ThreadInfo>,
    frames: Vec<StackFrame>,
    selected_frame: Option<i64>,
    scopes: Vec<Scope>,
    variables: HashMap<i64, Vec<Variable>>,
    output: Vec<OutputLine>,
    evaluations: Vec<(String, Result<String, String>)>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum Pending {
    Initialize,
    Launch,
    SetBreakpoints { path: PathBuf, lines: Vec<usize> },
    ConfigurationDone,
    Threads,
    StackTrace { thread_id: i64 },
    Scopes { frame_id: i64 },
    Variables { reference: i64 },
    Evaluate { expression: String },
    Step,
    Disconnect,
}

/// Maximum console lines kept per session.
const MAX_OUTPUT: usize = 20_000;

impl Session {
    /// Starts a session; the `initialize` request is queued immediately.
    pub fn new(launch: LaunchRequest, breakpoints: BTreeMap<PathBuf, BTreeSet<usize>>) -> Self {
        let mut session = Self {
            seq: 0,
            state: SessionState::Initializing,
            launch,
            breakpoints,
            statuses: Vec::new(),
            pending: HashMap::new(),
            outgoing: Vec::new(),
            capabilities: Value::Null,
            threads: Vec::new(),
            frames: Vec::new(),
            selected_frame: None,
            scopes: Vec::new(),
            variables: HashMap::new(),
            output: Vec::new(),
            evaluations: Vec::new(),
            error: None,
        };
        let adapter_id = session.launch.adapter_id.clone();
        session.request(
            "initialize",
            json!({
                "clientID": "jetthing",
                "clientName": "Jetthing",
                "adapterID": adapter_id,
                "pathFormat": "path",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "supportsVariableType": true,
                "supportsRunInTerminalRequest": false,
            }),
            Pending::Initialize,
        );
        session
    }

    fn request(&mut self, command: &str, arguments: Value, pending: Pending) -> i64 {
        self.seq += 1;
        let seq = self.seq;
        self.outgoing.push(json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        }));
        self.pending.insert(seq, pending);
        seq
    }

    /// Requests queued for the adapter since the last call.
    pub fn take_outgoing(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.outgoing)
    }

    /// Processes one message from the adapter.
    pub fn handle(&mut self, message: &Value) {
        match message["type"].as_str() {
            Some("response") => self.on_response(message),
            Some("event") => self.on_event(message),
            Some("request") => {
                // Reverse requests (runInTerminal, startDebugging) are not supported;
                // answer so the adapter does not wait forever.
                self.seq += 1;
                self.outgoing.push(json!({
                    "seq": self.seq,
                    "type": "response",
                    "request_seq": message["seq"],
                    "command": message["command"],
                    "success": false,
                    "message": "not supported by Jetthing",
                }));
            }
            _ => {}
        }
    }

    fn on_response(&mut self, message: &Value) {
        let Some(pending) = message["request_seq"]
            .as_i64()
            .and_then(|seq| self.pending.remove(&seq))
        else {
            return;
        };
        let body = &message["body"];
        if message["success"].as_bool() != Some(true) {
            let reason = message["message"]
                .as_str()
                .or_else(|| body["error"]["format"].as_str())
                .unwrap_or("request failed")
                .to_owned();
            match pending {
                Pending::Evaluate { expression } => {
                    self.evaluations.push((expression, Err(reason)))
                }
                Pending::Launch | Pending::Initialize => {
                    self.error = Some(format!("debugger failed to start: {reason}"));
                    self.state = SessionState::Terminated { exit_code: None };
                }
                Pending::SetBreakpoints { path, lines } => {
                    for line in lines {
                        self.set_status(&path, line, false, Some(reason.clone()));
                    }
                }
                _ => self.push_output(OutputCategory::Console, format!("{reason}\n")),
            }
            return;
        }
        match pending {
            Pending::Initialize => {
                self.capabilities = body.clone();
                let arguments = self.launch.arguments.clone();
                self.state = SessionState::Launching;
                self.request("launch", arguments, Pending::Launch);
            }
            Pending::Launch => {
                if self.state == SessionState::Launching {
                    self.state = SessionState::Running;
                }
            }
            Pending::SetBreakpoints { path, lines } => {
                let reported = body["breakpoints"].as_array().cloned().unwrap_or_default();
                for (index, line) in lines.into_iter().enumerate() {
                    let entry = reported.get(index);
                    let verified = entry
                        .and_then(|bp| bp["verified"].as_bool())
                        .unwrap_or(false);
                    let message = entry
                        .and_then(|bp| bp["message"].as_str())
                        .map(str::to_owned);
                    self.set_status(&path, line, verified, message);
                }
            }
            Pending::ConfigurationDone | Pending::Step | Pending::Disconnect => {}
            Pending::Threads => {
                self.threads = body["threads"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|thread| {
                        Some(ThreadInfo {
                            id: thread["id"].as_i64()?,
                            name: thread["name"].as_str().unwrap_or("thread").to_owned(),
                        })
                    })
                    .collect();
            }
            Pending::StackTrace { thread_id } => {
                if !matches!(self.state, SessionState::Stopped { thread_id: stopped, .. } if stopped == thread_id)
                {
                    return;
                }
                self.frames = body["stackFrames"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|frame| {
                        Some(StackFrame {
                            id: frame["id"].as_i64()?,
                            name: frame["name"].as_str().unwrap_or("?").to_owned(),
                            path: frame["source"]["path"].as_str().map(PathBuf::from),
                            line: frame["line"].as_i64().unwrap_or(0),
                            column: frame["column"].as_i64().unwrap_or(0),
                        })
                    })
                    .collect();
                // Select the topmost frame that has source, like the reference debugger.
                let first = self
                    .frames
                    .iter()
                    .find(|frame| frame.path.is_some())
                    .or(self.frames.first())
                    .map(|frame| frame.id);
                if let Some(frame_id) = first {
                    self.select_frame(frame_id);
                }
            }
            Pending::Scopes { frame_id } => {
                if self.selected_frame != Some(frame_id) {
                    return;
                }
                self.scopes = body["scopes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|scope| {
                        Some(Scope {
                            name: scope["name"].as_str().unwrap_or("Scope").to_owned(),
                            reference: scope["variablesReference"].as_i64()?,
                            expensive: scope["expensive"].as_bool().unwrap_or(false),
                        })
                    })
                    .collect();
                let cheap: Vec<i64> = self
                    .scopes
                    .iter()
                    .filter(|scope| !scope.expensive && scope.reference > 0)
                    .map(|scope| scope.reference)
                    .collect();
                for reference in cheap {
                    self.request_variables(reference);
                }
            }
            Pending::Variables { reference } => {
                let variables = body["variables"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|variable| Variable {
                        name: variable["name"].as_str().unwrap_or("?").to_owned(),
                        value: variable["value"].as_str().unwrap_or_default().to_owned(),
                        type_name: variable["type"].as_str().map(str::to_owned),
                        reference: variable["variablesReference"].as_i64().unwrap_or(0),
                    })
                    .collect();
                self.variables.insert(reference, variables);
            }
            Pending::Evaluate { expression } => {
                let value = body["result"].as_str().unwrap_or_default().to_owned();
                self.evaluations.push((expression, Ok(value)));
            }
        }
    }

    fn on_event(&mut self, message: &Value) {
        let body = &message["body"];
        match message["event"].as_str() {
            Some("initialized") => {
                let files: Vec<PathBuf> = self.breakpoints.keys().cloned().collect();
                for path in files {
                    self.send_breakpoints(&path);
                }
                if self.capabilities["supportsConfigurationDoneRequest"].as_bool() != Some(false) {
                    self.request("configurationDone", json!({}), Pending::ConfigurationDone);
                }
            }
            Some("stopped") => {
                let thread_id = body["threadId"]
                    .as_i64()
                    .or_else(|| self.threads.first().map(|thread| thread.id))
                    .unwrap_or(1);
                self.state = SessionState::Stopped {
                    thread_id,
                    reason: body["reason"].as_str().unwrap_or("pause").to_owned(),
                    description: body["description"]
                        .as_str()
                        .or_else(|| body["text"].as_str())
                        .map(str::to_owned),
                };
                self.frames.clear();
                self.scopes.clear();
                self.variables.clear();
                self.selected_frame = None;
                self.request("threads", json!({}), Pending::Threads);
                self.request(
                    "stackTrace",
                    json!({"threadId": thread_id, "startFrame": 0, "levels": 100}),
                    Pending::StackTrace { thread_id },
                );
            }
            Some("continued") => {
                if !matches!(self.state, SessionState::Terminated { .. }) {
                    self.state = SessionState::Running;
                    self.frames.clear();
                    self.scopes.clear();
                    self.variables.clear();
                }
            }
            Some("output") => {
                let category = match body["category"].as_str() {
                    Some("stdout") => OutputCategory::Stdout,
                    Some("stderr") => OutputCategory::Stderr,
                    Some("telemetry") => return,
                    _ => OutputCategory::Console,
                };
                let text = body["output"].as_str().unwrap_or_default().to_owned();
                self.push_output(category, text);
            }
            Some("exited") => {
                let code = body["exitCode"].as_i64();
                self.push_output(
                    OutputCategory::Console,
                    format!(
                        "\nProcess finished with exit code {}\n",
                        code.map_or("?".into(), |c| c.to_string())
                    ),
                );
                self.state = SessionState::Terminated { exit_code: code };
            }
            Some("terminated") => {
                if !matches!(self.state, SessionState::Terminated { .. }) {
                    self.state = SessionState::Terminated { exit_code: None };
                }
            }
            Some("breakpoint") => {
                // Verification updates for already requested breakpoints.
                let breakpoint = &body["breakpoint"];
                if let (Some(path), Some(line)) = (
                    breakpoint["source"]["path"].as_str(),
                    breakpoint["line"].as_i64(),
                ) {
                    let verified = breakpoint["verified"].as_bool().unwrap_or(false);
                    let line = usize::try_from(line - 1).unwrap_or(0);
                    if let Some(status) = self.statuses.iter_mut().find(|status| {
                        status.path == std::path::Path::new(path) && status.line == line
                    }) {
                        status.verified = verified;
                    }
                }
            }
            _ => {}
        }
    }

    fn push_output(&mut self, category: OutputCategory, text: String) {
        self.output.push(OutputLine { category, text });
        if self.output.len() > MAX_OUTPUT {
            let excess = self.output.len() - MAX_OUTPUT;
            self.output.drain(..excess);
        }
    }

    fn set_status(&mut self, path: &PathBuf, line: usize, verified: bool, message: Option<String>) {
        self.statuses
            .retain(|status| !(status.path == *path && status.line == line));
        self.statuses.push(BreakpointStatus {
            path: path.clone(),
            line,
            verified,
            message,
        });
    }

    fn send_breakpoints(&mut self, path: &PathBuf) {
        let lines: Vec<usize> = self
            .breakpoints
            .get(path)
            .map(|lines| lines.iter().copied().collect())
            .unwrap_or_default();
        self.statuses.retain(|status| status.path != *path);
        let breakpoints: Vec<Value> = lines.iter().map(|line| json!({"line": line + 1})).collect();
        self.request(
            "setBreakpoints",
            json!({
                "source": {"path": path, "name": path.file_name().map(|n| n.to_string_lossy().into_owned())},
                "breakpoints": breakpoints,
                "lines": lines.iter().map(|line| line + 1).collect::<Vec<_>>(),
            }),
            Pending::SetBreakpoints {
                path: path.clone(),
                lines,
            },
        );
    }

    /// Replaces the breakpoints of `path` (sent immediately when the adapter is configured).
    pub fn update_breakpoints(&mut self, path: PathBuf, lines: BTreeSet<usize>) {
        if lines.is_empty() {
            self.breakpoints.remove(&path);
        } else {
            self.breakpoints.insert(path.clone(), lines);
        }
        if matches!(
            self.state,
            SessionState::Running | SessionState::Stopped { .. }
        ) {
            self.send_breakpoints(&path);
        }
    }

    pub fn step(&mut self, step: Step) {
        let thread_id = match &self.state {
            SessionState::Stopped { thread_id, .. } => *thread_id,
            SessionState::Running if step == Step::Pause => {
                self.threads.first().map(|thread| thread.id).unwrap_or(1)
            }
            _ => return,
        };
        let command = match step {
            Step::Continue => "continue",
            Step::Next => "next",
            Step::StepIn => "stepIn",
            Step::StepOut => "stepOut",
            Step::Pause => "pause",
        };
        self.request(command, json!({"threadId": thread_id}), Pending::Step);
        if step != Step::Pause {
            // Optimistically running; a `stopped` event follows for steps.
            self.state = SessionState::Running;
            self.frames.clear();
            self.scopes.clear();
            self.variables.clear();
        }
    }

    pub fn select_frame(&mut self, frame_id: i64) {
        if !self.frames.iter().any(|frame| frame.id == frame_id) {
            return;
        }
        self.selected_frame = Some(frame_id);
        self.scopes.clear();
        self.variables.clear();
        self.request(
            "scopes",
            json!({"frameId": frame_id}),
            Pending::Scopes { frame_id },
        );
    }

    /// Loads the children of a structured variable or scope.
    pub fn request_variables(&mut self, reference: i64) {
        if reference <= 0 || self.variables.contains_key(&reference) {
            return;
        }
        self.request(
            "variables",
            json!({"variablesReference": reference}),
            Pending::Variables { reference },
        );
    }

    pub fn evaluate(&mut self, expression: &str) {
        let expression = expression.trim().to_owned();
        if expression.is_empty() {
            return;
        }
        let mut arguments = json!({"expression": expression, "context": "repl"});
        if let Some(frame) = self.selected_frame {
            arguments["frameId"] = json!(frame);
        }
        self.request("evaluate", arguments, Pending::Evaluate { expression });
    }

    /// Asks the adapter to end the debuggee.
    pub fn disconnect(&mut self) {
        if matches!(self.state, SessionState::Terminated { .. }) {
            return;
        }
        self.request(
            "disconnect",
            json!({"terminateDebuggee": true}),
            Pending::Disconnect,
        );
    }

    /// Records that the adapter process ended without a `terminated` event.
    pub fn adapter_exited(&mut self, detail: &str) {
        if !matches!(self.state, SessionState::Terminated { .. }) {
            if !detail.trim().is_empty() {
                self.push_output(OutputCategory::Stderr, detail.to_owned());
            }
            self.state = SessionState::Terminated { exit_code: None };
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }
    pub fn frames(&self) -> &[StackFrame] {
        &self.frames
    }
    pub fn selected_frame(&self) -> Option<&StackFrame> {
        self.selected_frame
            .and_then(|id| self.frames.iter().find(|frame| frame.id == id))
    }
    pub fn threads(&self) -> &[ThreadInfo] {
        &self.threads
    }
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }
    pub fn variables(&self, reference: i64) -> Option<&[Variable]> {
        self.variables.get(&reference).map(Vec::as_slice)
    }
    pub fn output(&self) -> &[OutputLine] {
        &self.output
    }
    pub fn breakpoint_statuses(&self) -> &[BreakpointStatus] {
        &self.statuses
    }
    pub fn evaluations(&self) -> &[(String, Result<String, String>)] {
        &self.evaluations
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub fn is_terminated(&self) -> bool {
        matches!(self.state, SessionState::Terminated { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(request: &Value, body: Value) -> Value {
        json!({
            "type": "response",
            "request_seq": request["seq"],
            "command": request["command"],
            "success": true,
            "body": body,
        })
    }

    fn take(session: &mut Session, command: &str) -> Value {
        let outgoing = session.take_outgoing();
        outgoing
            .into_iter()
            .find(|message| message["command"] == command)
            .unwrap_or_else(|| panic!("expected `{command}` request"))
    }

    fn started() -> Session {
        let mut breakpoints = BTreeMap::new();
        breakpoints.insert(PathBuf::from("/w/src/main.rs"), BTreeSet::from([4, 9]));
        let mut session = Session::new(
            LaunchRequest {
                adapter_id: "lldb-dap".into(),
                arguments: json!({"program": "/w/target/debug/app"}),
            },
            breakpoints,
        );
        let initialize = take(&mut session, "initialize");
        session.handle(&response(
            &initialize,
            json!({"supportsConfigurationDoneRequest": true}),
        ));
        let launch = take(&mut session, "launch");
        assert_eq!(launch["arguments"]["program"], "/w/target/debug/app");
        session.handle(&json!({"type": "event", "event": "initialized"}));
        let outgoing = session.take_outgoing();
        let set = outgoing
            .iter()
            .find(|m| m["command"] == "setBreakpoints")
            .unwrap();
        assert_eq!(
            set["arguments"]["breakpoints"],
            json!([{"line": 5}, {"line": 10}])
        );
        assert!(outgoing.iter().any(|m| m["command"] == "configurationDone"));
        session.handle(&response(
            set,
            json!({"breakpoints": [{"verified": true, "line": 5}, {"verified": false, "message": "no code"}]}),
        ));
        session.handle(&response(&launch, json!({})));
        session
    }

    #[test]
    fn startup_sequence_configures_breakpoints() {
        let session = started();
        assert_eq!(session.state(), &SessionState::Running);
        let statuses = session.breakpoint_statuses();
        assert_eq!(statuses.len(), 2);
        assert!(statuses[0].verified);
        assert_eq!(statuses[1].message.as_deref(), Some("no code"));
    }

    #[test]
    fn stop_loads_frames_scopes_and_variables() {
        let mut session = started();
        session.handle(&json!({"type": "event", "event": "stopped", "body": {"reason": "breakpoint", "threadId": 7}}));
        let outgoing = session.take_outgoing();
        let threads = outgoing
            .iter()
            .find(|m| m["command"] == "threads")
            .unwrap()
            .clone();
        let trace = outgoing
            .iter()
            .find(|m| m["command"] == "stackTrace")
            .unwrap()
            .clone();
        session.handle(&response(
            &threads,
            json!({"threads": [{"id": 7, "name": "main"}]}),
        ));
        session.handle(&response(
            &trace,
            json!({"stackFrames": [
                {"id": 100, "name": "std::rt::lang_start", "line": 0, "column": 0},
                {"id": 101, "name": "app::main", "line": 5, "column": 9, "source": {"path": "/w/src/main.rs"}}
            ]}),
        ));
        assert_eq!(session.selected_frame().unwrap().name, "app::main");
        let scopes = take(&mut session, "scopes");
        assert_eq!(scopes["arguments"]["frameId"], 101);
        session.handle(&response(
            &scopes,
            json!({"scopes": [{"name": "Locals", "variablesReference": 1000}, {"name": "Registers", "variablesReference": 2000, "expensive": true}]}),
        ));
        let variables = take(&mut session, "variables");
        assert_eq!(variables["arguments"]["variablesReference"], 1000);
        session.handle(&response(
            &variables,
            json!({"variables": [{"name": "x", "value": "42", "type": "i32", "variablesReference": 0}, {"name": "v", "value": "size=2", "variablesReference": 1001}]}),
        ));
        assert_eq!(session.variables(1000).unwrap()[0].value, "42");
        session.request_variables(1001);
        assert_eq!(
            take(&mut session, "variables")["arguments"]["variablesReference"],
            1001
        );
        assert!(matches!(
            session.state(),
            SessionState::Stopped { thread_id: 7, .. }
        ));
        session.step(Step::Next);
        let next = take(&mut session, "next");
        assert_eq!(next["arguments"]["threadId"], 7);
        assert_eq!(session.state(), &SessionState::Running);
        assert!(session.frames().is_empty());
    }

    #[test]
    fn output_exit_and_failures() {
        let mut session = started();
        session.handle(&json!({"type": "event", "event": "output", "body": {"category": "stdout", "output": "Hello\n"}}));
        session.handle(&json!({"type": "event", "event": "output", "body": {"category": "telemetry", "output": "x"}}));
        session.handle(&json!({"type": "event", "event": "exited", "body": {"exitCode": 3}}));
        assert_eq!(session.output()[0].text, "Hello\n");
        assert_eq!(session.output().len(), 2);
        assert_eq!(
            session.state(),
            &SessionState::Terminated { exit_code: Some(3) }
        );

        let mut failing = Session::new(
            LaunchRequest {
                adapter_id: "gdb".into(),
                arguments: json!({}),
            },
            BTreeMap::new(),
        );
        let initialize = take(&mut failing, "initialize");
        failing.handle(&json!({"type": "response", "request_seq": initialize["seq"], "success": false, "message": "unsupported"}));
        assert!(failing.is_terminated());
        assert!(failing.error().unwrap().contains("unsupported"));
        // Reverse requests are answered, never left hanging.
        failing.handle(&json!({"type": "request", "seq": 9, "command": "runInTerminal"}));
        let answer = failing.take_outgoing();
        assert_eq!(answer[0]["request_seq"], 9);
        assert_eq!(answer[0]["success"], false);
    }

    #[test]
    fn breakpoint_changes_during_a_session_are_sent() {
        let mut session = started();
        session.update_breakpoints(PathBuf::from("/w/src/lib.rs"), BTreeSet::from([0]));
        let set = take(&mut session, "setBreakpoints");
        assert_eq!(set["arguments"]["source"]["path"], "/w/src/lib.rs");
        session.update_breakpoints(PathBuf::from("/w/src/lib.rs"), BTreeSet::new());
        let cleared = take(&mut session, "setBreakpoints");
        assert_eq!(cleared["arguments"]["breakpoints"], json!([]));
    }

    #[test]
    fn evaluate_uses_selected_frame() {
        let mut session = started();
        session.evaluate("  1 + 1 ");
        let evaluate = take(&mut session, "evaluate");
        assert_eq!(evaluate["arguments"]["expression"], "1 + 1");
        session.handle(&response(&evaluate, json!({"result": "2"})));
        assert_eq!(
            session.evaluations()[0],
            ("1 + 1".to_owned(), Ok("2".to_owned()))
        );
    }
}
