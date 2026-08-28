# Independent product verification 3 — PASS

Verified on 2026-08-28 for work order `remote-scene-cues-verify-3`.

- Candidate and checkout HEAD: `7cc8c6a061f1022afec90de4455d107d43e83960`
- Live URL: <https://remote-scene-cues.sociobot.in>
- Acceptance sources: `.factory/brief.json`, `.factory/design.md`, `AGENTS.md`, and the injected work order.
- Starting worktree: clean.

## Disposition

**PASS.** Fresh evidence shows that the deployed backend identifies itself as
the candidate and the deployed initial JS/CSS exactly match a fresh local
production build. The complete two-device rehearsal job works: a phone must be
explicitly approved, then ten concurrent GO requests are serialized into the
ten expected ordered receipts. Temporary QA rooms were explicitly deleted.

No P0, P1, P2, or P3 product defects were found in this verification.

## Clean local gates

`npm ci` installed 170 packages and reported zero npm audit vulnerabilities.
The following all passed from the clean candidate:

```sh
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
npm run test:e2e
BUILD_SHA=7cc8c6a061f1022afec90de4455d107d43e83960 cargo build --locked --release
```

- `npm run check`: zero Svelte/TypeScript errors and warnings.
- `npm test`: 3 Vitest tests and 8 Rust router/API tests passed.
- `npm run test:e2e`: 12/12 Playwright tests passed across the configured
  desktop Chromium and 390 × 844 mobile Chromium projects. This includes real
  host → join → approval → GO, keyboard arrows/Enter/Space/G, axe checks,
  error recovery, service-worker update, and offline reload.
- The exact Vite production build produced `dist/`. Initial JS is 61,084 bytes
  (23.77 KB gzip), the lazy QR chunk is 25,881 bytes (10.17 KB gzip), CSS is
  13,787 bytes (3.67 KB gzip), self-hosted font 37,364 bytes, and mobile AVIF
  26,645 bytes: all within the stated budgets.
- The locked release binary was also started from a fresh temporary directory
  with only `PATH` and `PORT=8092`. It logged configuration provenance without
  values, returned the candidate SHA, created a room using the default SQLite
  database, and retained that room across a graceful stop/restart.
- A fresh local 100-request `/health` smoke at concurrency 100 returned 100
  HTTP 200 responses.

The verifier image has neither `docker` nor `podman`, so it could not invoke
the multi-stage Dockerfile itself (`docker: command not found`). This is an
environment limitation, not a deployed-product failure: its exact frontend
build and locked release build were run, and the live service below proves the
candidate container is serving.

## Live functional and backend evidence

`GET /health` returned the exact candidate identity:

```json
{"status":"ok","build_sha":"7cc8c6a061f1022afec90de4455d107d43e83960"}
```

I created a new live room with the exact free boundary of 12 cues. Its private
join URL used the canonical HTTPS deployment origin (secret redacted). A
separate controller identity joined, could not fire while pending (`403 This
controller is still waiting for host approval`), and was approved by the host
(`204`). Ten simultaneous controller `POST /fire` requests then returned ten
`200` results. The final authenticated snapshot had `current_index: 9`, ten
events, and unique ordered sequences `1..10` for cue names `Cue 01` through
`Cue 10`. The host deletion returned `204`; a fresh snapshot thereafter
returned `404`.

Boundary and recovery requests were independently checked live:

- blank show title: `400 Show name must be 1–80 characters`;
- 13 cues: `400 Add between 1 and 12 cues`;
- `http:` webhook: `400 Webhook URL must use HTTPS`;
- `https://127.0.0.1/...` webhook: `400 Webhook cannot target a private address`;
- short webhook secret: `400 Webhook signing secret must be at least 12 characters`.

This satisfies the brief's smallest useful product: a short-lived private
room, host-approved phone controller, deterministic shared cue state, ordered
receipts, clear invalid-input recovery, and log/room deletion.

## Deployment match, privacy, and browser quality

- Fresh local and live SHA-256 values matched for both deployed initial assets:
  `assets/index-wt-Sg26m.js` =
  `9ccfd4c4ac68c03dcc1bf89564dbe00827a5912b4b9c90207455b4f3ef793124`;
  `assets/index-BDGbChvu.css` =
  `bb85f584e94c7908f16aefcb76c4db2b45faa206cc099b230e6e23b615a8da21`.
- Fresh live desktop and 390 px controller runs had zero page/console errors;
  Playwright axe found zero serious/critical violations on landing and approved
  controller states. The page has a title, `lang=en`, one h1, a main landmark,
  labelled inputs, descriptive hero alt text, and a keyboard-visible skip-link
  focus ring (3 px outline plus 6 px ring). The mobile GO target measured
  350 × 379.8 px. Keyboard activation moved the skip link to `#main`.
- Under reduced motion, the controller's computed transition duration was
  `0s`. The PWA suite confirmed service-worker update/control and an offline
  landing-page reload.
- A fresh browser context had no cookies, local storage, or session storage on
  the landing page. Runtime landing requests were same-origin only; there are
  no analytics, tracking, third-party fonts, or runtime CDNs. `/privacy` and
  `/terms` rendered normally.
- HTTPS responses provide CSP self-only script/style/connect/form policy,
  `nosniff`, `DENY`, `no-referrer`, HSTS, and disabled camera/microphone/
  geolocation. API responses are `private, no-store`; hashed JS/CSS are
  one-year immutable; stable pages/assets are `no-cache`.

Mobile Lighthouse 12.8.2 against the live origin scored **98 performance** and
**100 accessibility**: LCP 1.897 s, TBT/max potential FID 30.9 ms, CLS 0,
and total transfer 75,996 bytes.

## Reproduce

```sh
npm ci
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
npm run test:e2e
BUILD_SHA=7cc8c6a061f1022afec90de4455d107d43e83960 cargo build --locked --release
```
