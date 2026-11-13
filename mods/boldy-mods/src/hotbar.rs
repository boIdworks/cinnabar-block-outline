//! Labels for the keys assigned to the nine hotbar slots.
macro_rules! document {
    () => {
        concat!(
            r##"{"hotkey_shadow_0":{"type":"label","text":"#hotkey_0","size":[20,9],"offset":[-79,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_0"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_0":{"type":"label","text":"#hotkey_0","size":[20,9],"offset":[-80,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_0"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_1":{"type":"label","text":"#hotkey_1","size":[20,9],"offset":[-59,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_1"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_1":{"type":"label","text":"#hotkey_1","size":[20,9],"offset":[-60,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_1"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_2":{"type":"label","text":"#hotkey_2","size":[20,9],"offset":[-39,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_2"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_2":{"type":"label","text":"#hotkey_2","size":[20,9],"offset":[-40,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_2"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_3":{"type":"label","text":"#hotkey_3","size":[20,9],"offset":[-19,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_3"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_3":{"type":"label","text":"#hotkey_3","size":[20,9],"offset":[-20,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_3"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_4":{"type":"label","text":"#hotkey_4","size":[20,9],"offset":[1,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_4"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_4":{"type":"label","text":"#hotkey_4","size":[20,9],"offset":[0,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_4"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_5":{"type":"label","text":"#hotkey_5","size":[20,9],"offset":[21,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_5"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_5":{"type":"label","text":"#hotkey_5","size":[20,9],"offset":[20,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_5"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_6":{"type":"label","text":"#hotkey_6","size":[20,9],"offset":[41,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_6"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_6":{"type":"label","text":"#hotkey_6","size":[20,9],"offset":[40,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_6"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_7":{"type":"label","text":"#hotkey_7","size":[20,9],"offset":[61,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_7"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_7":{"type":"label","text":"#hotkey_7","size":[20,9],"offset":[60,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_7"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_shadow_8":{"type":"label","text":"#hotkey_8","size":[20,9],"offset":[81,-14],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":24,"bindings":[{"binding_name":"#hotkey_8"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}},"##,
            r##"{"hotkey_8":{"type":"label","text":"#hotkey_8","size":[20,9],"offset":[80,-15],"anchor_from":"bottom_middle","anchor_to":"bottom_middle","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":25,"bindings":[{"binding_name":"#hotkey_8"},{"binding_name":"#hotkeys_visible","binding_name_override":"#visible"}]}}"##,
        )
    };
}
pub(crate) use document;

pub fn initial_bindings() -> String {
    let mut out = String::from(r##""#hotkeys_visible":false"##);
    for i in 0..9 {
        out.push_str(&format!(r##","#hotkey_{i}":"""##));
    }
    out
}

/// Physical key names use Cinnabar's control naming contract.
pub fn key_label(key: &str, shift: bool, control: bool, alt: bool, super_key: bool) -> String {
    let key = key
        .strip_prefix("Digit")
        .or_else(|| key.strip_prefix("Key"))
        .unwrap_or(key);
    let key = key
        .strip_prefix("Numpad")
        .map_or_else(|| key.to_owned(), |digit| format!("Num{digit}"));
    let mut out = String::new();
    if control {
        out.push_str("Ctrl+");
    }
    if shift {
        out.push_str("Shift+");
    }
    if alt {
        out.push_str("Alt+");
    }
    if super_key {
        out.push_str("Super+");
    }
    out.push_str(&key);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readable_keys_and_modifiers() {
        assert_eq!(key_label("Digit1", false, false, false, false), "1");
        assert_eq!(key_label("KeyQ", true, true, false, false), "Ctrl+Shift+Q");
        assert_eq!(key_label("Numpad1", false, false, false, false), "Num1");
    }
}
