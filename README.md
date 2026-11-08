# Boldy Mods

[![CI](https://github.com/boIdworks/cinnabar-block-outline/actions/workflows/ci.yml/badge.svg)](https://github.com/boIdworks/cinnabar-block-outline/actions/workflows/ci.yml)

Personal mods for [Cinnabar](https://cinnabar.gg), written in Rust and built as WebAssembly
components with `mod-api`.

| Mod | What it is |
| --- | --- |
| **Boldy Mods** (`mods/boldy-mods`) | Outlines the block or mob you aim at in your own colours, plus a set of HUD and gameplay tools. Press **F8** for its menu. |
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
| **FPS counter** | Frames per second. | HUD |
| **Crosshair** | Replaces the game's crosshair: preset shapes or a pixel drawing of your own (up to 64x64). | HUD |
| **Zoom** | Hold **Z** for a smooth zoom; scroll or **=** / **-** to change it. | Camera rotation |
| **Fullbright** | Local fullbright, optional key. | Fullbright |
| **Jump reset** | Tells you whether you jumped too early or too late after a hit. Timing is measured; the `Reset!` verdict is [not calibrated yet](docs/jump-reset.md). | Controls, Player status, HUD |

The menu works fully from the keyboard, shows help when you rest the pointer on a control, and
has a colour picker (bars, wheel, hex code, saved colours).

Details of every tool: **[docs/features.md](docs/features.md)** and
**[docs/jump-reset.md](docs/jump-reset.md)**.

### Good to know

- Cinnabar draws its own selection on top of mod drawing and no mod can hide it. Turn off
  **Settings → Video → Outline Selection** and use **Hide game highlight** in the menu; see
  [docs/settings.md](docs/settings.md#cinnabars-own-selection).
- A mod cannot read or block the mouse wheel during gameplay, so **scroll zoom moves your hotbar
  slot** while you scroll. Switch it off on the Zoom page to keep your slot still (**=** / **-**
  still work).
- The zoom is limited to 10x by the camera API.

## Install

1. Install the Rust toolchain from [rustup.rs](https://rustup.rs). The pinned compiler and the
   `wasm32-unknown-unknown` target come from `rust-toolchain.toml` automatically.
2. Build and install:

   ```powershell
   python scripts\build-mod.py
   ```

   This writes `dist\block-outline.wasm` and replaces the installed copy in
   `%LOCALAPPDATA%\Cinnabar\mods`, keeping your settings. Pass `--no-install` to only build, or set
   `CINNABAR_MODS_DIR` to install elsewhere. You can also import the file in Cinnabar through
   **Settings → Mods → Import**.
3. Restart Cinnabar, allow the permissions the mod asks for, and press **F8**.

Hive Auto CS builds the same way with `python scripts\build-autocs.py`.

The installed file is called `block-outline.wasm` for historical reasons (Cinnabar keys settings
and permission grants by that name); the mod is shown as **Boldy Mods**.

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
- [docs/architecture.md](docs/architecture.md): module map, the per-frame fuel budget that shapes
  the design, and how the host check works.
