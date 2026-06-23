# CLAUDE.md — grim

**Read [`AGENTS.md`](AGENTS.md) first** — it is the canonical guide for working in this repo
(architecture, the "keep `grim-core` pure" rule, build/test/lint, conventions, where things go). This
file holds only Claude Code–specific notes.

## Claude-specific notes

- **Verify clippy raw.** The `rtk` cargo wrapper in this environment compresses output and has
  reported "No issues found" while clippy was failing. Run
  `rtk proxy cargo clippy --all-targets -- -D warnings` to see real output, or rely on CI.
- **Skills.** Repo-local skills for grim development live under `.claude/skills/` — e.g.
  `grim-architecture` (the mental model) and `add-a-provider` (the package-provider checklist). Use
  them when the task matches.
- **Design docs are canonical.** `docs/` is the source of truth and the website source. Update it
  alongside behavior changes; add a `docs/decisions/` ADR for anything architectural.
- **The roadmap** (`docs/roadmap.md`) is the living task list — keep it current as phases land.
