//! "Check for Updates" and in-place installation.

use crate::{IdeState, spec};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_update::{CheckOutcome, Release, UpdateEvent, UpdateSource, Version};
use serde_json::{Value, json};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate(Version),
    Available {
        version: Version,
        release: Box<Release>,
    },
    Installing(Version),
    /// Installed; takes effect after restarting the IDE.
    Installed(Version),
    Failed(String),
}

pub fn current_version() -> Version {
    Version::parse(CURRENT_VERSION).unwrap_or(Version {
        major: 0,
        minor: 0,
        patch: 0,
        pre: None,
    })
}

/// Only release layouts (`jetthing` executable from a distribution archive) can
/// update themselves; development builds (`target/*/jetthing-app`) cannot.
pub fn installable_executable() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    if exe.file_stem().and_then(|s| s.to_str()) == Some("jetthing") {
        Ok(exe)
    } else {
        Err("This is a development build; download the update from the release page.".into())
    }
}

impl IdeState {
    pub fn check_for_updates(&mut self, silent: bool) {
        if self.updates.is_busy() {
            return;
        }
        self.update_silent = silent;
        self.update_state = UpdateState::Checking;
        self.updates
            .check(UpdateSource::from_env(), current_version());
    }

    pub fn install_update(&mut self) -> Result<(), String> {
        let UpdateState::Available { version, release } = &self.update_state else {
            return Err("no update is available".into());
        };
        let exe = self
            .update_executable
            .clone()
            .map_or_else(installable_executable, Ok)?;
        self.update_executable = Some(exe.clone());
        let version = version.clone();
        self.updates
            .install(UpdateSource::from_env(), (**release).clone(), exe);
        self.status = format!("Downloading Jetthing {version}…");
        self.update_state = UpdateState::Installing(version);
        Ok(())
    }

    pub fn poll_updates(&mut self) {
        let Some(event) = self.updates.poll() else {
            return;
        };
        self.update_state = match event {
            UpdateEvent::Checked(Ok(CheckOutcome::Available { version, release })) => {
                self.status = format!("Jetthing {version} is available");
                self.update_notification = true;
                UpdateState::Available {
                    version,
                    release: Box::new(release),
                }
            }
            UpdateEvent::Checked(Ok(CheckOutcome::UpToDate { latest })) => {
                if !self.update_silent {
                    self.status = format!("Jetthing {CURRENT_VERSION} is up to date");
                    self.update_notification = true;
                }
                UpdateState::UpToDate(latest)
            }
            UpdateEvent::Checked(Err(error)) => {
                if !self.update_silent {
                    self.status = format!("Update check failed: {error}");
                    self.update_notification = true;
                }
                UpdateState::Failed(error)
            }
            UpdateEvent::Installed(Ok(version)) => {
                self.status = format!("Jetthing {version} installed; restart to finish");
                self.update_notification = true;
                UpdateState::Installed(version)
            }
            UpdateEvent::Installed(Err(error)) => {
                self.status = format!("Update failed: {error}");
                self.update_notification = true;
                UpdateState::Failed(error)
            }
        };
    }

    /// On Linux, current_exe follows the running inode into the `.old` backup.
    /// Use the installation target captured before replacement instead.
    pub fn update_restart_target(&self) -> Option<&std::path::Path> {
        self.update_executable.as_deref()
    }
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec(
                "app.prepareRestart",
                "Save Session for Restart",
                "Help",
                None,
                "{}",
            ),
            |_| true,
            |state: &mut IdeState, _| {
                if state.project.is_some() {
                    state.save_all().map_err(CommandError::Failed)?;
                    state.save_workspace().map_err(CommandError::Failed)?;
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "app.checkForUpdates",
                "Check for Updates…",
                "Help",
                None,
                r#"{"silent":"bool?"}"#,
            ),
            |state: &IdeState| !state.updates.is_busy(),
            |state: &mut IdeState, params| {
                let silent = params
                    .get("silent")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                state.check_for_updates(silent);
                Ok(json!({"current": CURRENT_VERSION}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "app.installUpdate",
                "Update and Restart",
                "Help",
                None,
                "{}",
            ),
            |state: &IdeState| matches!(state.update_state, UpdateState::Available { .. }),
            |state: &mut IdeState, _| {
                state.install_update().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "app.dismissUpdate",
                "Dismiss Update Notification",
                "Help",
                None,
                "{}",
            ),
            |state: &IdeState| state.update_notification,
            |state: &mut IdeState, _| {
                state.update_notification = false;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("app.about", "About Jetthing", "Help", None, "{}"),
            |_| true,
            |state: &mut IdeState, _| {
                let build = crate::build_info::BUILD;
                state.status = build.version_line("Jetthing");
                state.about_open = true;
                Ok(build.to_json())
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_update::parse_release;

    #[test]
    fn restart_preparation_saves_dirty_files_and_propagates_save_errors() {
        let root = std::env::temp_dir().join(format!("jetthing-restart-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("notes.txt");
        std::fs::write(&path, "old").unwrap();
        let commands = crate::built_in_commands();
        let mut state = IdeState::default();
        state.open_project(&root).unwrap();
        state.open_file(std::path::Path::new("notes.txt")).unwrap();
        state.documents[0].text.edit(0, 3, "saved").unwrap();
        commands
            .execute("app.prepareRestart", &mut state, Value::Null)
            .unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "saved");
        assert!(root.join(".jetthing/workspace.json").is_file());
        assert!(!state.documents[0].text.is_dirty());
        state.documents[0].text.edit(0, 5, "keep this").unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(
            commands
                .execute("app.prepareRestart", &mut state, Value::Null)
                .is_err()
        );
        assert!(state.documents[0].text.is_dirty());
        assert_eq!(state.documents[0].text.text(), "keep this");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn welcome_session_can_prepare_for_restart() {
        crate::built_in_commands()
            .execute("app.prepareRestart", &mut IdeState::default(), Value::Null)
            .unwrap();
    }

    #[test]
    fn about_opens_the_dialog_and_reports_the_build() {
        let mut state = IdeState::default();
        let info = crate::built_in_commands()
            .execute("app.about", &mut state, Value::Null)
            .unwrap();
        assert!(state.about_open);
        assert_eq!(info["version"], CURRENT_VERSION);
        assert_eq!(info["commit"], crate::build_info::BUILD.commit);
        assert!(
            state
                .status
                .starts_with(&format!("Jetthing {CURRENT_VERSION} ("))
        );
    }

    #[test]
    fn version_comes_from_the_workspace() {
        assert_eq!(current_version().to_string(), CURRENT_VERSION);
    }

    #[test]
    fn install_requires_an_available_update_and_a_release_build() {
        let mut state = IdeState::default();
        assert!(state.install_update().is_err());
        let release = parse_release(
            r#"{"tag_name":"v99.0.0","html_url":"https://example.invalid","assets":[]}"#,
        )
        .unwrap();
        state.update_state = UpdateState::Available {
            version: release.version().unwrap(),
            release: Box::new(release),
        };
        // Test binaries are not named `jetthing`, so they never replace themselves.
        let error = state.install_update().unwrap_err();
        assert!(error.contains("development build"));
        assert!(matches!(state.update_state, UpdateState::Available { .. }));
    }
}
