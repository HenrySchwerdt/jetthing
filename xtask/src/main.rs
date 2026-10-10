use anyhow::{Context, Result, bail};
use std::{fs, path::Path, process::Command};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("parity") => parity(args.any(|a| a == "--check")),
        Some("dist") => dist(),
        _ => bail!("usage: cargo xtask parity [--check] | cargo xtask dist"),
    }
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

fn parity(check: bool) -> Result<()> {
    // Every inventory in docs/parity/ counts, so new plugins only add a file.
    let mut docs: Vec<_> = fs::read_dir("docs/parity")
        .context("read docs/parity")?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "md")
                && path.file_name().is_some_and(|name| name != "dashboard.md")
        })
        .collect();
    docs.sort();
    let mut totals = [0usize; 5];
    for doc in &docs {
        let text = fs::read_to_string(doc).with_context(|| format!("read {}", doc.display()))?;
        for (i, status) in ["Missing", "Planned", "Partial", "Implemented", "Verified"]
            .iter()
            .enumerate()
        {
            totals[i] += text
                .lines()
                .filter(|line| line.starts_with('|') && line.contains(&format!("| {status} |")))
                .count();
        }
    }
    let total: usize = totals.iter().sum();
    let report = format!(
        "# Parity dashboard\n\nGenerated from `docs/parity/*.md`.\n\n| Status | Count |\n|---|---:|\n| Missing | {} |\n| Planned | {} |\n| Partial | {} |\n| Implemented | {} |\n| Verified | {} |\n| **Total** | **{}** |\n",
        totals[0], totals[1], totals[2], totals[3], totals[4], total
    );
    let path = Path::new("docs/parity/dashboard.md");
    if check {
        let current = fs::read_to_string(path).unwrap_or_default();
        if current != report {
            bail!(
                "{} is stale; run `cargo run -p xtask -- parity`",
                path.display()
            );
        }
    } else {
        fs::write(path, &report)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
