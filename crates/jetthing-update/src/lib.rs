//! Self-update from GitHub releases ("Check for Updates" in the reference IDE).
//!
//! Release archives are produced by `cargo xtask dist` and the release workflow:
//! `jetthing-<version>-<os>-<arch>.tar.gz` (`.zip` on Windows) with a
//! `<archive>.sha256` checksum next to it. Updates are verified against that
//! checksum before the running executable is replaced.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use thiserror::Error;

pub const DEFAULT_REPOSITORY: &str = "HenrySchwerdt/jetthing";

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("network error: {0}")]
    Network(String),
    #[error("unexpected release data: {0}")]
    Format(String),
    #[error("no download for {0} in this release")]
    NoAsset(String),
    #[error("checksum mismatch for {0}")]
    Checksum(String),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
}

/// `major.minor.patch` with an optional pre-release suffix (`1.2.3-beta.1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
}

impl Version {
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim().trim_start_matches('v');
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre.to_owned())),
            None => (text, None),
        };
        let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
        let version = Version {
            major: parts.next()??,
            minor: parts.next()??,
            patch: parts.next().unwrap_or(Some(0))?,
            pre,
        };
        parts.next().is_none().then_some(version)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (&self.pre, &other.pre) {
                // A release sorts after its pre-releases.
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    pub name: String,
    /// API URL; with `Accept: application/octet-stream` it serves the file (works with tokens).
    pub url: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn version(&self) -> Option<Version> {
        Version::parse(&self.tag_name)
    }

    /// The archive built for `os`/`arch` (values of `std::env::consts`).
    pub fn asset_for(&self, os: &str, arch: &str) -> Option<&Asset> {
        let suffix = format!("-{os}-{arch}.{}", archive_extension(os));
        self.assets
            .iter()
            .find(|asset| asset.name.starts_with("jetthing-") && asset.name.ends_with(&suffix))
    }

    pub fn checksum_for(&self, asset: &Asset) -> Option<&Asset> {
        let name = format!("{}.sha256", asset.name);
        self.assets.iter().find(|candidate| candidate.name == name)
    }
}

pub fn archive_extension(os: &str) -> &'static str {
    if os == "windows" { "zip" } else { "tar.gz" }
}

/// Where to look for releases and how to authenticate (private repositories).
#[derive(Debug, Clone)]
pub struct UpdateSource {
    pub repository: String,
    pub token: Option<String>,
}

impl UpdateSource {
    /// Repository from `JETTHING_UPDATE_REPOSITORY`, token from
    /// `JETTHING_GITHUB_TOKEN` or `GITHUB_TOKEN`.
    pub fn from_env() -> Self {
        let token = ["JETTHING_GITHUB_TOKEN", "GITHUB_TOKEN"]
            .iter()
            .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()));
        Self {
            repository: std::env::var("JETTHING_UPDATE_REPOSITORY")
                .unwrap_or_else(|_| DEFAULT_REPOSITORY.to_owned()),
            token,
        }
    }

    fn request(&self, url: &str) -> ureq::Request {
        // System trust store and HTTP(S)_PROXY, so corporate proxies work like in
        // the reference IDE.
        let agent = ureq::AgentBuilder::new()
            .try_proxy_from_env(true)
            .timeout_connect(std::time::Duration::from_secs(15))
            .timeout_read(std::time::Duration::from_secs(60))
            .build();
        let mut request = agent
            .get(url)
            .set(
                "User-Agent",
                concat!("jetthing/", env!("CARGO_PKG_VERSION")),
            )
            .set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(token) = &self.token {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        request
    }

    pub fn latest_release(&self) -> Result<Release, UpdateError> {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.repository
        );
        let response = self
            .request(&url)
            .set("Accept", "application/vnd.github+json")
            .call()
            .map_err(|error| match error {
                ureq::Error::Status(404, _) => UpdateError::Network(format!(
                    "no published release found for {} (private repositories need \
                     JETTHING_GITHUB_TOKEN)",
                    self.repository
                )),
                other => UpdateError::Network(other.to_string()),
            })?;
        let body = response
            .into_string()
            .map_err(|error| UpdateError::Network(error.to_string()))?;
        parse_release(&body)
    }

    fn download(&self, asset: &Asset, mut sink: impl Write) -> Result<(), UpdateError> {
        let request = if self.token.is_some() {
            self.request(&asset.url)
                .set("Accept", "application/octet-stream")
        } else {
            self.request(&asset.browser_download_url)
        };
        let response = request
            .call()
            .map_err(|error| UpdateError::Network(error.to_string()))?;
        let mut reader = response.into_reader().take(512 * 1024 * 1024);
        std::io::copy(&mut reader, &mut sink)?;
        Ok(())
    }
}

pub fn parse_release(json: &str) -> Result<Release, UpdateError> {
    serde_json::from_str(json).map_err(|error| UpdateError::Format(error.to_string()))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Accepts `sha256sum` output (`<hex>  <name>`) or a bare hex digest.
pub fn parse_checksum(text: &str) -> Option<String> {
    let digest = text.split_whitespace().next()?.to_ascii_lowercase();
    (digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit())).then_some(digest)
}

/// Extracts the `jetthing` and `jetthing-cli` executables from a release archive.
pub fn extract_binaries(
    archive: &[u8],
    os: &str,
    destination: &Path,
) -> Result<Vec<PathBuf>, UpdateError> {
    fs::create_dir_all(destination)?;
    let exe = if os == "windows" { ".exe" } else { "" };
    let wanted = [format!("jetthing{exe}"), format!("jetthing-cli{exe}")];
    let mut extracted = Vec::new();
    let mut store = |name: &str, reader: &mut dyn Read| -> Result<(), UpdateError> {
        let file_name = Path::new(name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        if wanted.contains(&file_name) {
            let target = destination.join(&file_name);
            let mut file = fs::File::create(&target)?;
            std::io::copy(reader, &mut file)?;
            extracted.push(target);
        }
        Ok(())
    };
    if os == "windows" {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive))
            .map_err(|error| UpdateError::Format(error.to_string()))?;
        for index in 0..zip.len() {
            let mut entry = zip
                .by_index(index)
                .map_err(|error| UpdateError::Format(error.to_string()))?;
            if entry.is_file() {
                let name = entry.name().to_owned();
                store(&name, &mut entry)?;
            }
        }
    } else {
        let decoder = flate2::read::GzDecoder::new(archive);
        let mut tar = tar::Archive::new(decoder);
        for entry in tar.entries()? {
            let mut entry = entry?;
            if entry.header().entry_type().is_file() {
                let name = entry.path()?.to_string_lossy().into_owned();
                store(&name, &mut entry)?;
            }
        }
    }
    if !extracted.iter().any(|path| path.ends_with(&wanted[0])) {
        return Err(UpdateError::Format(format!(
            "{} missing from archive",
            wanted[0]
        )));
    }
    Ok(extracted)
}

/// Replaces `target` with `replacement`, keeping `<target>.old` as a backup so a
/// running executable can be swapped (also on Windows). Restores on failure.
pub fn replace_executable(replacement: &Path, target: &Path) -> Result<(), UpdateError> {
    let backup = target.with_extension("old");
    let _ = fs::remove_file(&backup);
    let had_target = target.exists();
    if had_target {
        fs::rename(target, &backup)?;
    }
    let result = fs::rename(replacement, target).or_else(|_| {
        // Different filesystem: copy instead of rename.
        fs::copy(replacement, target).map(|_| ())
    });
    if let Err(error) = result {
        if had_target {
            let _ = fs::rename(&backup, target);
        }
        return Err(error.into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

/// Result of a completed check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    UpToDate { latest: Version },
    Available { version: Version, release: Release },
}

/// Compares the latest release with `current`.
pub fn evaluate(current: &Version, release: Release) -> Result<CheckOutcome, UpdateError> {
    let version = release.version().ok_or_else(|| {
        UpdateError::Format(format!("tag `{}` is not a version", release.tag_name))
    })?;
    Ok(if version > *current && !release.draft {
        CheckOutcome::Available { version, release }
    } else {
        CheckOutcome::UpToDate { latest: version }
    })
}

/// Downloads, verifies and installs the release next to `current_exe`.
pub fn install(
    source: &UpdateSource,
    release: &Release,
    current_exe: &Path,
    os: &str,
    arch: &str,
) -> Result<(), UpdateError> {
    let asset = release
        .asset_for(os, arch)
        .ok_or_else(|| UpdateError::NoAsset(format!("{os}-{arch}")))?;
    let mut archive = Vec::with_capacity(asset.size as usize);
    source.download(asset, &mut archive)?;
    if let Some(checksum_asset) = release.checksum_for(asset) {
        let mut text = Vec::new();
        source.download(checksum_asset, &mut text)?;
        let expected = parse_checksum(&String::from_utf8_lossy(&text))
            .ok_or_else(|| UpdateError::Format("unreadable checksum file".into()))?;
        if sha256_hex(&archive) != expected {
            return Err(UpdateError::Checksum(asset.name.clone()));
        }
    } else {
        return Err(UpdateError::Format(format!(
            "{} has no .sha256 checksum; refusing to install",
            asset.name
        )));
    }
    let directory = current_exe
        .parent()
        .ok_or_else(|| UpdateError::Format("executable has no directory".into()))?;
    let staging = directory.join(".jetthing-update");
    let _ = fs::remove_dir_all(&staging);
    let binaries = extract_binaries(&archive, os, &staging)?;
    for binary in binaries {
        let name = binary.file_name().unwrap_or_default();
        let target = if Path::new(name).file_stem() == current_exe.file_stem() {
            current_exe.to_owned()
        } else {
            directory.join(name)
        };
        replace_executable(&binary, &target)?;
    }
    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

pub enum UpdateEvent {
    Checked(Result<CheckOutcome, String>),
    Installed(Result<Version, String>),
}

/// Runs checks and installs on background threads; poll from the UI loop.
#[derive(Default)]
pub struct UpdateService {
    pending: Option<Receiver<UpdateEvent>>,
}

impl UpdateService {
    pub fn is_busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn check(&mut self, source: UpdateSource, current: Version) {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = source
                .latest_release()
                .and_then(|release| evaluate(&current, release))
                .map_err(|error| error.to_string());
            let _ = sender.send(UpdateEvent::Checked(result));
        });
        self.pending = Some(receiver);
    }

    pub fn install(&mut self, source: UpdateSource, release: Release, current_exe: PathBuf) {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let os = std::env::consts::OS;
            let arch = std::env::consts::ARCH;
            let result = install(&source, &release, &current_exe, os, arch)
                .map_err(|error| error.to_string())
                .and_then(|_| release.version().ok_or_else(|| "invalid version".into()));
            let _ = sender.send(UpdateEvent::Installed(result));
        });
        self.pending = Some(receiver);
    }

    pub fn poll(&mut self) -> Option<UpdateEvent> {
        let receiver = self.pending.as_ref()?;
        match receiver.try_recv() {
            Ok(event) => {
                self.pending = None;
                Some(event)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                Some(UpdateEvent::Checked(Err("update worker stopped".into())))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE: &str = r#"{
        "tag_name": "v0.0.2",
        "name": "Jetthing 0.0.2",
        "body": "Notes",
        "html_url": "https://github.com/HenrySchwerdt/jetthing/releases/tag/v0.0.2",
        "draft": false,
        "prerelease": false,
        "assets": [
            {"name": "jetthing-0.0.2-linux-x86_64.tar.gz", "url": "https://api/1", "browser_download_url": "https://dl/1", "size": 10},
            {"name": "jetthing-0.0.2-linux-x86_64.tar.gz.sha256", "url": "https://api/2", "browser_download_url": "https://dl/2", "size": 1},
            {"name": "jetthing-0.0.2-windows-x86_64.zip", "url": "https://api/3", "browser_download_url": "https://dl/3", "size": 10}
        ]
    }"#;

    #[test]
    fn parses_and_orders_versions() {
        let v = |s| Version::parse(s).unwrap();
        assert_eq!(
            v("v0.0.1"),
            Version {
                major: 0,
                minor: 0,
                patch: 1,
                pre: None
            }
        );
        assert!(v("0.0.2") > v("0.0.1"));
        assert!(v("0.1.0") > v("0.0.9"));
        assert!(v("1.0.0") > v("1.0.0-beta.1"));
        assert_eq!(v("1.2").to_string(), "1.2.0");
        assert_eq!(Version::parse("x.1.2"), None);
        assert_eq!(Version::parse("1.2.3.4"), None);
    }

    #[test]
    fn finds_platform_assets_and_checksums() {
        let release = parse_release(RELEASE).unwrap();
        let linux = release.asset_for("linux", "x86_64").unwrap();
        assert_eq!(linux.name, "jetthing-0.0.2-linux-x86_64.tar.gz");
        assert_eq!(
            release.checksum_for(linux).unwrap().name,
            "jetthing-0.0.2-linux-x86_64.tar.gz.sha256"
        );
        assert!(release.asset_for("windows", "x86_64").is_some());
        assert!(release.asset_for("macos", "aarch64").is_none());
    }

    #[test]
    fn evaluates_against_current_version() {
        let release = parse_release(RELEASE).unwrap();
        let older = Version::parse("0.0.1").unwrap();
        assert!(matches!(
            evaluate(&older, release.clone()).unwrap(),
            CheckOutcome::Available { .. }
        ));
        let same = Version::parse("0.0.2").unwrap();
        assert!(matches!(
            evaluate(&same, release).unwrap(),
            CheckOutcome::UpToDate { .. }
        ));
    }

    #[test]
    fn checksums_parse_and_match() {
        let digest = sha256_hex(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            parse_checksum(&format!("{digest}  file.tar.gz\n")),
            Some(digest.clone())
        );
        assert_eq!(parse_checksum("nope"), None);
    }

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("jetthing-update-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extracts_binaries_from_tar_gz() {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for (path, data) in [
            ("jetthing-0.0.2-linux-x86_64/jetthing", &b"new app"[..]),
            ("jetthing-0.0.2-linux-x86_64/jetthing-cli", b"new cli"),
            ("jetthing-0.0.2-linux-x86_64/README.md", b"readme"),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, path, data).unwrap();
        }
        let archive = builder.into_inner().unwrap().finish().unwrap();
        let dir = temp("extract");
        let files = extract_binaries(&archive, "linux", &dir).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(fs::read(dir.join("jetthing")).unwrap(), b"new app");
        assert!(!dir.join("README.md").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn extracts_binaries_from_zip() {
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("jetthing-0.0.2-windows-x86_64/jetthing.exe", options)
                .unwrap();
            zip.write_all(b"exe").unwrap();
            zip.finish().unwrap();
        }
        let dir = temp("zip");
        let files = extract_binaries(cursor.get_ref(), "windows", &dir).unwrap();
        assert_eq!(files, vec![dir.join("jetthing.exe")]);
        assert!(extract_binaries(cursor.get_ref(), "linux", &dir).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn replaces_executable_and_keeps_backup() {
        let dir = temp("replace");
        let target = dir.join("jetthing");
        let replacement = dir.join("staged");
        fs::write(&target, "old").unwrap();
        fs::write(&replacement, "new").unwrap();
        replace_executable(&replacement, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "new");
        assert_eq!(fs::read_to_string(dir.join("jetthing.old")).unwrap(), "old");
        // A missing replacement restores the original.
        assert!(replace_executable(&dir.join("missing"), &target).is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "new");
        fs::remove_dir_all(dir).unwrap();
    }
}
