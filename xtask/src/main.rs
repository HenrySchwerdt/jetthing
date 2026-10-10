mod changelog;
mod dist;
mod parity;
mod scenes;

use anyhow::{Result, anyhow, bail};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: cargo xtask parity [--check] | dist [--installer] [--target <triple>] | changelog [--from REF] [--to REF] [--version X.Y.Z] [--output FILE] | scenes [--only 01,05] [--out DIR] [--release]";

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
        Some("dist") => dist::dist(dist::Options::parse(args.into_iter().skip(1))?),
        Some("changelog") => changelog::run(changelog::Options::parse(args.into_iter().skip(1))?),
        Some("scenes") => scenes::run(args.into_iter().skip(1)),
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
