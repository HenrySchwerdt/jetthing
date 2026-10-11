//! Measures opening and editing a large file: `cargo run --release -p
//! jetthing-language --example highlight_bench`.

use jetthing_text::{EditJournal, TextDocument};
use std::path::Path;
use std::time::Instant;

fn rust_source(items: usize) -> String {
    let mut s = String::new();
    for i in 0..items {
        s.push_str(&format!(
            "/// Item {i}.\npub fn item_{i}(x: u32) -> String {{\n    let v = vec![x, {i}, 3];\n    format!(\"{{}} {{:?}}\", x + {i}, v)\n}}\n\n"
        ));
    }
    s
}

fn main() {
    let path = Path::new("big.rs");
    let source = rust_source(8_000);
    println!("{} bytes, {} lines", source.len(), source.lines().count());

    let t = Instant::now();
    let full = jetthing_language::analyze(path, &source);
    println!("open, full highlight:      {:>8.1} ms", ms(t));
    drop(full);

    let t = Instant::now();
    let mut lazy = jetthing_language::analyze_lazy(path, &source);
    println!("open, viewport-first:      {:>8.1} ms", ms(t));
    let t = Instant::now();
    lazy.ensure_highlighted(source.len() / 2..source.len() / 2 + 4_000);
    println!("scroll to middle (4 KB):   {:>8.1} ms", ms(t));

    let mut document = TextDocument::new(&source);
    document.take_edits();
    let at = source.chars().count() / 2;
    let t = Instant::now();
    for i in 0..20 {
        document.edit(at + i, at + i, "x").unwrap();
        let text = document.text();
        if let EditJournal::Edits(edits) = document.take_edits() {
            lazy.apply_edits(path, &text, &edits);
        }
    }
    println!("20 keystrokes (lazy):      {:>8.1} ms", ms(t));
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
