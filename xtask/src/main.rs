use anyhow::{Context, Result, bail};
use std::{fs, path::Path};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("parity") {
        bail!("usage: cargo xtask parity [--check]");
    }
    let check = args.any(|a| a == "--check");
    let docs = ["docs/parity/platform.md", "docs/parity/android-studio.md"];
    let mut totals = [0usize; 5];
    for doc in docs {
        let text = fs::read_to_string(doc).with_context(|| format!("read {doc}"))?;
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
