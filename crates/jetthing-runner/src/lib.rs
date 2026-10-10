use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("no supported build system found in {0}")]
    NoBuildSystem(PathBuf),
    #[error("failed to start task: {0}")]
    Start(#[from] std::io::Error),
    #[error("task process lock is unavailable")]
    Lock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputChunk {
    pub stream: OutputStream,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Idle,
    Running,
    Exited(i32),
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPlan {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

impl TaskPlan {
    pub fn detect_build(root: &Path) -> Result<Self, RunnerError> {
        let plan = if root.join("Cargo.toml").is_file() {
            ("Cargo Build", "cargo", vec!["build"])
        } else if root.join("gradlew").is_file() {
            ("Gradle Build", "./gradlew", vec!["build"])
        } else if root.join("mvnw").is_file() {
            ("Maven Verify", "./mvnw", vec!["verify"])
        } else if root.join("pom.xml").is_file() {
            ("Maven Verify", "mvn", vec!["verify"])
        } else if root.join("package.json").is_file() {
            ("npm Test", "npm", vec!["test"])
        } else if root.join("go.mod").is_file() {
            ("Go Test", "go", vec!["test", "./..."])
        } else {
            return Err(RunnerError::NoBuildSystem(root.to_owned()));
        };
        Ok(Self {
            name: plan.0.into(),
            program: plan.1.into(),
            args: plan.2.into_iter().map(Into::into).collect(),
            cwd: root.to_owned(),
        })
    }
}

struct ActiveTask {
    child: Arc<Mutex<Child>>,
    output: Receiver<OutputChunk>,
}

pub struct TaskService {
    active: Option<ActiveTask>,
    state: TaskState,
    output: Vec<OutputChunk>,
    plan: Option<TaskPlan>,
}

impl Default for TaskService {
    fn default() -> Self {
        Self {
            active: None,
            state: TaskState::Idle,
            output: Vec::new(),
            plan: None,
        }
    }
}

impl TaskService {
    pub fn start(&mut self, plan: TaskPlan) -> Result<(), RunnerError> {
        self.cancel()?;
        let mut command = Command::new(&plan.program);
        command
            .args(&plan.args)
            .current_dir(&plan.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn()?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let child = Arc::new(Mutex::new(child));
        let (sender, output) = mpsc::channel();
        if let Some(reader) = stdout
            && let Err(error) = spawn_reader(reader, OutputStream::Stdout, sender.clone())
        {
            if let Ok(mut process) = child.lock() {
                let _ = process.kill();
            }
            return Err(error.into());
        }
        if let Some(reader) = stderr
            && let Err(error) = spawn_reader(reader, OutputStream::Stderr, sender)
        {
            if let Ok(mut process) = child.lock() {
                let _ = process.kill();
            }
            return Err(error.into());
        }
        self.output.clear();
        self.plan = Some(plan);
        self.state = TaskState::Running;
        self.active = Some(ActiveTask { child, output });
        Ok(())
    }

    pub fn poll(&mut self) -> Result<(), RunnerError> {
        let Some(active) = &self.active else {
            return Ok(());
        };
        self.output.extend(active.output.try_iter());
        if self.state == TaskState::Running {
            let mut child = active.child.lock().map_err(|_| RunnerError::Lock)?;
            if let Some(status) = child.try_wait()? {
                self.state = TaskState::Exited(status.code().unwrap_or(-1));
            }
        }
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), RunnerError> {
        if let Some(active) = &self.active
            && self.state == TaskState::Running
        {
            active.child.lock().map_err(|_| RunnerError::Lock)?.kill()?;
            self.state = TaskState::Cancelled;
        }
        Ok(())
    }

    pub fn state(&self) -> &TaskState {
        &self.state
    }
    pub fn is_running(&self) -> bool {
        self.state == TaskState::Running
    }
    pub fn output(&self) -> &[OutputChunk] {
        &self.output
    }
    pub fn plan(&self) -> Option<&TaskPlan> {
        self.plan.as_ref()
    }
}

impl Drop for TaskService {
    fn drop(&mut self) {
        let _ = self.cancel();
    }
}

fn spawn_reader(
    mut reader: impl Read + Send + 'static,
    stream: OutputStream,
    sender: mpsc::Sender<OutputChunk>,
) -> Result<(), std::io::Error> {
    thread::Builder::new()
        .name(format!("jetthing-task-{stream:?}"))
        .spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => {
                        if sender
                            .send(OutputChunk {
                                stream,
                                text: String::from_utf8_lossy(&buffer[..read]).into_owned(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        })?;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn streams_real_process_output_and_exit_code() {
        let mut service = TaskService::default();
        service
            .start(TaskPlan {
                name: "test".into(),
                program: "/bin/sh".into(),
                args: vec!["-c".into(), "printf out; printf err >&2; exit 7".into()],
                cwd: PathBuf::from("/tmp"),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while service.is_running() && Instant::now() < deadline {
            service.poll().unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        service.poll().unwrap();
        assert_eq!(service.state(), &TaskState::Exited(7));
        assert!(
            service
                .output()
                .iter()
                .any(|chunk| chunk.stream == OutputStream::Stdout && chunk.text.contains("out"))
        );
        assert!(
            service
                .output()
                .iter()
                .any(|chunk| chunk.stream == OutputStream::Stderr && chunk.text.contains("err"))
        );
    }

    #[test]
    fn cancels_running_process() {
        let mut service = TaskService::default();
        service
            .start(TaskPlan {
                name: "sleep".into(),
                program: "/bin/sh".into(),
                args: vec!["-c".into(), "sleep 30".into()],
                cwd: PathBuf::from("/tmp"),
            })
            .unwrap();
        service.cancel().unwrap();
        assert_eq!(service.state(), &TaskState::Cancelled);
    }

    #[test]
    fn detects_cargo_build() {
        let root = std::env::temp_dir().join(format!("jetthing-runner-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        let plan = TaskPlan::detect_build(&root).unwrap();
        assert_eq!(plan.program, "cargo");
        assert_eq!(plan.args, ["build"]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
