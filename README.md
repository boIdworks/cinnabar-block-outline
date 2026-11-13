# Boldy Mods

[![CI](https://github.com/boIdworks/cinnabar-block-outline/actions/workflows/ci.yml/badge.svg)](https://github.com/boIdworks/cinnabar-block-outline/actions/workflows/ci.yml)

Personal mods for [Cinnabar](https://cinnabar.gg), written in Rust and built as WebAssembly
components with `mod-api`.

| Mod | What it is |
| --- | --- |
| **Boldy Mods** (`mods/boldy-mods`) | A PvP practice and HUD toolkit: outlines the block or mob you aim at in your own colours, shows your combat numbers (CPS, arrows, armor, effect timers) and trains your timing. Press **F8** for its menu. |
| **Hive Auto CS** (`mods/hive-autocs`) | A packet mod that sets up a Bedwars Squads custom server on The Hive: type `!autocs`. |

## Boldy Mods at a glance

Everything is configured from one in-game menu (**F8**, changeable). Each tool has its own page,
colours and on/off switch, and every change is saved.

| Tool | What it does | Permissions |
| --- | --- | --- |
| **Block outline** | Outlines the aimed block along its real shape (stairs, fences, slabs), with optional fill, rainbow and pulse. Bridging-aware. | Terrain, World drawing |
| **Mob hitbox** | Outlines the mob you aim at, or every mob within 12 blocks, in separate colours. | Players, Entities |
| **Arrow counter** | Arrows you carry, next to the hotbar. Movable and resizable on screen. | HUD, Player status |
| **CPS counter** | Left and right clicks in the last second. | HUD, Controls |
| **FPS counter** | Frames per second, with an optional coordinates line (`XYZ: 12 64 -30`) under it. | HUD, Player status |
| **Armor HUD** | Equipped armor with durability bars, in the shared HUD editor. | HUD, Player status |
| **Effect timers** | Remaining time under each status-effect icon. | HUD, Player status |
| **Hotbar keys** | The key bound to each hotbar slot, on the slot. | HUD, Controls |
| **Crosshair** | Replaces the game's crosshair: preset shapes or a pixel drawing of your own (up to 64x64). | HUD |
| **Zoom** | Hold **Z** for a smooth zoom; **=** / **-** change it (scroll zoom is optional, see below). | Camera rotation |
| **Fullbright** | Local fullbright, optional key. | Fullbright |
| **Jump reset** *(experimental)* | Tells you whether you jumped too early or too late after a hit. Timing is measured; the `Reset!` verdict is [not calibrated yet](docs/jump-reset.md), so treat it as a guide. | Controls, Player status, HUD |

The menu works fully from the keyboard, shows help when you rest the pointer on a control, and
has a colour picker (bars, wheel, hex code, saved colours).

Details of every tool: **[docs/features.md](docs/features.md)** and
**[docs/jump-reset.md](docs/jump-reset.md)**.

### Good to know

- Cinnabar draws its own selection on top of mod drawing and no mod can hide it. Turn off
  **Settings → Video → Outline Selection** and use **Hide game highlight** in the menu; see
  [docs/settings.md](docs/settings.md#cinnabars-own-selection).
- A mod cannot read or block the mouse wheel during gameplay, so **scroll zoom moves your hotbar
  slot** while you scroll. It is therefore **off by default**; switch it on at the Zoom page if you
  accept that. **=** / **-** change the zoom without touching your slot.
- The zoom is limited to 10x by the camera API.

## Install

### Easiest: download the build

1. Download `block-outline.wasm` from the
   [latest release](https://github.com/boIdworks/cinnabar-block-outline/releases/latest). Each
   release says which Cinnabar version it was tested on and lists a `SHA256SUMS` file to check the
   download.
2. In Cinnabar open **Settings → Mods → Import** and pick the file (or copy it into
   `%LOCALAPPDATA%\Cinnabar\mods`).
3. Allow the permissions the mod asks for and press **F8**. A fresh install starts with **every
   feature switched off**: open the menu and turn on the ones you want (the switches beside each
   page button on the start page), then set them up to your liking. For the best outline look, also
   turn off Cinnabar's own selection outline, see *Good to know* above.

`hive-autocs.wasm` in the same release is the separate Hive Auto CS mod.

### Build it yourself

1. Install the Rust toolchain from [rustup.rs](https://rustup.rs). The pinned compiler and the
   `wasm32-unknown-unknown` target come from `rust-toolchain.toml` automatically.
2. Build and install:

   ```powershell
   python scriptsuild-mod.py
   ```

   This writes `distlock-outline.wasm` and replaces the installed copy in
   `%LOCALAPPDATA%\Cinnabar\mods`, keeping your settings. Pass `--no-install` to only build, or set
   `CINNABAR_MODS_DIR` to install elsewhere.
3. Restart Cinnabar, allow the permissions the mod asks for, and press **F8**.

Hive Auto CS builds the same way with `python scriptsuild-autocs.py`.

The installed file is called `block-outline.wasm` for historical reasons (Cinnabar keys settings
and permission grants by that name); the mod is shown as **Boldy Mods**.

### Keep it working

Cinnabar updates itself, and a mod is built against one fixed API revision. If something breaks
after an update, run `python scripts\doctor.py` first; see [docs/stability.md](docs/stability.md).
Choosing **Settings → Updates → Beta** with *Install updates when I quit* off keeps the game from
changing under the mod.

## Settings

The menu writes `block-outline.settings.json` next to the installed mod. You can edit it by hand;
missing keys take their defaults and out-of-range values are clamped. Every key is documented in
[docs/settings.md](docs/settings.md).

## Development

```powershell
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
python scripts\check-host.py        # asks Cinnabar's own code to accept every HUD/menu document
```

CI runs formatting, clippy (native and wasm), the tests, both component builds and the host check
on every push.

- [CONTRIBUTING.md](CONTRIBUTING.md): build, checks and rules for changes.
- [docs/releasing.md](docs/releasing.md): how a release is checked, tagged and published.
- [docs/architecture.md](docs/architecture.md): module map, the per-frame fuel budget that shapes
  the design, and how the host check works.

## License

[MIT](LICENSE).
