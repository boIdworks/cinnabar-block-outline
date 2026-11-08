//! Every live binding of the menu: captions, values, colours and which parts are shown.

use super::*;

/// What a key button says while it waits for a key (the menu key cannot be cleared).
pub(super) const CAPTURE_MENU: &str = "Press key, Esc cancels";
/// What an unbound key button says.
pub(super) const KEY_UNSET: &str = "Click to set";
/// The same for the optional keys, which Backspace or Delete clear.
pub(super) const CAPTURE_OPTIONAL: &str = "Esc cancels, Del clears";

/// Every surface binding: row captions, fill fractions, toggles, favourites and the preview.
pub(super) fn bindings(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let mut out = String::with_capacity(1024);
    bindings_into(&mut out, settings, seconds, state);
    out
}

pub(super) fn bindings_into(out: &mut String, settings: &Settings, seconds: f32, state: MenuState) {
    let toggle = |out: &mut String, name: &str, text: &str, on: bool| {
        if SWITCHES.contains(&name) {
            // A switch has no caption to change: its pill colour and knob show the state.
            out.push_str("\"#");
            out.push_str(name);
            out.push_str("_bg\":");
            vector(out, if on { ACTIVE } else { SWITCH_OFF });
            out.push_str(",\"#");
            out.push_str(name);
            out.push_str("_knob\":[");
            fixed(
                out,
                if on {
                    PILL_W - KNOB - KNOB_OFF
                } else {
                    KNOB_OFF
                },
            );
            out.push_str(",0],");
            return;
        }
        out.push_str("\"#");
        out.push_str(name);
        out.push_str("_text\":\"");
        out.push_str(text);
        out.push_str("\",\"#");
        out.push_str(name);
        out.push_str("_bg\":");
        vector(out, if on { ACTIVE } else { TRACK });
        out.push(',');
    };
    out.push('{');
    // The hover help starts hidden and any change of settings hides it again.
    out.push_str(r##""#tip_visible":false,"#tip_text":"","#tip_offset":[0,0],"##);
    // The card fits the page; the hint line sits at its foot.
    let height = card_height(state);
    out.push_str(r##""#card_height":"##);
    fixed(out, height);
    out.push_str(r##","#card_border_height":"##);
    fixed(out, height + 2.0);
    out.push_str(r##","#hint_offset":[0,"##);
    fixed(out, height - 16.0);
    out.push_str(r##"],"#options_offset":[0,"##);
    fixed(
        out,
        if state.page == Page::Mob {
            -MOB_OPTIONS_SHIFT
        } else {
            0.0
        },
    );
    out.push_str("],");
    out.push_str(
        r##""#hover_visible":false,"#hover_offset":[0,0],"#hover_width":1,"#hover_height":1,"##,
    );
    // The keyboard focus ring and the bottom line, which shows the focused control's help.
    focus_bindings(out, state, settings.fill_rgb.is_none());
    out.push(',');
    for (index, (id, _)) in SLIDERS.iter().enumerate() {
        let (min, max, _) = slider_range(index);
        let value = slider_value(settings, state, index);
        let typing = state
            .typing
            .filter(|typing| usize::from(typing.field) == index);
        out.push_str("\"#");
        out.push_str(id);
        out.push_str("_value\":\"");
        if let Some(typing) = typing {
            out.push_str(typing.text());
            out.push('|');
        } else {
            fixed(out, value);
            if index == 4 {
                out.push_str("px");
            }
        }
        out.push_str("\",\"#");
        out.push_str(id);
        out.push_str("_box\":");
        vector(out, if typing.is_some() { ACTIVE } else { NUMBER_BOX });
        out.push_str(",\"#");
        out.push_str(id);
        out.push_str("_fill\":");
        fixed(out, ((value - min) / (max - min)).clamp(0.0, 1.0));
        out.push(',');
    }
    toggle(
        out,
        "rainbow",
        if settings.rainbow {
            "Rainbow: On"
        } else {
            "Rainbow: Off"
        },
        settings.rainbow,
    );
    toggle(
        out,
        "pulse",
        if settings.pulse {
            "Pulse: On"
        } else {
            "Pulse: Off"
        },
        settings.pulse,
    );
    out.push_str(r##""#hide_game_highlight_text":"Hide game highlight: "##);
    if settings.highlight_cancel == 0 {
        out.push_str("Off");
    } else {
        let _ = write!(out, "{}%%", settings.highlight_cancel);
    }
    out.push_str(r##"","#hide_game_highlight_bg":"##);
    // Neutral, like the other - / + values: the text says Off or the strength.
    vector(out, TRACK);
    out.push(',');
    let matching = settings.fill_rgb.is_none();
    toggle(
        out,
        "fill_match",
        if matching {
            "Match fill: On"
        } else {
            "Match fill: Off"
        },
        matching,
    );
    let mob = state.page == Page::Mob;
    toggle(
        out,
        "edit_line",
        if matching && !mob {
            "Line & fill colour"
        } else {
            "Line colour"
        },
        state.colour_open && !state.editing_fill && !state.editing_aim,
    );
    toggle(
        out,
        "edit_fill",
        "Fill colour",
        state.colour_open && state.editing_fill,
    );
    toggle(
        out,
        "edit_aim",
        "Aimed colour",
        state.colour_open && state.editing_aim,
    );
    toggle(
        out,
        "fullbright",
        if settings.fullbright {
            "Fullbright: On"
        } else {
            "Fullbright: Off"
        },
        settings.fullbright,
    );
    toggle(
        out,
        "fullbright_key",
        match (state.capturing_fullbright, &settings.fullbright_key) {
            (true, _) => CAPTURE_OPTIONAL,
            (false, Some(key)) => key.display(),
            (false, None) => KEY_UNSET,
        },
        state.capturing_fullbright,
    );
    // Each reset asks for a second click: the button turns red and says so.
    for (name, asked, idle) in [
        ("reset", Reset::Page, "Reset page"),
        ("counter_reset", Reset::Counter, "Reset place and size"),
        ("cps_reset", Reset::Cps, "Reset place and size"),
        ("fps_reset", Reset::Fps, "Reset place and size"),
        ("zoom_reset", Reset::Zoom, "Reset zoom"),
        ("crosshair_reset", Reset::Crosshair, "Reset crosshair"),
        ("crosshair_clear", Reset::Drawing, "Clear drawing"),
    ] {
        let asking = state.confirm == asked;
        out.push_str("\"#");
        out.push_str(name);
        out.push_str("_text\":\"");
        out.push_str(if asking { "Sure? Click again" } else { idle });
        out.push_str("\",\"#");
        out.push_str(name);
        out.push_str("_bg\":");
        vector(out, if asking { DANGER } else { TRACK });
        out.push(',');
    }
    toggle(
        out,
        "show_all_mobs",
        if settings.show_all_mobs {
            "All mobs: On"
        } else {
            "All mobs: Off"
        },
        settings.show_all_mobs,
    );
    let counter = settings.counter;
    toggle(
        out,
        "counter",
        if counter.on {
            "Arrow counter: On"
        } else {
            "Arrow counter: Off"
        },
        counter.on,
    );
    toggle(
        out,
        "counter_bow",
        if counter.bow_only {
            "Only with a bow: On"
        } else {
            "Only with a bow: Off"
        },
        counter.bow_only,
    );
    let cps = settings.cps;
    toggle(
        out,
        "cps_on",
        if cps.on { "CPS: On" } else { "CPS: Off" },
        cps.on,
    );
    toggle(
        out,
        "cps_right",
        if cps.right {
            "Right clicks: On"
        } else {
            "Right clicks: Off"
        },
        cps.right,
    );
    toggle(
        out,
        "cps_zeros",
        if cps.zeros {
            "Leading zeros: On"
        } else {
            "Leading zeros: Off"
        },
        cps.zeros,
    );
    let mut size = String::from("Size: ");
    digits(&mut size, (cps.scale * 100.0).round().max(0.0) as u64);
    size.push_str("%%");
    toggle(out, "cps_size", &size, false);
    let fps = settings.fps;
    toggle(
        out,
        "fps_on",
        if fps.on { "FPS: On" } else { "FPS: Off" },
        fps.on,
    );
    let mut size = String::from("Size: ");
    digits(&mut size, (fps.scale * 100.0).round().max(0.0) as u64);
    size.push_str("%%");
    toggle(out, "fps_size", &size, false);
    let jump = settings.jump_reset;
    toggle(
        out,
        "jump_on",
        if jump.on {
            "Jump reset: On"
        } else {
            "Jump reset: Off"
        },
        jump.on,
    );
    let mut window = String::from("Perfect window: ");
    digits(&mut window, u64::from(jump.window));
    window.push_str(" ms");
    toggle(out, "jump_window", &window, false);
    let crosshair = settings.crosshair;
    toggle(out, "crosshair_on", "", crosshair.on);
    toggle(out, "crosshair_outline", "", crosshair.outline);
    toggle(out, "crosshair_mirror", "", crosshair.mirror);
    // A drawing's pixels are sized in screen pixels, and it has no length or gap of its own.
    let custom = crosshair.style == crate::crosshair::CUSTOM;
    // Beside the Colour button there is room for the size alone: clicking it goes back to 16.
    let mut text = String::new();
    let size = crosshair.pixels.size() as u64;
    digits(&mut text, size);
    text.push('x');
    digits(&mut text, size);
    toggle(out, "crosshair_res", &text, false);
    out.push_str(r##""#crosshair_res_reset":"","##);
    toggle(
        out,
        "crosshair_colour",
        "Colour",
        state.page == Page::Crosshair && state.colour_open,
    );
    let defaults = crate::crosshair::DEFAULT;
    for (name, caption, _, _) in CROSS_VALUES {
        let mut text = String::from(match name {
            "crosshair_thickness" if custom => "Pixel size",
            _ => caption,
        });
        text.push_str(": ");
        let changed = match name {
            "crosshair_length" | "crosshair_gap" if custom => {
                text.push_str("not used by a drawing");
                false
            }
            "crosshair_style" => {
                text.push_str(crate::crosshair::style_name(crosshair.style));
                crosshair.style != defaults.style
            }
            "crosshair_length" => {
                fixed(&mut text, crosshair.length);
                crosshair.length != defaults.length
            }
            "crosshair_thickness" if custom => {
                digits(&mut text, u64::from(crosshair.pixel_size));
                text.push_str(" px");
                crosshair.pixel_size != defaults.pixel_size
            }
            "crosshair_thickness" => {
                fixed(&mut text, crosshair.thickness);
                crosshair.thickness != defaults.thickness
            }
            "crosshair_gap" => {
                fixed(&mut text, crosshair.gap);
                crosshair.gap != defaults.gap
            }
            _ => {
                digits(&mut text, u64::from(crosshair.opacity));
                text.push_str("%%");
                crosshair.opacity != defaults.opacity
            }
        };
        toggle(out, name, &text, false);
        out.push_str("\"#");
        out.push_str(name);
        out.push_str("_reset\":\"");
        out.push_str(if changed { "reset" } else { "" });
        out.push_str("\",");
    }
    let zoom = settings.zoom;
    toggle(
        out,
        "zoom",
        if zoom.on { "Zoom: On" } else { "Zoom: Off" },
        zoom.on,
    );
    toggle(
        out,
        "zoom_mode",
        if zoom.toggle {
            "Mode: Press to toggle"
        } else {
            "Mode: Hold the key"
        },
        zoom.toggle,
    );
    toggle(
        out,
        "zoom_scroll",
        if zoom.scroll {
            "Scroll zoom: On"
        } else {
            "Scroll zoom: Off"
        },
        zoom.scroll,
    );
    toggle(
        out,
        "zoom_sens",
        if zoom.sensitivity {
            "Relative sensitivity: On"
        } else {
            "Relative sensitivity: Off"
        },
        zoom.sensitivity,
    );
    toggle(
        out,
        "zoom_key",
        match (state.capturing_zoom, &zoom.key) {
            (true, _) => CAPTURE_OPTIONAL,
            (false, Some(key)) => key.display(),
            (false, None) => KEY_UNSET,
        },
        state.capturing_zoom,
    );
    let mut text = String::from("Zoom level: ");
    fixed(&mut text, zoom.level);
    text.push('x');
    toggle(out, "zoom_level", &text, false);
    text = if zoom.time <= 0.0 {
        String::from("Zoom time: instant")
    } else {
        let mut text = String::from("Zoom time: ");
        fixed(&mut text, zoom.time);
        text.push('s');
        text
    };
    toggle(out, "zoom_time", &text, false);
    text = String::from("Scroll step: ");
    digits(&mut text, u64::from(zoom.step));
    text.push_str("%%");
    toggle(out, "zoom_step", &text, false);
    // A value that is not its default says so, and that clicking it resets it.
    let defaults = crate::zoom::Zoom::default();
    for (name, changed) in [
        ("zoom_level", zoom.level != defaults.level),
        ("zoom_time", zoom.time != defaults.time),
        ("zoom_step", zoom.step != defaults.step),
        ("counter_size", counter.scale != 1.0),
        ("cps_size", cps.scale != 1.0),
        ("fps_size", fps.scale != 1.0),
        (
            "jump_window",
            jump.window != crate::jumpreset::DEFAULT_WINDOW,
        ),
    ] {
        out.push_str("\"#");
        out.push_str(name);
        out.push_str("_reset\":\"");
        out.push_str(if changed { "reset" } else { "" });
        out.push_str("\",");
    }
    out.push_str(r##""#open_zoom_color":[0.5,0.85,1,1],"##);
    out.push_str(r##""#counter_size_text":"Size: "##);
    digits(out, (counter.scale * 100.0).round().max(0.0) as u64);
    out.push_str(r##"%%","#counter_size_bg":"##);
    vector(out, TRACK);
    // The start page's Arrow counter button: an arrow-shaft brown stripe.
    out.push_str(r##","#open_counter_color":[0.82,0.68,0.47,1],"##);
    // The CPS button's mouse and the Lighting button's sun.
    out.push_str(r##""#open_fps_color":[0.75,0.62,1,1],"##);
    out.push_str(r##""#open_cps_color":[0.6,0.9,0.65,1],"#open_light_color":[1,0.88,0.45,1],"##);
    // The Jump reset button's arrow.
    out.push_str(r##""#open_jump_color":[1,0.55,0.5,1],"##);
    // The start page's switches.
    toggle(out, "home_block", "", settings.block_on);
    toggle(out, "home_mob", "", settings.mob_on);
    toggle(out, "home_counter", "", counter.on);
    toggle(out, "home_cps", "", cps.on);
    toggle(out, "home_fps", "", fps.on);
    toggle(out, "home_zoom", "", zoom.on);
    toggle(out, "home_light", "", settings.fullbright);
    toggle(out, "home_jump", "", jump.on);
    toggle(out, "home_crosshair", "", crosshair.on);
    out.push_str(r##""#fuel_text":""##);
    fuel_text(out, state);
    out.push_str(r##"","#title_text":""##);
    // A trail: the start page, then the page, then (in the colour tools) the colour edited.
    let page = match state.page {
        Page::Home => "Boldy Mods",
        Page::Block => "Block outline",
        Page::Mob => "Mob hitbox",
        Page::Counter => "Arrow counter",
        Page::Cps => "CPS counter",
        Page::Fps => "FPS counter",
        Page::Zoom => "Zoom",
        Page::Lighting => "Lighting",
        Page::JumpReset => "Jump reset",
        Page::Crosshair => "Crosshair",
    };
    if state.page == Page::Crosshair && state.colour_open {
        out.push_str("Crosshair > Colour");
    } else if state.page == Page::Crosshair && state.drawing {
        out.push_str("Crosshair > Draw");
    } else if state.page.outline() && state.colour_open {
        out.push_str(if state.page == Page::Mob {
            "Mob"
        } else {
            "Block"
        });
        out.push_str(match (state.editing_aim, state.editing_fill) {
            (true, _) => " > Aimed colour",
            (false, true) => " > Fill colour",
            (false, false) => " > Line colour",
        });
    } else {
        out.push_str(page);
    }
    let page = state.page != Page::Home;
    let flag = |out: &mut String, name: &str, on: bool| {
        out.push_str("\"#");
        out.push_str(name);
        out.push_str("\":");
        out.push_str(if on { "true," } else { "false," });
    };
    out.push_str("\",");
    flag(out, "home_visible", !page);
    flag(out, "page_visible", page);
    flag(out, "block_only_visible", state.page == Page::Block);
    flag(out, "mob_only_visible", state.page == Page::Mob);
    // The panels behind the settings view, which the colour tools replace.
    flag(
        out,
        "block_settings_visible",
        state.page == Page::Block && !state.colour_open,
    );
    flag(
        out,
        "mob_settings_visible",
        state.page == Page::Mob && !state.colour_open,
    );
    flag(out, "counter_page_visible", state.page == Page::Counter);
    flag(out, "cps_page_visible", state.page == Page::Cps);
    flag(out, "fps_page_visible", state.page == Page::Fps);
    flag(out, "zoom_page_visible", state.page == Page::Zoom);
    flag(out, "light_page_visible", state.page == Page::Lighting);
    flag(out, "jump_page_visible", state.page == Page::JumpReset);
    flag(
        out,
        "crosshair_page_visible",
        state.page == Page::Crosshair && !state.colour_open && !state.drawing,
    );
    flag(
        out,
        "draw_page_visible",
        state.page == Page::Crosshair && state.drawing,
    );
    // Only while the grid shows: its pixels cost a lot to write, and every menu update would
    // otherwise pay for them.
    if state.page == Page::Crosshair && state.drawing {
        draw_bindings(out, settings, state);
        out.push(',');
    }
    flag(out, "crosshair_top_visible", state.page == Page::Crosshair);
    flag(
        out,
        "crosshair_draw_button_visible",
        state.page == Page::Crosshair && !state.drawing,
    );
    flag(out, "card_visible", !state.moving);
    flag(
        out,
        "guide_top_visible",
        state.moving && !state.guide_bottom,
    );
    flag(
        out,
        "guide_bottom_visible",
        state.moving && state.guide_bottom,
    );
    let block = state.page == Page::Block;
    let outline = state.page.outline();
    // While the fill follows the line there is no fill colour to pick: the Line colour
    // button stretches over the Fill colour button's place instead.
    // The block page shows Line + Fill colour, or a wide Line & fill colour; the mob page
    // Line colour + Aimed colour.
    out.push_str(r##""#fill_own_visible":"##);
    out.push_str(if block && !matching { "true" } else { "false" });
    out.push_str(r##","#fill_match_visible":"##);
    out.push_str(if block && matching { "true" } else { "false" });
    out.push_str(r##","#mob_top_visible":"##);
    out.push_str(if mob { "true" } else { "false" });
    out.push_str(r##","#colour_visible":"##);
    out.push_str(if state.page.has_colour() && state.colour_open {
        "true"
    } else {
        "false"
    });
    out.push_str(r##","#settings_visible":"##);
    out.push_str(if outline && !state.colour_open {
        "true"
    } else {
        "false"
    });
    out.push(',');
    toggle(
        out,
        "menu_key",
        if state.capturing {
            CAPTURE_MENU
        } else {
            settings.menu_key.display()
        },
        state.capturing,
    );
    for (slot, colour) in settings.favourites.iter().enumerate() {
        out.push_str("\"#favourite_");
        digits(out, slot as u64);
        out.push_str("_color\":");
        vector(
            out,
            colour.map_or(EMPTY_SLOT, |[r, g, b, _]| {
                [
                    f32::from(r) / 255.0,
                    f32::from(g) / 255.0,
                    f32::from(b) / 255.0,
                    1.0,
                ]
            }),
        );
        out.push(',');
    }
    swatches(out, settings, seconds);
    let hex_typing = state.typing.filter(|typing| typing.field == HEX_FIELD);
    out.push_str(r##","#hex_box":"##);
    vector(out, if hex_typing.is_some() { ACTIVE } else { TRACK });
    out.push_str(r##","#hex_text":""##);
    match hex_typing {
        Some(typing) => {
            out.push('#');
            out.push_str(typing.text());
            out.push('|');
        }
        None => out.push_str(&target_hex(settings, state)),
    }
    let [x, y] = wheel_position(target_rgb(settings, state));
    out.push_str(r##"","#wheel_marker":["##);
    fixed(out, x * WHEEL[2] - MARKER / 2.0);
    out.push(',');
    fixed(out, y * WHEEL[2] - MARKER / 2.0);
    out.push_str("]}");
}
