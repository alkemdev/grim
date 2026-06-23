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
    cargo run --bin grim -- stack --grimoire examples/demo

# Preview the example grimoire's file apply into /tmp/grim-demo.
diff-demo:
    cargo run --bin grim -- diff --grimoire examples/demo --target /tmp/grim-demo

# Apply the example grimoire's files into /tmp/grim-demo.
apply-demo:
    cargo run --bin grim -- apply --grimoire examples/demo --target /tmp/grim-demo

# Build the documentation website (Astro + Starlight) into web/dist.
site-build:
    cd web && npm ci && npm run build

# Serve the documentation website locally with live reload.
site-serve:
    cd web && npm run dev

# Deploys are automatic: Cloudflare Pages builds web/ and publishes on every push to main.
# This manages the project/domain/DNS itself (OpenTofu).
infra-apply:
    cd infra/cloudflare && tofu init && tofu apply

# Enable the local git hooks (pre-push quality gate). This repo uses no GitHub Actions.
install-hooks:
    git config core.hooksPath .githooks
    @echo "git hooks enabled (.githooks/pre-push runs fmt + clippy + tests on push)"
