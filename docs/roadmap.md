# Roadmap

The order is chosen so the **highest-pain, highest-leverage** logic (typed platform resolution and
package orchestration) lands first, and the riskier file-engine cutover happens against a core that's
already proven. Each phase is independently useful and lands as coherent, well-tested commits.

### Phase 0 — Foundation *(in progress)*

Repo, workspace, design docs (this directory), compiling skeleton. Establishes the vocabulary and the
locked decisions: engine vs. grimoire split, own-the-file-layer, optional pluggable secrets, sync
deferred.

### Phase 1 — `grim-core`: facts + platforms

The conceptual core. `Facts` and its detection; the `Platform` model and precedence resolution (see
[platforms.md](concepts/platforms.md), pending the open-choice decisions); the typed manifest model;
`grim facts`. Table-driven tests are the deliverable as much as the code.

### Phase 2 — packages

`grim-pkg`: a provider trait with brew / cargo / uv / npm / go implementations; canonical-id
resolution against the platform stack; `grim sync` and `grim resolve --explain`. Import the existing
package inventory verbatim into the new manifests.

### Phase 3 — apply

`grim-apply`: the MiniJinja render → diff → atomic-write engine; the source→target naming convention;
`grim apply` and `grim diff`. Import the chezmoi `home/` sources. Optionally stage via shelling to
chezmoi first, then swap to native.

### Phase 4 — secrets

`grim-secrets`: the provider trait and the `env` / `onepassword` / `age` implementations; lazy
resolution; template injection. All optional.

### Phase 5 — orchestration + TUI

`grim up` (the idempotent setup replacing `bootstrap.sh`'s install/update/upgrade), `grim doctor`,
and the interactive TUI wizard. Transactional execution with structured logging.

### Phase 6 — migrate the grimoire

Restructure the `dotfiles` repo into a `grim` grimoire. Stand up the private work grimoire that
`extends` it via submodule. Prove the home ← work layering end-to-end.

### Phase 7 — sync + backup *(deferred)*

The two-layer design researched but not yet built: Syncthing 2.x over Tailscale for shared folders
(Downloads/Documents everywhere), restic to a hub + offsite for versioned recovery. `grim` as the
declarative reconciler over both; a clean seam is reserved for it now.

### Phase 8 — the docs website

Build the public site from this `docs/` tree so the documentation and the "personal management
engine" website are one source of truth.
