# Rehearsal cue control review — FAIL

Reviewed 2026-09-06 for work order `remote-scene-cues-review-1`.

- Live URL: <https://remote-scene-cues.sociobot.in>
- Implementation reviewed: `a6d08f292e1f1071d86ad5746228a279104e605c` (`fix: recognize Vite asset hashes`), the last product-code commit.
- Documentation checkout SHA: `0c728b8ff0514af792cf9b2deaf734eddf86000f`.
- Live `/health` build SHA: `7cc8c6a061f1022afec90de4455d107d43e83960`, a report-only commit between the implementation and checkout SHAs. Fresh `index.html`, initial JS, and CSS from the implementation checkout byte-match live.

## Verdict

**FAIL.** There are 5 findings, including 2 P1 findings, and 12 untested public claims. This is not a product PASS.

## Job, audience, and first action

The job is to let a rehearsal host advance ordered scene cues from approved phones. The intended audience is small live-game, escape-room, and interactive-theatre teams. Before scrolling, the live desktop and 390 px phone pages present **Open a rehearsal room** as the first action. They do not present the required **Try it with sample data** action.

Fresh browser evidence was captured at `/work/.evidence/live-desktop.png` and `/work/.evidence/live-phone.png`. The product has a distinct, legible broadsheet visual system, but the required try-out path is absent.

## Findings

### P1 — live rooms alternate between present and missing

A new live room was created, then read eight times at 1.1-second intervals with the same valid host token. The results were exactly:

```text
404, 200, 404, 200, 404, 200, 404, 200
```

The first delete returned `404`; the retry returned `204`. A second temporary room showed the same `404` then `204` cleanup pattern. This is the same non-shared SQLite replica behavior reported as P1 in `verification-2.md`. It makes host approval, a controller's refresh, GO delivery, receipts, and cleanup depend on which live instance receives the request. The core two-device rehearsal job is therefore not reliable.

Required repair: deploy exactly one replica while using product-local SQLite on `/data`, or replace it with shared transactional state and a shared event mechanism. Re-test independent host and controller connections without retries.

### P1 — no one-click demo sandbox

The landing page has no **Try it with sample data** control. `/demo` returns the ordinary live room-creation page, with no seeded sample, isolated storage/tenant, persistent `Demo — sample data, nothing is saved` label, Reset demo, or Start for real action. `.factory/demo.md` is absent. The normal form creates real server rooms, so it cannot serve as the required sandbox.

Required repair: add a first-screen one-click sample rehearsal in a separate demo namespace or ephemeral demo tenant, document it, label it persistently, and make reset/exit discard all demo data.

### P2 — claims inventory and claim tests are missing

`.factory/claims.json` is absent and the test suite contains no `@claim:` tests. The clean checkout therefore declares no claim commands to run, while visitor-facing claims remain untested in the required demo sandbox. I counted 12 untested public claims: no account; nothing to install; eight-hour expiry; private QR; host approval; 12-cue limit; once-only ordered GO; signed webhook receipts; CSV export; manual log deletion; offline reload; and keyboard `G` operation.

Some were manually exercised below, but that does not replace one observable clean-demo test per claim. Add the manifest and tagged tests, or remove claims that cannot be tested.

### P2 — route metadata, discovery, and the error route are incomplete

`/privacy` and `/terms` retain the home title instead of route titles. The live document has no canonical link, Open Graph/Twitter metadata, or apple-touch icon. `/sitemap.xml`, `/404`, and an arbitrary unknown route all return the landing page with HTTP 200; there is no designed HTTP 404 page or useful sitemap. These are required site-structure elements and make deep links and search results misleading.

Required repair: set route-specific titles, add the required metadata and discovery files, and return a designed 404 response for unknown routes.

### P2 — required plain wording is not followed

The first screen and legal pages use mood/metaphor copy that the contract explicitly forbids: `Skip the lobby`, `One cue light. One shared truth.`, `Make the call sheet`, `The running order`, `Short rooms. Short memory.`, and `Ask to join the cue desk.` The product job is understandable from the supporting paragraph, but not stated plainly enough by the headline alone. `.factory/copy-audit.md` is also absent.

Required repair: replace these lines with direct task, state, and action wording, keep the job in the headline, and add the required copy audit.

## What passed

- Fresh desktop and phone browsers loaded without page or console errors. `verify-url.sh` passed: title, `lang=en`, one `h1`, `main`, image alt text, and labelled buttons were present. Playwright Axe found no serious or critical issue on the landing page. The standalone `npx @axe-core/cli` command could not start Chrome in this root container; this is an environment limitation, not an Axe pass.
- The visual layout is responsive and readable at 1440 x 1000 and 390 x 844. Reduced motion sets scroll behavior to `auto`.
- On an individual live instance, a fresh room accepted a phone join, rejected a pending GO with `403`, approved the controller with `204`, and serialized ten simultaneous GO requests into sequences 1 through 10. Clearing the test log returned `204`; deletion eventually returned `204` and subsequent read returned `404`.
- Invalid live requests returned clear `400` errors for a blank show name and 13 cues. A 42-request allowance test returned 34 `429` responses with `Retry-After: 1`.
- A cross-room-token attempt returned `404`; however the alternating replica behavior prevents this from being conclusive live tenant-isolation evidence.
- `/health` is live, API responses have `Cache-Control: private, no-store`, HTTPS sends CSP, HSTS, nosniff, DENY framing, no-referrer, and permissions-policy headers. The corrected join URL uses the canonical HTTPS product origin.
- Service-worker control and offline landing reload worked in a fresh live browser context. Landing requests were same-origin only; no analytics, third-party scripts, or CDN fonts were observed. Privacy and Terms render.
- A clean local restart smoke using only `PATH` and `PORT` returned health, retained a room across graceful restart, deleted it afterward, and logged configuration provenance without values.

## Clean checkout commands

The following passed from the clean checkout: `npm ci`, `npm run check`, `npm test`, `npm run build`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `npm run test:e2e` (12/12), and `cargo build --locked --release`. Docker and Podman are not installed in this worker, so the README's Docker build command could not be exercised.

The existing browser tests cover useful local host/controller, keyboard, offline, and Axe paths, but they do not run against `/demo`, do not use a demo tenant, and are not tagged claim tests.

## Earlier review disposition

`verification.md`'s unavailable checkout/free-tier issue is resolved by removal of the paid checkout claim; the current live page has no buy link. Its build-identity and arrow-key findings are resolved. `verification-2.md`'s public join origin, API no-store policy, 44 px target tests, HSTS, recovery wording, stable-asset caching, and startup-provenance concerns are resolved in the reviewed source/live responses. Its replica-state P1 has regressed and is reproduced above. `verification-3.md` recorded a PASS, but its assertion that the live two-device path is reliable is no longer true on the currently served service.

## Re-test order

1. Fix shared/persistent live room state and prove independent host/controller reads, approval, ten GO events, cleanup, and restart persistence without retries.
2. Add the required demo sandbox and claims manifest/tests.
3. Repair route metadata, sitemap, 404 behavior, and plain wording.
