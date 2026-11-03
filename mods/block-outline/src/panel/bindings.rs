//! Every live binding of the menu: captions, values, colours and which parts are shown.

use super::*;

/// Every surface binding: row captions, fill fractions, toggles, favourites and the preview.
pub(super) fn bindings(settings: &Settings, seconds: f32, state: MenuState) -> String {
    let mut out = String::with_capacity(1024);
    bindings_into(&mut out, settings, seconds, state);
    out
}

pub(super) fn bindings_into(out: &mut String, settings: &Settings, seconds: f32, state: MenuState) {
    let toggle = |out: &mut String, name: &str, text: &str, on: bool| {
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
    out.push_str(r##""#hide_game_highlight_text":"Game highlight fix: "##);
    if settings.highlight_cancel == 0 {
        out.push_str("Off");
    } else {
        let _ = write!(out, "{}%%", settings.highlight_cancel);
    }
    out.push_str(r##"","#hide_game_highlight_bg":"##);
    vector(
        out,
        if settings.highlight_cancel > 0 {
            ACTIVE
        } else {
            TRACK
        },
    );
    out.push(',');
    let matching = settings.fill_rgb.is_none();
    toggle(
        out,
        "fill_match",
        if matching {
            "Fill: match line"
        } else {
            "Fill: own colour"
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
            (true, _) => "Press a key...",
            (false, Some(key)) => key.display(),
            (false, None) => "Not set",
        },
        state.capturing_fullbright,
    );
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
    out.push_str(r##""#title_text":""##);
    out.push_str(match state.page {
        Page::Home => "Block Outline",
        Page::Block => "Block outline",
        Page::Mob => "Mob hitbox",
    });
    // In the colour tools, say which colour is being edited.
    if state.page != Page::Home && state.colour_open {
        out.push_str(match (state.editing_aim, state.editing_fill) {
            (true, _) => " - aimed colour",
            (false, true) => " - fill colour",
            (false, false) => " - line colour",
        });
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
    // While the fill follows the line there is no fill colour to pick: the Line colour
    // button stretches over the Fill colour button's place instead.
    // The block page shows Line + Fill colour, or a wide Line & fill colour; the mob page
    // Line colour + Aimed colour.
    out.push_str(r##""#fill_own_visible":"##);
    out.push_str(if !mob && !matching { "true" } else { "false" });
    out.push_str(r##","#fill_match_visible":"##);
    out.push_str(if !mob && matching { "true" } else { "false" });
    out.push_str(r##","#mob_top_visible":"##);
    out.push_str(if mob { "true" } else { "false" });
    out.push_str(r##","#colour_visible":"##);
    out.push_str(if page && state.colour_open {
        "true"
    } else {
        "false"
    });
    out.push_str(r##","#settings_visible":"##);
    out.push_str(if page && !state.colour_open {
        "true"
    } else {
        "false"
    });
    out.push(',');
    toggle(
        out,
        "menu_key",
        if state.capturing {
            "Press a key..."
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
