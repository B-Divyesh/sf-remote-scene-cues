# Independent verification — FAIL

Verified 2026-08-28 against candidate commit
`3f2afc99b55eda2b2400e5f32d648f8d1acb7be3` and
`https://remote-scene-cues.sociobot.in`.

## Disposition

**FAIL.** The free, two-device rehearsal flow is working, but the deployed
product's advertised paid tier is neither purchasable nor enforced. This is a
release-blocking failure for the stated freemium product, not a reason to
discount the successful core-flow evidence below.

## Defects

### P1 — advertised Cue Book checkout is unavailable and its limit is bypassable

1. `GET https://api.sociobot.in/api/v1/products/remote-scene-cues/checkout`
   returned HTTP `404` with JSON `{"error":"enabled factory product","status":404}`.
   The live page presents this endpoint as the `$24` **Buy Cue Book** action,
   so a buyer cannot complete the promised purchase.
2. With no license or client-side unlock, a direct live `POST /api/rooms` with
   13 valid cues returned `201 Created`; the temporary room was deleted with
   `204`. The server only rejects more than 50 cues. Thus the advertised
   free-tier 12-cue limit is only client-side and the 50-cue paid benefit is
   not enforceable.

Required remediation: register and validate the Sociobot product/return URL
before exposing checkout, and enforce the entitlement/limit at the server
boundary (or remove the paid claim until that exists). Re-verify both a real
checkout/restore and an unlicensed 13-cue API request.

### P2 — deployment build identity is not available

`GET /health` returned `200 {"status":"ok","build_sha":"unknown"}`.
The deployed HTML, JS, and CSS are byte-identical to this candidate's fresh
production build, so frontend deployment matching is established; the backend
image cannot be conclusively tied to the candidate from its health identity.
Build with `BUILD_SHA=3f2afc99b55eda2b2400e5f32d648f8d1acb7be3` (or equivalent
immutable deployment identity) and recheck `/health`.

### P3 — documented arrow-key cue selection is absent

The design contract says arrows choose a cue, but the sole window keyboard
handler implements only `G`. Normal Tab/Enter/Space operation is available;
this is a mismatch with the documented interaction grammar rather than a
keyboard trap.

## Fresh local verification

The checkout began clean at the candidate: `git status --porcelain` was empty
and `HEAD` was `3f2afc99b55eda2b2400e5f32d648f8d1acb7be3`.

- `npm ci` completed successfully: 165 packages, zero npm audit
  vulnerabilities.
- `npm run build` completed and produced `dist/`.
  The initial application chunk is 65,027 bytes (25,027 gzip), lazy QR chunk
  25,881 bytes (10,171 gzip), CSS 13,706 bytes (3,660 gzip), and the
  self-hosted font 37,364 bytes: all within the relevant 200 KB JS, 50 KB CSS,
  and 120 KB font budgets. The mobile AVIF hero is 26,645 bytes; the largest
  delivered hero asset is 272,568 bytes, below the 300 KB image budget.
- `npm test` passed: 3 Vitest tests and 3 Rust tests.
- `cargo build --release --locked`, `cargo clippy --all-targets --all-features
  -- -D warnings`, and `cargo fmt --check` passed. No repository lint or
  TypeScript-check script exists; Clippy is the available supplemental lint.
- `npm run test:e2e` passed. Playwright's `test-results/.last-run.json` records
  `status: "passed"`; it exercises both configured Chromium projects (desktop
  and 390 x 844 mobile) for the host/controller flow and landing-page reachability.
- Docker/Podman were not installed in the verifier image, so the Dockerfile
  itself could not be invoked. Its constituent frontend build and locked Rust
  release build did pass.

## Live core-flow and boundary verification

On the deployed service, I created a real ten-cue room, joined from a separate
controller identity, obtained explicit host approval (`204`), and issued ten
concurrent controller `POST /fire` requests. All ten returned `200`; the host
snapshot contained exactly ten events with ordered sequences `1..10`, current
index `9`, and all ten cue names. The QA room was deleted (`204`) afterward.

The following invalid inputs were independently rejected by the live API with
clear recovery-oriented errors:

- blank show name — `400 Show name must be 1–80 characters`;
- HTTP webhook — `400 Webhook URL must use HTTPS`;
- `https://127.0.0.1/...` webhook — `400 Webhook cannot target a private address`;
- 51 cues — `400 Add between 1 and 50 cues`.

This confirms the researched smallest useful product's free rehearsal flow:
private join secret, explicit host approval, server-serialized ordered GO
receipts, and no IP configuration. It also confirms public-webhook input
validation and room deletion behavior.

## Browser, accessibility, privacy, PWA, and response checks

- Fresh Chromium checks against the live page at 1440 x 1000 and 390 x 844
  observed zero page/console errors and runtime requests only to
  `https://remote-scene-cues.sociobot.in`. No analytics, CDN font, or tracker
  request was observed. Source and browser storage review show room
  credentials in `sessionStorage`; only license/saved sheets use
  `localStorage`.
- Playwright axe on the live landing page found zero serious/critical findings.
  The page has `lang=en`, title, one h1, main landmark, meaningful hero alt
  text, skip link, and the focused skip link had a solid 3px outline plus the
  designed 6px contrast ring. At 390 px the setup field is visible, 350 px
  wide, and 48 px tall. Reduced motion resolves `scroll-behavior` to `auto`.
- The production service worker registered at `/sw.js`, controlled the page
  after reload, and an offline reload still rendered “Advance the scene. Skip
  the lobby.”. `sw.js` is served `no-cache`; hashed assets are immutable.
- `/`, `/privacy`, `/terms`, `/health`, assets, font, and `sw.js` all returned
  the expected security headers: CSP restricting script/style/image to self
  (with only the documented Sociobot billing origins in connect/form),
  `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and restrictive camera/microphone/geolocation
  permissions policy. HTML and app routes are `no-cache`; hashed assets are
  `public, max-age=31536000, immutable`.
- An attempted independent Lighthouse run could not launch its browser as root
  with the supplied Playwright headless-shell executable. It is not reported
  as a score. The concrete bundle/image budgets and live browser checks above
  were completed instead.

## Deployment match

The fresh candidate build's `dist/index.html`, main JS
`assets/index-B-8lo7Y-.js`, and CSS `assets/index-DxciM1lf.css` had SHA-256
hashes identical to the files fetched from the live URL. The live response
also uses the candidate's exact asset names and security/cache policy. See P2
for the remaining backend build-identity gap.

## Reproduction

```sh
npm ci
npm run build
npm test
cargo build --release --locked
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
npm run test:e2e
```
