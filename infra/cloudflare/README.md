# Cloudflare infrastructure (OpenTofu)

Manages the hosting for [grim.alkem.dev](https://grim.alkem.dev) declaratively — **no GitHub
Actions**. It creates:

- a Cloudflare **Pages** project `grim` (Direct Upload type),
- the custom domain `grim.alkem.dev` bound to that project,
- a proxied `CNAME` `grim → <project>.pages.dev` in the `alkem.dev` zone.

The site *content* is uploaded separately with `wrangler` (`just site-deploy`); this directory only
manages the project, domain, and DNS.

## Prerequisites

- [OpenTofu](https://opentofu.org) (`tofu`).
- `CLOUDFLARE_API_TOKEN` in the environment, with **Account → Cloudflare Pages → Edit** and
  **Zone → DNS → Edit** on the `alkem.dev` zone.
- `terraform.tfvars` with the account id (copy `terraform.tfvars.example`; the account id is found via
  the `curl` in that file). It is git-ignored.

## Use

```bash
cd infra/cloudflare
tofu init
tofu plan
tofu apply        # or, from the repo root: just infra-apply
```

State is local (`terraform.tfstate`, git-ignored). After the project exists, deploy the site:

```bash
just site-deploy  # from the repo root
```

## Why Direct Upload (and not git integration)

`alkemdev/grim` is private. Cloudflare's git integration (where Cloudflare builds the site itself on
push) needs the Cloudflare GitHub App connected to the repo — a one-time dashboard step. Until then,
Direct Upload keeps everything scriptable and under our control. To switch later, connect the app and
add a `source { type = "github" }` + `build_config` block to `cloudflare_pages_project` (see the
mdBook-era config in the old dotfiles repo for the shape). Either way, **no GitHub Actions** are
involved.
