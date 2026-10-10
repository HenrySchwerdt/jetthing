//! Built-in providers against recorded-shape responses on a local server.

mod support;

use jetthing_toolchains::download::{HttpClient, sha256_hex};
use jetthing_toolchains::providers::github::GithubReleaseProvider;
use jetthing_toolchains::providers::{jdk::FoojayJdk, jdtls::EclipseJdtls, rust_analyzer};
use jetthing_toolchains::{
    ArchiveKind, Checksum, ComponentProvider, ComponentSettings, Platform, ToolchainError,
    VersionRequest,
};
use std::sync::atomic::AtomicBool;
use support::FakeServer;

fn linux() -> Platform {
    Platform::parse("linux", "x86_64").unwrap()
}

fn foojay(server: &FakeServer) {
    server.serve(
        "/disco/major_versions",
        r#"{"result":[{"major_version":25,"term_of_support":"LTS"},{"major_version":27,"term_of_support":"STS"},{"major_version":21,"term_of_support":"LTS"}]}"#,
    );
    let package = |id: &str, version: &str, libc: &str, fx: bool| {
        format!(
            r#"{{"java_version":"{version}","archive_type":"tar.gz","javafx_bundled":{fx},"lib_c_type":"{libc}","size":100,"links":{{"pkg_info_uri":"{}"}}}}"#,
            server.url(&format!("/disco/ids/{id}"))
        )
    };
    server.serve(
        "/disco/packages",
        format!(
            r#"{{"result":[{},{},{},{}]}}"#,
            package("a", "21.0.3+9", "glibc", false),
            package("b", "21.0.4+7", "glibc", false),
            package("c", "21.0.5+1", "musl", false),
            package("d", "21.0.9+1", "glibc", true),
        ),
    );
    for (id, file) in [("a", "jdk-21.0.3.tar.gz"), ("b", "jdk-21.0.4.tar.gz")] {
        server.serve(
            &format!("/disco/ids/{id}"),
            format!(
                r#"{{"result":[{{"filename":"{file}","direct_download_uri":"{}","checksum":"{}","checksum_type":"sha256","checksum_uri":""}}]}}"#,
                server.url(&format!("/dl/{file}")),
                "ab".repeat(32)
            ),
        );
    }
}

#[test]
fn foojay_picks_newest_matching_glibc_jdk_without_javafx() {
    let server = FakeServer::start();
    foojay(&server);
    let provider = FoojayJdk::with_api(&server.url("/disco"));
    let http = HttpClient::new();
    let release = provider
        .resolve(&VersionRequest::parse("21"), &linux(), &http, None)
        .unwrap();
    assert_eq!(release.version.as_str(), "21.0.4+7");
    assert_eq!(release.artifact.url, server.url("/dl/jdk-21.0.4.tar.gz"));
    assert_eq!(release.artifact.checksum, Checksum::Sha256("ab".repeat(32)));
    assert_eq!(release.artifact.archive, ArchiveKind::TarGz);
    let query = &server.requests("/disco/packages")[0].path;
    assert!(query.contains("version=21&distribution=temurin"), "{query}");
    assert!(query.contains("lib_c_type=glibc") && query.contains("latest=available"));

    // `lts` resolves the newest LTS feature release first.
    let _ = provider.resolve(&VersionRequest::parse("lts"), &linux(), &http, None);
    let query = &server.requests("/disco/packages")[1].path;
    assert!(query.contains("version=25&"), "{query}");

    // Distribution from settings; invalid names are refused before any request.
    let settings = ComponentSettings {
        distribution: Some("zulu".into()),
        ..Default::default()
    };
    let _ = provider.resolve(
        &VersionRequest::parse("21.0.3"),
        &linux(),
        &http,
        Some(&settings),
    );
    let query = &server.requests("/disco/packages")[2].path;
    assert!(query.contains("version=21.0.3&distribution=zulu") && !query.contains("latest="));
    let bad = ComponentSettings {
        distribution: Some("zulu&x=1".into()),
        ..Default::default()
    };
    assert!(matches!(
        provider.resolve(&VersionRequest::Latest, &linux(), &http, Some(&bad)),
        Err(ToolchainError::Settings { .. })
    ));
    assert!(matches!(
        provider.resolve(&VersionRequest::parse("11"), &linux(), &http, None),
        Err(ToolchainError::NoMatchingVersion { .. })
    ));
}

#[test]
fn jdtls_resolves_milestones_and_installs_with_sidecar_checksum() {
    let server = FakeServer::start();
    // jdtls tarballs have no single root directory.
    let archive = {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for (path, data) in [
            ("bin/jdtls", &b"launcher"[..]),
            ("plugins/x.jar", &b"jar"[..]),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, path, data).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    };
    server.serve(
        "/jdtls/milestones/",
        "<a href='/jdtls/milestones/1.9.0'> 1.9.0</a><a href='/jdtls/milestones/1.40.0'>1.40.0</a>",
    );
    server.serve(
        "/jdtls/milestones/1.40.0/latest.txt",
        "jdt-language-server-1.40.0-202409261450.tar.gz\n",
    );
    let file = "/jdtls/milestones/1.40.0/jdt-language-server-1.40.0-202409261450.tar.gz";
    server.serve(file, archive.clone());
    server.serve(&format!("{file}.sha256"), sha256_hex(&archive));
    let provider = EclipseJdtls::with_base(&server.url("/jdtls"));
    let http = HttpClient::new();
    let release = provider
        .resolve(&VersionRequest::Latest, &linux(), &http, None)
        .unwrap();
    assert_eq!(release.version.as_str(), "1.40.0");
    assert_eq!(
        release.artifact.checksum,
        Checksum::Sha256Url(server.url(&format!("{file}.sha256")))
    );
    let dir = tempfile::tempdir().unwrap();
    let store = jetthing_toolchains::ToolchainStore::new(dir.path());
    let installed = store
        .install(&release, &http, &mut |_| {}, &AtomicBool::new(false))
        .unwrap();
    assert!(installed.path.join("bin/jdtls").is_file());
    assert!(installed.path.join("plugins/x.jar").is_file());

    server.serve("/jdtls/milestones/1.9.0/latest.txt", "../../evil.tar.gz");
    assert!(matches!(
        provider.resolve(&VersionRequest::parse("1.9"), &linux(), &http, None),
        Err(ToolchainError::Catalog { .. })
    ));
}

fn github_release(server: &FakeServer, tag: &str, assets: &[(&str, Option<String>)]) -> String {
    let assets: Vec<String> = assets
        .iter()
        .map(|(name, digest)| {
            format!(
                r#"{{"name":"{name}","browser_download_url":"{}","size":3,"digest":{}}}"#,
                server.url(&format!("/dl/{tag}/{name}")),
                digest
                    .as_ref()
                    .map_or("null".to_string(), |d| format!("\"sha256:{d}\""))
            )
        })
        .collect();
    format!(
        r#"{{"tag_name":"{tag}","draft":false,"assets":[{}]}}"#,
        assets.join(",")
    )
}

#[test]
fn github_releases_use_asset_digests_or_checksum_assets() {
    let server = FakeServer::start();
    let digest = "cd".repeat(32);
    server.serve(
        "/api/repos/rust-lang/rust-analyzer/releases/latest",
        github_release(
            &server,
            "2026-10-06",
            &[
                (
                    "rust-analyzer-x86_64-unknown-linux-gnu.gz",
                    Some(digest.clone()),
                ),
                ("rust-analyzer-x86_64-apple-darwin.gz", None),
            ],
        ),
    );
    server.serve(
        "/api/repos/rust-lang/rust-analyzer/releases/tags/nightly",
        github_release(
            &server,
            "nightly",
            &[("rust-analyzer-x86_64-apple-darwin.gz", None)],
        ),
    );
    let provider = rust_analyzer().with_api(&server.url("/api"));
    let http = HttpClient::new();
    let release = provider
        .resolve(&VersionRequest::Latest, &linux(), &http, None)
        .unwrap();
    assert_eq!(release.version.as_str(), "2026-10-06");
    assert_eq!(release.artifact.checksum, Checksum::Sha256(digest));
    assert!(matches!(release.artifact.archive, ArchiveKind::Gz { .. }));

    // No digest and no checksum asset: refuse instead of installing unverified.
    let mac = Platform::parse("macos", "aarch64").unwrap();
    assert!(matches!(
        provider.resolve(&VersionRequest::Latest, &mac, &http, None),
        Err(ToolchainError::UnsupportedPlatform { .. })
    ));
    let intel_mac = Platform::parse("macos", "x86_64").unwrap();
    let error = provider
        .resolve(&VersionRequest::parse("nightly"), &intel_mac, &http, None)
        .unwrap_err();
    assert!(error.to_string().contains("refusing"), "{error}");

    // `v`-prefixed tags and checksum-list assets.
    let tool = GithubReleaseProvider::new(
        jetthing_toolchains::ComponentSpec::new(
            "demo",
            "Demo",
            jetthing_toolchains::ComponentKind::Formatter,
            "demo",
        ),
        "o/demo",
        |_, tag| Some((format!("demo-{tag}.tar.gz"), ArchiveKind::TarGz)),
    )
    .with_api(&server.url("/api"));
    server.serve(
        "/api/repos/o/demo/releases/tags/v1.2.3",
        github_release(
            &server,
            "v1.2.3",
            &[("demo-v1.2.3.tar.gz", None), ("checksums.txt", None)],
        ),
    );
    let release = tool
        .resolve(&VersionRequest::parse("1.2.3"), &linux(), &http, None)
        .unwrap();
    assert_eq!(release.version.as_str(), "1.2.3");
    assert_eq!(
        release.artifact.checksum,
        Checksum::Sha256Url(server.url("/dl/v1.2.3/checksums.txt"))
    );
    assert!(matches!(
        tool.resolve(&VersionRequest::parse("9.9.9"), &linux(), &http, None),
        Err(ToolchainError::NoMatchingVersion { .. })
    ));
}

/// Live network check of the real JDK + jdtls sources; run with `--ignored`.
#[test]
#[ignore = "downloads ~380 MB from api.foojay.io and download.eclipse.org"]
fn live_jdk_and_jdtls_install() {
    let dir = tempfile::tempdir().unwrap();
    let store = jetthing_toolchains::ToolchainStore::new(dir.path());
    let http = HttpClient::new();
    let platform = Platform::current().expect("supported platform");
    for provider in [
        Box::new(FoojayJdk::default()) as Box<dyn ComponentProvider>,
        Box::new(EclipseJdtls::default()),
    ] {
        let release = provider
            .resolve(&VersionRequest::parse("21"), &platform, &http, None)
            .or_else(|_| provider.resolve(&VersionRequest::Latest, &platform, &http, None))
            .unwrap();
        let installed = store
            .install(&release, &http, &mut |_| {}, &AtomicBool::new(false))
            .unwrap();
        assert!(
            installed.executable(provider.spec(), &platform).is_some(),
            "{} has no executable",
            provider.spec().id
        );
    }
}
