# grim docs

Design documentation for the `grim` engine. These are written as the design happens — they are the
canonical record of *why* the system is shaped the way it is, and they are the source the docs
website will eventually build from.

## Start here

- [Architecture](architecture.md) — the engine at a glance: crates, data flow, the command set.
- [Roadmap](roadmap.md) — what's being built, in what order.

## Concepts

The five load-bearing abstractions:

1. [Grimoire](concepts/grimoire.md) — the configuration repository `grim` reads, and how grimoires
   layer (home ← work).
2. [Platforms](concepts/platforms.md) — facts, named platform layers, and precedence resolution.
   **This is the conceptual core.**
3. [Packages](concepts/packages.md) — one canonical id, many providers; resolving the best one.
4. [Apply](concepts/apply.md) — templating and file placement (the chezmoi replacement).
5. [Secrets](concepts/secrets.md) — optional, pluggable secret providers.
