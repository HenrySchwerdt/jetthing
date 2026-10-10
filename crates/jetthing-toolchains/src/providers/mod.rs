//! Built-in component providers (the first-wave catalog).
//!
//! | Component | Source |
//! |---|---|
//! | `jdk` | foojay Disco API (Temurin by default) |
//! | `jdtls` | download.eclipse.org milestones |
//! | `rust-analyzer` | GitHub releases of rust-lang/rust-analyzer |
//! | `bazelisk` | GitHub releases of bazelbuild/bazelisk |
//! | `taplo` | GitHub releases of tamasfe/taplo |

pub mod github;
pub mod jdk;
pub mod jdtls;

use crate::{Arch, ArchiveKind, Catalog, ComponentKind, ComponentSpec, Os, Platform};
use github::GithubReleaseProvider;
use std::sync::Arc;

/// The catalog Jetthing ships with.
pub fn builtin_catalog() -> Catalog {
    let mut catalog = Catalog::new();
    catalog.register(Arc::new(jdk::FoojayJdk::default()));
    catalog.register(Arc::new(jdtls::EclipseJdtls::default()));
    catalog.register(Arc::new(rust_analyzer()));
    catalog.register(Arc::new(bazelisk()));
    catalog.register(Arc::new(taplo()));
    catalog
}

/// Rust target triple for a platform (`x86_64-unknown-linux-gnu`).
pub fn rust_triple(platform: &Platform) -> &'static str {
    match (platform.os, platform.arch) {
        (Os::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu",
        (Os::Linux, Arch::Aarch64) => "aarch64-unknown-linux-gnu",
        (Os::Macos, Arch::X86_64) => "x86_64-apple-darwin",
        (Os::Macos, Arch::Aarch64) => "aarch64-apple-darwin",
        (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc",
        (Os::Windows, Arch::Aarch64) => "aarch64-pc-windows-msvc",
    }
}

pub fn rust_analyzer() -> GithubReleaseProvider {
    let mut spec = ComponentSpec::new(
        "rust-analyzer",
        "rust-analyzer",
        ComponentKind::LanguageServer,
        "rust-analyzer",
    );
    spec.homepage = Some("https://rust-analyzer.github.io".into());
    GithubReleaseProvider::new(spec, "rust-lang/rust-analyzer", |platform, _tag| {
        let triple = rust_triple(platform);
        Some(if platform.os == Os::Windows {
            (format!("rust-analyzer-{triple}.zip"), ArchiveKind::Zip)
        } else {
            (
                format!("rust-analyzer-{triple}.gz"),
                ArchiveKind::Gz {
                    file_name: "rust-analyzer".into(),
                },
            )
        })
    })
    .with_tag_channels(&["nightly"])
}

fn go_style(platform: &Platform) -> (&'static str, &'static str) {
    let os = match platform.os {
        Os::Linux => "linux",
        Os::Macos => "darwin",
        Os::Windows => "windows",
    };
    let arch = match platform.arch {
        Arch::X86_64 => "amd64",
        Arch::Aarch64 => "arm64",
    };
    (os, arch)
}

pub fn bazelisk() -> GithubReleaseProvider {
    let mut spec = ComponentSpec::new("bazelisk", "Bazelisk", ComponentKind::BuildTool, "bazelisk");
    spec.homepage = Some("https://github.com/bazelbuild/bazelisk".into());
    GithubReleaseProvider::new(spec, "bazelbuild/bazelisk", |platform, _tag| {
        let (os, arch) = go_style(platform);
        let exe = if platform.os == Os::Windows {
            ".exe"
        } else {
            ""
        };
        Some((
            format!("bazelisk-{os}-{arch}{exe}"),
            ArchiveKind::Raw {
                file_name: format!("bazelisk{exe}"),
            },
        ))
    })
}

pub fn taplo() -> GithubReleaseProvider {
    let mut spec = ComponentSpec::new(
        "taplo",
        "Taplo (TOML)",
        ComponentKind::LanguageServer,
        "taplo",
    );
    spec.homepage = Some("https://taplo.tamasfe.dev".into());
    GithubReleaseProvider::new(spec, "tamasfe/taplo", |platform, _tag| {
        let os = match platform.os {
            Os::Linux => "linux",
            Os::Macos => "darwin",
            Os::Windows => "windows",
        };
        let arch = match platform.arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        };
        Some(if platform.os == Os::Windows {
            (format!("taplo-{os}-{arch}.zip"), ArchiveKind::Zip)
        } else {
            (
                format!("taplo-{os}-{arch}.gz"),
                ArchiveKind::Gz {
                    file_name: "taplo".into(),
                },
            )
        })
    })
}
