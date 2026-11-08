//! The menu's HUD content: the full document, and the patches that update it.

use super::*;

/// The panel bundle the packer embeds in the component, so the host loads the menu itself:
/// building the whole document inside the guest costs more than a callback's fuel.
pub fn bundle() -> String {
    format!(
        r#"{{"panel":{},"surfaces":{{}}}}"#,
        content(&Settings::default(), 0.0)
    )
}

/// The whole panel for `settings`, as `panel.set-content` takes it. Build-time only.
pub fn content(settings: &Settings, seconds: f32) -> String {
    content_in(settings, seconds, MenuState::default())
}

/// [`content`] showing `state`, for rendering other pages in tests.
pub(super) fn content_in(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let mut controls = String::new();
    for (index, (id, _)) in SLIDERS.iter().enumerate() {
        let (min, max, step) = slider_range(index);
        let value = slider_value(settings, state, index);
        let _ = write!(
            controls,
            r#"{{"kind":"slider","id":"{id}","label":"{}","value":{value},"min":{min},"max":{max},"step":{step}}},"#,
            title(id)
        );
    }
    let _ = write!(
        controls,
        r#"{{"kind":"toggle","id":"rainbow","label":"Rainbow","value":{}}}"#,
        settings.rainbow
    );
    for (name, _) in SWATCHES {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"swatch_{}","label":"{name}"}}"#,
            name.to_lowercase()
        );
    }
    let [x, y] = wheel_position(target_rgb(settings, state));
    let _ = write!(
        controls,
        r#",{{"kind":"slider","id":"wheel_x","label":"Wheel","value":{x},"min":0,"max":1,"step":0.001}},{{"kind":"slider","id":"wheel_y","label":"Wheel","value":{y},"min":0,"max":1,"step":0.001}},{{"kind":"text","id":"hex","label":"Hex","value":"{}","min_chars":0,"max_chars":9}}"#,
        target_hex(settings, state)
    );
    let _ = write!(
        controls,
        r#",{{"kind":"keybind","id":"menu_key","label":"Menu key","key":"{}"}}"#,
        settings.menu_key.as_str()
    );
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"pulse","label":"Pulse","value":{}}},{{"kind":"button","id":"reset","label":"Reset page"}},{{"kind":"button","id":"edit_line","label":"Line colour"}},{{"kind":"button","id":"edit_fill","label":"Fill colour"}},{{"kind":"toggle","id":"fill_match","label":"Match fill to line","value":{}}}"#,
        settings.pulse,
        settings.fill_rgb.is_none()
    );
    for slot in 0..FAVOURITES {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"favourite_{slot}","label":"Saved colour {}"}}"#,
            slot + 1
        );
    }
    controls.push_str(r#",{"kind":"button","id":"save_favourite","label":"Save colour"}"#);
    for (id, _) in SLIDERS {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"type_{id}","label":"Type {}"}}"#,
            title(id)
        );
    }
    controls.push_str(r#",{"kind":"button","id":"type_hex","label":"Type hex code"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"button","id":"hide_game_highlight","label":"Hide game highlight: {}%"}}"#,
        settings.highlight_cancel
    );
    controls.push_str(r#",{"kind":"button","id":"back","label":"Back to settings"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_block","label":"Block outline"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_mob","label":"Mob hitbox"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"show_all_mobs","label":"All mobs","value":{}}}"#,
        settings.show_all_mobs
    );
    controls.push_str(r#",{"kind":"button","id":"edit_aim","label":"Aimed colour"}"#);
    controls
        .push_str(r#",{"kind":"button","id":"highlight_less","label":"Weaker highlight hiding"}"#);
    controls.push_str(
        r#",{"kind":"button","id":"highlight_more","label":"Stronger highlight hiding"}"#,
    );
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"fullbright","label":"Fullbright","value":{}}}"#,
        settings.fullbright
    );
    controls.push_str(r#",{"kind":"button","id":"fullbright_key","label":"Fullbright key"}"#);
    controls.push_str(r#",{"kind":"button","id":"close","label":"Close menu"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_counter","label":"Arrow counter"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"counter","label":"Arrow counter","value":{}}},{{"kind":"toggle","id":"counter_bow","label":"Only with a bow","value":{}}}"#,
        settings.counter.on, settings.counter.bow_only
    );
    controls.push_str(r#",{"kind":"button","id":"counter_move","label":"Move and resize"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_size","label":"Counter size"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_smaller","label":"Smaller counter"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_bigger","label":"Bigger counter"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_reset","label":"Reset counter place"}"#);
    controls.push_str(r#",{"kind":"button","id":"counter_done","label":"Done moving"}"#);
    controls.push_str(r#",{"kind":"button","id":"open_zoom","label":"Zoom"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"zoom","label":"Zoom","value":{}}},{{"kind":"toggle","id":"zoom_mode","label":"Zoom mode","value":{}}},{{"kind":"toggle","id":"zoom_scroll","label":"Scroll zoom","value":{}}},{{"kind":"toggle","id":"zoom_sens","label":"Relative sensitivity","value":{}}}"#,
        settings.zoom.on, settings.zoom.toggle, settings.zoom.scroll, settings.zoom.sensitivity
    );
    for (id, label) in [
        ("zoom_key", "Zoom key"),
        ("zoom_level", "Zoom level"),
        ("zoom_level_less", "Smaller zoom"),
        ("zoom_level_more", "Bigger zoom"),
        ("zoom_time", "Zoom time"),
        ("zoom_time_less", "Faster zoom"),
        ("zoom_time_more", "Slower zoom"),
        ("zoom_step", "Scroll step"),
        ("zoom_step_less", "Smaller scroll step"),
        ("zoom_step_more", "Bigger scroll step"),
        ("zoom_reset", "Reset zoom"),
    ] {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"{id}","label":"{label}"}}"#
        );
    }
    controls.push_str(r#",{"kind":"button","id":"open_cps","label":"CPS counter"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"cps_on","label":"CPS counter","value":{}}},{{"kind":"toggle","id":"cps_right","label":"Right clicks","value":{}}}"#,
        settings.cps.on, settings.cps.right
    );
    for (id, label) in [
        ("cps_move", "Move and resize"),
        ("cps_size", "CPS size"),
        ("cps_size_less", "Smaller CPS"),
        ("cps_size_more", "Bigger CPS"),
        ("cps_reset", "Reset CPS place"),
    ] {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"{id}","label":"{label}"}}"#
        );
    }
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"cps_zeros","label":"Leading zeros","value":{}}}"#,
        settings.cps.zeros
    );
    controls.push_str(r#",{"kind":"button","id":"open_light","label":"Lighting"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"home_block","label":"Block outline on","value":{}}},{{"kind":"toggle","id":"home_mob","label":"Mob hitbox on","value":{}}},{{"kind":"toggle","id":"home_counter","label":"Arrow counter on","value":{}}},{{"kind":"toggle","id":"home_cps","label":"CPS counter on","value":{}}},{{"kind":"toggle","id":"home_zoom","label":"Zoom on","value":{}}},{{"kind":"toggle","id":"home_light","label":"Fullbright on","value":{}}}"#,
        settings.block_on,
        settings.mob_on,
        settings.counter.on,
        settings.cps.on,
        settings.zoom.on,
        settings.fullbright
    );
    controls.push_str(r#",{"kind":"button","id":"open_jump","label":"Jump reset"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"jump_on","label":"Jump reset","value":{}}}"#,
        settings.jump_reset.on
    );
    for (id, label) in [
        ("jump_window", "Perfect window"),
        ("jump_window_less", "Narrower window"),
        ("jump_window_more", "Wider window"),
    ] {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"{id}","label":"{label}"}}"#
        );
    }
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"home_jump","label":"Jump reset on","value":{}}}"#,
        settings.jump_reset.on
    );
    let crosshair = settings.crosshair;
    controls.push_str(r#",{"kind":"button","id":"open_crosshair","label":"Crosshair"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"home_crosshair","label":"Crosshair on","value":{}}}"#,
        crosshair.on
    );
    controls.push_str(r#",{"kind":"button","id":"crosshair_colour","label":"Crosshair colour"}"#);
    for (id, caption, _, _) in CROSS_VALUES {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"{id}","label":"Crosshair {}"}},{{"kind":"button","id":"{id}_less","label":"Less crosshair {}"}},{{"kind":"button","id":"{id}_more","label":"More crosshair {}"}}"#,
            caption.to_lowercase(),
            caption.to_lowercase(),
            caption.to_lowercase()
        );
    }
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"crosshair_on","label":"Crosshair","value":{}}},{{"kind":"toggle","id":"crosshair_outline","label":"Crosshair outline","value":{}}}"#,
        crosshair.on, crosshair.outline
    );
    controls.push_str(r#",{"kind":"button","id":"crosshair_reset","label":"Reset crosshair"}"#);
    controls.push_str(r#",{"kind":"button","id":"crosshair_draw","label":"Draw your own"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"crosshair_mirror","label":"Mirror drawing","value":{}}}"#,
        crosshair.mirror
    );
    controls.push_str(r#",{"kind":"button","id":"crosshair_clear","label":"Clear drawing"}"#);
    for (id, label) in [
        ("crosshair_res", "Drawing resolution"),
        ("crosshair_res_less", "Fewer pixels"),
        ("crosshair_res_more", "More pixels"),
    ] {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"{id}","label":"{label}"}}"#
        );
    }
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"fps_on","label":"FPS counter","value":{}}}"#,
        settings.fps.on
    );
    for (id, label) in [
        ("fps_move", "Move and resize FPS"),
        ("fps_size", "FPS size"),
        ("fps_size_less", "Smaller FPS"),
        ("fps_size_more", "Bigger FPS"),
        ("fps_reset", "Reset FPS place"),
    ] {
        let _ = write!(
            controls,
            r#",{{"kind":"button","id":"{id}","label":"{label}"}}"#
        );
    }
    controls.push_str(r#",{"kind":"button","id":"open_fps","label":"FPS counter"}"#);
    let _ = write!(
        controls,
        r#",{{"kind":"toggle","id":"home_fps","label":"FPS counter on","value":{}}}"#,
        settings.fps.on
    );
    format!(
        r#"{{"reference_size":[{},{}],"motion":true,"widget_colors":{{"background":[0.07,0.08,0.1,0.98],"surface":[0.2,0.21,0.24,1],"text":[0.92,0.93,0.95,1],"accent":[0.45,0.24,0.75,1],"border":[0.36,0.37,0.42,1]}},"controls":[{controls}],"surface":{{"screen":"bo.screen","document":{},"bindings":{}}}}}"#,
        // Shrinks the whole menu on windows too small for the card, with a small margin.
        SETTINGS_W + 16.0,
        CARD_H + 16.0,
        json_string(&document()),
        bindings(settings, seconds, state)
    )
}

/// A patch moving every control and label to `settings`. Runs in frame callbacks, so it
/// formats numbers with [`fixed`] rather than the much costlier float formatting.
pub fn update(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let shown = view(settings, state);
    let settings = &shown;
    let mut out = String::with_capacity(1536);
    out.push_str(r#"{"controls":[{"id":"menu_key","key":""#);
    out.push_str(settings.menu_key.as_str());
    out.push_str(if state.capturing {
        r#"","capturing":true}],"#
    } else {
        r#"","capturing":false}],"#
    });
    out.push_str(
        // Escape ends moving the counter instead of closing the menu.
        if state.capturing
            || state.capturing_fullbright
            || state.capturing_zoom
            || state.typing.is_some()
            || state.moving
        {
            r#""capture_key":true,"#
        } else {
            r#""capture_key":false,"#
        },
    );
    // The arrow counter lives on the HUD, which the menu hides on its other pages.
    out.push_str(
        if matches!(
            state.page,
            Page::Counter | Page::Cps | Page::Fps | Page::JumpReset | Page::Crosshair
        ) || state.moving
        {
            r#""show_gameplay_hud":true,"#
        } else {
            r#""show_gameplay_hud":false,"#
        },
    );
    out.push_str(r#""control_values":["#);
    let value = |out: &mut String, index: usize, write: &dyn Fn(&mut String)| {
        if !out.ends_with('[') {
            out.push(',');
        }
        out.push('[');
        fixed(out, index as f32);
        out.push(',');
        write(out);
        out.push(']');
    };
    for index in 0..SLIDERS.len() {
        let v = slider_value(settings, state, index);
        value(&mut out, index, &|out| fixed(out, v));
    }
    let boolean =
        |on: bool| move |out: &mut String| out.push_str(if on { "true" } else { "false" });
    value(&mut out, RAINBOW, &boolean(settings.rainbow));
    let wheel = wheel_position(target_rgb(settings, state));
    value(&mut out, WHEEL_X, &|out| fixed(out, wheel[0]));
    value(&mut out, WHEEL_Y, &|out| fixed(out, wheel[1]));
    let hex = target_hex(settings, state);
    value(&mut out, HEX, &|out| {
        out.push('"');
        out.push_str(&hex);
        out.push('"');
    });
    value(&mut out, PULSE, &boolean(settings.pulse));
    value(&mut out, FILL_MATCH, &boolean(settings.fill_rgb.is_none()));
    value(&mut out, SHOW_ALL, &boolean(settings.show_all_mobs));
    value(&mut out, FULLBRIGHT, &boolean(settings.fullbright));
    value(&mut out, COUNTER, &boolean(settings.counter.on));
    value(&mut out, COUNTER_BOW, &boolean(settings.counter.bow_only));
    value(&mut out, ZOOM_ON, &boolean(settings.zoom.on));
    value(&mut out, ZOOM_MODE, &boolean(settings.zoom.toggle));
    value(&mut out, ZOOM_SCROLL, &boolean(settings.zoom.scroll));
    value(&mut out, ZOOM_SENS, &boolean(settings.zoom.sensitivity));
    value(&mut out, CPS_ON, &boolean(settings.cps.on));
    value(&mut out, CPS_RIGHT, &boolean(settings.cps.right));
    value(&mut out, CPS_ZEROS, &boolean(settings.cps.zeros));
    value(&mut out, FPS_ON, &boolean(settings.fps.on));
    value(&mut out, HOME_BLOCK_ON, &boolean(settings.block_on));
    value(&mut out, HOME_MOB_ON, &boolean(settings.mob_on));
    value(&mut out, HOME_COUNTER_ON, &boolean(settings.counter.on));
    value(&mut out, HOME_CPS_ON, &boolean(settings.cps.on));
    value(&mut out, HOME_FPS_ON, &boolean(settings.fps.on));
    value(&mut out, HOME_ZOOM_ON, &boolean(settings.zoom.on));
    value(&mut out, HOME_LIGHT_ON, &boolean(settings.fullbright));
    value(&mut out, JUMP_ON, &boolean(settings.jump_reset.on));
    value(&mut out, HOME_JUMP_ON, &boolean(settings.jump_reset.on));
    value(&mut out, HOME_CROSS_ON, &boolean(settings.crosshair.on));
    value(&mut out, CROSS_ON, &boolean(settings.crosshair.on));
    value(
        &mut out,
        CROSS_OUTLINE,
        &boolean(settings.crosshair.outline),
    );
    value(&mut out, DRAW_MIRROR, &boolean(settings.crosshair.mirror));
    out.push_str(r#"],"binding_changes":"#);
    bindings_into(&mut out, settings, seconds, state);
    out.push('}');
    out
}

/// A patch of only the preview colour, for animating the rainbow while the menu is open.
pub fn preview_update(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let mut out = String::with_capacity(96);
    let shown = view(settings, state);
    let settings = &shown;
    out.push_str(r##"{"binding_changes":{"##);
    swatches(&mut out, settings, seconds);
    out.push_str("}}");
    out
}

/// Appends the line and fill swatch colours shown inside the Line / Fill colour buttons.
pub(super) fn swatches(out: &mut String, settings: &Settings, seconds: f32) {
    out.push_str(r##""#line_swatch":"##);
    let [r, g, b, _] = settings.srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#fill_swatch":"##);
    let [r, g, b] = settings.fill_srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#open_block_color":"##);
    let [r, g, b, _] = settings.srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#open_mob_color":"##);
    let [r, g, b, _] = settings.for_mobs().srgb(seconds);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#aim_swatch":"##);
    let [r, g, b] = settings.aim_rgb.map(|c| f32::from(c) / 255.0);
    vector(out, [r, g, b, 1.0]);
    // The Crosshair page's colour button and the start page's crosshair icon.
    let [r, g, b] = settings.crosshair.rgb.map(|c| f32::from(c) / 255.0);
    out.push_str(r##","#crosshair_swatch":"##);
    vector(out, [r, g, b, 1.0]);
    out.push_str(r##","#open_crosshair_color":"##);
    vector(out, [r, g, b, 1.0]);
}

/// Whether the colour tools' hex code and saved colours carry the line opacity: only for the
/// line, the colour that owns it. (The crosshair's opacity is a value of its own.)
pub(super) fn edits_alpha(state: MenuState) -> bool {
    !state.editing_fill && !state.editing_aim && state.page != Page::Crosshair
}

/// The colour the colour tools are editing: the crosshair's on its page, else the fill's own
/// colour, the aimed mob's or the line colour.
pub(super) fn target_rgb(settings: &Settings, state: MenuState) -> [u8; 3] {
    if state.page == Page::Crosshair {
        return settings.crosshair.rgb;
    }
    if state.editing_aim {
        return settings.aim_rgb;
    }
    let line = [settings.rgba[0], settings.rgba[1], settings.rgba[2]];
    if state.editing_fill {
        settings.fill_rgb.unwrap_or(line)
    } else {
        line
    }
}

pub(super) fn target_hex(settings: &Settings, state: MenuState) -> String {
    if !edits_alpha(state) {
        let [r, g, b] = target_rgb(settings, state);
        hex_code([r, g, b, 255])
    } else {
        hex_code(settings.rgba)
    }
}

/// Sets the edited colour. Editing the fill gives it its own colour; editing the line turns
/// rainbow off, since a fixed colour was chosen.
pub(super) fn set_target_rgb(settings: &mut Settings, state: MenuState, rgb: [u8; 3]) {
    if state.page == Page::Crosshair {
        settings.crosshair.rgb = rgb;
        return;
    }
    if state.editing_aim {
        settings.aim_rgb = rgb;
        return;
    }
    if state.editing_fill {
        settings.fill_rgb = Some(rgb);
    } else {
        settings.rgba = [rgb[0], rgb[1], rgb[2], settings.rgba[3]];
        settings.rainbow = false;
    }
}

/// Whether slider `index` (red, green, blue) belongs to the colour tools, and its row's top.
pub(super) fn slider_place(index: usize) -> (bool, f32) {
    if index < 3 {
        (true, RGB_Y + (ROW_STEP - 2.0) * index as f32)
    } else {
        (false, ROW_Y + ROW_STEP * (index - 3) as f32)
    }
}

pub(super) fn slider_range(index: usize) -> (f32, f32, f32) {
    match index {
        0..=2 => (0.0, 255.0, 1.0),
        3 => (0.0, 100.0, 1.0),
        4 => (MIN_WIDTH, MAX_WIDTH, 0.5),
        _ => (0.0, 100.0, 1.0),
    }
}

pub(super) fn slider_value(settings: &Settings, state: MenuState, index: usize) -> f32 {
    match index {
        0..=2 => f32::from(target_rgb(settings, state)[index]),
        3 => (f32::from(settings.rgba[3]) / 2.55).round(),
        4 => settings.width,
        _ => f32::from(settings.fill),
    }
}

pub(super) fn title(id: &str) -> &'static str {
    match id {
        "red" => "Red",
        "green" => "Green",
        "blue" => "Blue",
        "opacity" => "Opacity",
        "width" => "Thickness",
        _ => "Fill",
    }
}

/// The drawing grid's pixel under `point` on the card, as (row, column), if any.
pub fn draw_cell(point: [f32; 2], size: usize) -> Option<(usize, usize)> {
    let [x, y, side] = DRAW_GRID;
    let size = size.max(1);
    let cell = side / size as f32;
    let (dx, dy) = (point[0] - x, point[1] - y);
    if !(dx >= 0.0 && dy >= 0.0 && dx < side && dy < side) {
        return None;
    }
    let at = |d: f32| ((d / cell) as usize).min(size - 1);
    Some((at(dy), at(dx)))
}

/// The drawing view's live parts: the pixels drawn, in the crosshair's colour, and the help line.
pub(super) fn draw_bindings(out: &mut String, _settings: &Settings, state: MenuState) {
    // The grid itself is the HUD's `drawgrid`, which the menu embeds: a binding cannot hold
    // its pixels.
    out.push_str(r##""#draw_note_text":""##);
    out.push_str(if state.draw_full {
        "Too detailed: try fewer separate pixels"
    } else {
        "Left click draws, right click erases"
    });
    out.push('"');
}

/// A patch of only the drawing view's help line, sent when it changes while the player draws.
pub fn draw_update(settings: &Settings, state: MenuState) -> String {
    let mut out = String::with_capacity(900);
    out.push_str(r#"{"binding_changes":{"#);
    draw_bindings(&mut out, settings, state);
    out.push_str("}}");
    out
}

/// A patch of only the start page's fuel line. Runs in a frame callback, so it writes the
/// numbers with [`digits`].
pub fn fuel_update(state: MenuState) -> String {
    let mut out = String::with_capacity(112);
    out.push_str(r##"{"binding_changes":{"#fuel_text":""##);
    fuel_text(&mut out, state);
    out.push_str(r##""}}"##);
    out
}

/// The fuel line's text: nothing until a frame has been measured.
pub(super) fn fuel_text(out: &mut String, state: MenuState) {
    let [last, peak] = state.fuel_k;
    if peak == 0 {
        return;
    }
    out.push_str("Fuel per frame: ");
    digits(out, u64::from(last));
    out.push_str("k now, ");
    digits(out, u64::from(peak));
    out.push_str("k busiest of 200k");
}

/// A patch showing the moving guide at the top or the bottom of the screen.
pub fn guide_update(state: MenuState) -> String {
    let (top, bottom) = (
        state.moving && !state.guide_bottom,
        state.moving && state.guide_bottom,
    );
    format!(
        r##"{{"binding_changes":{{"#guide_top_visible":{top},"#guide_bottom_visible":{bottom}}}}}"##
    )
}
