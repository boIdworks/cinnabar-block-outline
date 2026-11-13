# Architecture

This workspace builds two Cinnabar mods:

| Crate | Mod | Kind |
| --- | --- | --- |
| `mods/boldy-mods` (+ `pack`) | **Boldy Mods**: block and mob outline, HUD counters, crosshair, zoom, jump reset trainer | personal mod |
| `mods/hive-autocs` (+ `pack`) | **Hive Auto CS**: sets up a Bedwars Squads custom server on The Hive | packet mod |

Each mod is a library crate compiled to `wasm32-unknown-unknown`; its `pack` crate turns the core
module into a validated Cinnabar component (`scripts/build-mod.py`, `scripts/build-autocs.py`).

## The one rule: logic is host-independent, the guest is thin

Anything that can be written without Cinnabar's bindings is, so `cargo test` exercises it natively.
Only the `guest` modules call the host. They compile natively too (so clippy and the tests see them),
but the host calls only work inside Cinnabar, so tests cover just the helpers around them. That is why
most of the code is testable and the part that is not is thin glue.

### `mods/boldy-mods/src`

| Path | Responsibility | Tested natively |
| --- | --- | --- |
| `outline/` | `settings` (the settings file), `colour`, `fields` (one-pass JSON reader), `raycast` (targeting, bridging support, shape edges, line width) | yes |
| `crosshair/` | `model` (styles and the pixel drawing), `shape` (pixel runs, circles), `hud` (the documents and patches that show it) | yes |
| `counter.rs`, `cps.rs`, `fps.rs` | the arrow counter, CPS readout and FPS counter | yes |
| `hudedit.rs` | the on-screen editor that moves and resizes those three | yes |
| `jumpreset.rs` | jump timing, knock-left measurement, the recorded log | yes |
| `zoom.rs` | smooth zoom, scroll steps | yes |
| `autoq.rs` | auto queue: reading Hive's "Finding you a game of ...", "Game OVER!", party-out and "FINAL KILL!" chat lines (`read_line`), telling the player's own elimination (Hive's "You have been eliminated from the game!", or `alive` turning off plus a final kill), the `/p list` party check, and Hive's "Custom" sidebar row (`custom_game`), so a custom game is never queued, and the timing of the `/q` command (`Queue`) | yes |
| `panel/` | the F8 menu: `ids` and `metrics` (control ids and layout constants), `layout/` (widgets, pages, sections), `controls/` (the control table, `nav` for keyboard focus), `events` (what clicks and typing change), `bindings` (live captions and colours), `content` (the full document and its patches), `help` (hover help) | yes |
| `arena.rs` | bump allocator for big host replies (see below) | yes |
| `guest.rs` + `guest/` | lifecycle and per-frame behaviour: `world` (terrain sample, mobs, drawing), `menu` (menu events, key capture, saving), `fuel` (budget and profile), `hud_elements` (HUD elements), `queue` (the auto queue's chat reads, the life read, the sidebar read once a second, the party question and the command) | only the helpers that do not call the host (terrain lookup, targeting, crosshair send order, fuel report); the host calls themselves run only in the game |
| `host_dump.rs` | writes every document the mod can send, for `scripts/check-host.py` | test-only |

### `mods/hive-autocs/src`

| Path | Responsibility |
| --- | --- |
| `wire.rs`, `packets.rs` | Bedrock wire basics (batch framing, headers) and the packets the mod reads or sends, decoded |
| `recorder.rs` | writes every form, screen, chat line and command of a session to Cinnabar's log, so a hand-made setup can be replayed (and recorded as a fixture) |
| `commands.rs` | adds `/autocs` to the command list the server sends, so the game autocompletes it |
| `autocs/` | the state machine: `input` (what the player types), `server` (what the server says), `setup` (create the party, apply settings), `teams` (roles and reports), `control` (ticks, answers, reset), plus `data` (maps and pairings), `menu` and `text` helpers |
| `tests/` | scripted whole-run tests against recorded Hive menus in `src/fixtures/` |

## Fuel: the budget that shapes the design

Cinnabar gives `init` and every `frame` 200,000 instructions of fuel. Running out quarantines the
mod with `wasm trap: all fuel consumed` (see `%LOCALAPPDATA%\Cinnabar\local-mod.status.json`).
Most of the unusual structure follows from that:

- **The menu document is embedded at build time** as the `cinnabar:extension/panel-bundle@1`
  section (see `pack/src/main.rs`), and per-frame menu patches format numbers with integer
  arithmetic.
- **Loading is spread over frames.** `init` only reads the settings file (split into its fields in
  one pass by `outline::Fields`) and does nothing else heavy. The first frames then take one heavy
  job each: the HUD layer, the outline's geometry, then the menu's saved values.
- **A frame keeps to 200,000 by the mod's own count**, whatever the game allows it, and only a
  frame that took a heavy job to itself may go past 150,000 without being reported.
- **`arena.rs`** bump-allocates the large host replies (terrain regions, actor lists), which would
  otherwise cost hundreds of small allocations and frees. Callers copy the little they keep into
  normal memory, and the whole arena is reclaimed at once. The arena logic is an ordinary `Arena`
  type with unit tests; only the `#[global_allocator]` that routes the guest's allocations
  through one is wasm-only. All `unsafe` in it carries a `SAFETY` comment and
  `clippy::undocumented_unsafe_blocks` keeps it that way.
- **Player-state reads are capped at eight per frame, and going over quarantines the mod** (`player-state read budget exhausted`; the host traps rather than refusing). A frame with every feature on already makes about eight, so every `player_state::read_*` call in the mod is preceded by `note_read()` (`guest/queue.rs`), the auto queue runs after the rest of the frame and makes at most one read, and only when the count says the frame has one left. Any new feature that reads player state must do the same; a loop of reads in one frame is never safe.
- **`guest/fuel.rs`** tracks fuel two ways. `FrameFuel` is always on and nearly free (no host
  calls, a few integer operations a frame): the start page shows its average, the busiest frame of
  the last few seconds ("recent") and the busiest ever ("peak"). The per-feature profile
  (`PROFILE`) says where a frame's fuel goes, but costs fuel itself, so it only exists in a build
  made with `python scripts\build-mod.py --fuel-profile` (the cargo feature `fuel-profile`); it
  saves `fuel_report` (`name=average/most/calls` per feature) to the settings file. Rebuild without
  the flag afterwards. Read the report with two things in mind: each average is **per call**, not
  per frame (the third number, the calls, says how often a section ran), and some sections lie
  inside others (`armor` in `arrow`, `hotbar` in `cps`, `xh_grid` in `crosshair`), so adding them
  up counts work twice. The `total` is the whole frame: the bookkeeping that costs fuel itself
  (`fuel_save`, the profile's periodic save, and `fuel_show`, the start page's fuel line) runs
  *before* it is taken, on the previous frames' numbers, so a frame that saved is not understated.
- **What the profile found, so it is not found again.** (1) Comparing a large array with the
  derived `==` is a byte-by-byte `bcmp` in wasm: the crosshair's 64-row drawing cost about 12,000
  fuel *every frame* just to check it had not changed. `Drawing` now compares only its rows in use,
  and the crosshair plan is re-checked only when the settings revision (`SETTINGS_REV`, bumped by
  `set_settings`, the one place settings are replaced) or the HUD size changes: 38 fuel a frame.
  Never compare or copy a big settings field per frame; read the few fields with `settings_with`.
  (2) The actor read costs about 1,000 fuel an actor, so it asks only for as many as were in range
  last time plus a few (`ACTOR_ASK`, up to `ACTOR_LIMIT`), and the most again when every actor read
  was in range. (3) The game's HUD-element list (about 30,000 fuel) is read five times a second for
  both the hidden-HUD check and the custom crosshair. (4) Every drawn line is an instance sent to
  the host, about 400 fuel each, so a mob's box is drawn as one instance of a wire cube
  (`outline::WIRE_PATH`, 16 points through all twelve edges; the three retraced ones are the least
  a single path can do) instead of up to nine edge lines: `draw` fell from 18,000-31,000 to 2,700 a
  frame in a crowd. A block keeps its per-edge lines (`Shape::wireframe` tells them apart), so it
  still shows only the side facing the camera. What is left is mostly the actor read, which costs
  about 800 fuel an actor and has no lighter form in the API.
- **Shared work, found by reading, not yet measured** (check these against a `fuel_report` before
  claiming a saving). (5) The armor HUD's bindings (about forty formatted fields) used to be rebuilt
  on every equipment change, so using an arrow or moving any item paid for them; now a
  `crate::armor::Print` of what the four cells would show is compared first, and nothing is built
  while the armor HUD is off (turning it on reads the equipment again). (6) The selected hotbar slot
  was asked for separately by the bow-only arrow counter, the bow-aware mob hitboxes and scroll zoom;
  `selected_slot()` in `guest.rs` reads it at most once a frame, a failed read included, and never
  keeps it to the next frame. (7) The fullbright key check no longer copies the whole settings every
  frame; only a press does. (8) A moved HUD element's save waits for a frame with the headroom and
  no other heavy job, stays pending if the write fails (retried after five seconds), and the new
  place shows at once. (9) The coordinates line (`fps_coords`) reads the player's feet position
  with `player_state::read_world` at most ten times a second (`COORDS_NEXT`) and only while it is
  on and in a world; the line is patched only when a block number changes. Its fuel is the `coords`
  section (a call every frame, about one in six reading): unmeasured until a `fuel_report` has it.

## Checking what Cinnabar will accept

Unit tests cannot catch a document Cinnabar refuses: a binding may only hold a flag, a number,
short text or two to four numbers, and a mod that keeps sending refused patches can be
quarantined. So `scripts/check-host.py`:

1. builds `tools/host-check` (Cinnabar's own `ui` crate at the pinned revision) to wasm,
2. runs the `host_dump` test, which writes every HUD document, panel bundle and patch the mod can
   send, in order,
3. feeds each to Cinnabar's admission code and fails if any is rejected.

CI runs it on every push. Run it yourself after changing anything the mod sends to the HUD or the
menu (`pip install wasmtime` first).

## Naming

The crate is `boldy-mods` and the game shows **Boldy Mods**, but the installed file is still
`block-outline.wasm` (and its settings `block-outline.settings.json`). Cinnabar keys settings and
permission grants by that name, so renaming it would reset every install. The build script
(`INSTALL_NAME`) is the one place that knows.

## Conventions

- Add only the permissions a feature needs, in `mods/boldy-mods/pack/grants.json`.
- Check the result of every host call, and keep frame work inside the fuel budget.
- Boldy Mods has no OS, filesystem, network or WASI access: its packer refuses WASI imports. (Hive Auto CS is a packet mod with its own grants, including `wasi` for log output.)
- Put new logic in a host-independent module with tests first; touch `guest` only for glue.
- Keep files focused. Split by responsibility when one grows past ~800 lines.
