//! Deterministic project templates. Generation never runs downloaded code.
use crate::{Project, ProjectError};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Empty,
    RustBinary,
    RustLibrary,
    JavaGradleKotlin,
    JavaGradleGroovy,
}

impl ProjectKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "rust-binary" => Some(Self::RustBinary),
            "rust-library" => Some(Self::RustLibrary),
            "java-gradle-kotlin" => Some(Self::JavaGradleKotlin),
            "java-gradle-groovy" => Some(Self::JavaGradleGroovy),
            _ => None,
        }
    }
}

/// Creates a new child directory only; existing directories are never reused.
pub fn create_project(
    parent: &Path,
    name: &str,
    kind: ProjectKind,
    group: &str,
) -> Result<Project, ProjectError> {
    if name.is_empty()
        || name.len() > 80
        || !name.starts_with(|c: char| c.is_ascii_alphabetic())
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ProjectError::InvalidName(name.into()));
    }
    let java = matches!(
        kind,
        ProjectKind::JavaGradleKotlin | ProjectKind::JavaGradleGroovy
    );
    if java && !valid_package(group) {
        return Err(ProjectError::InvalidName(format!(
            "Java group/package: {group}"
        )));
    }
    let parent = parent.canonicalize()?;
    if !parent.is_dir() {
        return Err(ProjectError::NotDirectory(parent));
    }
    let root = parent.join(name);
    match std::fs::create_dir(&root) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ProjectError::AlreadyExists(root));
        }
        Err(e) => return Err(e.into()),
    }
    let mut files: Vec<(PathBuf, String)> = vec![
        ("README.md".into(), format!("# {name}\n")),
        (
            ".gitignore".into(),
            ".jetthing/\n.idea/\ntarget/\nbuild/\n.gradle/\n".into(),
        ),
    ];
    match kind {
        ProjectKind::Empty => {}
        ProjectKind::RustBinary | ProjectKind::RustLibrary => {
            files.push(("Cargo.toml".into(), format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n")));
            files.push(if kind == ProjectKind::RustBinary {
                ("src/main.rs".into(), "fn main() {\n    println!(\"Hello, world!\");\n}\n".into())
            } else {
                ("src/lib.rs".into(), "pub fn add(left: u64, right: u64) -> u64 {\n    left + right\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add(2, 2), 4);\n    }\n}\n".into())
            });
        }
        ProjectKind::JavaGradleKotlin | ProjectKind::JavaGradleGroovy => {
            let kotlin = kind == ProjectKind::JavaGradleKotlin;
            files.push((
                if kotlin {
                    "settings.gradle.kts"
                } else {
                    "settings.gradle"
                }
                .into(),
                format!("rootProject.name = \"{name}\"\n"),
            ));
            let build = if kotlin {
                format!(
                    "plugins {{\n    application\n}}\n\ngroup = \"{group}\"\nversion = \"1.0-SNAPSHOT\"\n\nrepositories {{\n    mavenCentral()\n}}\n\njava {{\n    toolchain {{\n        languageVersion.set(JavaLanguageVersion.of(21))\n    }}\n}}\n\napplication {{\n    mainClass.set(\"{group}.Main\")\n}}\n"
                )
            } else {
                format!(
                    "plugins {{\n    id 'application'\n}}\n\ngroup = '{group}'\nversion = '1.0-SNAPSHOT'\n\nrepositories {{\n    mavenCentral()\n}}\n\njava {{\n    toolchain {{\n        languageVersion = JavaLanguageVersion.of(21)\n    }}\n}}\n\napplication {{\n    mainClass = '{group}.Main'\n}}\n"
                )
            };
            files.push((
                if kotlin {
                    "build.gradle.kts"
                } else {
                    "build.gradle"
                }
                .into(),
                build,
            ));
            files.push((format!("src/main/java/{}/Main.java", group.replace('.', "/")).into(), format!("package {group};\n\npublic class Main {{\n    public static void main(String[] args) {{\n        System.out.println(\"Hello, world!\");\n    }}\n}}\n")));
            files[0].1.push_str("\nRequires JDK 21 and Gradle on PATH. Run `gradle run` or `gradle build`.\nGenerate a pinned wrapper with `gradle wrapper` if desired.\n");
        }
    }
    for (path, contents) in files {
        let target = root.join(path);
        let write = || -> std::io::Result<()> {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            use std::io::Write;
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&target)?
                .write_all(contents.as_bytes())
        };
        if let Err(error) = write() {
            return Err(ProjectError::Generation {
                root,
                source: error,
            });
        }
    }
    Project::open(root)
}

fn valid_package(group: &str) -> bool {
    const RESERVED: &[&str] = &[
        "class",
        "public",
        "private",
        "static",
        "void",
        "int",
        "package",
        "import",
        "new",
        "return",
        "if",
        "else",
        "while",
        "for",
        "switch",
        "case",
        "default",
        "true",
        "false",
        "null",
        "enum",
        "record",
        "interface",
        "extends",
        "implements",
        "var",
        "_",
        "assert",
        "boolean",
        "break",
        "byte",
        "catch",
        "char",
        "const",
        "continue",
        "do",
        "double",
        "final",
        "finally",
        "float",
        "goto",
        "instanceof",
        "long",
        "native",
        "protected",
        "short",
        "strictfp",
        "super",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "try",
        "volatile",
        "abstract",
    ];
    !group.is_empty()
        && group.len() <= 160
        && group.split('.').all(|part| {
            !RESERVED.contains(&part)
                && part.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn templates_and_collisions_are_safe() {
        let parent =
            std::env::temp_dir().join(format!("jetthing-generators-{}", std::process::id()));
        std::fs::create_dir_all(&parent).unwrap();
        for (name, kind, expected) in [
            ("app", ProjectKind::RustBinary, "src/main.rs"),
            ("library", ProjectKind::RustLibrary, "src/lib.rs"),
            (
                "java",
                ProjectKind::JavaGradleKotlin,
                "src/main/java/org/example/Main.java",
            ),
            ("groovy", ProjectKind::JavaGradleGroovy, "build.gradle"),
            ("empty", ProjectKind::Empty, "README.md"),
        ] {
            let root = parent.join(name);
            // Unique test-owned directory; support rerunning after a failed test.
            if root.exists() {
                std::fs::remove_dir_all(&root).unwrap();
            }
            let project = create_project(&parent, name, kind, "org.example").unwrap();
            assert!(project.root().join(expected).is_file());
            assert!(matches!(
                create_project(&parent, name, kind, "org.example"),
                Err(ProjectError::AlreadyExists(_))
            ));
        }
        assert!(create_project(&parent, "../escape", ProjectKind::Empty, "").is_err());
        assert!(
            create_project(&parent, "bad", ProjectKind::JavaGradleKotlin, "org.class").is_err()
        );
        assert!(!parent.join("bad").exists());
        let source =
            std::fs::read_to_string(parent.join("java/src/main/java/org/example/Main.java"))
                .unwrap();
        assert_eq!(
            source,
            "package org.example;\n\npublic class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, world!\");\n    }\n}\n"
        );
        std::fs::remove_dir_all(parent).unwrap();
    }
}
