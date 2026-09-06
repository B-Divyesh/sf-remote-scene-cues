# Scene Cues review handoff — FAIL

Reviewed 2026-09-06 for `remote-scene-cues-review-1`. This work changed no
product code. The detailed report is `.factory/review-1.md`.

**Result: FAIL.** The live service again alternates valid room reads between
`404` and `200`, reproducing the earlier non-shared SQLite replica failure.
The required one-click demo sandbox and claims manifest/tests are also absent.

The reviewed implementation is `a6d08f292e1f1071d86ad5746228a279104e605c`;
the documentation checkout is `0c728b8ff0514af792cf9b2deaf734eddf86000f`.
Live health identifies report-only commit `7cc8c6a061f1022afec90de4455d107d43e83960`.
The fresh initial HTML, JS, and CSS byte-match the live output.

Local checks passed: `npm ci`, `npm run check`, `npm test`, `npm run build`,
`cargo fmt --check`, strict Clippy, `npm run test:e2e` (12/12), and
`cargo build --locked --release`. A local empty-environment restart/persistence
smoke also passed. Docker/Podman are unavailable in this worker, so the Docker
build was not run.

Before release, run the repair and re-test order in `.factory/review-1.md`:

1. Use one replica with `/data` SQLite or shared transactional state, then
   prove independent host/controller connections work without retries.
2. Add an isolated one-click sample demo plus claims manifest and tagged tests.
3. Repair the route metadata, sitemap/404 path, and non-plain copy.
