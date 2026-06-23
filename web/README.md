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

Deploys are **automatic**: the Cloudflare Pages project is git-integrated, so Cloudflare builds `web/`
(`npm ci && npm run build`) and publishes to `grim.alkem.dev` on every push to `main`. No GitHub
Actions are involved — Cloudflare does the building.

The project, custom domain, and DNS are managed declaratively by OpenTofu in
[`../infra/cloudflare/`](../infra/cloudflare/); run `just infra-apply` to change them. The build uses
Node `web/.nvmrc`.

## Astro version note

This site is pinned to **Astro 6 + Starlight 0.40**. Astro 7 is released, but Starlight does not yet
support it (its peer dependency is `astro: ^6.4.5`). Bump both together — `npm install astro@latest
@astrojs/starlight@latest` — once Starlight ships Astro 7 support.
