# Deployment

evo-1 is intended to run on Cloudflare Pages at:

- Production URL: `https://evo-1.tre.systems`
- Cloudflare Pages project: `evo-1`
- Pages fallback URL: `https://evo-1.pages.dev`

## Build Output

Cloudflare Pages should serve the generated `dist/` directory, not the repository root.

```bash
./scripts/build-pages.sh
```

The build script:

1. Runs the threaded WASM package build.
2. Creates a fresh `dist/` directory.
3. Copies `index.html`, `LICENSE`, the generated `pkg/` files, and `public/_headers`.

## Direct Upload

Use Wrangler for a manual deployment:

```bash
npx wrangler pages deploy dist --project-name evo-1
```

The same command is available through npm:

```bash
npm run deploy
```

## Git Integration

GitHub Actions deploys to Cloudflare Pages after CI passes on pushes to `main`
and the active `feature/parallel-evolution-refactor` branch. The deploy job uses
the organization/repository Actions secrets:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`

The action deploys the generated `dist/` bundle with:

```bash
npx wrangler pages deploy dist --project-name evo-1 --branch main
```

For a Cloudflare Pages project connected directly to GitHub instead:

| Setting | Value |
| --- | --- |
| Project name | `evo-1` |
| Production branch | `main` after the repository rename lands |
| Build command | `./scripts/build-pages.sh` |
| Build output directory | `dist` |

## Custom Domain

Add `evo-1.tre.systems` under the Pages project's **Custom domains** tab.

If `tre.systems` is managed in the same Cloudflare account, Cloudflare should create the DNS record during custom-domain setup. If DNS is managed elsewhere, add a CNAME from `evo-1.tre.systems` to `evo-1.pages.dev` after associating the custom domain with the Pages project. Cloudflare documents that adding only the CNAME, without first adding the custom domain to the Pages project, can fail.

Relevant Cloudflare docs:

- [Pages custom domains](https://developers.cloudflare.com/pages/configuration/custom-domains/)
- [Pages custom headers](https://developers.cloudflare.com/pages/configuration/headers/)
- [Wrangler Pages deploy](https://developers.cloudflare.com/workers/wrangler/commands/pages/#deploy)

## Headers

The browser app can use `SharedArrayBuffer` and threaded WASM only when served with cross-origin isolation headers. `public/_headers` is copied into `dist/_headers` and applies:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`
- conservative browser security headers
- `Cache-Control: no-cache` for the app shell and WASM package files

Keep the cache policy conservative until the build produces fingerprinted asset filenames.
