# 0002 — Own the file-templating layer; retire chezmoi

**Status:** accepted

## Context

The previous system used chezmoi as its file layer and drove it from shell via `run_onchange_`
hooks. chezmoi's templating works well, but the two-tool split meant a second config language, a
bridge between "chezmoi rendered a file" and "now reconcile packages", and a second tool to install.
The vision for grim is *one tool for everything*.

## Decision

`grim` embeds its own file-apply engine: [MiniJinja](https://docs.rs/minijinja) for templating
(Jinja2-compatible, single pure-Rust dependency) plus a small source→target naming convention and a
render → diff → atomic-write apply step. chezmoi is retired.

The feature set we actually use is a small fraction of chezmoi's universe-of-use-cases, so a clean,
scoped reimplementation is tractable and removes a whole class of integration seams.

## Consequences

- One tool, one config model, one render context shared by files, packages, and secrets.
- We own the apply semantics (dry-run, diff, transactional writes) instead of shelling to them.
- We carry the maintenance of a templating/apply engine — bounded, because it serves one well-defined
  job, not everyone's.
- Migration may *stage* through chezmoi (shell out first, swap to native at parity) but takes no
  runtime dependency on it.
