# Cloudflare infrastructure (OpenTofu)

Manages the hosting for [grim.alkem.dev](https://grim.alkem.dev) declaratively — **no GitHub
Actions**. It creates:

- a Cloudflare **Pages** project `grim`, git-integrated with `alkemdev/grim` (Cloudflare builds
  `web/` and deploys on every push to `main`),
- the custom domain `grim.alkem.dev` bound to that project,
- a proxied `CNAME` `grim → <project>.pages.dev` in the `alkem.dev` zone.

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

State is local (`terraform.tfstate`, git-ignored). Once the project exists, **deploys are
automatic** — push to `main` and Cloudflare builds `web/` and publishes. There's no manual deploy
step and no GitHub Actions.

## Notes

- The project is connected to GitHub via the Cloudflare GitHub App on the `alkemdev` org (shared with
  the account's other Pages projects). Build settings live in the `build_config` block of
  `cloudflare_pages_project` (root `web/`, `npm ci && npm run build`, output `dist`).
- Replacing the project (e.g. changing its git source) requires detaching the custom domain first;
  the cleanest path is `tofu destroy` then `tofu apply`, since Terraform orders the domain/record
  teardown before the project.
