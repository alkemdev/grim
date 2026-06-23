# AGENTS.md — working on grim

This is the canonical guide for coding agents (Claude Code, Codex, Cursor, …) and humans working in
this repo. `CLAUDE.md` and editor rule files point here.

`grim` is a single Rust CLI for declarative, cross-platform environment management. It reads a
*grimoire* (a configuration repo — a dotfiles repo is one) and casts it onto a machine: installing
packages across managers, rendering config files from templates, and wiring up secrets. The engine
(`grim`) and the data (a grimoire) are separate by design.

**Read [`docs/`](docs/) before changing behavior** — it is the canonical design record, including the
[architecture](docs/architecture.md), the [concepts](docs/README.md), and the
[decision records](docs/decisions/) that explain *why* things are the way they are.

## Repo map

```
crates/
  grim-core/   pure logic: facts, platforms, precedence, manifests, resolution. NO I/O.
  grim-apply/  file templating + placement (the chezmoi replacement).      [coming]
  grim-pkg/    package-manager providers behind one trait.                  [coming]
  grim-secrets/ optional secret providers (env / 1Password / age).         [coming]
  grim/        the CLI binary; wires everything together.
docs/          architecture, concepts, guides, decision records (source for the website)
web/           the documentation website (grim.alkem.dev)                   [coming]
examples/      example grimoires
```

## The one golden rule

**`grim-core` is pure.** Facts detection is the only machine-dependent code in it, and it is isolated
behind small functions whose parsing helpers are pure and tested. Everything else in `grim-core` is a
function of typed inputs. Anything that runs a command, reads beyond a single declared file, or hits
the network belongs in a sibling crate. Keeping the core pure is what makes the engine testable.

## Build, test, lint

```bash
cargo build
cargo nextest run          # unit + integration tests (or: cargo test)
cargo test --doc           # doctests
cargo clippy --all-targets -- -D warnings
cargo fmt                  # and `cargo fmt --check` in CI
```

A `justfile` wraps these: `just check` runs the whole CI bar locally.

> **Caveat for Claude Code in this environment:** the `rtk` wrapper compresses cargo output and has
> been observed to report "No issues found" while clippy was actually failing. Verify clippy with
> `rtk proxy cargo clippy --all-targets -- -D warnings` (raw output) or trust CI.

## Conventions

- **Edition 2024**, toolchain pinned in `rust-toolchain.toml` (1.96). `cargo fmt` + `clippy -D
  warnings` must be clean.
- **Errors:** `thiserror` for library error types (in `grim-core` and siblings), `anyhow` at the CLI
  boundary. No `unwrap()`/`expect()` outside tests and `main`.
- **Tests live with the logic.** Pure logic gets table-driven unit tests in its module; the CLI gets
  integration tests against fixture grimoires. A green test that doesn't exercise the change is worth
  nothing — prefer tests that would fail if the behavior regressed.
- **No silent failures.** `deny_unknown_fields` on every config struct so a misspelled key is a hard
  error, never an ignored constraint. Don't swallow errors to make output clean.
- **Commits:** conventional-commits (`type(scope): summary`), imperative, summary under ~70 chars.
  Explain the *why* and anything surprising in the body, not just the *what*.

## Where things go

| Task                              | Where                                                          |
| --------------------------------- | -------------------------------------------------------------- |
| A new fact to detect              | `grim-core/src/facts.rs` (pure parser + isolated detection)    |
| A new platform match axis         | `grim-core/src/platform.rs` (add to `Match`, update inference) |
| A new package manager             | `grim-pkg` (implement the provider trait)                      |
| A new file-naming convention      | `grim-apply` (the source→target mapper)                        |
| A new secret backend              | `grim-secrets` (implement the provider trait)                  |
| A new CLI command                 | `grim/src/` + document it in `docs/architecture.md`            |
| A concept or guide                | `docs/` (it builds the website)                                |
| A design decision                 | `docs/decisions/` (add an ADR)                                 |

## Definition of done

Behavior change → tests updated/added; `just check` clean; docs updated if the change is
user-visible; a decision record added if it's architectural. CI is the backstop, not the bar.
