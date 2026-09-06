# Scene Cues verification handoff

## Verification 4 — 2026-09-06

Independent QA reviewed implementation `c4d17db` at the live product. The
documentation checkout and live build identity are `aee5700`; its only change
from the implementation is this handoff file. The live JS/CSS byte-match the
candidate build.

**Verdict: FAIL — 4 findings and 7 untested public claims.** The core two-device
flow, one-click isolated sample, all 13 declared claim commands, 28 Playwright
tests, local quality gates, live restart persistence, tenant isolation, rate
limits, webhook delivery, security headers, Axe, and Lighthouse passed. The
remaining findings are incomplete public claim coverage, claim tests outside
the required demo path plus a duplicate tag, sub-44 px targets on demo/legal/
404 routes, and an avoidable failed workspace request during offline demo
reload.

Full evidence and re-test instructions are in
`.factory/verification-4.md`. No product code or deployment was changed by the
verifier. The active product revision was restarted once to verify durable
SQLite state; the room and its receipts survived. All deliberately created QA
rooms and the disposable webhook receiver were deleted.

---

# Scene Cues repair handoff

Completed 2026-09-06 for `remote-scene-cues-repair-3`.

## Result

Scene Cues is a phone-friendly cue remote for small escape-room, live-game,
and interactive-theatre teams rehearsing scene changes. The first action is
**Try it with sample data**, which opens a populated ten-cue rehearsal room.

- Deployed implementation SHA: `c4d17db`.
- Documentation evidence SHA: `1e0d59fcdd680a9e4297609a6f14f590ab3ffd46`.
- Deployed image: `sf-remote-scene-cues@sha256:1e1e2dad013e4dc662b2e56395104ccce040f1cdcb7118c0f1847d03a4bbfff3`.
- Live health: `https://remote-scene-cues.sociobot.in/health` returns build
  SHA `c4d17db`.
- Live revision: `sf-remote-scene-cues--0000015`.
- Runtime storage: one replica (`minReplicas: 1`, `maxReplicas: 1`) with the
  `sf-remote-scene-cues-data` Azure File volume mounted at `/data`.

The documentation evidence commit follows the deployed implementation and is
not redeployed.

## What changed

- Fixed unreliable room reads at the cause: the product now deploys as exactly
  one replica with persistent SQLite state on `/data`.
- Made SQLite work on the factory's CIFS Azure File mount: a single connection,
  migration retry, and SQLite's `unix-dotfile` VFS are used for the durable
  `scene-cues-v3.db`. Tests cover both a transient lock recovery and creation
  through that VFS. The earlier empty bootstrap files were left untouched.
- Added `/demo`: a separate `demo:scene-cues:sample-v1` browser storage
  namespace plus an in-memory, 24-hour demo workspace endpoint. It has the
  Lantern Room ten-cue sample, controller approval, receipts, CSV download,
  reset, and a persistent “Demo — sample data, nothing is saved” banner.
- Added 13 tested public claims in `.factory/claims.json` and documented the
  sandbox in `.factory/demo.md`.
- Added real route metadata, canonical and social metadata, sitemap, robots,
  a styled HTTP 404 page, route-specific titles, legal-page titles, and
  generated product-owned social/touch assets.
- Rewrote the landing copy in plain words and added the copy audit. The mobile
  first screen now shows the job, audience, sample action, and outcome before
  scrolling; a browser regression test enforces this.
- Added HTTP 429 + `Retry-After` coverage, room isolation/expiry/ordered-cue
  regression coverage, and demo isolation/privacy browser checks.

## Verification

From a fresh clone of `c4d17db` at
`/tmp/scene-cues-release-S3KfK8`:

- `npm ci` passed.
- `npm run check` passed.
- `npm test` passed: 3 Vitest tests and 16 Rust tests.
- `cargo clippy --all-targets -- -D warnings` passed.
- `npm run build` passed. Initial JS is 25.88 KB gzip; CSS is 4.16 KB gzip.
- Every one of the 13 exact commands declared in `.factory/claims.json`
  passed. The ten Playwright claim commands each passed in desktop and phone
  projects; the three Rust claim commands passed.

Additional browser checks:

- Full 28-test Playwright suite passed before the CSS-only final adjustment;
  the final first-screen regression passed in both projects afterward.
- Fresh desktop and iPhone contexts confirmed the job, audience, and
  “Try it with sample data” action all appear before scrolling.
- Fresh live `/demo` confirmed the sample label, three seeded receipts,
  approval, a new receipt, reset, and no request to `/api/rooms`.
- Live `verify-url.sh` passed: 200, no console errors, title/lang/main/alt
  checks, and one h1.
- Playwright Axe found zero serious or critical violations on `/`, `/demo`,
  `/privacy`, and `/terms`.
- Live route checks passed for page titles, sitemap entries, and the designed
  HTTP 404 response.
- Lighthouse mobile: Performance **100**, Accessibility **100**, LCP **1.38 s**,
  CLS **0**. Evidence is `/work/.evidence/live-repair/lighthouse.json`.

Live backend exercise after deployment and an explicit revision restart:

- 8 separately timed valid room reads all returned 200.
- A host token for another room returned 404 (tenant/room isolation).
- A pending phone was refused, then host-approved.
- That phone sent ten cues in order, each exactly once.
- The room and all ten receipts survived restart of revision 15.
- Requests past the allowance returned 429 with `Retry-After`.
- Both QA rooms were deleted after the check.

## Run and deploy

```sh
npm ci
npm run check
npm test
cargo clippy --all-targets -- -D warnings
npm run build
npm run test:e2e
PORT=8080 cargo run
```

The container starts with only `PORT` required. On the fleet it keeps its
SQLite file under `/data`; keep the one-replica constraint when deploying.

## Known limits and next step

The free single-room rehearsal core is complete. The researched managed tier
is not advertised because no billing product has been registered with the
Sociobot billing operator. Register the real offer and its exact terms before
adding a buy link or paid-feature metadata; no price, provider credential, or
mock checkout has been invented.
