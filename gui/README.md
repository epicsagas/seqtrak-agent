# SEQTRAK Web GUI (`gui/`)

English | [한국어](README.ko.md)

Svelte 5, TypeScript, and Vite frontend served by `st serve` on `http://127.0.0.1:8787`.

> **Note:** This web GUI is currently a prototype and may not work accurately in all situations. Use the `st` CLI or `st-mcp` for reliable hardware control.

## Views

- **DEVICE** (`src/components/device/DeviceFace.svelte`): SVG hardware faceplate replica with pattern selection knobs (SysEx `30:5p:0F`), 2x8 drum step grid synced to the unit's MIDI clock, `[REC]` plus 7 scale keys, Sound Design knobs, `[FX]` type rotary, latching `MASTER/SINGLE` switch, and momentary `HIGH PASS` and `REPEATER` touch sliders.
- **MIXER** (`src/App.svelte`): 11 track strips (`TrackStrip.svelte`), 7 drum trigger pads (`PadGrid.svelte`), arpeggiator and portamento controls (`ArpBlock.svelte`), master `AllKnob.svelte`, `Transport.svelte`, project P1 to P8 loader, and incoming MIDI `EventLog.svelte`.
- **HELP** (`src/components/HelpPanel.svelte`): In-app manual drawer with an `EN` / `KO` language toggle.

## Development and build

```bash
npm install
npm run check   # runs svelte-check and tsc
npm run build   # writes production bundle to gui/dist/
```

Then from the repository root:

```bash
st serve        # serves gui/dist/ and /api/* on http://127.0.0.1:8787
```

## References

- UX and hardware mapping: `../docs/DEVICE_UX.md` ([한국어](../docs/DEVICE_UX.ko.md))
- Design tokens and layout: `../docs/DESIGN.md` ([한국어](../docs/DESIGN.ko.md))
- Backend API surface: `../docs/GUI_PLAN.md` ([한국어](../docs/GUI_PLAN.ko.md))
