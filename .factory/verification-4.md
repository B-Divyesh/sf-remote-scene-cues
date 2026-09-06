# Shared rehearsal cue control verification — FAIL

Verified 2026-09-06 for work order `remote-scene-cues-verify-4`.

- Live URL: <https://remote-scene-cues.sociobot.in>
- Implementation candidate: `c4d17dba672eac1df2a11c51a5aa97747fb1833f`
- Documentation base: `aee570019087e5e0b549afc73e2825a8fd0a6d1d`
- Verification report commit: `8cb364f816c85fd7176f879c44b2f02767a167c0`
- Live `/health` build: `aee570019087e5e0b549afc73e2825a8fd0a6d1d`
- Starting repository worktree: clean

The live build identity is the later documentation commit. Its only difference
from the implementation candidate is `.factory/handoff.md`. Fresh candidate
JS and CSS byte-match the live assets, so the reviewed implementation is
`c4d17db`.

## Verdict

**FAIL.** There are 4 findings and 7 untested public claims. All declared
commands pass and the core live rehearsal flow works, but the contract permits
a PASS only with zero findings and zero untested claims.

## Job, audience, and first action

The job is to run shared rehearsal cues from approved phones. The audience is
small escape-room, live-game, and theatre teams rehearsing scene changes. In a
fresh desktop browser and a fresh 390 × 844 phone browser, the first action
before scrolling is **Try it with sample data**. Its adjacent text says it opens
a ten-cue sample room.

## Findings

### P2 — seven public claims are absent from, or not proved by, the claims manifest

`.factory/claims.json` declares 13 claims, and every declared command passed.
It does not inventory and test these separate statements that visitors can
rely on:

1. Ending a room immediately deletes controller access and the server-side cue
   log (`/host`, `/privacy`, README).
2. Room access tokens use session storage and disappear when the browser
   session closes (`/privacy`).
3. Real room data is not stored in local storage (`/privacy`).
4. The product uses no analytics, advertising or tracking cookies, tracking
   pixels, third-party scripts, or CDN fonts (`/privacy`, README). The existing
   `privacy-same-origin` claim only observes requests during the demo and does
   not check cookies, same-origin analytics, or the real-room flow.
5. Webhook delivery is limited to a configured public HTTPS address and rejects
   private and local-network destinations (`/privacy`, setup help).
6. A failed webhook never changes cue order (live host room rail).
7. An existing app can receive the optional webhook (README). The declared
   `webhook-receipt` command calls the signature helper with a fixed byte string;
   it does not exercise delivery.

Independent live checks did confirm room deletion, session-only room keys,
empty local storage during a real flow, rejection of HTTP/private webhook
targets, one real webhook delivery, and its exact HMAC signature. That evidence
shows no observed falsehood, but it does not satisfy the required permanent
claim inventory and one-command regression coverage.

Required repair: add one manifest entry and one observable sandbox test for
each statement, or remove/narrow the public copy. The webhook delivery test
should use a recorded local receiver rather than a paid or live dependency.

### P2 — declared claim tests do not have one demo-only sandbox path

- `private-join` and `approved-phone` open `/` and POST to `/api/rooms`; they do
  not enter `/demo` or use the ephemeral demo workspace required for claim
  verification.
- `demo-sandbox` appears on two test functions: the Playwright test and
  `demo_workspace_is_ephemeral_and_never_creates_a_real_room`. The contract
  requires exactly one test tagged for each claim. Its manifest command runs
  only the Playwright test.
- The `/demo` UI stores and changes its working state in local storage. The
  provisioned backend demo workspace contains only an id and expiry and is not
  used for the join, approval, or GO actions, so it cannot replace the two
  real-room claim paths above.

The clean checkout tests did not touch deployed user data, and the live sample
itself made no `/api/rooms` request. This is a claim-sandbox contract defect,
not evidence of live data contamination.

Required repair: make the ephemeral demo workspace exercise the private join
and approval flow, point both claims at that sandbox, and leave exactly one
tagged test per claim.

### P2 — several phone touch targets are below 44 px

At a 390 px viewport, measured visible targets include:

- `/demo`: **Reset demo** and **Start for real** are 40 px high.
- `/privacy`: the privacy email link is 21 px high.
- the designed 404 page: the header home link is 19 px high.

The main page's 24 px checkbox has a larger associated label and is not counted
as a separate failure. Main navigation, footer links, forms, and cue controls
meet the target size. All keyboard-focused controls show the designed 3 px
outline and 6 px outer ring.

Required repair: give every affected link/button at least a 44 × 44 CSS-pixel
hit area, then add route-specific regression coverage.

### P3 — offline demo reload makes an avoidable failed request

After service-worker control, a live offline reload of `/demo` rendered the
sample and persistent demo label correctly, but the page still attempted
`POST /api/demo/workspaces`. Chromium logged
`net::ERR_INTERNET_DISCONNECTED`. The application catches the fetch failure,
so the promised offline sample works, but this violates the no-console-errors
load gate and needlessly attempts network work while `navigator.onLine` is
false.

Required repair: skip workspace provisioning while offline and provision after
the next online event. Add a console-error assertion to the offline claim.

## Declared claims

All 13 exact commands in `.factory/claims.json` passed from a fresh checkout.
The ten Playwright commands passed in both desktop and phone projects; the
three Rust commands passed. Tag counting found one tag each except
`demo-sandbox`, which has two as described above.

| Claim | Result |
| --- | --- |
| private-join | PASS |
| demo-sandbox | PASS; duplicate tag finding |
| approved-phone | PASS |
| ordered-go | PASS |
| demo-approval | PASS |
| cue-limit | PASS |
| csv-export | PASS |
| log-deletion | PASS |
| keyboard-go | PASS |
| offline-reload | PASS; console finding |
| privacy-same-origin | PASS; incomplete public coverage finding |
| room-expiry | PASS |
| webhook-receipt | PASS; delivery claim remains untested |

## Demo and browser evidence

- `/demo` opened in one click with The Lantern Room, ten cues, three receipts,
  an approved controller, and a pending Alex controller.
- GO was disabled before approval. Approval and GO added receipt 4 and advanced
  to “Show rules camera.” Reload kept the sample state and banner. Reset restored
  three receipts. Start for real removed the `demo:` key and returned home.
- The full demo trace made no `/api/rooms` request and no cross-origin request.
- A fresh live desktop host and separate 390 px phone completed join, pending,
  approval, Space-key GO, `G`-key GO, and receipt display. The private secret
  was removed from the controller address. The QA room was deleted with 204.
- Online `/`, `/demo`, `/privacy`, `/terms`, host, and controller states had no
  page or console errors. A deliberate unknown route returned the designed
  HTTP 404 with a return link; its browser 404 resource message is expected and
  is not a defect.
- All landing links returned 200, apart from the intentional `mailto:` action.
  Route titles, one h1, main landmarks, canonical/social metadata, sitemap,
  robots, favicon, and 1200 × 630 social image are present.
- Reduced motion resolves to `scroll-behavior: auto` and zero transition and
  animation duration. The service worker updated, controlled a fresh context,
  and served `/demo` after offline reload.

## Backend evidence

- Eight separately timed reads of one live room all returned 200.
- A host token from a second room returned 404 against the first room.
- A pending controller received 403, approval returned 204, and ten GO actions
  produced exactly sequences 1–10 and cues 01–10.
- Three receipts and the current cue survived a restart of active revision
  `sf-remote-scene-cues--0000016`; the remaining seven cues then completed.
  Both QA rooms were deleted with 204.
- A 55-request burst produced 25 responses with HTTP 429. Every 429 included
  `Retry-After`; `/health` remained healthy.
- Blank names, 13 cues, HTTP webhook URLs, and private-IP webhook URLs returned
  clear 400 errors. A real disposable HTTPS webhook received exactly one
  `cue.fired` event; its HMAC-SHA256 matched and delivery state became
  `delivered`. The room and disposable receiver were deleted.
- A locked release binary started with only `PATH` and `PORT`, logged supplied
  versus defaulted configuration without values, retained a room across a
  graceful restart, and deleted it afterward.
- API responses use `private, no-store`; HTTPS provides HSTS, CSP, nosniff,
  frame denial, no-referrer, and restrictive permissions policy. Only hashed
  JS/CSS use immutable caching.

## Clean checkout and quality gates

The fresh checkout was `/tmp/scene-cues-verify4-jkrfFv` at `c4d17db`.

- `npm ci`: passed; 170 packages, zero audit vulnerabilities.
- `npm run check`: passed with zero errors and warnings.
- `npm test`: passed; 3 Vitest and 16 Rust tests.
- `npm run build`: passed and produced `dist/`.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `BUILD_SHA=c4d17db... cargo build --locked --release`: passed.
- `npm run test:e2e`: passed 28/28 across desktop and phone.
- `verify-url.sh`: passed with 200, title, `lang=en`, one h1, main, image alt,
  labelled buttons, and no online console errors.
- Playwright Axe reported no serious or critical violations on home, demo,
  legal, 404, host, and controller states. The standalone Axe CLI could not
  start Chrome in this root container; the repository-pinned Playwright Axe
  integration completed the required equivalent checks.
- Docker and Podman are unavailable in the worker. The frontend build and
  locked release binary were exercised directly.

Build output is 25.88 KB gzip initial JS, 10.17 KB gzip lazy QR JS, and
4.16 KB gzip CSS. Lighthouse mobile scored 100 performance, 100 accessibility,
100 best practices, and 100 SEO, with 1.35 s LCP, 84 ms total blocking time,
and CLS 0.

Evidence is under `/work/.evidence/verification-4/`. Screenshots containing a
temporary private join QR or secret were redacted; no credential is recorded.

## Earlier finding disposition

- The original unavailable paid checkout is resolved by removing the offer.
  The managed tier remains unadvertised pending real billing registration.
- Build identity, keyboard arrow selection, production join origin, API
  no-store policy, HSTS, recovery copy, stable-asset caching, and startup
  provenance are resolved.
- The replica-local SQLite failure is resolved: the live app has one active
  replica, repeated reads are stable, and state survives revision restart.
- The missing one-click sample, route titles, metadata, sitemap, designed 404,
  plain wording, and copy audit are resolved.
- The earlier 40 px masthead/footer target issue is resolved, but the new demo
  controls and the legal/404 links listed above still miss the same baseline.
- The earlier missing claim inventory is improved to 13 passing commands but
  remains incomplete as detailed in the two claims findings.

## Re-test order

1. Complete the public claim inventory and make all claim tests use one isolated
   demo path with exactly one tag per claim.
2. Increase the listed phone touch targets to at least 44 px.
3. Prevent demo workspace provisioning while offline and assert a clean console.
4. Re-run all 13 claim commands, the full suite, live demo isolation, Axe, and
   the offline reload before declaring PASS.
