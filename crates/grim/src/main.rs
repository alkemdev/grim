//! The `grim` command-line interface.
//!
//! Skeleton: only `grim facts` is wired so far, so the detection path is runnable end-to-end. The
//! full command surface (apply, sync, up, doctor, the TUI wizard) lands as the engine grows; see
//! `docs/architecture.md`.

use clap::{Parser, Subcommand};
use grim_core::Facts;

#[derive(Parser)]
#[command(name = "grim", version, about = "A grimoire for your machines.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print the detected machine facts.
    Facts,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Facts) => {
            let facts = Facts::detect();
            println!("{facts:#?}");
        }
        None => {
            println!("{} {}", grim_core::NAME, env!("CARGO_PKG_VERSION"));
            println!("run `grim --help` for commands");
        }
    }
    Ok(())
}
