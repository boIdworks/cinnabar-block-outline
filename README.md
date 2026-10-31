# Block Outline

A Rust/WebAssembly personal mod for Cinnabar, built from the Cinnabar mod template with `mod-api`. It draws a coloured outline around the block you are looking at.

## Block Outline

- **Targeting:** the mod raycasts from your eye against the blocks' selection boxes, so slabs, stairs and other partial blocks get their real outline. Only edges of faces turned towards the camera are drawn, so you never see edges hidden behind the block.
- **Bridging:** when the look ray hits nothing and you look steeply down (Cinnabar's bridging rule: look direction Y below −0.7, support within 2 blocks under your eyes), the block you stand on is outlined if your crosshair points into one of the eight gaps around it, sides or corners.
- **Thickness:** lines keep the chosen thickness within 2.5 blocks and thin out with distance beyond that, never below 1 pixel.
- **Fill:** an optional see-through fill in the outline colour over the block's visible faces.
- **Menu:** press **F8** in game (F8 or Esc closes it). Drag the Red, Green, Blue, Opacity, Thickness and Fill bars, toggle Rainbow, or click a quick colour. The outline stays visible while the menu is open, and every change is saved.
- **Settings file:** the menu writes `block-outline.settings.json` next to the installed mod; you can also edit it by hand:

  ```json
  {
    "color": "#FF8800FF",
    "rainbow": false,
    "width": 3,
    "fill": 0,
    "reach": 5.7
  }
  ```

  `color` accepts `#RGB`, `#RRGGBB` or `#RRGGBBAA`; `width` is the up-close line width in pixels (1–8); `fill` is the face fill opacity (0–100, 0 off); `reach` is the targeting distance in blocks: 5.7 by default, Cinnabar's normal keyboard-and-mouse reach (1–7, file only).
- **Installing:** `python scripts\build-mod.py` builds the mod and replaces `block-outline.wasm` in `%LOCALAPPDATA%\Cinnabar\mods` (keeping its settings; an install from before the rename, `mod-template.wasm`, is removed and its settings moved over); pass `--no-install` to skip that, or set `CINNABAR_MODS_DIR` for another folder. Restart Cinnabar to load the new build.

Permissions (`mods\block-outline\pack\grants.json`): `terrain` to read block selection shapes, `world_draw` for the outline and fill (round joins and caps), `render` as a fallback line renderer when World drawing is not allowed, `settings` to remember your choices, and `controls` for the F8 menu.

Cinnabar's own selection draws on top of mod drawing and cannot be hidden or recoloured by a mod. Turn off **Settings → Video → Outline Selection** to replace its black lines with its light highlight, which this mod's Fill can tint.

### Fuel

Cinnabar gives this mod's `init` and each `frame` 200,000 instructions of fuel; running out quarantines it with `wasm trap: all fuel consumed` in `%LOCALAPPDATA%\Cinnabar\local-mod.status.json`. The F8 menu document is therefore embedded at build time as the `cinnabar:extension/panel-bundle@1` section (see `mods\block-outline\pack\src\main.rs`), and per-frame menu patches format numbers with integer arithmetic.

## Build

Install the Rust toolchain and `wasm32-unknown-unknown` target pinned in `rust-toolchain.toml`, then run:

```powershell
python scripts\build-mod.py
cargo test --locked
```

The importable component is written to `dist\block-outline.wasm`. Import it in Cinnabar through **Settings → Mods → Import**.

## Developing with AI

Before implementing a feature, consult the [Cinnabar modding guide](https://cinnabar.gg/docs/modding/), its relevant guide and API reference, and a matching example in the sibling `cinnabar\examples\mods` checkout. The `mod-api` dependency is pinned to a Cinnabar source revision in the workspace `Cargo.toml`; its generated bindings are the source of truth for available interfaces.

Describe the intended player-facing behavior, target world, relevant inputs or game state, restrictions, and how to verify it. Ask the assistant to inspect the docs and examples, identify the minimum grants, implement the feature, and run the build and tests. Do not rely on guessed API names or add OS, filesystem, network, or WASI access: personal mods run in a sandbox and communicate through documented capabilities.

Personal mods do not have direct world-editing or raw-packet access. If the feature requires a server guest or trusted packet access, choose the corresponding mod type and follow its separate guide instead of extending this personal mod.

## Project conventions

You should:

- Keep the Rust/WASM component entry point and Cinnabar packaging working.
- Add only the permissions the feature actually needs in `mods\block-outline\pack\grants.json`.
- Implement lifecycle and per-frame behavior in `mods\block-outline\src\guest.rs`.
- Replace `assets\base-icon.png` to change the icon shown on the mod's card in Cinnabar's Mods UI. It is embedded when building the component; keep it a PNG no larger than 256 × 256 pixels and 256 KiB.
- Add dependencies only when necessary and keep the build command working.
- Check returned errors from host API calls and keep frame work within the documented fuel budget.
- Explain the required permissions and how to build/import the finished mod.

`init` runs when the mod is initialized, and `frame` runs on admitted frames. Use the Cinnabar API through `mod-api`; do not add operating-system or WASI imports.

## Project files

- `mods\block-outline\src\guest.rs` — mod lifecycle and behavior: terrain sampling, drawing, menu events.
- `mods\block-outline\src\outline.rs` — settings, raycast, bridging support, edges, faces and terrain region (unit-tested).
- `mods\block-outline\src\panel.rs` — the F8 menu document, its patches and event handling (unit-tested).
- `mods\block-outline\pack\grants.json` — host capabilities/permissions.
- `mods\block-outline\pack\src\main.rs` — converts the compiled module into a validated Cinnabar component.
- `assets\base-icon.png` — base mod-card icon, the Cinnabar logo embedded by the packer.
- `scripts\build-mod.py` — builds the component.
