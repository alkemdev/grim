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
- `grim`: `grim facts` (probe the machine) and `grim stack` (resolve the platform stack against a
  grimoire).
- Project foundation: design docs and concept records under `docs/`, decision records under
  `docs/decisions/`, CI (fmt + clippy + tests + doctests), and AI-agent tooling (`AGENTS.md`,
  `CLAUDE.md`, repo skills).

[Unreleased]: https://github.com/alkemdev/grim/commits/main
