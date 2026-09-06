# Scene Cues

Scene Cues lets small escape-room, live-game, and interactive-theatre teams run
shared rehearsal cues from approved phones. A host writes up to 12 cues, shares
a private QR link, approves each controller, and receives ordered cue receipts.
An existing game or media app can receive optional signed HTTPS webhooks.

Live product: <https://remote-scene-cues.sociobot.in>

## Try the sample

Open <https://remote-scene-cues.sociobot.in/demo> or choose **Try it with
sample data** on the landing page. It opens *The Lantern Room — tech rehearsal*
with ten cues and populated receipts. The persistent demo label explains that
sample data is separate from real rooms. **Reset demo** restores the sample;
**Start for real** discards it.

See [`.factory/demo.md`](.factory/demo.md) for the sample and storage boundary.

## What it does

- Hosts create a room with 1–12 ordered scene cues.
- Controllers use a private join link and wait for host approval.
- Each accepted GO creates an ordered receipt.
- Hosts can download receipts as CSV or clear the log.
- Rooms delete after eight hours. Hosts can delete them sooner.
- Optional public HTTPS webhooks receive HMAC-SHA256 signed cue receipts.

Scene Cues does not stream video, create player lobbies, replace game
networking, or control safety-critical equipment.

## Run locally

Requirements: Node 22+, npm, and current stable Rust with SQLite support.

```sh
npm ci
npm run build
PORT=8080 cargo run
```

Open <http://localhost:8080>. The server uses `/data/scene-cues.db` when a
`/data` mount exists; otherwise it creates `scene-cues.db` in the current
directory. For frontend development, start the backend and run `npm run dev`
in another terminal. Vite proxies `/api` and `/health` to port 8080.

Optional environment overrides:

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | HTTP listener port |
| `DATABASE_URL` | `/data` SQLite when mounted, otherwise local SQLite | Override database location |
| `PUBLIC_URL` | production Scene Cues origin | Origin in real private join links |
| `STATIC_DIR` | `dist` | Built frontend directory |
| `RUST_LOG` | `info` | Structured log filter |

## Test and verify

```sh
npm ci
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
npm run test:e2e
cargo build --locked --release
```

Every visitor-facing claim is listed in [`.factory/claims.json`](.factory/claims.json).
Run its exact commands from a clean checkout. Browser claim tests use `/demo`;
the room-expiry and HMAC fixture claims use isolated Rust tests.

Playwright is pinned to 1.58.2. Set `PLAYWRIGHT_BROWSERS_PATH` when the browser
is installed outside Playwright's default cache.

## Privacy and safety

There are no analytics, tracking pixels, third-party runtime scripts, or CDN
fonts. Real room access tokens remain in browser session storage. The sample
uses a separate `demo:` local-storage namespace and an in-memory demo
workspace. Read the live [privacy policy](https://remote-scene-cues.sociobot.in/privacy)
and [terms](https://remote-scene-cues.sociobot.in/terms).

Do not use Scene Cues for pyrotechnics, life-safety systems, access control, or
any action where a delayed or duplicated message could cause harm.

## Deploy

The container starts on `PORT` (default `8080`) without required secrets. It
uses `/data` for persistent SQLite state and must run as one replica. The
factory deployment command preserves the product volume, probes, environment,
and one-replica bound:

```sh
WO_DATA_DIR=/data /opt/fleet/lib/deploy-container.sh remote-scene-cues . Dockerfile 8080
```

The Docker image is multi-stage, runs as a non-root user, and reports its
build SHA at `/health`.

## License

MIT. League Gothic is redistributed under the SIL Open Font License in
`frontend/public/fonts/OFL-League-Gothic.md`.
