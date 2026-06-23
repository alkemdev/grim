//! The `grim` command-line interface.
//!
//! Commands wired so far: `facts` (probe the machine), `stack` (resolve a grimoire's platform stack
//! for this machine), and `apply` / `diff` (render and place the grimoire's files). The remaining
//! surface (sync, up, doctor, the TUI wizard) lands as the engine grows; see
//! `docs/architecture.md`.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::{Parser, Subcommand};
use grim_apply::{Change, Plan, RenderContext, execute, plan};
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
        /// Path to the grimoire directory (containing `grimoire.toml`).
        #[arg(long, default_value = ".")]
        grimoire: PathBuf,
    },
    /// Render the grimoire's files and place them under the target root (default `$HOME`).
    Apply {
        /// Path to the grimoire directory.
        #[arg(long, default_value = ".")]
        grimoire: PathBuf,
        /// Where to place files (default: `$HOME`).
        #[arg(long)]
        target: Option<PathBuf>,
        /// Show what would change without writing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Show the diff between the grimoire's rendered files and what's on disk (a dry-run apply).
    Diff {
        /// Path to the grimoire directory.
        #[arg(long, default_value = ".")]
        grimoire: PathBuf,
        /// Target root to diff against (default: `$HOME`).
        #[arg(long)]
        target: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Facts) => {
            println!("{:#?}", Facts::detect());
        }
        Some(Command::Stack { grimoire }) => cmd_stack(&grimoire)?,
        Some(Command::Apply {
            grimoire,
            target,
            dry_run,
        }) => cmd_apply(&grimoire, target, dry_run)?,
        Some(Command::Diff { grimoire, target }) => cmd_diff(&grimoire, target)?,
        None => {
            println!("{} {}", grim_core::NAME, env!("CARGO_PKG_VERSION"));
            println!("run `grim --help` for commands");
        }
    }
    Ok(())
}

fn cmd_stack(grimoire_dir: &Path) -> anyhow::Result<()> {
    let grimoire = Grimoire::load_dir(grimoire_dir)?;
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
    Ok(())
}

fn cmd_apply(grimoire_dir: &Path, target: Option<PathBuf>, dry_run: bool) -> anyhow::Result<()> {
    let (ctx, _) = build_context(grimoire_dir)?;
    let target = target_root(target)?;
    let p = plan(&grimoire_dir.join("files"), &target, &ctx)?;
    let (created, updated, unchanged) = p.counts();

    if dry_run {
        print_changes(&p, false);
        println!(
            "\nplan: {created} to create, {updated} to update, {unchanged} unchanged \
             (dry run, nothing written)"
        );
    } else {
        print_changes(&p, false);
        execute(&p, false)?;
        println!("\napplied: {created} created, {updated} updated, {unchanged} unchanged");
    }
    Ok(())
}

fn cmd_diff(grimoire_dir: &Path, target: Option<PathBuf>) -> anyhow::Result<()> {
    let (ctx, _) = build_context(grimoire_dir)?;
    let target = target_root(target)?;
    let p = plan(&grimoire_dir.join("files"), &target, &ctx)?;
    print_changes(&p, true);
    let (created, updated, _) = p.counts();
    if created + updated == 0 {
        println!("no changes");
    }
    Ok(())
}

/// Build the render context (facts + resolved platform names) for a grimoire.
fn build_context(grimoire_dir: &Path) -> anyhow::Result<(RenderContext, Vec<String>)> {
    let grimoire = Grimoire::load_dir(grimoire_dir)?;
    let facts = Facts::detect();
    let names: Vec<String> = grimoire
        .resolve(&facts)?
        .names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    Ok((RenderContext::new(facts, names.clone()), names))
}

fn target_root(target: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    match target {
        Some(t) => Ok(t),
        None => std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("no --target given and $HOME is not set"),
    }
}

fn print_changes(plan: &Plan, show_diff: bool) {
    for action in plan.changed() {
        let tag = match action.change {
            Change::Create => "create",
            Change::Update => "update",
            Change::Unchanged => "ok",
        };
        let kind = if action.is_dir { "dir " } else { "" };
        println!("  {tag:<7} {kind}{}", action.target.display());
        if show_diff && let Some(diff) = &action.diff {
            for line in diff.lines() {
                println!("    {line}");
            }
        }
    }
}
