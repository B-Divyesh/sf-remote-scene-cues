# Scene Cues visual thesis

## Direction: monochrome typographic broadsheet

Scene Cues should feel like the marked-up call sheet sitting beside a stage
manager, not like a generic SaaS control panel. The interface borrows the
authority, dense rhythm, and instant scanability of a black-and-cream theatre
broadsheet. Oversized condensed headlines announce the current cue; hairline
rules, folio numbers, and editorial labels organize everything else. The phone
controller reduces this language to one thumb-sized action and the next line.

## Tokens

- Ink `#151512`: primary type and hard outlines.
- Paper `#F2EEDF`: warm background, chosen for the working-script metaphor.
- Fresh paper `#FBF9F1`: lifted working surfaces.
- Pencil `#66645C`: secondary copy (7.0:1 on paper).
- Signal red `#B52B24`: the single live/action accent, inspired by a cue light.
- Signal dark `#7A1B17`: focus and pressed states.
- Go green `#235C3A`: receipt/success, always paired with words or symbols.
- Warning ochre `#7A4B00`; danger `#98231D`.

This is an intentionally single-mode product: a consistent light paper field
keeps every control legible in dim control booths without making the red GO
signal glow or bloom. There are no gradients.

## Type and rhythm

Headlines use self-hosted **League Gothic** (OFL), a narrow display face with
the spatial economy of playbills. Body and controls use a native humanist sans
stack (`Inter` when installed, then system UI) for fast loading and unambiguous
small text. The scale is 14 / 16 / 20 / 32 / clamp(56–112) px. Body text never
drops below 16 px. Labels are uppercase with generous tracking; numbers use
tabular figures.

Spacing follows a 4/8 px rhythm: 8, 12, 16, 24, 32, 48, 64. Content measures
at most 72 characters. Groups rely on whitespace and rules before boxes.
Buttons are at least 48 px tall; the primary GO target is at least 112 px.

## Interaction grammar

- The active cue owns the largest type on every screen.
- Solid ink means structural action; signal red is reserved for advancing a
  live room. Success is communicated by a check plus words, never color alone.
- A pressed control translates down 2 px, like a physical desk switch.
- New cue receipts use a 220 ms upward reveal from the log line that produced
  them. Approval drawers enter from their originating header control.
- Keyboard: `G` advances when focus is not in a field; arrows choose a cue;
  Enter/Space activate normal controls. Visible double-line focus is universal.
- At 390 px, supporting instructions and decorative illustration move below
  the working controls; host setup stacks, controller GO remains above fold.

Under `prefers-reduced-motion: reduce`, all transforms and smooth scrolling are
removed; state updates become instant opacity changes. Nothing loops or flashes.

## Original asset plan and prompt sheet

The hero is a wide editorial still life: a stage manager's paper cue stack,
handheld cue switch, coiled cable, and three cropped theatrical light apertures.
It explains the product's bridge between paper-simple operation and connected
scene control. It is reproduced like a coarse monochrome newspaper engraving,
with a small amount of signal-red registration ink. It contains no UI claims.

**Generation prompt (stylized-concept):** “Wide editorial still life for a web
landing page. A stage manager's marked cue sheets, a simple unbranded handheld
cue switch with one large round button, a coiled cable, and three abstract
theatrical light apertures arranged as a decisive diagonal. Monochrome black ink
woodcut and coarse newspaper halftone on warm cream uncoated paper, restrained
single spot color in deep signal red, hard side light, tactile paper grain,
high-contrast broadsheet illustration, generous quiet negative space, no people,
no readable text, no letters, no numbers, no logos, no watermark, no screens,
no gradients, no glossy 3D.”

Generated with the Param Factory Azure image deployment (`factory-image`) on
2026-08-28. The selected output is original to Scene Cues and may be used under
the repository's MIT license. Source PNG and prompt metadata are retained in
`assets/src/`; delivery variants are WebP and AVIF with explicit dimensions.

`frontend/public/og-scene-cues.jpg` is a 1200 × 630 crop derived from that
selected original hero. `apple-touch-icon.png` is a hand-composed ink-and-paper
SC monogram made from the same palette; it contains no external artwork.
