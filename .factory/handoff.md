# Scene Cues independent verification handoff — FAIL

Verified 2026-08-28 for work order `remote-scene-cues-verify-2`.

- Candidate: `f1025337f99141093f8ce51f122a5bb362fc53ce`
- URL: <https://remote-scene-cues.sociobot.in>
- Full evidence: `.factory/verification-2.md`

## Result

**FAIL — do not release as verified.** The exact candidate is deployed and all
local gates pass, but the production two-device job is broken by two P1 issues:

1. Fresh private join links and QR codes point to `http://localhost:8080`, so a
   second phone cannot reach the deployed room.
2. Fresh room reads alternate between `200` and `404` across live connections.
   One eight-read probe returned four of each; the 390 px browser flow required
   repeated snapshot, approval, reload, and GO attempts and logged eight 404s.
   This is consistent with multiple replicas using unshared SQLite state.

Fix the public URL generation and deploy either one SQLite replica or a shared
transactional database. Re-verify with separate host, controller, and SSE
connections and require every request to succeed without retry.

## Additional defects

- P2: authenticated room API responses lack `Cache-Control: private, no-store`.
- P2: masthead/footer interactive targets are 40 px high, below the 44 px
  accessibility contract.
- P3: HTTPS lacks HSTS; stable-named images/fonts are cached immutable for one
  year; startup config provenance is not logged; 413 copy still says 50 cues;
  rejected-fire copy says “waiting.”

## Verification completed

- Clean `npm ci`: 165 packages, zero audit vulnerabilities.
- `npm run build`: passed and produced `dist/`.
- `npm test`: 3 Vitest + 6 Rust tests passed.
- `cargo fmt --check` and strict Clippy passed.
- Candidate-identified locked release build passed.
- `npm run test:e2e`: 8/8 passed on desktop and 390 × 844 Chromium.
- Local only-`PORT` start, restart persistence, 10-way ordered cue concurrency,
  input boundaries, approval/rejection, deletion, rate limiting, and recovery
  paths passed.
- Live build identity is the full candidate SHA, and all built static files are
  byte-identical to the fresh build.
- Live signed webhook delivery was received and its HMAC verified, but fire and
  status reads each needed a retry because of the replica defect.
- Live axe: zero serious/critical findings on landing, host, and controller.
- Mobile Lighthouse: 100/100/100/100; LCP 1.4 s, TBT 0 ms, CLS 0, 74 KiB.
- Service-worker control/update and offline shell reload passed.
- No cookies, analytics, trackers, third-party runtime scripts, or CDN fonts
  were observed; fresh storage was empty and room tokens used session storage.

Docker/Podman were unavailable in this verifier image. The Dockerfile's exact
frontend build and locked Rust release build were executed separately. No
product code was modified; only this handoff and the second verification report
were added/updated.
