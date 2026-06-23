# Changelog

All notable changes to grim are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and grim aims to follow
[Semantic Versioning](https://semver.org/) once it reaches `0.1.0`.

## [Unreleased]

### Added

- `grim-core`: typed `Facts` and machine detection (OS, arch, CPU features, distro, libc, hostname,
  GPUs, container), with pure, tested parsers for `/proc/cpuinfo`, `/etc/os-release`, and container
  signals.
- `grim-core`: the `Platform`/`Match` model and the precedence resolver — inferred precedence bands,
  `refines` tie-breaking with cycle and unknown-target validation, and `resolve_override` /
  `resolve_merged` value folds.
- `grim-core`: grimoire loading (`grimoire.toml` → typed model) and stack resolution.
- `grim-apply`: the file-apply engine — the source→target naming convention
  (`dot_`/`executable_`/`private_`/`.tmpl`), a MiniJinja render context (`facts`, `platforms`,
  `isolation_id`), and render → diff → atomic-write planning with Unix mode support.
- `grim-core`: the package model — one canonical name, ordered provider rows (`{ brew = "rg" }`
  shorthand with optional `platform` gate, closed manager set), and pure resolution against the
  active stack (`resolve_package` / `preferred_provider`).
- `grim`: `grim facts`, `grim stack`, `grim apply` / `grim diff` (with `--dry-run`), and
  `grim packages` / `grim resolve` (explainable package resolution).
- Project foundation: design docs and concept records under `docs/`, decision records under
  `docs/decisions/`, a local pre-push quality gate (fmt + clippy + tests + doctests; no GitHub
  Actions), and AI-agent tooling (`AGENTS.md`, `CLAUDE.md`, repo skills).
- Website (`web/`): the [grim.alkem.dev](https://grim.alkem.dev) documentation site (Astro +
  Starlight), generated from `docs/` via a sync script so there's one source of truth. Hosted on
  git-integrated Cloudflare Pages (provisioned with OpenTofu in `infra/cloudflare/`) — Cloudflare
  builds and deploys on every push; no GitHub Actions.

[Unreleased]: https://github.com/alkemdev/grim/commits/main
