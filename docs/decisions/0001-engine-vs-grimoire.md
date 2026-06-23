# 0001 — Separate the engine from the data

**Status:** accepted

## Context

The system grim replaces fused two things: a configuration repository (dotfiles) and the shell logic
that applied it. They grew together until the logic couldn't be reused and the data couldn't be
shared. We also need a clean way to layer a private "work" configuration over a public "home" one.

## Decision

`grim` is an **engine** — a general-purpose CLI that anyone can install. A **grimoire** is **data** —
the configuration `grim` reads. A dotfiles repo is just a grimoire. The engine has no knowledge of
any particular user's setup; everything personal lives in the grimoire.

A grimoire can `extend` another (composition), so a private work grimoire includes the public one as
a submodule and overrides it — replacing the old "clone the public repo inside the private one" hack
with a declarative, well-ordered layering.

## Consequences

- The engine is reusable and independently versioned and tested.
- The boundary is a well-defined interface (the grimoire format), which forces clarity.
- A grimoire carries no engine logic, so it stays small and portable.
- Two repos to manage instead of one — an acceptable cost for the separation.
