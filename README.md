# grim

**A grimoire for your machines.** Declarative, cross-platform environment management in Rust.

`grim` reads a *grimoire* — a repository of declarative configuration — and *casts* it onto
the current machine: installing packages across every package manager, rendering config files
from templates, wiring up secrets, and isolating per-platform install trees so a single shared
`$HOME` can serve a fleet of heterogeneous machines.

The engine and the configuration are deliberately separate:

| Thing       | What it is                              | Example                                  |
| ----------- | --------------------------------------- | ---------------------------------------- |
| **`grim`**  | the engine (this repo) — a single CLI   | `grim apply`, `grim sync`, `grim doctor` |
| a *grimoire* | your data — a repo `grim` reads          | [`dotfiles`](https://github.com/cadebrown/dotfiles) |

A dotfiles repo is just a grimoire. Your work machine's private repo is *another* grimoire that
**extends** the public one (as a submodule) and overrides it. Same engine, layered data.

## Why

The system this replaces had a clean declarative *file* layer (chezmoi) sitting on top of ~6,700
lines of untyped, untested shell that did all the real work: package orchestration, platform
detection (duplicated four times), config merging (reimplemented four times), and secret loading
(plaintext env files globbed into every shell). It worked, but it drifted by hand and there was no
dry-run, no transactionality, and no tests on the riskiest scripts.

`grim` moves that logic into one typed, tested Rust binary built on a small set of clean
abstractions:

- **Facts** — the machine, probed once into a typed struct (OS, arch, CPU features, distro, libc,
  GPU, hostname, container, …).
- **Platforms** — named, precedence-ordered layers matched against facts. A dev container is just a
  high-precedence platform. See [Platforms](docs/concepts/platforms.md).
- **Packages** — a canonical id resolved to the best provider (brew / cargo / uv / npm / go / …) for
  the current platform stack. One manifest, every package manager.
- **Apply** — a scoped templating + file-placement engine (MiniJinja) that retires chezmoi.
- **Secrets** — *optional*, pluggable providers (1Password, age/sops, plain env). Core works with
  none configured.

## Status

Green-field and early. The architecture and concepts are being designed in the open under
[`docs/`](docs/); the core engine is taking shape under [`crates/`](crates/). Nothing here is
stable yet.

## Layout

```
grim/
├── crates/
│   ├── grim-core/   pure logic: facts, platforms, precedence, manifest model
│   └── grim/        the CLI binary
├── docs/            architecture + concept docs (source for the docs site)
└── Cargo.toml       workspace
```

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).
