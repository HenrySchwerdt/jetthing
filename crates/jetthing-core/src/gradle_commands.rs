//! Gradle sync and task execution (Gradle tool window actions).

use crate::{IdeState, spec, string_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_gradle::{GradleBuild, GradleModel};
use jetthing_runner::{BuildSystemKind, RunnerError, TaskKind, TaskState};
use serde_json::{Value, json};

impl IdeState {
    pub fn is_gradle_project(&self) -> bool {
        self.detected_build
            .as_ref()
            .is_some_and(|build| build.kind == BuildSystemKind::Gradle)
    }

    fn gradle_build(&self) -> Result<GradleBuild, RunnerError> {
        let root = self
            .project
            .as_ref()
            .ok_or_else(|| RunnerError::NotRunnable("no project is open".into()))?
            .root();
        GradleBuild::detect(root).ok_or_else(|| RunnerError::NoBuildSystem(root.to_owned()))
    }

    /// Loads the model cached by the last sync (no Gradle process is started).
    pub fn load_gradle_model(&mut self) {
        self.gradle_model = None;
        self.gradle_error = None;
        let Some(project) = &self.project else {
            return;
        };
        match GradleModel::load_cached(project.root()) {
            Some(Ok(model)) => self.gradle_model = Some(model),
            Some(Err(error)) => self.gradle_error = Some(error),
            None => {}
        }
    }

    pub fn start_gradle_sync(&mut self) -> Result<(), String> {
        let plan = self.gradle_build().and_then(|build| build.sync_plan());
        self.start_task(plan, "Gradle Sync", TaskKind::Sync)
    }

    /// Runs task paths and options as typed in Execute Gradle Task
    /// (`:app:run`, `test --tests 'Foo*'` is passed token by token).
    pub fn run_gradle_task(&mut self, command: &str) -> Result<(), String> {
        let tokens: Vec<String> = command.split_whitespace().map(str::to_owned).collect();
        let Some(first) = tokens.iter().find(|token| !token.starts_with('-')) else {
            return Err("enter at least one Gradle task, e.g. :app:test".into());
        };
        let name = first.rsplit(':').next().unwrap_or(first);
        let kind = match name {
            "run" | "bootRun" => TaskKind::Run,
            "test" | "check" => TaskKind::Test,
            _ => TaskKind::Build,
        };
        let label = format!("Gradle {}", tokens.join(" "));
        let plan = self
            .gradle_build()
            .and_then(|build| build.task_plan(kind, label.clone(), &tokens));
        self.start_task(plan, &label, kind)
    }

    /// Applies the result of a finished sync.
    pub(crate) fn finish_gradle_sync(&mut self) {
        match self.runner.state() {
            TaskState::Exited(0) => {
                self.load_gradle_model();
                if let Some(model) = &self.gradle_model {
                    let plural = |count: usize, word: &str| {
                        format!("{count} {word}{}", if count == 1 { "" } else { "s" })
                    };
                    self.status = format!(
                        "Gradle sync finished: {}, {} (Gradle {})",
                        plural(model.projects.len(), "project"),
                        plural(model.task_count(), "task"),
                        model.gradle_version
                    );
                } else {
                    self.gradle_error =
                        Some("Gradle finished but did not write the project model".into());
                    self.status = "Gradle sync did not produce a model".into();
                }
            }
            TaskState::Exited(_) => {
                self.gradle_error = Some("Gradle sync failed; see the Build tool window".into());
            }
            _ => {}
        }
    }
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec(
                "gradle.sync",
                "Sync All Gradle Projects",
                "Gradle",
                None,
                "{}",
            ),
            |state: &IdeState| state.is_gradle_project() && !state.runner.is_running(),
            |state: &mut IdeState, _| {
                state.start_gradle_sync().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "gradle.runTask",
                "Execute Gradle Task",
                "Gradle",
                None,
                r#"{"task":"string"}"#,
            ),
            |state: &IdeState| state.is_gradle_project() && !state.runner.is_running(),
            |state: &mut IdeState, params| {
                let task = string_param(&params, "task")?;
                state.run_gradle_task(&task).map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "gradle.refresh",
                "Reload Gradle Model from Cache",
                "Gradle",
                None,
                "{}",
            ),
            |state: &IdeState| state.is_gradle_project(),
            |state: &mut IdeState, _| {
                state.load_gradle_model();
                if let Some(error) = &state.gradle_error {
                    return Err(CommandError::Failed(error.clone()));
                }
                Ok(match &state.gradle_model {
                    Some(model) => json!({
                        "gradleVersion": model.gradle_version,
                        "projects": model.projects.iter().map(|p| &p.path).collect::<Vec<_>>(),
                        "tasks": model.task_count(),
                    }),
                    None => Value::Null,
                })
            },
        ))
        .expect("unique built-in id");
}

#[cfg(all(test, unix))]
mod tests {
    use crate::{IdeState, built_in_commands};
    use jetthing_runner::TaskState;
    use serde_json::{Value, json};
    use std::time::{Duration, Instant};

    fn fixture(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("jetthing-gradle-cmd-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("gradle")).unwrap();
        std::fs::write(root.join("settings.gradle.kts"), "include(\":app\")\n").unwrap();
        root
    }

    fn wait(state: &mut IdeState) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while state.runner.is_running() && Instant::now() < deadline {
            state.poll_runner();
            std::thread::sleep(Duration::from_millis(10));
        }
        state.poll_runner();
    }

    #[test]
    fn sync_with_fake_wrapper_loads_model_and_tasks_run_through_wrapper() {
        let root = fixture("sync");
        // A fake wrapper that writes a model where Gradle would, and echoes task runs.
        std::fs::write(
            root.join("gradlew"),
            r#"#!/bin/sh
for arg in "$@"; do
  case "$arg" in -Pjetthing.modelFile=*) out="${arg#-Pjetthing.modelFile=}";; esac
done
if [ -n "$out" ]; then
  printf '{"gradleVersion":"8.14.3","rootDir":"%s","projects":[{"path":":","name":"r","dir":"%s"},{"path":":app","name":"app","dir":"%s/app","tasks":[{"name":"run","path":":app:run","group":"application"}]}]}' "$PWD" "$PWD" "$PWD" > "$out"
  echo "> Task :jetthingModel"
else
  echo "ran $*"
fi
"#,
        )
        .unwrap();
        std::process::Command::new("chmod")
            .args(["+x", "gradlew"])
            .current_dir(&root)
            .status()
            .unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        assert!(state.gradle_model.is_none());
        assert!(commands.is_enabled("gradle.sync", &state));
        commands
            .execute("gradle.sync", &mut state, Value::Null)
            .unwrap();
        wait(&mut state);
        assert_eq!(state.runner.state(), &TaskState::Exited(0));
        let model = state
            .gradle_model
            .as_ref()
            .expect("model loaded after sync");
        assert_eq!(model.project(":app").unwrap().tasks[0].path, ":app:run");
        assert!(
            state.status.contains("2 projects, 1 task "),
            "{}",
            state.status
        );

        commands
            .execute("gradle.runTask", &mut state, json!({"task": ":app:run"}))
            .unwrap();
        wait(&mut state);
        assert_eq!(
            state.runner.console().lines()[0].text,
            "ran :app:run --console=plain"
        );
        assert!(
            commands
                .execute(
                    "gradle.runTask",
                    &mut state,
                    json!({"task": "  --offline "})
                )
                .is_err()
        );
        commands
            .execute(
                "gradle.runTask",
                &mut state,
                json!({"task": "clean :app:test --offline"}),
            )
            .unwrap();
        wait(&mut state);
        assert_eq!(
            state.runner.console().lines()[0].text,
            "ran clean :app:test --offline --console=plain"
        );

        // Reopening restores the cached model without running Gradle.
        let mut reopened = IdeState::default();
        commands
            .execute("project.open", &mut reopened, json!({"path": root}))
            .unwrap();
        assert_eq!(reopened.gradle_model, state.gradle_model);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn gradle_commands_are_disabled_for_other_projects() {
        let root =
            std::env::temp_dir().join(format!("jetthing-gradle-cmd-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("go.mod"), "module x\n").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        for id in ["gradle.sync", "gradle.runTask", "gradle.refresh"] {
            assert!(!commands.is_enabled(id, &state), "{id}");
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
