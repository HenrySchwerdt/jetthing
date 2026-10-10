//! Prints what Jetthing can use from a downloaded IntelliJ plugin:
//! `cargo run -p jetthing-marketplace --example inspect -- plugin.zip`
fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: inspect <plugin.zip|plugin.jar>");
        std::process::exit(2);
    };
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("{path}: {error}");
            std::process::exit(1);
        }
    };
    match jetthing_marketplace::inspect(&bytes) {
        Ok(inspection) => {
            println!(
                "{} {} ({} classes)",
                inspection.plugin.id, inspection.plugin.version, inspection.class_count
            );
            println!("{:?}", inspection.verdict());
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
