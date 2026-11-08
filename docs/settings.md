# Settings file, installing and permissions

## The settings file

The menu writes `block-outline.settings.json` next to the installed mod; you can also edit it by hand. Every change in the menu is saved. A file that is missing keys, or that holds out-of-range values, is read with the defaults and clamped values.

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
  "zoom_key": "KeyZ",
  "jump_reset": true,
  "jump_reset_window": 75,
  "jump_reset_carry": 0.08,
  "jump_reset_log": "",
  "crosshair": false,
  "crosshair_style": "cross",
  "crosshair_color": "#FFFFFF",
  "crosshair_opacity": 100,
  "crosshair_length": 4,
  "crosshair_thickness": 1,
  "crosshair_gap": 2,
  "crosshair_outline": true,
  "crosshair_resolution": 16,
  "crosshair_pixels": "0000/0180/0180/0180/0180/0180/0000/7c3e/7c3e/0000/0180/0180/0180/0180/0180/0000",
  "crosshair_pixel_size": 2,
  "crosshair_mirror": true
}
```

Keys:

- `color` accepts `#RGB`, `#RRGGBB` or `#RRGGBBAA`.
- `width` is the up-close line width in pixels (1–8).
- `fill` is the face fill opacity (0–100, 0 off).
- `fill_color` is `line` or a hex code.
- `favourites` holds up to four comma-separated codes.
- `reach` is the targeting distance in blocks: 5.7 by default, Cinnabar's normal keyboard-and-mouse reach (1–7, file only).
- `arrow_counter_x` / `arrow_counter_y` with `arrow_counter_anchor_x` / `arrow_counter_anchor_y` (`start`, `centre` or `end`) are the arrow counter's centre, in GUI units from that edge or the middle (without anchors they are shares of the screen, 0–1, and are converted when the mod starts), and `arrow_counter_scale` its size (0.5–4).
- `cps`, `cps_right`, `cps_x` / `cps_y` with `cps_anchor_x` / `cps_anchor_y` (as for the arrow counter) and `cps_scale` (0.5–4) set the CPS counter.
- `fps`, `fps_x` / `fps_y` with `fps_anchor_x` / `fps_anchor_y` and `fps_scale` (0.5–4) likewise set the FPS counter.
- `zoom_level` is the zoom factor (1.5–10), `zoom_time` its duration in seconds (0–1), `zoom_step` the percent each scroll notch changes it by (5–50) and `zoom_key` the physical key (`KeyZ`, `F6`, …; empty means unbound).
- `jump_reset` switches the jump reset timer on or off `jump_reset_window` is its perfect window in milliseconds (20–200, on a step of 5) and `jump_reset_carry` the horizontal speed in blocks per tick (0.01–0.4, 0.08 by default) that must be left one tick after the shove to count as knocked back, below it the readout says `Reset!` (file only).
- `crosshair` switches the custom crosshair on or off, `crosshair_style` is `cross`, `cross_dot`, `t`, `x`, `circle`, `circle_dot` or `dot`, `crosshair_color` a hex code, `crosshair_opacity` 10–100 in steps of 10, and `crosshair_length` (1–20), `crosshair_thickness` (0.5–5) and `crosshair_gap` (0–10) are in GUI pixels, on steps of 0.5.
- `crosshair_resolution` is the drawing's grid (16, 32 or 64), `crosshair_pixels` the drawing, one row of the grid each in hexadecimal (pixel 0 the lowest bit) joined by `/` (an older file's 15 rows of `.` and `#` are read into the 16x16 grid), `crosshair_pixel_size` its pixel size in screen pixels (1–8) and `crosshair_mirror` the Mirror switch.

## Installing

`python scripts\build-mod.py` builds the mod and replaces `block-outline.wasm` in `%LOCALAPPDATA%\Cinnabar\mods` (keeping its settings; an install from before the rename, `mod-template.wasm`, is removed and its settings moved over); pass `--no-install` to skip that, or set `CINNABAR_MODS_DIR` for another folder. Restart Cinnabar to load the new build.

The installed file name stays `block-outline.wasm` although the crate is called `boldy-mods`: Cinnabar keys a mod's settings file and permission grants by that name, so renaming it would reset every existing install. The name shown in the game is **Boldy Mods**.

## Permissions

Permissions (`mods\block-outline\pack\grants.json`): `terrain` to read block selection shapes, `world_draw` for the outline and fill (round joins and caps), `render` as a fallback line renderer when World drawing is not allowed, `settings` to remember your choices, `controls` for the menu, its key and your jump key, `hud` to notice Hide HUD (F1), draw the arrow counter and the crosshair, and hide the game's crosshair, `player_state` to count the arrows in your inventory and see the hotbar slot move for scroll zoom and to see when you are hurt for jump reset, `camera` (Camera rotation) to apply the zoom, `players` and `entities` to see mobs and players in front of blocks, and `fullbright` for the Fullbright switch.

## Cinnabar's own selection

Cinnabar's own selection draws on top of mod drawing and cannot be hidden or recoloured by a mod. Turn off **Settings → Video → Outline Selection** to replace its black lines with its light face highlight, then use **Game highlight fix** in the menu: click it to switch the fix off or on, and use **-** / **+** to make it 5% weaker or stronger; it darkens the selected block's faces to cancel Cinnabar's brightening. In theory 100% matches it exactly (Cinnabar brightens by 1.3), but pick whatever looks untouched in game. It starts at 50%, which matched best in testing. Leave it Off with Outline Selection on, or blocks would look darker.
