//! The `grim` command-line interface.
//!
//! Skeleton: `grim facts` (probe the machine) and `grim stack` (resolve a grimoire's platform stack
//! for this machine) are wired so the detect → load → resolve path runs end-to-end. The full
//! command surface (apply, sync, up, doctor, the TUI wizard) lands as the engine grows; see
//! `docs/architecture.md`.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use grim_core::{Facts, Grimoire};

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
    /// Resolve and print the active platform stack for this machine against a grimoire.
    Stack {
        /// Path to the grimoire's `grimoire.toml`.
        #[arg(long, default_value = "grimoire.toml")]
        grimoire: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Facts) => {
            let facts = Facts::detect();
            println!("{facts:#?}");
        }
        Some(Command::Stack { grimoire }) => {
            let grimoire = Grimoire::load(&grimoire)?;
            let facts = Facts::detect();
            let stack = grimoire.resolve(&facts)?;
            if stack.platforms().is_empty() {
                println!("no platforms match this machine");
            } else {
                println!("active platform stack (highest precedence first):");
                for p in stack.platforms() {
                    println!("  {:>5}  {:<18} {:?}", p.score(), p.name, p.band);
                }
            }
        }
        None => {
            println!("{} {}", grim_core::NAME, env!("CARGO_PKG_VERSION"));
            println!("run `grim --help` for commands");
        }
    }
    Ok(())
}
