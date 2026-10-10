//! Build-system service trait and the built-in providers.
//!
//! Every build tool Jetthing understands (Cargo, Gradle, Maven, Bazel, npm, Go)
//! implements [`BuildSystem`]. The platform asks the first provider that detects a
//! project root for build/run/test plans; providers that cannot run a project
//! return an actionable [`RunnerError`] instead of a plan that is known to fail.
//! Richer integrations (the Gradle model, Bazel targets, Android deploy) live in
//! their own crates and register additional or replacement providers.

use crate::{RunnerError, TaskKind, TaskPlan};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildSystemKind {
    Cargo,
    Gradle,
    Maven,
    Bazel,
    Npm,
    Go,
}

impl BuildSystemKind {
    pub fn id(self) -> &'static str {
        match self {
            BuildSystemKind::Cargo => "cargo",
            BuildSystemKind::Gradle => "gradle",
            BuildSystemKind::Maven => "maven",
            BuildSystemKind::Bazel => "bazel",
            BuildSystemKind::Npm => "npm",
            BuildSystemKind::Go => "go",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            BuildSystemKind::Cargo => "Cargo",
            BuildSystemKind::Gradle => "Gradle",
            BuildSystemKind::Maven => "Maven",
            BuildSystemKind::Bazel => "Bazel",
            BuildSystemKind::Npm => "npm",
            BuildSystemKind::Go => "Go",
        }
    }
}

/// A build tool integration. Implementations must not panic on malformed projects.
pub trait BuildSystem: Send + Sync {
    fn kind(&self) -> BuildSystemKind;

    /// Whether `root` is a project of this build system.
    fn detect(&self, root: &Path) -> bool;

    /// Plan for Build | Build Project.
    fn build_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError>;

    /// Plan for the default Run action.
    fn run_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError>;

    /// Plan that runs all tests.
    fn test_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError>;
}

/// Ordered provider list; the first provider that detects a root wins.
pub struct BuildSystemRegistry {
    providers: Vec<Box<dyn BuildSystem>>,
}

impl Default for BuildSystemRegistry {
    /// The built-in providers of this crate (Gradle and others are added by their crates).
    fn default() -> Self {
        Self {
            providers: vec![
                Box::new(Cargo),
                Box::new(Maven),
                Box::new(Bazel),
                Box::new(Npm),
                Box::new(Go),
            ],
        }
    }
}

impl BuildSystemRegistry {
    pub fn empty() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Adds a provider with precedence over the existing ones.
    pub fn register_first(&mut self, provider: Box<dyn BuildSystem>) {
        self.providers.insert(0, provider);
    }

    /// Adds a provider before the first provider of `kind` (or last if absent).
    pub fn register_before(&mut self, kind: BuildSystemKind, provider: Box<dyn BuildSystem>) {
        let index = self
            .providers
            .iter()
            .position(|existing| existing.kind() == kind)
            .unwrap_or(self.providers.len());
        self.providers.insert(index, provider);
    }

    pub fn detect(&self, root: &Path) -> Option<&dyn BuildSystem> {
        self.providers
            .iter()
            .find(|provider| provider.detect(root))
            .map(Box::as_ref)
    }

    pub fn kinds(&self) -> Vec<BuildSystemKind> {
        self.providers
            .iter()
            .map(|provider| provider.kind())
            .collect()
    }
}

/// Searches `PATH` for an executable, honouring `PATHEXT` on Windows.
pub fn find_program(name: &str) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.components().count() > 1 {
        return candidate.is_file().then(|| candidate.to_owned());
    }
    let paths = std::env::var_os("PATH")?;
    let extensions: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.BAT;.CMD".into())
            .split(';')
            .map(str::to_owned)
            .chain(std::iter::once(String::new()))
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&paths).find_map(|dir| {
        extensions.iter().find_map(|extension| {
            let path = dir.join(format!("{name}{extension}"));
            is_executable(&path).then_some(path)
        })
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Resolves a tool on `PATH` or returns an actionable error.
pub fn require_program(name: &str, hint: &str) -> Result<String, RunnerError> {
    find_program(name)
        .map(|_| name.to_owned())
        .ok_or_else(|| RunnerError::ToolNotFound {
            program: name.into(),
            hint: hint.into(),
        })
}

fn plan(
    kind: TaskKind,
    name: &str,
    program: String,
    args: &[&str],
    root: &Path,
) -> Result<TaskPlan, RunnerError> {
    Ok(TaskPlan::new(name, program, args.iter().copied(), root).with_kind(kind))
}

pub struct Cargo;

impl BuildSystem for Cargo {
    fn kind(&self) -> BuildSystemKind {
        BuildSystemKind::Cargo
    }
    fn detect(&self, root: &Path) -> bool {
        root.join("Cargo.toml").is_file()
    }
    fn build_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let cargo = require_program("cargo", "Install Rust with rustup (https://rustup.rs).")?;
        plan(TaskKind::Build, "Cargo Build", cargo, &["build"], root)
    }
    fn run_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        if !cargo_has_binary(root) {
            // Libraries have nothing to launch; running their tests is the useful default.
            return self.test_plan(root);
        }
        let cargo = require_program("cargo", "Install Rust with rustup (https://rustup.rs).")?;
        plan(TaskKind::Run, "Cargo Run", cargo, &["run"], root)
    }
    fn test_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let cargo = require_program("cargo", "Install Rust with rustup (https://rustup.rs).")?;
        plan(TaskKind::Test, "Cargo Test", cargo, &["test"], root)
    }
}

fn cargo_has_binary(root: &Path) -> bool {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .ok()
        .and_then(|source| source.parse::<toml::Value>().ok());
    root.join("src/main.rs").is_file()
        || root.join("src/bin").is_dir()
        || manifest
            .as_ref()
            .and_then(|manifest| manifest.get("bin"))
            .and_then(toml::Value::as_array)
            .is_some_and(|bins| !bins.is_empty())
}

pub struct Maven;

impl Maven {
    fn program(root: &Path) -> Result<String, RunnerError> {
        let wrapper = if cfg!(windows) { "mvnw.cmd" } else { "mvnw" };
        if root.join(wrapper).is_file() {
            return Ok(if cfg!(windows) {
                root.join(wrapper).display().to_string()
            } else {
                format!("./{wrapper}")
            });
        }
        require_program(
            "mvn",
            "Install Apache Maven and add it to PATH, or add the Maven wrapper (mvnw).",
        )
    }
}

impl BuildSystem for Maven {
    fn kind(&self) -> BuildSystemKind {
        BuildSystemKind::Maven
    }
    fn detect(&self, root: &Path) -> bool {
        root.join("pom.xml").is_file()
    }
    fn build_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        plan(
            TaskKind::Build,
            "Maven Compile",
            Self::program(root)?,
            &["compile"],
            root,
        )
    }
    fn run_plan(&self, _root: &Path) -> Result<TaskPlan, RunnerError> {
        Err(RunnerError::NotRunnable(
            "Maven projects have no standard run goal; create a run configuration".into(),
        ))
    }
    fn test_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        plan(
            TaskKind::Test,
            "Maven Test",
            Self::program(root)?,
            &["test"],
            root,
        )
    }
}

pub struct Bazel;

impl Bazel {
    pub fn is_workspace(root: &Path) -> bool {
        ["MODULE.bazel", "WORKSPACE", "WORKSPACE.bazel"]
            .iter()
            .any(|name| root.join(name).is_file())
    }

    /// Prefers `bazelisk`, which honours `.bazelversion`, then `bazel`.
    pub fn program() -> Result<String, RunnerError> {
        if find_program("bazelisk").is_some() {
            return Ok("bazelisk".into());
        }
        require_program(
            "bazel",
            "Install Bazelisk (https://github.com/bazelbuild/bazelisk) or Bazel and add it to PATH.",
        )
    }
}

impl BuildSystem for Bazel {
    fn kind(&self) -> BuildSystemKind {
        BuildSystemKind::Bazel
    }
    fn detect(&self, root: &Path) -> bool {
        Self::is_workspace(root)
    }
    fn build_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        plan(
            TaskKind::Build,
            "Bazel Build",
            Self::program()?,
            &["build", "//..."],
            root,
        )
    }
    fn run_plan(&self, _root: &Path) -> Result<TaskPlan, RunnerError> {
        Err(RunnerError::NotRunnable(
            "Bazel runs a specific target; create a run configuration with a target such as //:app"
                .into(),
        ))
    }
    fn test_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        plan(
            TaskKind::Test,
            "Bazel Test",
            Self::program()?,
            &["test", "//..."],
            root,
        )
    }
}

pub struct Npm;

impl BuildSystem for Npm {
    fn kind(&self) -> BuildSystemKind {
        BuildSystemKind::Npm
    }
    fn detect(&self, root: &Path) -> bool {
        root.join("package.json").is_file()
    }
    fn build_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let npm = require_program("npm", "Install Node.js (https://nodejs.org) and npm.")?;
        if npm_script(root, "build") {
            plan(TaskKind::Build, "npm Build", npm, &["run", "build"], root)
        } else {
            plan(TaskKind::Build, "npm Install", npm, &["install"], root)
        }
    }
    fn run_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let npm = require_program("npm", "Install Node.js (https://nodejs.org) and npm.")?;
        plan(TaskKind::Run, "npm Start", npm, &["start"], root)
    }
    fn test_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let npm = require_program("npm", "Install Node.js (https://nodejs.org) and npm.")?;
        plan(TaskKind::Test, "npm Test", npm, &["test"], root)
    }
}

fn npm_script(root: &Path, name: &str) -> bool {
    std::fs::read_to_string(root.join("package.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .is_some_and(|package| package["scripts"][name].is_string())
}

pub struct Go;

impl BuildSystem for Go {
    fn kind(&self) -> BuildSystemKind {
        BuildSystemKind::Go
    }
    fn detect(&self, root: &Path) -> bool {
        root.join("go.mod").is_file()
    }
    fn build_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let go = require_program("go", "Install Go (https://go.dev/dl) and add it to PATH.")?;
        plan(TaskKind::Build, "Go Build", go, &["build", "./..."], root)
    }
    fn run_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let go = require_program("go", "Install Go (https://go.dev/dl) and add it to PATH.")?;
        plan(TaskKind::Run, "Go Run", go, &["run", "."], root)
    }
    fn test_plan(&self, root: &Path) -> Result<TaskPlan, RunnerError> {
        let go = require_program("go", "Install Go (https://go.dev/dl) and add it to PATH.")?;
        plan(TaskKind::Test, "Go Test", go, &["test", "./..."], root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "jetthing-build-system-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn registry_detects_in_order_and_accepts_overrides() {
        let root = temp("order");
        std::fs::write(root.join("MODULE.bazel"), "").unwrap();
        std::fs::write(root.join("package.json"), "{}").unwrap();
        let mut registry = BuildSystemRegistry::default();
        assert_eq!(
            registry.detect(&root).unwrap().kind(),
            BuildSystemKind::Bazel
        );
        registry.register_first(Box::new(Npm));
        assert_eq!(registry.detect(&root).unwrap().kind(), BuildSystemKind::Npm);
        assert!(BuildSystemRegistry::empty().detect(&root).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bazel_and_maven_refuse_to_guess_a_run_target() {
        let root = temp("bazel");
        std::fs::write(root.join("WORKSPACE"), "").unwrap();
        assert!(Bazel.detect(&root));
        let error = Bazel.run_plan(&root).unwrap_err().to_string();
        assert!(error.contains("//:app"), "{error}");
        assert!(matches!(
            Maven.run_plan(&root),
            Err(RunnerError::NotRunnable(_))
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_tools_produce_actionable_errors() {
        let error = require_program("jetthing-definitely-missing-tool", "Install it.").unwrap_err();
        assert_eq!(
            error.to_string(),
            "`jetthing-definitely-missing-tool` was not found on PATH. Install it."
        );
    }

    #[test]
    fn npm_build_uses_build_script_when_present() {
        if find_program("npm").is_none() {
            eprintln!("skipping: npm is not installed");
            return;
        }
        let root = temp("npm");
        std::fs::write(root.join("package.json"), r#"{"scripts":{"build":"tsc"}}"#).unwrap();
        assert_eq!(Npm.build_plan(&root).unwrap().args, ["run", "build"]);
        std::fs::write(root.join("package.json"), "{}").unwrap();
        assert_eq!(Npm.build_plan(&root).unwrap().args, ["install"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cargo_library_runs_tests() {
        if find_program("cargo").is_none() {
            eprintln!("skipping: cargo is not installed");
            return;
        }
        let root = temp("cargo");
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        let run = Cargo.run_plan(&root).unwrap();
        assert_eq!(
            (run.name.as_str(), run.kind),
            ("Cargo Test", TaskKind::Test)
        );
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        assert_eq!(Cargo.run_plan(&root).unwrap().args, ["run"]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
