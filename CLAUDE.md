# CLAUDE.md — grim

`grim` is a single Rust CLI for declarative, cross-platform environment management. It reads a
*grimoire* (a config repo — a dotfiles repo is one) and casts it onto a machine. **Read
[`docs/`](docs/) first** — it is the canonical design record, written as decisions are made.

## Locked decisions

- **Engine vs. data split.** `grim` (this repo) is the engine; a *grimoire* (e.g. the `dotfiles`
  repo) is the data. Keep them separate; don't fold grimoire-specific logic into the engine.
- **Own the file layer.** `grim` retires chezmoi and embeds its own MiniJinja apply engine. Don't add
  a chezmoi runtime dependency (shelling to it as a *staging* step during migration is allowed).
- **Secrets are optional + pluggable.** Core must work with no secret provider. Never make a provider
  a hard dependency or let it block `apply`.
- **Sync/backup is deferred** (Phase 7). Leave the seam; don't build it yet.

## Architecture

Pure logic in `grim-core` (facts, platforms, precedence, manifests) — testable with no I/O. The
crates that touch the world (`grim-pkg`, `grim-apply`, `grim-secrets`) stay thin. See
[architecture.md](docs/architecture.md). The platform precedence model
([platforms.md](docs/concepts/platforms.md)) is the conceptual core and has open design choices still
being settled — check there before writing the platform types.

## Conventions

- Edition 2024; toolchain pinned in `rust-toolchain.toml`. `cargo fmt` + `cargo clippy` clean.
- Errors: `thiserror` in libraries, `anyhow` at the CLI boundary. No `unwrap()` outside tests.
- Tests live with the logic in `grim-core`; prefer table-driven tests over machine-dependent fixtures.
  Use `cargo nextest run` (doctests via `cargo test --doc`).
- Commits: conventional-commits, imperative, explain the *why* and anything surprising.
