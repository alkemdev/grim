# 0003 — Platform precedence by inferred bands, error on ties

**Status:** accepted

## Context

The old `PLAT` system isolated install trees by CPU microarchitecture. It was single-axis, its
precedence was implicit (a reverse sort of spec names), and its detection was copy-pasted into four
shell files. grim generalizes this to multi-axis platform layers (OS, arch, distro, libc, GPU,
hostname, container) where *any* layered value resolves the same way. The open question was how to
express precedence.

The options weighed: plain integer weights (simple, but hand-managed and collision-prone); a full
declared partial order via `refines` (the most honest model, but front-loads cycle detection and
tie-break machinery); or **banded** weights where precedence is implied by the kind of fact a
platform constrains.

## Decision

Precedence is a **band**, *inferred* from which facts a platform's predicate constrains
(`os < arch < distro < hardware < host < container`), plus an optional `weight` nudge and an optional
`refines` relation as escape hatches. In the common case the author writes **zero** precedence
numbers. When two matching platforms tie on score and no `refines` relation orders them, resolution
**errors** rather than guessing.

## Consequences

- The natural layering (a host beats hardware beats OS; a dev container beats all) is free and needs
  no configuration.
- Silent precedence ambiguity — the exact bug class the rewrite exists to kill — is impossible;
  ambiguity is a loud error with a fix suggestion.
- `refines` is wired now (validated for unknown targets and cycles) so the schema won't break when a
  richer partial-order resolver lands later.
- Resolution is a lattice join: the active stack is a path down the platform poset, and a layered
  value is the join of contributions along it.
