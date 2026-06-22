# Grimoire

A **grimoire** is the repository of declarative configuration that `grim` reads — the data the engine
acts on. A classic "dotfiles" repo is a grimoire. It is the thing you version, share, and carry from
machine to machine.

## Anatomy (proposed)

```
my-grimoire/
├── grimoire.toml          # root manifest: identity, extends, isolation policy, platform defs
├── platforms.toml         # platform layer definitions (see platforms.md)
├── packages/              # the package manifests (one canonical id, many providers)
│   ├── core.toml
│   └── ...
├── files/                 # source templates → rendered into $HOME (the chezmoi `home/` replacement)
│   ├── zsh/
│   ├── git/
│   └── ...
├── secrets.toml           # secret references (op://… / age:…), never secret values
└── hosts/                 # optional per-host overrides, keyed by hostname
    └── x1.toml
```

The exact tree is still being designed; the principle is **typed manifests + a templated file tree**,
with everything resolvable against the [platform stack](platforms.md).

## Layering: home ← work

The headline requirement. The public dotfiles grimoire is the base. A private work grimoire is a
*separate repo* that **extends** it — including the public one as a git submodule and bumping it
periodically — and overrides or adds on top:

```toml
# work-grimoire/grimoire.toml
[grimoire]
name = "work"
extends = ["vendor/dotfiles"]   # the public grimoire, pinned as a submodule
```

Resolution composes layers most-specific-first (work over home), then resolves each value's platform
map against the active stack. This replaces the old "clone the public repo *inside* the private one"
overlay hack with a clean, declarative `extends` and a well-defined override order — and keeps secrets
and work-only config strictly in the private grimoire, never leaking into the public one.

> Design note: `extends` is its own precedence axis, orthogonal to platform precedence. See the
> "Composition" section of [platforms.md](platforms.md#composition-with-grimoires).
