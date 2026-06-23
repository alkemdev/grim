# grim website

The source for [grim.alkem.dev](https://grim.alkem.dev), built with [Astro](https://astro.build) +
[Starlight](https://starlight.astro.build).

## Single source of truth

The concept and decision pages are **generated from the repo's `../docs/` tree** by
`scripts/sync-docs.mjs` — edit the markdown in `docs/`, not the copies under `src/content/docs/`
(which are git-ignored and regenerated). `npm run sync` runs automatically before `dev` and `build`.

Hand-authored pages that live only in the site: `src/content/docs/index.mdx` (the landing).

## Develop

```bash
cd web
npm install
npm run dev       # syncs docs/, starts the dev server with live reload
npm run build     # syncs docs/, builds to web/dist
npm run preview   # preview the production build
```

## Deploy (Cloudflare Pages — no GitHub Actions)

The Pages project, custom domain (`grim.alkem.dev`), and DNS are managed by OpenTofu in
[`../infra/cloudflare/`](../infra/cloudflare/) — a **Direct Upload** project, so Cloudflare never
builds from the repo. Deploys are explicit:

```bash
just site-deploy     # from the repo root: builds, then `wrangler pages deploy`
```

`wrangler` reads `CLOUDFLARE_API_TOKEN` from the environment (set `CLOUDFLARE_ACCOUNT_ID` too if your
token spans multiple accounts). To provision or change the project/domain/DNS, run `just infra-apply`.

> Want auto-deploy on push instead? Connect the Cloudflare GitHub App to the repo and switch the
> OpenTofu `cloudflare_pages_project` to a `source { type = "github" }` build — still no GitHub
> Actions, since Cloudflare does the building.

## Astro version note

This site is pinned to **Astro 6 + Starlight 0.40**. Astro 7 is released, but Starlight does not yet
support it (its peer dependency is `astro: ^6.4.5`). Bump both together — `npm install astro@latest
@astrojs/starlight@latest` — once Starlight ships Astro 7 support.
