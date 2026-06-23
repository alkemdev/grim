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

## Deploy (Cloudflare Pages)

CI (`.github/workflows/site.yml`) builds the site on every push. To turn on deploys to
`grim.alkem.dev`:

1. Create a Cloudflare Pages project named **`grim`** (Direct Upload type).
2. Add repo secrets `CLOUDFLARE_API_TOKEN` (with Pages:Edit) and `CLOUDFLARE_ACCOUNT_ID`.
3. Set the repo **variable** `DEPLOY_SITE=true` to enable the deploy step.
4. In the Pages project, add the custom domain `grim.alkem.dev`.

## Astro version note

This site is pinned to **Astro 6 + Starlight 0.40**. Astro 7 is released, but Starlight does not yet
support it (its peer dependency is `astro: ^6.4.5`). Bump both together — `npm install astro@latest
@astrojs/starlight@latest` — once Starlight ships Astro 7 support.
