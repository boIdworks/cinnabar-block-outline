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
Only the `guest` modules (compiled for wasm only) call the host. That is why most of the code is
testable and the wasm-only part is a few thousand lines of glue.

### `mods/boldy-mods/src`

| Path | Responsibility | Tested natively |
| --- | --- | --- |
| `outline/` | `settings` (the settings file), `colour`, `fields` (one-pass JSON reader), `raycast` (targeting, bridging support, shape edges, line width) | yes |
| `crosshair/` | `model` (styles and the pixel drawing), `shape` (pixel runs, circles), `hud` (the documents and patches that show it) | yes |
| `counter.rs`, `cps.rs`, `fps.rs` | the arrow counter, CPS readout and FPS counter | yes |
| `hudedit.rs` | the on-screen editor that moves and resizes those three | yes |
| `jumpreset.rs` | jump timing, knock-left measurement, the recorded log | yes |
| `zoom.rs` | smooth zoom, scroll steps | yes |
| `panel/` | the F8 menu: `ids` and `metrics` (control ids and layout constants), `layout/` (widgets, pages, sections), `controls/` (the control table, `nav` for keyboard focus), `events` (what clicks and typing change), `bindings` (live captions and colours), `content` (the full document and its patches), `help` (hover help) | yes |
| `arena.rs` | bump allocator for big host replies (see below) | yes |
| `guest.rs` + `guest/` | lifecycle and per-frame behaviour: `world` (terrain sample, mobs, drawing), `menu` (menu events, key capture, saving), `fuel` (budget and profile), `hud_elements` (HUD elements) | no (wasm only) |
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
- **`guest/fuel.rs`** has an optional per-feature profile (`PROFILE`), and the start page shows a
  fuel meter (last frame, busiest frame).

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
