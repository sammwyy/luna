mod commands;
mod lua;
mod platform;
mod renderer;
mod setup;
pub mod shell;

use crate::{
    platform::{CurrentPlatform, Platform},
    shell::Luna,
};
use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    CurrentPlatform::setup_terminal()?;

    let args: Vec<String> = env::args().collect();
    let mut execute_command: Option<String> = None;

    if args.len() >= 2 && args[1] == "init" {
        let force = args.len() >= 3 && args[2] == "--force";
        setup::setup_user_dir(force);
        println!(
            "Luna initialized successfully in {}{}",
            setup::luna_dir().display(),
            if force { " (forced)" } else { "" }
        );
        return Ok(());
    }

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--command" => {
                if i + 1 < args.len() {
                    execute_command = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("luna: -c requires an argument");
                    std::process::exit(1);
                }
            }
            "--version" | "-v" => {
                println!("luna {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!("luna - a modern shell written in Rust");
                println!("\nUsage: luna [options]");
                println!("\nOptions:");
                println!("  -c, --command <expr>    Execute a single command and exit");
                println!("  -v, --version           Print version and exit");
                println!("  -h, --help              Print this help and exit");
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    // 2. Initialize and start the shell
    let mut luna = Luna::init()?;

    if let Some(cmd) = execute_command {
        luna.execute_line(&cmd);
    } else {
        luna.run()?;
    }

    Ok(())
}
