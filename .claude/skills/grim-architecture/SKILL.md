---
name: grim-architecture
description: The mental model for working inside the grim engine — crate boundaries, the pure-core invariant, the resolution pipeline, and conventions. Use when editing grim's Rust internals (anything under crates/), adding a command, a fact, a platform axis, a provider, or when reasoning about how a value resolves.
---

# Working inside grim

grim is a single Rust CLI for declarative, cross-platform environment management. It reads a
*grimoire* (config data — a dotfiles repo) and casts it onto a machine. Engine and data are separate.

## The invariant that governs everything

**`grim-core` is pure.** It is a function of typed inputs (`Facts` + the grimoire). The only
machine-dependent code is detection, isolated behind functions whose parsing helpers are pure and
table-tested. If your change to `grim-core` makes it run a command, read an undeclared file, or hit
the network — it's in the wrong crate. Move the side effect to a sibling (`grim-apply`, `grim-pkg`,
`grim-secrets`) and keep the decision in the core.

## Crate map

- `grim-core` — `facts` (the machine, probed once), `platform` (named layers + the `Match` predicate
  + precedence bands), `resolve` (the active stack + folding layered values), `grimoire` (loading the
  typed model), `error`. Pure. Heavily tested.
- `grim-apply` — file templating (MiniJinja) + placement. [grows in Phase 2]
- `grim-pkg` — package-manager providers behind one trait. [grows in Phase 3]
- `grim-secrets` — optional secret providers. [grows in Phase 4]
- `grim` — the clap CLI; wires the pipeline.

## The resolution pipeline

```
detect Facts → load Grimoire → resolve active platform stack → plan (diff vs live) → execute
```

Steps 1–4 are pure and live in `grim-core`. Only execute touches the world. `--dry-run` stops after
plan.

## Platforms & precedence (the part people get wrong)

A `Platform` = a `Match` predicate over facts + a precedence. Precedence is a **band inferred from
which facts the predicate constrains** (`os < arch < distro < hardware < host < container`), plus an
optional `weight` and `refines` relation. The active stack = matching platforms ordered by score,
highest first. **Equal score with no `refines` relation is a hard error**, never a silent pick.
Resolving a value is a fold down the stack: `resolve_override` (highest wins) or `resolve_merged`
(union low→high). See `docs/concepts/platforms.md` and ADR 0003.

When adding a match axis: add the field to `Match` (with `deny_unknown_fields` intact), implement its
check in `Match::matches`, and update `Match::inferred_band`. Add tests for matching *and* inference.

## Conventions

- Edition 2024; `cargo fmt` + `cargo clippy --all-targets -- -D warnings` clean.
- `thiserror` in libraries, `anyhow` at the CLI; no `unwrap()` outside tests/`main`.
- Every config struct: `#[serde(deny_unknown_fields)]` — a grimoire typo is an error.
- Tests live with the logic; prefer table-driven and would-fail-on-regression over coverage theater.

## Verify your work

```bash
just check    # fmt + clippy + tests + doctests — the CI bar
```

In Claude Code here, `rtk` can hide clippy failures; verify with
`rtk proxy cargo clippy --all-targets -- -D warnings` or trust CI.
