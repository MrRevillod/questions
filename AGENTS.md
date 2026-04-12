# AGENTS.md

## Repo Shape
- This repo is not a JS/Rust monorepo in the usual sense: the Rust workspace only contains `apps/server`, while the frontend is a separate pnpm project in `apps/client` with its own lockfile.
- Main backend entrypoint: `apps/server/src/main.rs`. It wires feature modules through `sword::Application` (`auth`, `authz`, `attempts`, `users`, `quizzes`, `shared`).
- Main frontend shell: `apps/client/src/routes/+layout.svelte`. It installs the TanStack Query provider and Axios interceptors once for the whole SPA.
- The frontend is client-only: `apps/client/src/routes/+layout.ts` sets `ssr = false`.

## Commands
- Preferred local workflow is Docker-first. `make run` starts the full stack from `compose.yml`; `make detach` runs it in the background.
- First-time local setup is `make setup`. It deletes `apps/client/node_modules`, runs `corepack pnpm install --frozen-lockfile --ignore-scripts` in `apps/client`, then rebuilds/restarts the client container.
- Repo-wide formatting/lint shortcuts are `make fmt` and `make lint`.
- Focused backend verification:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test -p server`
- Focused frontend verification from `apps/client`:
  - `corepack pnpm run lint`
  - `corepack pnpm run check`
  - `corepack pnpm run build`
- CI does not run frontend tests because none are configured; client CI is `lint` + `check`, server CI is `fmt` + `clippy` + `test`.

## Docker And Env Quirks
- Root `.env` is loaded by `make`, `compose.yml`, and the client Vite app (`apps/client/vite.config.ts` uses `envDir: "../../"`). Do not assume `apps/client/.env` is used.
- Local HTTP entrypoint is the nginx container on port `80`, not the Vite dev server directly. Nginx proxies `/` to the client and `/api/` to the Rust server.
- The frontend Axios client is hardcoded to `baseURL: "/api"`; API work should preserve that proxy path unless the nginx/client wiring is intentionally changing.
- Server startup runs SQL migrations automatically in `Database::new()` using `apps/server/config/migrations`.
- `make clean-db` drops and recreates the public schema, then reruns migrations against `LOCAL_POSTGRES_DATABASE_URL`. Treat it as destructive.
- The server dev container starts through `apps/server/config/scripts/entrypoint.sh`, which attempts WireGuard setup before `cargo watch`. VPN env vars in `.env` are optional for local work.

## Code Navigation
- Backend feature layout is consistent: each feature lives under `apps/server/src/<feature>/` with `controller`, `service`, `repository`, and optional `policy`/`dtos`/`entity` files. Start there before searching broadly.
- Frontend feature code lives in `apps/client/src/lib/features/*`; route files are thin wrappers around feature services/stores.
- Auth/session behavior is centralized in `apps/client/src/lib/features/auth/auth.service.ts` and `apps/client/src/lib/shared/http/api.interceptors.ts`. Protected-route redirects happen in `apps/client/src/routes/(protected)/+layout.ts`.

## Style Notes
- Frontend formatting is shared from the root `.prettierrc`: tabs, no semicolons, `printWidth: 85`, and Tailwind class sorting via `prettier-plugin-tailwindcss` using `apps/client/src/routes/layout.css` as the Tailwind stylesheet.
- The frontend uses Svelte 5 runes (`$state`, `$derived`) rather than legacy store patterns in component code. Match the existing style when editing Svelte files.
