# 0005 — Defer file sync + backup to a later phase

**Status:** accepted

## Context

A goal for the broader system is to have certain folders (Downloads, Documents) shared across all
machines *and* backed up with versioned recovery. The research settled on a two-layer design:
Syncthing over Tailscale for same-state-everywhere, and restic to a hub plus offsite for versioned
recovery, with grim acting as a declarative reconciler over both. But this touches external daemons
and a fleet topology, and it is orthogonal to the core job of applying a configuration to one
machine.

## Decision

Sync and backup are **deferred** to a later phase. We reserve a clean architectural seam (a `grim
sync`/`grim backup` module slot and per-host config shape) but build none of it now. The core
engine — facts, platforms, packages, files, secrets — ships first.

## Consequences

- The first usable grim is smaller and focused; it does the universal job well before taking on a
  fleet-management job.
- The seam is documented so adding the sync layer later doesn't require reworking the core.
- Until it lands, sync/backup remains a manual, out-of-band concern.
