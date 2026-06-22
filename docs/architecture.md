# Architecture

`grim` is one binary built from a workspace of focused crates. The guiding split is **pure logic vs.
the outside world**: everything that can be decided by reasoning about typed data lives in
`grim-core` and is tested without touching the filesystem, network, or any package manager. The
crates that *do* things (run `brew`, write files, call `op`) are thin and sit on top.

## Crates

| Crate         | Responsibility                                                                 | Status   |
| ------------- | ------------------------------------------------------------------------------ | -------- |
| `grim-core`   | Facts, platforms, precedence resolution, the typed grimoire/manifest model     | seed     |
| `grim-pkg`    | Package-manager providers (brew, cargo, uv, npm, go, …) behind one trait       | planned  |
| `grim-apply`  | Templating (MiniJinja) + file placement: render → diff → atomic write          | planned  |
| `grim-secrets`| Secret-provider trait + impls (env, 1Password, age/sops)                       | planned  |
| `grim`        | The CLI: clap command surface + the TUI wizard; wires everything together      | seed     |

New crates earn their place only when there's real code to put in them. Today only `grim-core` and
`grim` exist.

## The pipeline

A run of `grim` is a small, deterministic pipeline:

```
  detect            load                resolve                 plan                 execute
 ┌────────┐      ┌──────────┐      ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
 │ Facts  │ ───▶ │ Grimoire │ ───▶ │ active stack │ ───▶ │ actions +    │ ───▶ │ apply files, │
 │ (probe │      │ (parse   │      │ + resolved   │      │ diff vs.     │      │ install pkgs,│
 │  once) │      │  TOML)   │      │ values       │      │ live state   │      │ inject secrets│
 └────────┘      └──────────┘      └──────────────┘      └──────────────┘      └──────────────┘
                                                              │
                                                         `--dry-run` stops here and prints the plan
```

1. **detect** — probe the machine into `Facts` (one typed struct, computed once).
2. **load** — parse the grimoire (a tree of TOML manifests), composing any grimoires it `extends`.
3. **resolve** — compute the *active platform stack* (which platform layers match these facts, in
   precedence order) and resolve every layered value against it.
4. **plan** — diff the desired state against what's actually on the machine, producing an ordered
   list of typed actions. This is the unit of `--dry-run`.
5. **execute** — run the plan transactionally where possible, with structured logging.

Steps 1–4 are pure (`grim-core`) and fully testable. Only step 5 touches the world.

## Command set (planned)

The CLI replaces `bootstrap.sh`'s three modes and the scattered `install/*.sh` scripts with typed
subcommands. The shape, subject to change as the core lands:

- `grim apply` — render and place config files (the chezmoi-apply replacement).
- `grim sync` — reconcile installed packages to the manifest.
- `grim up` — the full setup: apply + sync + everything, idempotent. (`install`/`update`/`upgrade`
  become flags or subcommands of this.)
- `grim facts` — print detected facts and the resolved platform stack. The debugging workhorse;
  the typed successor to "why did PLAT pick that?".
- `grim resolve <thing> --explain` — show how a package / value resolved, and why.
- `grim diff` / `grim status` — show drift between desired and live state.
- `grim doctor` — health checks (missing tools, broken links, stale isolation trees).
- `grim` (no args) — the interactive TUI wizard for first-time setup and common tasks.

## Conventions

- **Edition 2024**, toolchain pinned in `rust-toolchain.toml`.
- Errors: `thiserror` for library error types in `grim-core`; `anyhow` at the CLI boundary.
- Paths: UTF-8 paths (`camino`) at the boundaries that touch the grimoire.
- Logging: `tracing`, with a human formatter by default and structured output under a flag.
- Tests: `grim-core` carries the logic and the tests. Prefer table-driven tests over fixtures that
  hit the real machine. Integration tests drive the CLI against fixture grimoires.
