# Platforms

> Status: **design proposal.** This is the conceptual core of `grim` and the part most worth getting
> right. The open choices are flagged inline; resolve them before the types land in `grim-core`.

## The problem

The system this replaces had `PLAT`: per-CPU-microarchitecture isolation of install trees. It
detected one of six `plat_{os}_{cpu}` specs from `uname` + `/proc/cpuinfo` flags, sourced tuned
`-march` flags, and (when enabled) namespaced every tool root under `~/.local/$PLAT` so one
NFS-shared `$HOME` could serve machines with different CPUs without binaries clobbering each other.

It worked, but it was **single-axis** (microarch only), its precedence was implicit (a `sort -r` on
spec names), and the detection logic was copy-pasted into four different shell files.

`grim` generalizes this to a **multi-axis, explicitly-precedenced** model where *any* layered value —
a package's provider, an env var, a config-file variant, an install path — resolves the same way:
walk the platform layers that apply to this machine, in precedence order.

## Facts

`grim` probes the machine **once** into a typed `Facts` struct. Everything downstream is a pure
function of `Facts` (plus the grimoire). Probing is the only machine-dependent step.

```
Facts {
    os:           Os,              // macos | linux | windows
    arch:         Arch,            // aarch64 | x86_64 | ...
    cpu_features: Set<Feature>,    // avx2, avx512f, neon, sve, ...
    distro:       Option<Distro>,  // { id: "ubuntu", version: "24.04", like: ["debian"] }
    libc:         Option<Libc>,    // glibc | musl
    hostname:     String,
    gpus:         Vec<Gpu>,        // { vendor: nvidia|amd|apple|intel, model }
    container:    Option<Container>, // { kind: docker|podman|devcontainer }
    // extensible: new fact kinds are additive and never break existing grimoires
}
```

`grim facts` prints this struct and the resolved stack — the typed successor to squinting at
`uname -a` and guessing why `PLAT` chose what it did.

## Platforms

A **platform** is a named layer, declared in the grimoire, with a **match predicate** over facts and
a **precedence**. A machine is described by *all* the platforms whose predicate matches it — usually
several at once.

```toml
[platform.linux]
match = { os = "linux" }

[platform.x86-64-v3]
match = { os = "linux", arch = "x86_64", cpu_features = { all = ["avx2", "fma", "bmi2"] } }

[platform.cuda]
match = { gpu = { vendor = "nvidia" } }

[platform.workstation]            # a specific machine, by hostname
match = { hostname = "x1" }
```

The set of matching platforms, ordered by precedence (highest first), is the **active stack**. It is
a totally-ordered chain selected from the partially-ordered set of all declared platforms — i.e. the
active stack is a *path down a lattice*. Resolution is a fold along that chain.

## Resolution

Every layered value in a grimoire is, conceptually, a map `platform-name → contribution`. To resolve
it for the current machine, walk the active stack from highest precedence to lowest. Two modes:

- **override** (scalars): the highest-precedence contribution wins, the rest are shadowed.
  *Example:* the `EDITOR` env var, or which variant of `gitconfig` to render.
- **merge** (collections): contributions are unioned/concatenated, with higher precedence winning on
  conflicts. *Example:* the set of packages to install, or `PATH` entries.

In lattice terms, the resolved value is the **join** of the contributions along the active chain.
This single mechanism covers what the old system did with ad-hoc `if [ "$PLAT" = ... ]` branches
scattered across templates and scripts.

## Isolation

Install-tree isolation — the original point of `PLAT` — falls out for free. A designated function of
the active stack produces an **isolation id** (default: `{os}-{arch}-{microarch}`, preserving today's
behavior). Every derived tool root (`CARGO_HOME`, `RUSTUP_HOME`, `GOPATH`, `UV_*`, `NVM_DIR`, brew
prefix, …) is computed **once** from that id in one typed place — replacing the derived-variable list
that was hand-maintained in four shell files.

Crucially, the two axes the old system fused are kept **decoupled**:

- **capability** (which `-march` flags / which provider) — always active.
- **directory isolation** (namespacing `~/.local/$id`) — opt-in, off by default (most people have one
  machine and don't need it).

## Composition with grimoires

Two precedence axes exist and must not be confused:

1. **Platform precedence** (this document) — *which machine variant*, within one composed grimoire.
2. **Grimoire layering** ([grimoire.md](grimoire.md)) — *which source repo*: a work grimoire extends
   the public one and overrides it.

Both feed the same resolver. A value's final form is: take the most-specific grimoire layer that
defines it, then resolve *its* platform map against the active stack.

## A dev container is just a platform

Containerization-by-default drops out of this model rather than bolting on. A dev container is the
**highest-precedence platform**: when `Facts.container` is set, the `container` platform matches and
sits at the top of the stack, so its package providers and config win. The same manifest that drives
a host install generates the `devcontainer.json`. No separate code path. See
[packages.md](packages.md) and [the roadmap](../roadmap.md).

---

## Open design choices

These genuinely shape the `grim-core` types. They're the parts worth deciding together.

### 1. How is precedence represented?  *(the big one)*

- **(a) Integer weights.** Each platform gets a number; sort descending. Dead simple, total order,
  but *you* manage the numbers and they collide as the system grows.
- **(b) Declared partial order.** Platforms declare relationships (`workstation refines linux`); the
  active stack is a topological sort of the matching sub-poset. This is the honest model — precedence
  *is* a partial order, and "refines" reads naturally — but it needs cycle detection and a
  tie-break rule when two incomparable platforms both match.
- **(c) Banded weights (hybrid).** Named bands with implied ranges — `os` (100s), `arch` (200s),
  `distro` (300s), `hardware` (400s), `host` (900s) — and you only nudge within a band. Keeps the
  simplicity of integers while encoding the natural layering, and most platforms never need an
  explicit number.

> **Recommendation:** start at **(c)**, designed so a platform *may* also declare `refines = [...]`
> relations (b) when a band isn't expressive enough. Pure integers (a) age badly; a full poset (b)
> is elegant but front-loads machinery before we know we need it. (c) gives the formal layering for
> free and leaves the door open to (b).

### 2. Tie-breaking when two matching platforms have equal precedence

- **error** (force the author to disambiguate — safest, most explicit),
- **definition order** (first-declared wins — convenient, but order-dependent and surprising), or
- **specificity** (the platform whose predicate constrains more facts wins — clever, but the rule is
  subtle).

> **Recommendation:** **error** by default, with an explicit `refines`/weight to break it. Silent
> precedence bugs are exactly the class of problem this redesign exists to kill.

### 3. How rich is the match predicate language?

Start with: equality, set membership (`all`/`any`/`none` over `cpu_features`, `gpus`), and nested
keys (`gpu = { vendor = "nvidia" }`). Defer regex and arbitrary boolean expressions until something
real needs them.

### 4. Isolation id default

Keep `{os}-{arch}-{microarch}` to preserve the current NFS-shared-`$HOME` behavior exactly, but make
it a configurable function of the stack per grimoire. A single-machine user can set it to a constant
and never think about it.
