//! User options for Gradle invocations (Settings | Build Tools | Gradle).
//!
//! The IDE copies the settings here when they change, so every plan built by
//! [`crate::GradleBuild`] (build, run, test, sync) follows them.

use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Distribution {
    /// The project's `gradlew` (falls back to `gradle` on PATH without one).
    #[default]
    Wrapper,
    /// `gradle` from PATH, ignoring the wrapper.
    Local,
}

impl Distribution {
    /// Parses the settings option id; unknown ids mean the wrapper.
    pub fn from_id(id: &str) -> Self {
        if id == "local" {
            Self::Local
        } else {
            Self::Wrapper
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GradleOptions {
    pub distribution: Distribution,
    pub offline: bool,
    /// `GRADLE_USER_HOME`; `None` keeps the default.
    pub user_home: Option<String>,
    /// `JAVA_HOME` for Gradle; `None` keeps the environment's.
    pub jvm: Option<String>,
}

static OPTIONS: RwLock<Option<GradleOptions>> = RwLock::new(None);

/// The current options (defaults until [`set_options`] is called).
pub fn options() -> GradleOptions {
    OPTIONS
        .read()
        .map(|guard| guard.clone().unwrap_or_default())
        .unwrap_or_default()
}

pub fn set_options(options: GradleOptions) {
    if let Ok(mut guard) = OPTIONS.write() {
        *guard = Some(options);
    }
}
