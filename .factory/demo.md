# Scene Cues demo sandbox

Open [https://remote-scene-cues.sociobot.in/demo](https://remote-scene-cues.sociobot.in/demo) or choose **Try it with sample data** on the landing page.

The demo shows **The Lantern Room — tech rehearsal** with ten ordered cues,
three received cues, one approved controller, and one pending controller. It
lets a visitor approve Alex — props, send the next cue, add sample cues to the
12-cue boundary, download CSV, and clear the sample log.

The browser stores the sample only under the `demo:scene-cues:sample-v1` local
storage key. `/api/demo/workspaces` creates a separate in-memory, random demo
workspace with a 24-hour TTL; it never reads or writes the SQLite `rooms`
table. **Reset demo** restores the shipped sample. **Start for real** removes
the demo storage and asks the server to discard its ephemeral workspace.

The demo URL is safe for automated claim checks and does not create a real
rehearsal room.
