<div align="center">

# grim

**A grimoire for your machines.**
Declarative, cross-platform environment management in a single Rust CLI.

[Documentation](https://grim.alkem.dev) · [Concepts](docs/README.md) · [Roadmap](docs/roadmap.md) · [Decisions](docs/decisions/)

</div>

---

`grim` reads a *grimoire* — a repository of declarative configuration — and *casts* it onto the
current machine: installing packages across every package manager, rendering config files from
templates, wiring up secrets, and isolating per-platform install trees so one shared `$HOME` can
serve a fleet of heterogeneous machines.

The engine and the configuration are deliberately separate:

| Thing        | What it is                            | Example                                   |
| ------------ | ------------------------------------- | ----------------------------------------- |
| **`grim`**   | the engine (this repo) — a single CLI | `grim apply`, `grim sync`, `grim doctor`  |
| a *grimoire* | your data — a repo `grim` reads       | a dotfiles repo                           |

A dotfiles repo is just a grimoire. Your work machine's private repo is *another* grimoire that
**extends** the public one and overrides it. Same engine, layered data.

## Why

A typical dotfiles setup is a clean declarative file layer sitting on a pile of untyped, untested
shell that does the real work — package orchestration, platform detection, config merging, secret
loading. It works until it drifts. `grim` moves that logic into one typed, tested Rust binary built
on a few clean abstractions:

- **Facts** — the machine, probed once into a typed struct.
- **Platforms** — named, precedence-ordered layers matched against facts. A dev container is just a
  high-precedence platform. ([concept](docs/concepts/platforms.md))
- **Packages** — one canonical id resolved to the best provider (brew / cargo / uv / npm / go / …) for
  the current platform stack. ([concept](docs/concepts/packages.md))
- **Apply** — a scoped templating + file-placement engine. ([concept](docs/concepts/apply.md))
- **Secrets** — *optional*, pluggable providers (1Password, age/sops, plain env). Core works with
  none. ([concept](docs/concepts/secrets.md))

## Install

> grim is green-field and pre-`0.1`; install from source.

```bash
git clone https://github.com/alkemdev/grim
cd grim
cargo install --path crates/grim
```

## Quickstart

```bash
grim facts                                   # probe this machine into typed Facts
grim stack --grimoire examples/grimoire.toml # resolve the active platform stack
```

```text
$ grim stack --grimoire examples/grimoire.toml
active platform stack (highest precedence first):
  1000  devcontainer       Container
   900  workstation        Host
   400  cuda               Hardware
   100  linux              Os
```

## Status

Early. The detect → load → resolve core is built and tested; the apply, packages, and secrets layers
are landing per the [roadmap](docs/roadmap.md). The design is documented in the open under
[`docs/`](docs/) and recorded in [decision records](docs/decisions/).

## Layout

```
crates/
  grim-core/   pure logic: facts, platforms, precedence, manifests, resolution
  grim/        the CLI binary
docs/          architecture, concepts, guides, decisions (source for grim.alkem.dev)
examples/      example grimoires
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) (and [AGENTS.md](AGENTS.md) if you're a coding agent). The
short version: `grim-core` stays pure, `just check` stays green, and decisions get an ADR.

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
