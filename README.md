# Scene Cues

Scene Cues is a narrow live cue remote for small escape-room, interactive-
theatre, and live-game teams. A host writes an ordered cue sheet, opens an
eight-hour room, and shares a private QR. Phone controllers request access,
the host approves each device, and every **GO** advances exactly once on the
server. Receipts remain synchronized across devices and can also be delivered
as signed webhooks.

Live product: <https://remote-scene-cues.sociobot.in>

## What v1 includes

- Ordered scene/cue setup (up to 12 cues per room).
- Short-lived private join links and explicit controller approval.
- Server-Sent Events for live state and receipt synchronization.
- Deterministic, serialized cue advancement and host-only cue selection.
- Optional HTTPS webhooks signed with HMAC-SHA256.
- CSV receipt export, manual log deletion, and automatic eight-hour expiry.
- Offline/error/empty states, keyboard `G`, and a 390 px phone remote.

It intentionally does not stream video, perform matchmaking, replace game
networking, or control safety-critical equipment.

## Stack

The frontend is Svelte 5 + TypeScript built by Vite. The Rust 2021 backend uses
Axum, Tokio, SQLx, and SQLite, serves `dist/`, and exposes the API and SSE stream
from the same origin. The production image runs as a non-root Alpine user.

## Run locally

Requirements: Node 22+, npm, Rust 1.88+, and SQLite build support.

```sh
npm install
npm run build
DATABASE_URL='sqlite://scene-cues.db?mode=rwc' \
PUBLIC_URL='http://localhost:8080' \
cargo run
```

Open <http://localhost:8080>. For frontend-only development, run the backend as
above and `npm run dev` in another terminal; Vite proxies `/api` to port 8080.

Configuration is environment-only:

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | HTTP listen port |
| `DATABASE_URL` | `sqlite://scene-cues.db?mode=rwc` | SQLite file |
| `PUBLIC_URL` | `https://remote-scene-cues.sociobot.in` | Origin placed in private join links; override for local development |
| `STATIC_DIR` | `dist` | Built frontend directory |
| `RUST_LOG` | library default | Structured JSON log filter |

## Test and verify

```sh
npm test             # Vitest unit tests + Rust API integration tests
npm run test:e2e     # desktop + 390 px Chromium, axe, console checks
npm run build        # reproducible frontend output in dist/
docker build -t scene-cues .
```

Playwright is pinned to 1.58.2. Its browser path can be supplied with
`PLAYWRIGHT_BROWSERS_PATH`. The end-to-end test starts the real Rust server and
uses two isolated browser contexts to create, join, approve, and fire a cue.

## Webhook contract

For each accepted cue the server POSTs JSON like:

```json
{
  "type": "cue.fired",
  "event": {
    "sequence": 1,
    "scene": "Opening",
    "cue_name": "House lights",
    "controller_name": "Sam — booth",
    "created_at": "2026-08-28T01:00:00Z"
  }
}
```

`X-Scene-Cues-Signature` is `sha256=<hex HMAC-SHA256 of the exact body>` using
the host-provided secret. Only public HTTPS destinations are accepted; local,
private, credential-bearing, and redirecting targets are rejected. A failed
delivery is recorded but never changes cue order.

## Privacy and deployment

There are no analytics or third-party runtime fonts/scripts. Access tokens are
stored in browser session storage. Server rooms and receipts expire after eight
hours and can be deleted immediately by the host. See `/privacy` and `/terms`
in the app.

For deployment, mount writable storage at `/data` and run exactly one replica,
because SQLite is a single-instance store. The image needs only `PORT`; it
defaults private join links to the canonical production HTTPS origin. Override
`DATABASE_URL` and `PUBLIC_URL` for local or non-production environments. TLS,
persistent volume, and replica policy belong to the factory deployment layer;
horizontal scaling requires a shared transactional database and event bus.

The generated hero source, prompt metadata, and optimized variants live under
`assets/src/` and `frontend/public/assets/`. Visual rationale and provenance are
in [`.factory/design.md`](.factory/design.md).

## License

MIT. League Gothic is redistributed under the SIL Open Font License in
`frontend/public/fonts/OFL-League-Gothic.md`.
