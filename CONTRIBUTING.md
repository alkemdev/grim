# Contributing to grim

Thanks for your interest in grim. This guide is for humans; agents should read
[`AGENTS.md`](AGENTS.md) (which this complements).

## Development setup

You need a Rust toolchain — the repo pins it in `rust-toolchain.toml` (1.96), so `rustup` will fetch
the right one automatically. Optional but recommended:

```bash
cargo install cargo-nextest --locked   # faster test runner (or use `cargo test`)
cargo install just --locked            # task runner (or run the cargo commands directly)
```

## The loop

```bash
just check        # the full local CI bar: fmt, clippy, tests, doctests
# or individually:
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo nextest run
cargo test --doc
```

`just check` must pass before you push. CI runs the same bar on every push and PR.

## Architecture in one breath

`grim-core` is pure (facts, platforms, precedence, manifests, resolution) and exhaustively tested.
The crates around it (`grim-apply`, `grim-pkg`, `grim-secrets`, `grim`) do the I/O — running package
managers, writing files, calling secret backends. Keep logic in the core and side effects at the
edges. The full picture is in [`docs/architecture.md`](docs/architecture.md); the *why* behind the
big choices is in [`docs/decisions/`](docs/decisions/).

## Conventions

- **Commits:** conventional-commits (`feat(core): …`, `fix(apply): …`, `docs: …`), imperative mood,
  summary under ~70 chars. The body explains *why* and anything surprising.
- **Tests:** add them with the logic. Prefer tests that would fail if the behavior regressed over
  tests that merely execute the code.
- **Errors:** `thiserror` in libraries, `anyhow` at the CLI boundary; no `unwrap()` outside tests.
- **Config structs** use `deny_unknown_fields` — a typo in a grimoire is an error, not a silent
  no-op.
- **Docs:** update `docs/` for user-visible changes; add an ADR under `docs/decisions/` for
  architectural ones.

## Pull requests

Keep PRs focused. Describe the change and the reasoning. Make sure `just check` is green and the docs
reflect any user-visible behavior change.
