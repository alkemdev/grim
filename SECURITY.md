# Security policy

## Reporting a vulnerability

Please report security issues privately rather than opening a public issue. Use GitHub's
[private vulnerability reporting](https://github.com/alkemdev/grim/security/advisories/new) on this
repository. You'll get an acknowledgement and a fix or mitigation plan.

## How grim handles secrets

grim is designed so that secret *values* never live in a grimoire — only *references*
(`op://…`, `age:…`) that a [secret provider](docs/concepts/secrets.md) resolves at apply-time.
Principles:

- Secret backends are **optional**; the core never depends on one and never logs a resolved value.
- References resolve **lazily** — only when something being applied uses them.
- The 1Password provider shells out to the `op` CLI; the age provider decrypts in-process. grim takes
  no dependency on unofficial vault SDKs.

If you find a path where grim writes, logs, or otherwise exposes a secret value, treat it as a
vulnerability and report it as above.
