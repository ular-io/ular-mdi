mod alfred;
mod cli;
mod config;
mod meta;
mod notify;
mod npm;
mod paths;
mod render;
mod search;
mod update;

use anyhow::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("mdi: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("search") => cli::search(&args[1..]),
        Some("update") => cli::update(&args[1..]),
        Some("bake") => cli::bake(&args[1..]),
        Some("--version") | Some("-V") => {
            println!("ular-mdi {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprintln!("usage: mdi <search|update|bake> [args]");
            std::process::exit(2);
        }
    }
}
