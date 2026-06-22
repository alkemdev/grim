# Apply

`grim apply` renders the grimoire's `files/` tree into `$HOME` (and other targets). It is the
replacement for `chezmoi apply` — scoped deliberately to *one user's needs* rather than the universe
of everyone's, which is what keeps it small.

## Why own this instead of wrapping chezmoi

chezmoi's templating layer works well; the reason to absorb it is **one tool, one model**. With the
file engine in-process, there is no `run_onchange_` hook bridge between "chezmoi rendered a file" and
"grim should now reconcile packages", no second config language, and no second tool to install. The
file-apply feature set we actually use is a small fraction of chezmoi's, so reimplementing it cleanly
is tractable.

## The engine

- **Templating: [MiniJinja](https://docs.rs/minijinja).** Jinja2-compatible, a single pure-Rust
  dependency, actively maintained. Templates see a context built from `Facts`, the resolved platform
  values, and resolved [secrets](secrets.md).
- **Source naming.** A small, explicit convention maps a source path to a target path, mode, and
  type (file / template / symlink). The plan is to keep chezmoi's legible `dot_`, `executable_`,
  `private_`, `.tmpl` ideas where they pull their weight, but defined by `grim`, not inherited.
- **Apply = render → diff → atomic write.** Render to memory, diff against what's on disk, and only
  write what changed — to a temp file, then atomic rename. `--dry-run` stops at the diff and prints
  it. This gives the transactionality and previewability the shell pipeline never had.

## Migration

A one-time importer maps the existing chezmoi `home/` sources (146 `dot_` files, 32 `.tmpl`
templates, the `.chezmoidata.toml` / `.chezmoitemplates/` partials) into the new `files/` layout. The
v0 path may even shell out to chezmoi to de-risk the cutover, then swap in the native engine once it
reaches parity — the [decision](../../README.md) was to own this, but we can stage *how* we get there.
