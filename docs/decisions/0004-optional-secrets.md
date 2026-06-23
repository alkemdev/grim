# 0004 — Secrets are optional, pluggable providers

**Status:** accepted

## Context

The old system stored secrets as plaintext `~/.<svc>.env` files, chmod 600, globbed into every login
shell — so any process could read every token at once, with no encryption, rotation, or expiry. We
want better, but we also can't make secret management a hard dependency: grim must apply cleanly on a
machine with no vault, no network, and no `op` binary.

## Decision

Secret management is an **optional, pluggable** subsystem. The core engine works with no provider
configured. A grimoire stores secret *references* (`op://…`, `age:…`), never values; a
`SecretProvider` trait resolves a reference at apply-time, lazily (only if something being applied
uses it). Providers ship for plain `env`, 1Password (shelling out to `op`), and age/sops (offline,
in-process). A grimoire may use one, both, or none.

## Consequences

- `grim apply` never fails because a secret backend is missing or offline; it fails only if something
  being applied actually needs an unavailable secret.
- 1Password gives the best workstation UX (biometric unlock, SSH agent, git signing); age/sops is the
  robust offline break-glass for headless boxes, CI, and containers.
- There is no official 1Password Rust SDK (as of 2026), so that provider shells out to `op`; we take
  no dependency on unofficial FFI crates.
- Rotation and expiry are out of scope for now but the trait is shaped to admit them later.
