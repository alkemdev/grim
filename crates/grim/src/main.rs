//! The `grim` command-line interface.
//!
//! Skeleton for now: the CLI surface (clap commands, the TUI wizard) lands once the
//! core model is settled. See `docs/architecture.md` for the planned command set.

fn main() {
    println!("{} {}", grim_core::NAME, env!("CARGO_PKG_VERSION"));
}
