use jetthing_core::{IdeState, built_in_commands};
use serde_json::Value;

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "commands".into());
    let registry = built_in_commands();
    if mode == "commands" {
        println!(
            "{}",
            serde_json::to_string_pretty(&registry.specs()).expect("specs serialize")
        );
        return;
    }
    if mode == "run" {
        let Some(id) = args.next() else {
            eprintln!("usage: jetthing-cli run <command-id> [json-params]");
            std::process::exit(2);
        };
        let params = args
            .next()
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .unwrap_or_else(|e| {
                eprintln!("invalid JSON: {e}");
                std::process::exit(2)
            })
            .unwrap_or(Value::Null);
        let mut state = IdeState::default();
        match registry.execute(&id, &mut state, params) {
            Ok(value) => println!("{value}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }
    eprintln!("usage: jetthing-cli <commands|run>");
    std::process::exit(2);
}
