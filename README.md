# Boldy Mods

A personal mod for Cinnabar that outlines the block or mob you aim at in your own colours, with optional fills, rainbow and pulse effects, mob hitboxes, a movable arrow counter, a smooth zoom you can scroll and a fullbright switch. Written in Rust and built as a WebAssembly component with `mod-api`. Its files keep the original `block-outline` names, so existing installs and settings carry over.

## Features

- **Targeting:** the mod raycasts from your eye against the blocks' selection boxes, so slabs, stairs and other partial blocks get their real outline. Only edges of faces turned towards the camera are drawn, so you never see edges hidden behind the block.
- **Exact shapes:** stairs, fences, walls and other blocks made of several boxes are outlined along their real shape (up to 4 boxes; bigger shapes get their bounding box), with no lines across flat faces; the fill and the game-highlight fix follow the shape too.
- **Aimed mob:** a mob counts as aimed exactly when Cinnabar says your crosshair would hit it (attack reach, blocks in the way), not by the mod's own guess.
- **Bridging:** when the look ray hits nothing and you look steeply down (Cinnabar's bridging rule: look direction Y below −0.7, support within 2 blocks under your eyes), the block you stand on is outlined if your crosshair points into one of the eight gaps around it, sides or corners.
- **Thickness:** lines keep the chosen thickness within 2.5 blocks and thin out with distance beyond that, never below 1 pixel.
- **Fill:** an optional see-through fill in the outline colour over the block's visible faces.
- **Menu:** press **F8** in game (the menu key, changeable in the menu: click **Menu key**, then press a key; Esc cancels). Esc closes it. Rest the mouse on any control for a short explanation. Buttons are 18 units tall; every on/off setting is a pill switch (knob right and purple = on) the sliders are slim bars with a round knob, each page's settings sit in lighter inset panels under small-capital headings, the start page's buttons carry drawn icons (a block, a mob in its hitbox, an arrow, a magnifier) in the page's colour, key buttons look like keys (a button that is not set says **Click to set**), a value that is not its default says **reset** beside it (click it to reset), and the counter guide shares the menu's look; buttons that do something (Reset, Move and resize) have their caption in the middle. The bottom line of the card shows the help of whatever the pointer or the keyboard is on straight away; the floating box follows after a short rest.
  - **Keyboard:** the whole menu works without the mouse. The **arrow keys** move a thin frame the way they point: **Up** / **Down** to the control above or below (staying in the same column, so Rainbow goes down to Match fill and Pulse to Reset), **Left** / **Right** to the one beside it (along the swatches, the saved colours and side-by-side buttons); past the last control they wrap round, and the first arrow press only brings the frame in. On a **slider** or a value with **-** / **+**, Left and Right change it instead (by 5, or 0.5 px for thickness; hold **Shift** for five times that), and **holding** the key repeats, so you can sweep a slider; on a switch with nothing beside it they turn it off / on. **Enter** or **Space** presses the focused control (on a slider it starts typing the number; on a value with **-** / **+** it resets the value). **Tab** and **Shift+Tab** walk every control in reading order, **Home** / **End** jump to the first and last, **Backspace** goes back a page and **Esc** closes the menu. Clicking with the mouse puts the frame away. The keys do nothing while a box is typed into or a key is awaited. The reset buttons ask **Sure? Click again** and take the question back after 4 seconds, or when you move on.
  - The menu opens on a start page: pick **Block outline**, **Mob hitbox**, **Arrow counter**, **CPS counter**, **Zoom** or **Lighting**. Each page has its own colours, opacity, thickness, fill and rainbow (pulse, and a separate fill colour, are for blocks only: mob hitboxes have no fill; the Mob hitbox page has **Line colour** and **Aimed colour** at the top instead); **< Back** (top right) goes back one step and **X** beside it closes the menu.
  - **Line colour / Fill colour** show the colour tools, which stay hidden until then and replace the settings in the same narrow menu, so the middle of the screen stays clear (click **< Back**, or the same button again, to go back). They edit that colour: the Red, Green and Blue bars, the wheel (hue and saturation, keeping brightness), the hex code field (click it and type 0-9 and A-F; Enter applies, Escape cancels, invalid codes are dropped), the quick colours and the saved colours.
  - Drag a bar, or click the number beside it and type an exact value (Enter or a click elsewhere applies, Escape cancels, Backspace deletes). The mod reads these keys itself, because Cinnabar's native text editor does not open for this menu in game; copy and paste are therefore not available. Whatever is under the mouse is highlighted.
  - **Opacity, Thickness, Fill** set the line opacity, up-close line width and face fill strength (the fill hides while you hold your attack key to break a block, so the cracks stay visible; the mod reads your key bindings, so a remapped attack key works too). **Match fill** makes the fill follow the line colour (rainbow included); turn it off to give the fill its own colour; while it is on, the Fill colour button is hidden and **Line & fill colour** stretches across its place.
  - **Rainbow** cycles the line colour; **Pulse** gently fades the outline in and out.
  - **Saved colours:** **Save colour** stores the colour being edited in the next of four slots (oldest replaced); click a slot to use it.
  - **Reset page** restores that page's defaults, keeping the other page, the menu key and saved colours. All three reset buttons (page, counter, zoom) turn red and ask **Sure? Click again** first; clicking anything else cancels. The menu key is set on the start page, under **Menu** (**Fullbright** and its key are under **Lighting**); **Hide game highlight** (the old *Game highlight fix*) is on the Block outline page.
  - The outline stays visible while the menu is open, and every change is saved.
- **Fullbright:** the **Lighting** page's **Fullbright** switch (and its key) uses Cinnabar's own local fullbright, so caves and nights look like daylight on your screen only (nothing is sent to the server). Like the Simple Fullbright mod, it asks Cinnabar for fullbright every frame, so it stays on. **Fullbright key** (start page, under Lighting, not set by default) switches it on and off in game with an on-screen message: click it, then press a key (Escape cancels; Backspace or Delete unbinds it). It needs the **Fullbright** permission.
- **Arrow counter:** an arrow icon with the number of arrows you carry (inventory and offhand, tipped arrows included) on your HUD; the number turns red at 0. On the menu's **Arrow counter** page: switch it on or off, **Only with a bow** (shown only while a bow or crossbow is in your hand or offhand), **Size** with **-** / **+** (50%-400%; click the size to go back to 100%) and **Reset place and size** (beside the hotbar, 100%). **Move and resize on screen** hides the menu and frames the counter: drag it anywhere with the left mouse button, and drag any of the small squares on its corners to resize it, the opposite corner staying put (the size shows below it). Moving and resizing snap to the hotbar (its left and right edges, just beside it, its top, its middle height and the bottom of the screen), to the middle of the screen and to the screen edges; a purple line shows what it lined up with. Resizing until the top edge snaps to the hotbar's top makes the counter exactly as tall as the hotbar, and click **Done** (or right-click, Enter or Esc) when finished; **Reset** in the guide puts it back. Scrolling or **+** / **-** also resize it. The guide moves to the bottom of the screen while the counter is near the top. Its place is measured from the nearest screen edge or the middle (where the hotbar is), so it stays beside the hotbar when the window or GUI scale changes. It needs the **Player status** permission (Mods tab; to read the inventory) and **HUD**; without Player status it shows a yellow **?** instead of the number.
- **CPS counter:** a line of text such as `CPS: 7 | 3` (left | right mouse button clicks in the last second) on your HUD, top left by default. Open the **CPS counter** page from the start page: switch it on or off, **Right clicks** (show only the left button), **Size** with **-** / **+** (50%-400%; click the size to go back to 100%) and **Reset place and size**. **Move and resize on screen** (here or on the Arrow counter page) shows the arrow counter and the CPS readout together, each in its own frame, so you can place both in one go: drag either one anywhere, drag a corner square to resize, it snaps to the hotbar and the screen's middle and edges. Clicks count only in gameplay with the menu closed. Its place is measured from the nearest screen edge or the middle, so it stays there when the window or GUI scale changes. Needs the **Controls** and **HUD** permissions.
- **Zoom:** hold **Z** (changeable on the menu's **Zoom** page: click **Zoom key**, then press a key; **Escape** cancels; **Backspace** or **Delete** unbinds it, so the zoom does nothing until you pick a key again) and the view narrows smoothly; let go and it eases back. The Zoom page has: **Zoom** on/off; **Mode** (hold the key, or press it to switch the zoom on and off); **Scroll zoom** (it moves your hotbar slot; see below); **Relative sensitivity** (you look more slowly while zoomed, in proportion to the zoom); **Zoom level** (1.5x-10x, 4x by default), **Zoom time** (0 = instant to 1 s) and **Scroll step** (5%-50% each notch), each with **-** / **+** buttons (click the value itself to go back to its default); and **Reset zoom**.
  - **Changing the zoom while zooming:** **=** zooms in and **-** zooms out, one step each press. They never touch your hotbar.
  - **Scroll zoom:** scrolling the mouse wheel while the zoom key is held zooms in (wheel up) or out (wheel down). A mod cannot read or block the wheel during gameplay: the game moves your selected hotbar slot on every wheel turn, and the zoom only notices that. So with this on, **your hotbar slot moves while you scroll**, and no mod can stop that (it needs a change in Cinnabar itself). Switch it off to keep your slot still; **=** / **-** still work. Pressing a number key to pick a slot is not counted as a scroll.
  - Each zoom starts from the **Zoom level** setting; scrolling only changes the zoom you are holding. The zoom needs the **Camera rotation** permission and works only in gameplay, with no menu open. It is a fresh implementation inspired by Zoomify by isXander, not a copy of its code. The camera API limits the zoom to 10x and cannot widen the view past normal.
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
    "menu_key": "F8",
    "arrow_counter": true,
    "arrow_counter_bow_only": false,
    "arrow_counter_x": 0.8,
    "arrow_counter_y": 0.95,
    "arrow_counter_scale": 1,
    "zoom": true,
    "zoom_toggle": false,
    "zoom_scroll": true,
    "zoom_sensitivity": true,
    "zoom_level": 4,
    "zoom_time": 0.25,
    "zoom_step": 20,
    "zoom_key": "KeyZ"
  }
  ```

  `color` accepts `#RGB`, `#RRGGBB` or `#RRGGBBAA`; `width` is the up-close line width in pixels (1–8); `fill` is the face fill opacity (0–100, 0 off); `fill_color` is `line` or a hex code; `favourites` holds up to four comma-separated codes; `reach` is the targeting distance in blocks: 5.7 by default, Cinnabar's normal keyboard-and-mouse reach (1–7, file only); `arrow_counter_x` / `arrow_counter_y` with `arrow_counter_anchor_x` / `arrow_counter_anchor_y` (`start`, `centre` or `end`) are the arrow counter's centre, in GUI units from that edge or the middle (without anchors they are shares of the screen, 0–1, and are converted when the mod starts), and `arrow_counter_scale` its size (0.5–4); `cps`, `cps_right`, `cps_x` / `cps_y` with `cps_anchor_x` / `cps_anchor_y` (as for the arrow counter) and `cps_scale` (0.5–4) set the CPS counter; `zoom_level` is the zoom factor (1.5–10), `zoom_time` its duration in seconds (0–1), `zoom_step` the percent each scroll notch changes it by (5–50) and `zoom_key` the physical key (`KeyZ`, `F6`, …; empty means unbound).
- **Installing:** `python scripts\build-mod.py` builds the mod and replaces `block-outline.wasm` in `%LOCALAPPDATA%\Cinnabar\mods` (keeping its settings; an install from before the rename, `mod-template.wasm`, is removed and its settings moved over); pass `--no-install` to skip that, or set `CINNABAR_MODS_DIR` for another folder. Restart Cinnabar to load the new build.

Permissions (`mods\block-outline\pack\grants.json`): `terrain` to read block selection shapes, `world_draw` for the outline and fill (round joins and caps), `render` as a fallback line renderer when World drawing is not allowed, `settings` to remember your choices, `controls` for the menu and its key, `hud` to notice Hide HUD (F1) and draw the arrow counter, `player_state` to count the arrows in your inventory and see the hotbar slot move for scroll zoom, `camera` (Camera rotation) to apply the zoom, `players` and `entities` to see mobs and players in front of blocks, and `fullbright` for the Fullbright switch.

Cinnabar's own selection draws on top of mod drawing and cannot be hidden or recoloured by a mod. Turn off **Settings → Video → Outline Selection** to replace its black lines with its light face highlight, then use **Game highlight fix** in the menu: click it to switch the fix off or on, and use **-** / **+** to make it 5% weaker or stronger; it darkens the selected block's faces to cancel Cinnabar's brightening. In theory 100% matches it exactly (Cinnabar brightens by 1.3), but pick whatever looks untouched in game. It starts at 50%, which matched best in testing. Leave it Off with Outline Selection on, or blocks would look darker.

### Fuel

The start page's last line is a fuel meter: the fuel the last frame used and the busiest frame so far, out of the 200,000 Cinnabar allows each frame; a frame over 150,000 is reported once on screen.

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

- `mods\block-outline\src\guest.rs` — mod lifecycle and behavior: terrain sampling, drawing, menu events, zoom. `guest\hud_elements.rs` holds the HUD's arrow counter, CPS readout and editor frame (game-only code).
- `mods\block-outline\src\counter.rs`, `cps.rs`, `hudedit.rs` — the arrow counter, the CPS readout and the editor that moves them (unit-tested).
- `mods\block-outline\src\outline.rs` — settings, raycast, bridging support, edges, faces and terrain region (unit-tested).
- `mods\block-outline\src\panel\` — the F8 menu (unit-tested): `mod.rs` (menu state, controls and patches), `events.rs` (what clicks and typing change), `bindings.rs` (live captions and colours), `layout.rs` (the menu document), `help.rs` (hover help), `json.rs` (number writers), `tests.rs`.
- `mods\block-outline\pack\grants.json` — host capabilities/permissions.
- `mods\block-outline\pack\src\main.rs` — converts the compiled module into a validated Cinnabar component.
- `assets\base-icon.png` — base mod-card icon, the Cinnabar logo embedded by the packer.
- `scripts\build-mod.py` — builds the component.
