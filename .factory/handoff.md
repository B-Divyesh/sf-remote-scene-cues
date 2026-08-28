# Scene Cues repair handoff

Completed 2026-08-28 for work order `remote-scene-cues-repair-1`, repairing
the independent-verification failures recorded at commit
`8278b7f7991ee61cec504549e8df1e3287f38af4`.

## Repairs

- The advertised Cue Book checkout, restore UI, paid-limit copy, billing
  storage, and billing CSP permissions were removed. The registered billing
  product still returns 404 and repository rules prohibit registering or
  changing billing from this repo. This is the verifier's explicitly allowed
  honest fallback: no visitor is offered an unavailable $24 purchase.
- The API now has a single authoritative `FREE_CUE_LIMIT` of 12. A direct
  `POST /api/rooms` with 13 valid cues returns `400` and
  `Add between 1 and 12 cues`; the browser limit is no longer bypassable.
- Docker now defaults `BUILD_SHA` to `dev`, rather than `unknown`. The factory
  deployment helper supplies the immutable committed SHA as its build arg, and
  `/health` exposes that compiled identity.
- Host cue-list buttons now support the documented Arrow Up/Down/Left/Right
  focus selection without firing a cue; Enter and Space retain their normal
  button activation behavior.
- The privacy response policy no longer lists the removed Sociobot billing
  origins. No analytics, trackers, third-party runtime scripts, or billing
  requests are shipped.

The original free two-device rehearsal flow, host approval, ordered GO
receipts, signed webhook behavior, export, mobile controller, service worker,
and broadsheet visual system are unchanged.

## Regression coverage

- Rust API tests cover the 13th-cue rejection, non-`unknown` health identity,
  response CSP, existing webhook validation, and the full create → join →
  approve → fire → clear → delete flow.
- Playwright covers the desktop and 390 × 844 mobile projects: the two-device
  host/controller flow, axe serious/critical violations, console errors, no
  checkout/billing origin exposed, arrow-key host selection, and an offline
  reload controlled by the versioned service-worker shell.

## Verification run locally

All commands ran from `/work/repo` after a clean `npm ci`.

- `npm ci` — 165 packages installed; `npm audit` reported 0 vulnerabilities.
- `npm test` — 3 Vitest tests and 6 Rust tests passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --check` — passed.
- `npm run build` — passed; initial JavaScript is 61.08 KB (23.77 KB gzip),
  lazy QR JavaScript is 25.88 KB (10.17 KB gzip), and CSS is 13.71 KB
  (3.66 KB gzip), all within the configured budgets.
- `cargo build --release --locked` — passed.
- `npm run test:e2e` — 8/8 passed across desktop Chromium and the 390 × 844
  mobile project; axe reported no serious or critical issues.
- A locally compiled `BUILD_SHA=repair-identity-check` server returned
  `{"status":"ok","build_sha":"repair-identity-check"}` from `/health`.
  `/` and `/sw.js` returned the CSP, `nosniff`, `DENY`, no-referrer,
  restrictive permissions policy, and `no-cache` headers expected by the
  service worker/update policy.
- Docker/Podman are not installed in this worker, so the container was not
  built locally. The multi-stage Dockerfile's frontend and locked release
  stages were independently run above; cloud deployment uses the factory ACR
  build with `BUILD_SHA`, `GIT_SHA`, and `SOURCE_COMMIT` arguments.

## Deployment

The container deployment is performed after this repair commit with
`/opt/fleet/lib/deploy-container.sh remote-scene-cues /work/repo Dockerfile 8080`.
It builds the committed source in ACR, passes the committed SHA as `BUILD_SHA`,
and serves the same Rust/Axum + SQLite artifact on port 8080. Record the
resulting revision and live `/health` identity with the deployment log.

## Known follow-up

The researched product is freemium, but Cue Book is intentionally not offered
until the factory registers a real Sociobot product and return URL. At that
point, reintroduce the purchase/restore flow together with server-side license
verification before allowing more than 12 cues; do not re-enable a client-only
limit or an unverified checkout link.
