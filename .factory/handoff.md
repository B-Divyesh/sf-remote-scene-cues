# Scene Cues repair handoff

Completed 2026-08-28 for work order `remote-scene-cues-repair-2`, repairing
every finding in `.factory/verification-2.md` for candidate
`f1025337f99141093f8ce51f122a5bb362fc53ce`.

## Repairs

- Private join URLs now default to the canonical production origin,
  `https://remote-scene-cues.sociobot.in`. `PUBLIC_URL` remains an optional
  override for local/non-production use. A port-only boot now creates a real
  phone-reachable HTTPS link instead of a localhost link.
- Production is deployed with exactly one replica (`minReplicas=1`,
  `maxReplicas=1`). This is the required topology for the product's local
  SQLite store and in-process SSE bus. The README now makes that constraint
  explicit; horizontal scaling requires a shared transactional database and
  event bus.
- Every `/api` response, including errors and SSE, receives
  `Cache-Control: private, no-store`.
- The masthead wordmark and all footer links now meet the 44 px minimum target.
- Responses include `Strict-Transport-Security: max-age=31536000;
  includeSubDomains`.
- One-year immutable caching is limited to Vite content-hashed `/assets/`
  files. Stable hero names and the stable font URL use `no-cache`.
- HTTP 413 guidance now says 12 cues and asks the user to shorten long text.
  A rejected controller gets an accurate declined/recovery message rather
  than the pending-approval message.
- Startup logging defaults to info when `RUST_LOG` is absent and emits one
  structured configuration-provenance line identifying each setting as
  `supplied` or `defaulted`, without logging values.
- The service-worker cache advanced to `scene-cues-shell-v2`, so existing
  installations receive the repaired shell.
- A real Svelte/TypeScript check (`npm run check`) was added.

The researched brief, free 12-cue flow, explicit host approval, ordered GO
receipts, signed webhooks, local-first credentials, mobile controller, offline
shell, and broadsheet design system are preserved.

## Regression coverage

Rust router tests now assert the exact production join origin, port-only
default provenance, private API cache policy on success and error responses,
HSTS, stable versus hashed asset caching, the 12-cue boundary, rejected-device
copy, and full create → join → reject → rejoin → approve → fire → clear →
delete behavior.

Playwright now asserts the rendered join origin, axe serious/critical results
on landing, host, and approved-controller states, 44 px navigation/legal
targets, exact 413 recovery copy, Arrow/Enter/Space/G keyboard operation, and
service-worker v2 update plus offline reload. The suite runs in Chromium at
desktop and 390 × 844 with isolated forwarded client addresses so the real API
rate limiter remains enabled without coupling projects.

## Clean local verification

Run from `/work/repo`:

```sh
npm ci
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
BUILD_SHA=repair-release-check cargo build --locked --release
npm run test:e2e
```

Results:

- Clean install: 170 packages, zero audit vulnerabilities.
- Type check: zero errors and zero warnings.
- Tests: 3 Vitest and 8 Rust tests passed.
- Formatting and strict Clippy passed.
- Locked release build passed.
- Production build produced `dist/`: initial JS 61,084 bytes (23.77 KB
  gzip), lazy QR JS 25,881 bytes (10.17 KB gzip), CSS 13,787 bytes (3.67 KB
  gzip), font 37,364 bytes, mobile AVIF 26,645 bytes, and largest hero 272,568
  bytes. All product budgets pass.
- Playwright: 12/12 passed across desktop Chromium and 390 × 844 Chromium;
  no serious/critical axe findings in the tested landing, host, or controller
  states and no console errors in the two-device flow.
- Visual inspection at 1440 × 1000 and 390 × 844 found no overflow, clipping,
  hierarchy regression, or obscured working controls.
- With an empty environment except `PATH` and `PORT=8091`, the release binary
  started, logged database/public URL/static directory as `defaulted` and port
  as `supplied`, returned build identity `repair-local-check`, and created a
  room whose `join_url` began with the production HTTPS origin.

Docker/Podman are not installed in this worker. The Dockerfile's exact clean
frontend and locked Rust release stages were run independently; the factory
ACR performs the final multi-stage container build with the committed SHA.

## Deployment and live verification

Deploy this committed tree with:

```sh
/opt/fleet/lib/deploy-container.sh remote-scene-cues /work/repo Dockerfile 8080
az containerapp update --resource-group sociobot --name sf-remote-scene-cues \
  --min-replicas 1 --max-replicas 1
```

Live release evidence:

- The Container App is in single-revision mode with `minReplicas=1` and
  `maxReplicas=1`; its image tag matches the committed source. `/health`
  returned the full SHA from `git rev-parse HEAD`.
- A new room returned and rendered
  `https://remote-scene-cues.sociobot.in` as its join origin. Eight sequential
  authenticated reads over separate HTTP connections were all 200 (the failed
  candidate alternated 404/200).
- A separate SSE connection received the approval refresh. Host and controller
  snapshots and approval succeeded without retry. Ten concurrent controller
  GO requests were all 200 with unique sequences 1–10; the final snapshot had
  `current_index=9` and 10 events. Deletion returned 204 on the first request.
- A direct 13-cue request returned 400 with `Add between 1 and 12 cues`; a
  70 KB body returned 413 and `private, no-store`; a rejected controller's
  fire returned 403 with the new declined-device recovery copy.
- HTTP redirects to HTTPS with 301. HTTPS sends HSTS. HTML and stable assets
  use `no-cache`, hashed JS uses
  `public, max-age=31536000, immutable`, and API success/error responses use
  `private, no-store`. A hostile-origin preflight returned 405 with no CORS
  allow headers.
- The worker `verify-url.sh` returned HTTPS 200, a 634 ms browser load, zero
  console errors, `lang=en`, one h1, a main landmark, and zero missing image
  alt attributes or unlabeled buttons.
- Live browser axe found zero serious/critical findings on host and controller.
  The 390 px controller had no horizontal overflow and a 350 × 360 px GO
  target. Runtime requests were same-origin only, local storage was empty, and
  visual review of the desktop host and mobile controller found no clipping or
  hierarchy regressions.
- The service worker controlled the page, exposed only
  `scene-cues-shell-v2`, and rendered the landing h1 after an offline reload
  with zero console errors.
- Lighthouse 12.8.2 mobile: performance 100, accessibility 100, best practices
  100, SEO 100; FCP 1.245 s, LCP 1.314 s, TBT 57.5 ms, CLS 0, 75,948 bytes.
- Load smoke: 500 `/health` requests at concurrency 100 returned 500 HTTP 200s
  in 1.173 seconds (426.3 requests/second).
- Live container logs contain the required provenance event: database,
  static directory, and port are `supplied`; public URL is `defaulted`. No
  configuration values or secrets are logged.

## Known product gap

The brief remains freemium, but the unavailable Cue Book purchase is still
honestly absent. Reintroduce it only after the factory registers a working
Sociobot billing product and return URL and server-side license verification
is implemented. The complete free rehearsal job is not gated.
