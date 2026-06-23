# 0006 — A Cargo workspace with a pure, I/O-free core

**Status:** accepted

## Context

The system grim replaces had all its logic in shell: no types, no tests on the riskiest paths, no
dry-run, no transactionality. The first-principles fix is to make the logic a pure function of typed
data so it can be tested exhaustively, and to push side effects to the edges.

## Decision

grim is a Cargo workspace. `grim-core` holds the pure logic — facts, platforms, precedence,
manifests, resolution — with the *only* machine-dependent code being detection, itself isolated
behind functions whose parsing helpers are pure and tested. Crates that touch the world
(`grim-apply`, `grim-pkg`, `grim-secrets`) sit on top and stay thin. New crates are added only when
there is real code to put in them.

## Consequences

- The hard part — platform resolution, package selection, template context construction — is unit-
  tested without a filesystem, network, or package manager.
- I/O is small, at the edges, and easy to reason about (and to dry-run).
- A bit more ceremony than a single crate, paid back immediately in testability and clear seams.
- Contributors have an unambiguous rule for where code goes: pure → core, effectful → a sibling.
