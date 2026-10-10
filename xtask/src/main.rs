mod parity;
mod scenes;

use anyhow::{Context, Result, anyhow, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const USAGE: &str = "usage: cargo xtask parity [--check] | dist | scenes [--only 01,05] [--out DIR] [--release] | pack-plugin <dir> [output]";

fn workspace_root() -> PathBuf {
    // Prefer the current directory when it holds the inventory (normal `cargo run`
    // from the workspace root); fall back to the xtask manifest's parent.
    let cwd = std::env::current_dir().unwrap_or_default();
    if cwd.join("docs/parity").is_dir() {
        cwd
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("parity") => {
            let mut check = false;
            for arg in &args[1..] {
                match arg.as_str() {
                    "--check" => check = true,
                    other => bail!("unknown argument `{other}`; {USAGE}"),
                }
            }
            parity(check)
        }
        Some("dist") => dist(),
        Some("scenes") => scenes::run(args.into_iter().skip(1)),
        Some("pack-plugin") => {
            let dir = args
                .get(1)
                .context("usage: cargo xtask pack-plugin <plugin dir> [output]")?;
            pack_plugin(Path::new(dir), args.get(2).map(Path::new))
        }
        _ => bail!(USAGE),
    }
}

fn parity(check: bool) -> Result<()> {
    match parity::run(&workspace_root(), check).map_err(|e| anyhow!("{e}"))? {
        parity::Outcome::Valid { rows } => println!("parity tables valid: {rows} rows"),
        parity::Outcome::UpToDate => println!("parity dashboard and README are up to date"),
        parity::Outcome::Written(paths) => {
            for path in paths {
                println!("wrote {}", path.display());
            }
        }
    }
    Ok(())
}

/// Builds release binaries and stages a distributable directory under `dist/`.
fn dist() -> Result<()> {
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "build",
            "--release",
            "-p",
            "jetthing-app",
            "-p",
            "jetthing-cli",
        ])
        .status()
        .context("run cargo build")?;
    if !status.success() {
        bail!("release build failed");
    }
    let version = env!("CARGO_PKG_VERSION");
    let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let name = format!("jetthing-{version}-{target}");
    let out = Path::new("dist").join(&name);
    if out.exists() {
        fs::remove_dir_all(&out)?;
    }
    fs::create_dir_all(&out)?;
    let exe = std::env::consts::EXE_SUFFIX;
    for (built, shipped) in [
        ("jetthing-app", "jetthing"),
        ("jetthing-cli", "jetthing-cli"),
    ] {
        let from = Path::new("target/release").join(format!("{built}{exe}"));
        let to = out.join(format!("{shipped}{exe}"));
        fs::copy(&from, &to).with_context(|| format!("copy {}", from.display()))?;
    }
    for file in ["README.md", "docs/spec/data/NOTICE.md"] {
        let target_name = if file.ends_with("NOTICE.md") {
            "NOTICE-intellij-data.md"
        } else {
            "README.md"
        };
        fs::copy(file, out.join(target_name)).with_context(|| format!("copy {file}"))?;
    }
    let fonts = out.join("licenses");
    fs::create_dir_all(&fonts)?;
    for license in ["Inter-OFL.txt", "JetBrainsMono-OFL.txt"] {
        fs::copy(
            Path::new("crates/jetthing-ui/assets/fonts").join(license),
            fonts.join(license),
        )
        .with_context(|| format!("copy {license}"))?;
    }
    // Forward slashes on every platform so CI scripts can parse the path.
    println!("dist/{name}");
    Ok(())
}

/// Packs a plugin directory into `dist/plugins/<id>-<version>.jtplugin`.
fn pack_plugin(dir: &Path, output: Option<&Path>) -> Result<()> {
    let manifest_text = fs::read_to_string(dir.join(jetthing_plugins::MANIFEST_FILE))
        .with_context(|| {
            format!(
                "{} has no {}",
                dir.display(),
                jetthing_plugins::MANIFEST_FILE
            )
        })?;
    let manifest = jetthing_plugins::PluginManifest::parse(&manifest_text)?;
    let default = Path::new("dist/plugins").join(format!(
        "{}-{}.{}",
        manifest.id,
        manifest.version,
        jetthing_plugins::archive::ARCHIVE_EXTENSION
    ));
    let output = output.unwrap_or(&default);
    jetthing_plugins::archive::pack(dir, output)?;
    // Installing validates the archive the same way the IDE will.
    jetthing_plugins::archive::read_manifest(output)?;
    println!("{}", output.display());
    Ok(())
}
