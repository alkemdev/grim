# Apply

`grim apply` renders the grimoire's `files/` tree and places it under a target root (default
`$HOME`). It is grim's replacement for `chezmoi apply` — scoped deliberately to one user's needs,
which is what keeps it small ([ADR 0002](../decisions/0002-own-the-file-layer.md)).

## The source tree

A grimoire's `files/` directory mirrors the target tree. Attributes are encoded in path-component
names and applied per component, so they work for both files and directories:

| source component   | target            | effect                                  |
| ------------------ | ----------------- | --------------------------------------- |
| `dot_zshrc`        | `.zshrc`          | leading-dot rename                      |
| `dot_config/`      | `.config/`        | same, for directories                   |
| `executable_setup` | `setup` (mode 755)| mark executable                         |
| `private_dot_ssh/` | `.ssh/` (mode 700)| restrict permissions (file: 600)        |
| `gitconfig.tmpl`   | `gitconfig`       | render as a template (suffix stripped)  |

Attributes combine in the order `[executable_|private_] dot_ name [.tmpl]` — e.g.
`executable_dot_script.tmpl` → `.script`, mode 755, templated. Source entries beginning with a literal
`.` (like `.git`) are ignored.

## The template context

Templates are [MiniJinja](https://docs.rs/minijinja) (Jinja2-compatible). A template sees:

- `facts` — the detected machine, e.g. `{{ facts.os }}`, `{{ facts.arch }}`, `{{ facts.hostname }}`,
  `{{ facts.cpu_features }}`.
- `platforms` — the active platform names, highest precedence first, e.g.
  `{% if "workstation" in platforms %}…{% endif %}`.
- `isolation_id` — the per-platform install-tree namespace, e.g. `{{ isolation_id }}`.

```jinja
# ~/.demo-grimrc, rendered for {{ facts.hostname }} ({{ facts.os }}/{{ facts.arch }})
{% if "workstation" in platforms -%}
EDITOR=nvim
{%- else -%}
EDITOR=vim
{%- endif %}
```

Trailing newlines are preserved (config files should end with one).

## Apply = render → diff → atomic write

For each source entry, grim computes the target path, mode, and content (rendering templates), then
compares against what's on disk:

- **create** — the target doesn't exist.
- **update** — it exists but differs (a unified diff is computed for text files).
- **unchanged** — it already matches; skipped.

Writes go to a temp file in the target directory and are `rename`d into place, so a reader never sees
a partial file. `--dry-run` stops after the diff and writes nothing.

## Commands

```bash
grim apply --grimoire <dir> [--target <root>] [--dry-run]
grim diff  --grimoire <dir> [--target <root>]   # a dry-run apply that shows the diffs
```

`--grimoire` is the grimoire *directory* (containing `grimoire.toml` and `files/`); `--target`
defaults to `$HOME`. Try it against the bundled example:

```bash
grim diff  --grimoire examples/demo --target /tmp/grim-demo
grim apply --grimoire examples/demo --target /tmp/grim-demo
```

## Status & what's next

Implemented: the naming convention, the MiniJinja render context, plan/diff/atomic-apply, modes on
Unix. Not yet: symlink sources, deletion of files grim previously managed (state tracking), and
sourcing the chezmoi `home/` tree via an importer. See the [roadmap](../roadmap.md).
