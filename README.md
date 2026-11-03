# Block Outline

A Rust/WebAssembly personal mod for Cinnabar, built from the Cinnabar mod template with `mod-api`. It draws a coloured outline around the block you are looking at.

## Block Outline

- **Targeting:** the mod raycasts from your eye against the blocks' selection boxes, so slabs, stairs and other partial blocks get their real outline. Only edges of faces turned towards the camera are drawn, so you never see edges hidden behind the block.
- **Exact shapes:** stairs, fences, walls and other blocks made of several boxes are outlined along their real shape (up to 4 boxes; bigger shapes get their bounding box), with no lines across flat faces; the fill and the game-highlight fix follow the shape too.
- **Aimed mob:** a mob counts as aimed exactly when Cinnabar says your crosshair would hit it (attack reach, blocks in the way), not by the mod's own guess.
- **Bridging:** when the look ray hits nothing and you look steeply down (Cinnabar's bridging rule: look direction Y below −0.7, support within 2 blocks under your eyes), the block you stand on is outlined if your crosshair points into one of the eight gaps around it, sides or corners.
- **Thickness:** lines keep the chosen thickness within 2.5 blocks and thin out with distance beyond that, never below 1 pixel.
- **Fill:** an optional see-through fill in the outline colour over the block's visible faces.
- **Menu:** press **F8** in game (the menu key, changeable in the menu: click **Menu key**, then press a key; Esc cancels). F10 also opens it and Esc closes it. Rest the mouse on any control for a short explanation.
  - The menu opens on a start page: pick **Block outline** or **Mob hitbox**. Each page has its own colours, opacity, thickness, fill and rainbow (pulse, and a separate fill colour, are for blocks only: mob hitboxes have no fill; the Mob hitbox page has **Line colour** and **Aimed colour** at the top instead); **< Back** (top right) goes back one step and **X** beside it closes the menu.
  - **Line colour / Fill colour** show the colour tools, which stay hidden until then and replace the settings in the same narrow menu, so the middle of the screen stays clear (click **< Back**, or the same button again, to go back). They edit that colour: the Red, Green and Blue bars, the wheel (hue and saturation, keeping brightness), the hex code field (click it and type 0-9 and A-F; Enter applies, Escape cancels, invalid codes are dropped), the quick colours and the saved colours.
  - Drag a bar, or click the number beside it and type an exact value (Enter or a click elsewhere applies, Escape cancels, Backspace deletes). The mod reads these keys itself, because Cinnabar's native text editor does not open for this menu in game; copy and paste are therefore not available. Whatever is under the mouse is highlighted.
  - **Opacity, Thickness, Fill** set the line opacity, up-close line width and face fill strength (the fill hides while you hold your attack key to break a block, so the cracks stay visible; the mod reads your key bindings, so a remapped attack key works too). **Fill: match line** makes the fill follow the line colour (rainbow included); turn it off to give the fill its own colour; while it is on, the Fill colour button is hidden and **Line & fill colour** stretches across its place.
  - **Rainbow** cycles the line colour; **Pulse** gently fades the outline in and out.
  - **Saved colours:** **Save colour** stores the colour being edited in the next of four slots (oldest replaced); click a slot to use it.
  - **Reset** restores that page's defaults, keeping the other page, the menu key and saved colours. The menu key is set on the start page; **Game highlight fix** is on the Block outline page.
  - The outline stays visible while the menu is open, and every change is saved.
- **Fullbright:** the start page's **Fullbright** switch uses Cinnabar's own local fullbright, so caves and nights look like daylight on your screen only (nothing is sent to the server). Like the Simple Fullbright mod, it asks Cinnabar for fullbright every frame, so it stays on. **Light key** (start page, not set by default) switches it on and off in game with an on-screen message: click it, then press a key (Backspace removes it, Escape keeps the old one). It needs the **Fullbright** permission.
- **Hides with the HUD:** like Cinnabar's own selection, the outline disappears with **F1** (Hide HUD) and on the death screen.
- **Mobs and players:** when one stands in front of the block, Cinnabar targets it instead, so its hitbox is outlined rather than the block behind it. Hitboxes are drawn with the **Mob hitbox** page's settings. With **All mobs: On** (the default) every mob within 12 blocks is outlined (the nearest 8 hitboxes), and the one you aim at switches to the **Aimed colour**; with it off only the aimed mob is outlined, in the mob colours.
- **Settings file:** the menu writes `block-outline.settings.json` next to the installed mod; you can also edit it by hand:

  ```json
  {
    "color": "#FF8800FF",
    "rainbow": false,
    "width": 3,
    "fill": 0,
    "fill_color": "line",
    "pulse": false,
    "favourites": "#FF8800,,,",
    "reach": 5.7,
    "menu_key": "F8"
  }
  ```

  `color` accepts `#RGB`, `#RRGGBB` or `#RRGGBBAA`; `width` is the up-close line width in pixels (1–8); `fill` is the face fill opacity (0–100, 0 off); `fill_color` is `line` or a hex code; `favourites` holds up to four comma-separated codes; `reach` is the targeting distance in blocks: 5.7 by default, Cinnabar's normal keyboard-and-mouse reach (1–7, file only).
- **Installing:** `python scripts\build-mod.py` builds the mod and replaces `block-outline.wasm` in `%LOCALAPPDATA%\Cinnabar\mods` (keeping its settings; an install from before the rename, `mod-template.wasm`, is removed and its settings moved over); pass `--no-install` to skip that, or set `CINNABAR_MODS_DIR` for another folder. Restart Cinnabar to load the new build.

Permissions (`mods\block-outline\pack\grants.json`): `terrain` to read block selection shapes, `world_draw` for the outline and fill (round joins and caps), `render` as a fallback line renderer when World drawing is not allowed, `settings` to remember your choices, `controls` for the menu and its key, `hud` to notice Hide HUD (F1), `players` and `entities` to see mobs and players in front of blocks, and `fullbright` for the Fullbright switch.

Cinnabar's own selection draws on top of mod drawing and cannot be hidden or recoloured by a mod. Turn off **Settings → Video → Outline Selection** to replace its black lines with its light face highlight, then use **Game highlight fix** in the menu: click it to switch the fix off or on, and use **-** / **+** to make it 5% weaker or stronger; it darkens the selected block's faces to cancel Cinnabar's brightening. In theory 100% matches it exactly (Cinnabar brightens by 1.3), but pick whatever looks untouched in game. It starts at 50%, which matched best in testing. Leave it Off with Outline Selection on, or blocks would look darker.

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
- `mods\block-outline\src\panel\` — the F8 menu (unit-tested): `mod.rs` (menu state, controls and patches), `events.rs` (what clicks and typing change), `bindings.rs` (live captions and colours), `layout.rs` (the menu document), `help.rs` (hover help), `json.rs` (number writers), `tests.rs`.
- `mods\block-outline\pack\grants.json` — host capabilities/permissions.
- `mods\block-outline\pack\src\main.rs` — converts the compiled module into a validated Cinnabar component.
- `assets\base-icon.png` — base mod-card icon, the Cinnabar logo embedded by the packer.
- `scripts\build-mod.py` — builds the component.
