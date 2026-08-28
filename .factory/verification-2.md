# Independent product verification 2 — FAIL

Verified on 2026-08-28 for work order `remote-scene-cues-verify-2`.

- Candidate: `f1025337f99141093f8ce51f122a5bb362fc53ce`
- Live URL: <https://remote-scene-cues.sociobot.in>
- Starting state: clean checkout, `HEAD` and `origin/main` both at the candidate
- Acceptance source: `.factory/brief.json`, `.factory/design.md`, `AGENTS.md`,
  and the injected verifier work order

## Disposition

**FAIL.** The candidate builds and passes its local suite, and the deployed
static files and `/health` identity match the candidate. The live product does
not complete its core two-device job reliably:

1. Every newly generated QR/private controller link uses
   `http://localhost:8080`, not the deployed origin.
2. A room created on the live service is present on one request and absent on
   the next. Fresh sequential reads of one room alternated four `200` responses
   with four `404` responses. This is consistent with multiple live replicas
   using separate SQLite state.

Both defects independently prevent a normal second phone from joining and
firing ten cues with no manual network configuration and no missed receipt.

## Defects by severity

### P1 — live QR and private join links point to localhost

A fresh production `POST /api/rooms` returned a `join_url` beginning with
`http://localhost:8080`. A room created through the live desktop UI displayed
the same value as both the controller hyperlink and QR source:

```text
http://localhost:8080/join/YY64RJ?secret=<redacted>
```

The browser assertion `href.startsWith("https://remote-scene-cues.sociobot.in")`
was false. On a second phone, `localhost` means that phone, so the advertised
scan-to-join path cannot reach Scene Cues. This violates the researched
no-manual-IP success measure and the requirement not to expose a host-local
address.

Required remediation: generate controller URLs from the canonical public
origin (or a strictly validated forwarded origin) in the factory's default
runtime configuration. Add a deployed smoke test that creates a room and
asserts the returned and rendered join origins are the production HTTPS origin.

### P1 — live replicas do not share room state

For fresh room `FC6KZZ`, eight sequential authenticated reads returned this
exact pattern:

```text
404, 200, 404, 200, 404, 200, 404, 200
```

The `200` bodies contained the correct room and host role; the `404` bodies said
`Room or access token not found`. Deletion likewise returned `404` on one
attempt and `204` on the next. A separate 12-cue boundary room required the
same `[404, 204]` deletion sequence.

The real 390 px controller flow, after manually replacing the broken localhost
origin, needed two host-snapshot attempts, two approval attempts, two reloads
to see the remote, and three GO attempts to receive cue 1. Chromium logged eight
failed `404` resource requests during that flow. A signed webhook fire and its
status read each also required two attempts.

This behavior is consistent with independent replica-local SQLite databases.
It makes controller approval, SSE refresh, GO delivery, deletion, and receipt
history nondeterministic across browser connections. A single sticky connection
can appear healthy: one live ten-request HTTP/2 burst produced sequences 1–10,
which is why the stronger multi-connection and sequential tests were necessary.

Required remediation: run one replica for the SQLite deployment, or move all
room state to a genuinely shared database with transactional cue sequencing.
Then exercise host fetch, controller fetch, SSE, approve, and fire across
separate connections/instances without retries.

### P2 — authenticated API responses are cacheable by default

`GET /api/rooms/:code?token=...` responses contain cue text, controller names,
and receipts but have no `Cache-Control` header. The same is true of API error
responses. HTML and the service worker correctly use `no-cache`, but the
security middleware explicitly skips cache policy for `/api/`.

Required remediation: return `Cache-Control: private, no-store` for all API
responses, especially token-bearing room snapshots and SSE endpoints.

### P2 — minimum 44 px target contract is missed

At both tested widths, the masthead home link measured 40 px high. Desktop
footer links (`Privacy`, `Terms`, and `A Param Factory product`) also measured
40 px high. They are keyboard reachable and have excellent visible focus, but
do not meet the product's explicit 44 × 44 CSS px target baseline.

### P3 — HTTPS response hardening is incomplete

Plain HTTP correctly redirects to HTTPS with `301`, and the application sends
CSP, `nosniff`, `DENY`, `no-referrer`, and restrictive permissions policy.
HTTPS responses do not include `Strict-Transport-Security`, so a first visit is
not protected from an HTTP downgrade. This can be fixed at the platform edge.

### P3 — two recovery messages are stale or inaccurate

- The frontend maps HTTP `413` to “Keep it under 50 short cues,” while the
  authoritative free limit is 12. A 70 KB live body correctly returned `413`.
- A rejected controller's direct fire request returns `403` saying it is
  “still waiting for host approval,” although its state is `rejected`. The
  normal rejected UI hides GO, so this is mainly an API/error-copy issue.

### P3 — immutable caching is applied to stable asset names

All `/assets/` and `/fonts/` responses receive one-year `immutable` caching,
including stable names such as `scene-cues-hero-mobile.avif` and
`league-gothic.ttf`. Future bytes at those URLs would remain stale. Restrict
immutable caching to content-hashed files or version every long-lived asset.

### P3 — mandatory startup configuration provenance is not logged

With only `PORT` supplied, the service started but emitted no startup line
identifying generated/defaulted versus supplied configuration. The only coded
startup event is `Scene Cues listening` with the port, and it is filtered out
at the default log level. This misses the backend runtime contract's required
configuration-provenance line (without secret values).

## Clean local verification

- `npm ci` passed: 165 packages installed and zero audit vulnerabilities.
- `npm run build` passed and produced `dist/`.
- `npm test` passed: 3 Vitest tests and 6 Rust tests.
- `cargo fmt --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `BUILD_SHA=f1025337f99141093f8ce51f122a5bb362fc53ce cargo build
  --locked --release` passed.
- `npm run test:e2e` passed all 8 tests across desktop Chromium and the
  configured 390 × 844 Chromium project.
- No separate TypeScript check or JavaScript lint script exists in the
  repository. Vite did compile the Svelte/TypeScript production input.
- Docker and Podman are not installed in this verifier image, so the Dockerfile
  could not be invoked. Its exact Vite build and locked Rust release build were
  executed separately.

The release binary was started from a temporary directory with an empty
environment except `PORT` and `PATH`. It migrated a default SQLite database,
served successfully, returned the candidate SHA from `/health`, and retained a
room across a graceful stop/restart. This meets the local persistence boundary.

Local API coverage included:

- ten simultaneous approved GO requests: ten `200` responses, ordered unique
  sequences 1–10, current index 9, and ten matching cue names;
- fire before approval: `403`; fire after the final cue: `400`;
- clear logs: `204` and zero remaining events; delete room: `204`, then `404`;
- exact 80/60/100-character title/scene/cue boundaries accepted; 81/61/101,
  control characters, zero cues, and 13 cues rejected;
- malformed JSON rejected and a following valid request succeeded;
- bad join secret and bad access token rejected;
- controller decline persisted and prevented fire;
- HTTP and localhost/private-IP webhook destinations and a short signing secret
  were rejected in fresh requests; source review also confirmed credential and
  DNS-resolution checks;
- a 100-request local API burst returned 30 `200` and 70 intentional `429`
  responses, after which `/health` remained `200`.

## Live behavior and webhook evidence

The following production features work when requests reach the replica holding
the room:

- 12 cues are accepted; 13 are rejected with `Add between 1 and 12 cues`.
- Blank title, HTTP webhook, and `https://127.0.0.1` webhook inputs return clear
  `400` errors.
- Explicit approval is enforced; a pending controller's fire returns `403`.
- A live ten-cue burst returned ten `200` responses and sequences 1–10, and the
  temporary room/log were deleted.
- A disposable Webhook.site endpoint received exactly one HTTPS `POST`. The
  body contained `type: cue.fired` and sequence 1; recomputing HMAC-SHA256 over
  the exact raw body matched `X-Scene-Cues-Signature`. The stored delivery state
  changed to `delivered`. The room and disposable webhook endpoint were deleted.

These successes do not mitigate the two P1 defects: they required a connection
that happened to reach the correct replica and, for the browser controller,
manual correction of the join origin plus retries.

## Deployment identity and static match

`GET /health` returned:

```json
{"status":"ok","build_sha":"f1025337f99141093f8ce51f122a5bb362fc53ce"}
```

Fresh local and live `index.html`, initial JS, CSS, lazy QR chunk, service
worker, favicon, robots file, all five hero variants, and the font were compared
byte for byte; every file matched. Representative SHA-256 values:

```text
d275d408fa561dbf807f545a947c4387480d44004815f1457a9f75fba2525ecd  index.html
ad5fc3f35a7d7b870480a6ba8e44294282df618ab406437ad97bdb33191eeee0  index-CmWbGLMB.js
d463992bb284d15e978f7cf484713ef357d48bb56d9542cbc1ece27e1b7be8a2  index-DxciM1lf.css
```

The live candidate match is confirmed for both build identity and static bytes.

## Accessibility, responsive UI, and browser quality

- Playwright axe found zero serious/critical violations on the live landing,
  host, and approved controller states.
- Lighthouse mobile scored 100 accessibility. Its experimental
  `label-content-name-mismatch` diagnostic flagged the wordmark, although the
  repository-pinned axe 4.10.2 rule did not reproduce that finding.
- `lang=en`, descriptive title, one `h1`, one `main`, heading hierarchy, hero
  alt text, skip link, labels, status/alert regions, and native controls are
  present.
- Keyboard creation, controller request, GO, host arrow selection, Enter/Space,
  and `G` were exercised locally. On live host controls, Tab moved through the
  private link, Copy, GO, and cue buttons with a 3 px outline plus 6 px ring.
- At 390 × 844 there was no horizontal overflow. The join input was 342 × 48 px
  and the approved GO target was 350 × 360 px. The private secret was stripped
  from the address after join.
- With reduced motion requested, computed `scroll-behavior` was `auto` and the
  motion-removal CSS applied.
- Landing, privacy, and terms pages produced no page/console errors. The
  corrected live controller flow produced the eight expected 404 console
  errors caused by the replica defect.
- Visual inspection at 1440 × 1000 and 390 × 844 confirmed the documented
  broadsheet system, responsive stacking, legible form, and no clipping.

## Privacy, requests, PWA, and response policy

- Fresh desktop and mobile sessions had no cookies, local storage, analytics,
  trackers, CDN fonts, or third-party runtime scripts. Runtime resource requests
  were all same-origin. Room credentials were confined to session storage.
- The only external page link is the disclosed Param Factory footer link; it
  makes no request unless activated.
- Cross-origin preflight from `https://evil.example` received `405` with no
  access-control allow headers.
- CSP is self-only for scripts, styles, connections, and forms; images allow
  self/data. `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and camera/microphone/geolocation denial are
  present.
- HTML, legal routes, health, and `sw.js` use `no-cache`; hashed JS/CSS use
  one-year immutable caching. See the API and stable-asset defects above.
- The service worker registered, controlled the page after reload, accepted an
  explicit update check, and served the app shell during an offline reload.
  The service worker does not cache API requests.

## Performance and budgets

Fresh build sizes:

- initial JS: 61,061 bytes (23.75 KB gzip)
- lazy QR JS: 25,881 bytes (10.17 KB gzip)
- CSS: 13,706 bytes (3.66 KB gzip)
- font: 37,364 bytes
- mobile AVIF hero: 26,645 bytes
- largest hero variant: 272,568 bytes

All prescribed static budgets pass. Lighthouse 12.8.2 mobile results against
the live URL were:

```text
Performance 100 | Accessibility 100 | Best Practices 100 | SEO 100
FCP 1.4 s | LCP 1.4 s | Speed Index 1.4 s | TBT 0 ms | CLS 0
Initial transfer: 74 KiB across 6 requests
```

Lighthouse did not expose a lab INP value because the audit has no user
interaction; no INP claim is made.

## Contract gaps and known follow-up

The researched monetization is freemium, but this candidate intentionally ships
only the genuinely useful free 12-cue room. It has no purchase/restore UI or
managed retention feature because the earlier Sociobot checkout was unavailable.
That is more honest than a broken purchase path, but the retained-shows/teams/
history tier remains unimplemented.

## Reproduction commands

```sh
npm ci
npm run build
npm test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
BUILD_SHA=f1025337f99141093f8ce51f122a5bb362fc53ce cargo build --locked --release
npm run test:e2e
CHROME_PATH=/opt/pw-browsers/chromium-1208/chrome-linux64/chrome \
  npx --yes lighthouse@12.8.2 https://remote-scene-cues.sociobot.in \
  --chrome-flags='--headless --no-sandbox --disable-dev-shm-usage'
```
