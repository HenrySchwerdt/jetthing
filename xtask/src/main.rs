mod dist;

use anyhow::{Context, Result, bail};
use std::{fs, path::Path};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("parity") => parity(args.any(|a| a == "--check")),
        Some("dist") => dist::dist(dist::Options::parse(args)?),
        _ => bail!(
            "usage: cargo xtask parity [--check] | cargo xtask dist [--installer] [--target <triple>]"
        ),
    }
}

const PARITY_DOCS: [&str; 2] = ["docs/parity/platform.md", "docs/parity/android-studio.md"];
const STATUSES: [&str; 5] = ["Missing", "Planned", "Partial", "Implemented", "Verified"];

/// Validates the parity tables and, unless `check` is set, rewrites the dashboard.
///
/// `--check` validates rows (known status, unique IDs) instead of comparing the
/// generated dashboard, so parallel PRs that each add rows do not conflict on it.
fn parity(check: bool) -> Result<()> {
    let mut totals = [0usize; STATUSES.len()];
    let mut seen = std::collections::HashMap::new();
    let mut problems = Vec::new();
    for doc in PARITY_DOCS {
        let text = fs::read_to_string(doc).with_context(|| format!("read {doc}"))?;
        for (number, line) in text.lines().enumerate() {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let Some(id) = cells.get(1).filter(|id| is_parity_id(id)) else {
                continue;
            };
            let location = format!("{doc}:{}", number + 1);
            if let Some(first) = seen.insert(id.to_string(), location.clone()) {
                problems.push(format!("{location}: duplicate ID {id} (first at {first})"));
            }
            match STATUSES.iter().position(|status| cells.contains(status)) {
                Some(i) => totals[i] += 1,
                None => problems.push(format!(
                    "{location}: {id} has no status; use one of {}",
                    STATUSES.join(", ")
                )),
            }
        }
    }
    if !problems.is_empty() {
        bail!("parity tables are invalid:\n{}", problems.join("\n"));
    }
    if check {
        println!("parity tables valid: {} rows", seen.len());
        return Ok(());
    }
    let total: usize = totals.iter().sum();
    let report = format!(
        "# Parity dashboard\n\nGenerated from `docs/parity/*.md`.\n\n| Status | Count |\n|---|---:|\n| Missing | {} |\n| Planned | {} |\n| Partial | {} |\n| Implemented | {} |\n| Verified | {} |\n| **Total** | **{}** |\n",
        totals[0], totals[1], totals[2], totals[3], totals[4], total
    );
    let path = Path::new("docs/parity/dashboard.md");
    fs::write(path, &report)?;
    println!("wrote {}", path.display());
    Ok(())
}

/// Matches row IDs such as `PLAT-001` or `AND-010`.
fn is_parity_id(cell: &str) -> bool {
    let Some((prefix, number)) = cell.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
}
