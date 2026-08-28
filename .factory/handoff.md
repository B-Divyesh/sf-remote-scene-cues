# Scene Cues v1 handoff

Completed 2026-08-28 for work order `remote-scene-cues-build-1`.

## What shipped

- A production Svelte host flow for naming a rehearsal, authoring ordered
  `Scene | Cue` lines, and optionally configuring a signed webhook.
- Eight-hour SQLite-backed rooms with 6-character codes, 24/40-character
  secrets, hashed access tokens, explicit host approval, and immediate host
  deletion. Expired rooms are deleted in a recurring cleanup task.
- A private QR/controller link, pending and rejected states, a 390 px remote,
  keyboard `G`, server-serialized GO actions, live SSE refresh, and durable cue
  receipts. Reconnects retrieve the latest snapshot, so a missed SSE message
  does not mean a missed cue receipt.
- Host-only cue selection, controller list, webhook delivery state, CSV export,
  log deletion, and complete empty/error/offline states.
- HMAC-SHA256 webhook POSTs. Targets must be public HTTPS, are DNS-checked again
  at delivery, cannot contain credentials, and cannot redirect into a private
  network. Failure never alters cue order.
- Cue Book: a $24 one-time Sociobot checkout link, callback token storage,
  once-daily verification cache, offline optimistic unlock, restore field, 50
  cue limit, and 20 device-local saved cue sheets. The free 12-cue room, CSV
  export, accessibility, and safety controls remain ungated.
- `/privacy` and `/terms`, no analytics, session-only room credentials, a
  versioned offline shell, self-hosted League Gothic, strict security headers,
  compressed responses, request size limits, and proxy-aware API rate limits.
- Original broadsheet illustration with retained source/prompt metadata and
  AVIF/WebP delivery variants (mobile AVIF 27 KB; mobile WebP 119 KB; largest
  delivered hero 267 KB). The design system and provenance are in
  `.factory/design.md`.
- A non-root, multi-stage container that serves the built frontend and Axum API
  together on `PORT=8080`, with `/data` reserved for the SQLite volume.

## Verification performed

All commands ran from `/work/repo`.

- `npm ci` — clean install, 0 npm audit vulnerabilities.
- `npm test` — 3 Vitest unit tests and 3 Rust tests pass. The Rust integration
  test exercises create → join → reject pre-approval fire → approve → fire →
  clear log → delete room through the real Axum router.
- `npm run test:e2e` — 4/4 Playwright 1.58.2 tests pass across desktop Chromium
  and a 390 × 844 Chromium viewport. A two-browser-context test completes the
  real host/controller flow; axe finds no serious/critical violations; both
  pages assert zero console errors.
- `npm run build` — `dist/index.html` produced. Initial route payload: 65.03 KB
  JS (25.03 KB gzip) and 13.71 KB CSS (3.66 KB gzip). The 25.88 KB QR module is
  lazy-loaded only on host pages. Self-hosted font: 37 KB.
- `cargo build --release --locked` — optimized backend succeeds.
- Lighthouse 12.8.2, mobile emulation against the production build: performance
  **100**, accessibility **100**, best practices **100**, SEO **100**; LCP
  **1.5 s**, CLS **0**, total blocking time **30 ms**, max potential input delay
  **80 ms**.
- Load smoke: 500 `/health` requests at concurrency 100 returned 500 HTTP 200s
  in 2.139 seconds (about 234 requests/second).
- Visual review performed at 1440 × 1000 and 390 × 844. Focus, reduced-motion,
  mobile stacking, generated-artifact review, and contrast were checked. The
  landing page axe/Lighthouse contrast checks pass.

The container stages were also reproduced individually (`npm ci`, frontend
build, and locked Rust release build). A Docker daemon/CLI was not present in
the worker image, so `docker build` itself could not be invoked here.

## Deployment

Build with `docker build -t scene-cues .`. Set:

- `PUBLIC_URL=https://remote-scene-cues.sociobot.in`
- `DATABASE_URL=sqlite:///data/scene-cues.db?mode=rwc`
- a persistent writable volume at `/data`
- `PORT=8080` (already the image default)

For staging, build with
`VITE_BILLING_BASE=https://pilot-api.sociobot.in`; use the production default at
release. The factory still needs to register the `remote-scene-cues` paid
product and return URL. No product ID is hardcoded.

## Known limits and next steps

- Webhooks make one five-second attempt. Failures are visible in the receipt
  log but are not retried; a bounded retry queue would be the next reliability
  improvement for retained shows.
- Cue Books intentionally remain local to one browser because v1 has no account
  system. The restore flow covers licenses, not saved sheets.
- Room availability depends on the deployed single-container SQLite volume.
  Horizontal scaling would require PostgreSQL plus a shared event bus.
- Payments cannot be end-to-end charged until the factory registers the paid
  product; the client implements the supplied buy/return/verify contract.
