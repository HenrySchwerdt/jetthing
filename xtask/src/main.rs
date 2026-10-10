mod parity;

use anyhow::{Result, anyhow, bail};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: cargo run -p xtask -- parity [--check]";

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
    if args.first().map(String::as_str) != Some("parity") {
        bail!(USAGE);
    }
    let mut check = false;
    for arg in &args[1..] {
        match arg.as_str() {
            "--check" => check = true,
            other => bail!("unknown argument `{other}`; {USAGE}"),
        }
    }
    match parity::run(&workspace_root(), check).map_err(|e| anyhow!("{e}"))? {
        parity::Outcome::UpToDate => println!("parity dashboard and README are up to date"),
        parity::Outcome::Written(paths) => {
            for path in paths {
                println!("wrote {}", path.display());
            }
        }
    }
    Ok(())
}
