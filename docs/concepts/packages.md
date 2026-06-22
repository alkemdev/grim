# Packages

One declaration, every package manager. A package has a **canonical id** and an ordered list of
**provider rows**; resolution picks the best provider for the current [platform stack](platforms.md).

## The model

```toml
[package.ripgrep]
providers = [
    { brew  = "ripgrep" },
    { cargo = "ripgrep" },              # binstall-able; fallback when no brew
]

[package.docker]
providers = [
    { brew = "docker", platform = "macos" },
    { apt  = "docker.io", platform = "linux" },
]

[package.cuda-toolkit]
providers = [
    { apt = "cuda-toolkit-12", platform = "cuda" },   # only where the `cuda` platform matches
]
```

To resolve `ripgrep` on a given machine, walk its provider rows and pick the first whose `platform`
predicate is satisfied by the active stack (and whose manager is available). The dev container, being
the highest-precedence platform, naturally wins when present.

This subsumes the three incompatible conventions the old `packages/` directory used (Ruby
`if OS.mac?` in the Brewfile, `# linux-only` text markers, and no markers at all in `cargo.txt`)
under one typed, enforceable scheme. "One tool, one owning layer" stops being a prose comment and
becomes a validation the engine can run.

## Canonical ids across managers

The hard part is that managers name the same software differently (`ripgrep` vs `rg` vs
`ripgrep-bin`). The plan: a **vendored name map** (seeded from a Repology snapshot, which is
rate-limited and not authoritative, so it's a build-time snapshot) plus hand-curated overrides. The
grimoire can always state the exact per-manager id explicitly, as above; the map is only a
convenience for the common case.

## What carries over

The migration target is the existing inventory, verbatim: ~110 Brewfile entries (formulae + macOS
casks + taps), 31 cargo crates (binstall-able only), 7 uv-tool CLIs, 7 npm globals (with version
pins), 5 go modules, plus the MCP-server / Claude-plugin / agent-skill / editor-extension lists.
Importing these into the new manifest format is the first real package-layer task; see the
[roadmap](../roadmap.md).

## Resolution must be explainable

`grim resolve ripgrep --explain` should print exactly which provider was chosen and why (which
platform matched, which managers were available, what was shadowed). The old system's biggest
debugging cost was opacity; explainability is a first-class feature here.
