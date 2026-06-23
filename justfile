# grim task runner — run `just` to list recipes.
# Everything here is a thin wrapper over cargo; you never need `just` to work on grim.

# List available recipes.
default:
    @just --list

# Build everything.
build:
    cargo build

# Run the full local CI bar: format check, lint, tests, doctests.
check: fmt-check lint test doc-test

# Run unit + integration tests (uses nextest if available, else cargo test).
test:
    cargo nextest run || cargo test

# Run doctests (nextest does not run these).
doc-test:
    cargo test --doc

# Lint with clippy, warnings as errors.
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Format the tree.
fmt:
    cargo fmt

# Check formatting without changing files.
fmt-check:
    cargo fmt --check

# Build and open the API docs.
doc:
    cargo doc --no-deps --open

# Run the CLI (e.g. `just run facts`).
run *ARGS:
    cargo run --bin grim -- {{ARGS}}

# Resolve the example grimoire's platform stack for this machine.
stack:
    cargo run --bin grim -- stack --grimoire examples/grimoire.toml

# Build the documentation website into web/book (requires mdbook).
site-build:
    mdbook build web

# Serve the documentation website locally with live reload.
site-serve:
    mdbook serve web --open
