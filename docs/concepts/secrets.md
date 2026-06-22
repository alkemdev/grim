# Secrets

Secret management in `grim` is **optional and pluggable**. The core engine works with no secret
provider configured. Providers are an add-on you turn on per grimoire — never a hard dependency, and
never a thing that can block `grim apply` on a machine that doesn't have them.

## The model

A grimoire stores secret *references*, never secret *values*:

```toml
[secret.github_token]
ref = "op://Personal/GitHub/token"     # resolved by the 1Password provider

[secret.anthropic_key]
ref = "age:secrets/anthropic.age"      # resolved by the age/sops provider, offline
```

A `SecretProvider` trait resolves a reference to a value at apply-time, and the value is injected into
templates (e.g. an `.env` or a config file) — or, better, into a process environment for a single
command, so it never lands on disk. Resolution is **lazy**: a reference is only resolved if something
being applied actually uses it.

## Providers

| Provider     | Backend                          | Notes                                                      |
| ------------ | -------------------------------- | ---------------------------------------------------------- |
| `env`        | the current shell environment    | the no-dependency default; mirrors today's `~/.svc.env`    |
| `onepassword`| the `op` CLI (shelled out)       | `op://` refs, biometric unlock, SSH agent, git signing     |
| `age`        | age / sops encrypted files in-repo| fully offline break-glass; decrypts in-process in Rust     |

> There is no official 1Password Rust SDK as of 2026, so the `onepassword` provider shells out to
> `op`. The unofficial FFI crates are not a dependency worth taking.

## Why both 1Password *and* a local handler

1Password gives the best workstation UX — biometric unlock, the SSH agent (so ssh keys and git
commit signing never touch disk), and a real audit trail. But a workstation-only, network-dependent
vault is a single point of failure for headless boxes, CI, and containers. The `age` provider is the
robust offline escape hatch: sops-encrypted files committed to the grimoire, decrypted in-process,
no daemon required. A grimoire can use one, both, or neither.

## What this replaces

Today: 10 per-service `~/.<svc>.env` files, chmod 600, blanket-sourced (`for f in ~/.*.env`) into
every login shell — so any process can read every token at once, with no encryption, rotation, or
expiry. The one good pattern (deriving `GITHUB_TOKEN` from the `gh` keyring per-shell, never on disk)
generalizes into the provider model and is kept. The machine-wide gitleaks pre-push hook stays
regardless.

## Out of scope for now

Secret *rotation*, *expiry*, and the [sync/backup](../roadmap.md) layer are deferred. The provider
trait is designed so they can slot in later without reworking callers.
