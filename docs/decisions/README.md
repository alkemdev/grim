# Decision records

Architecture Decision Records (ADRs) capture *why* grim is shaped the way it is — the context, the
choice, and the consequences — so the reasoning survives the people who made it.

Each record is immutable once accepted: to change a decision, add a new ADR that supersedes the old
one rather than editing history. Keep them short.

| #    | Decision                                                              | Status   |
| ---- | -------------------------------------------------------------------- | -------- |
| [0001](0001-engine-vs-grimoire.md) | Separate the engine (`grim`) from the data (a grimoire) | accepted |
| [0002](0002-own-the-file-layer.md) | Own the file-templating layer; retire chezmoi          | accepted |
| [0003](0003-banded-precedence.md)  | Platform precedence by inferred bands, error on ties   | accepted |
| [0004](0004-optional-secrets.md)   | Secrets are optional, pluggable providers              | accepted |
| [0005](0005-defer-sync.md)         | Defer file sync + backup to a later phase              | accepted |
| [0006](0006-pure-core-workspace.md)| A Cargo workspace with a pure, I/O-free core           | accepted |
