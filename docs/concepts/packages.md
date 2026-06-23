# Packages

One declaration, every package manager. A package has a **canonical name** and an ordered list of
**provider** rows; resolution picks the best provider for the current
[platform stack](platforms.md).

## The model

```toml
[package.ripgrep]
providers = [{ brew = "ripgrep" }, { cargo = "ripgrep" }]

[package.docker]
providers = [
    { brew = "docker", platform = "macos" },
    { apt  = "docker.io", platform = "linux" },
]
```

A provider row is `{ <manager> = "<id>" }` with an optional `platform = "<name>"` gate. The
supported managers are `brew`, `cargo`, `uv`, `npm`, `go`, `pip`, `apt`, `dnf`, `pacman` — the set is
closed, so a typo'd manager name is a hard error, and a row may name exactly one manager.

## Resolution

To resolve a package for a machine, grim walks its provider rows **in declared order** and keeps each
whose `platform` gate is satisfied by the [active stack](platforms.md) (an ungated row always
applies). The first surviving row is the **preferred** provider; the rest are fallbacks. This is
pure — whether a manager is actually installed is a separate, runtime concern (see *Status* below).

This subsumes the three incompatible conventions the old `packages/` directory used (Ruby
`if OS.mac?`, `# linux-only` text markers, and unmarked lists) under one typed, enforceable scheme.

## Explaining a resolution

Resolution is meant to be legible — the old system's biggest debugging cost was opacity.

```text
$ grim resolve docker --grimoire examples/demo
docker:
  + brew = docker                   [preferred]
  - apt = docker.io                 (platform `linux` is not active)
```

```bash
grim packages --grimoire <dir>          # every package and its resolved provider for this machine
grim resolve  <name> --grimoire <dir>   # explain one package's resolution
```

## Status & what's next

Implemented: the typed manifest model, the provider shorthand and platform gates, eligibility
resolution, and `grim packages` / `grim resolve`. **Next** (`grim-pkg`): the `Manager` trait with
real backends, manager-availability checks (so resolution falls through to an installed fallback),
and `grim sync` to reconcile installed packages to the manifest — see the
[roadmap](../roadmap.md). A vendored canonical-id name map across managers is deferred until it earns
its place; today you state the per-manager id explicitly, which is unambiguous.
