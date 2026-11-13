//! Recorded Hive menus, the command list, pairings and the map menu.

use super::*;

/// Hive's menus as recorded on 2026-11-05, by the form ID they had then.
pub(super) fn recorded(form: u32) -> Vec<u8> {
    let line = include_str!("../fixtures/hive-forms.tsv")
        .lines()
        .find(|line| line.split('\t').next() == Some(&form.to_string()))
        .unwrap_or_else(|| panic!("form {form} not recorded"));
    let json = line.split_once('\t').unwrap().1;
    let mut batch = Vec::new();
    packets::form_request(&mut batch, form, json);
    batch
}

#[test]
fn recorded_hive_menus_get_the_recorded_answers() {
    let mut h = Harness::new();
    h.client(&chat("!autocs temple"));
    h.server(&system("You are not in a party."));
    h.run(40);
    // Custom Server Menu → Create Server → BedWars → Squads, as pressed by hand.
    h.press(&recorded(2), 2, 0);
    h.press(&recorded(3), 3, 0);
    h.press(&recorded(4), 4, 2);
    h.server(&system("§a§l» §r§7Boldworks joined. §8[1/16]"));
    // Manage Your Server → Manage Presets → Saved Presets → new scrim rules → Confirm.
    h.press(&recorded(10), 10, 2);
    h.press(&recorded(11), 11, 4);
    h.press(&recorded(12), 12, 2);
    h.press(&recorded(13), 13, 0);
    // Back out of Preset Settings (form 11 again), then Game Settings → Map → Temple (25).
    h.press(&recorded(11), 11, 5);
    h.press(&recorded(5), 5, 0);
    h.press(&recorded(6), 6, 3);
    h.press(&recorded(7), 7, 25);
    // Teams: Set Roles → Boldworks → RED, then everyone is placed.
    h.press(&recorded(6), 6, 17); // Go Back: teams wait for a full lobby
    h.open_to_all(&recorded(5), 5, 30);
    h.client(&chat("!teams"));
    h.press(&recorded(5), 5, 0);
    h.press(&recorded(6), 6, 2);
    let sent = h.close(&recorded(17), 17);
    // Alone in the server: Boldworks is on RED, but the teams wait for more players and no
    // half-empty "Teams:" line is shown.
    assert!(sent.said("so the teams aren't finished"), "{:?}", sent.said);
    assert!(!sent.said("Teams:"), "{:?}", sent.said);
}

// ---- /autocs in the command list ----------------------------------------------------------------

mod command_list {
    use bedrock_protocol::packets::AvailableCommands;
    use bedrock_protocol::types::{CommandData, CommandEnum, CommandOverload};
    use bedrock_protocol::wire::{self, Decode, Encode};

    use super::*;
    use crate::autocs::MAPS;
    use crate::commands::{ACTIONS, AVAILABLE_COMMANDS, NAME, slug, with_autocs};

    fn hive_list(extra: Option<&str>) -> Vec<u8> {
        let mut list = AvailableCommands {
            enum_values: vec!["temple".into(), "survival".into()],
            enum_data: vec![CommandEnum {
                name: "Mode".into(),
                values: vec![wire::U32LE(1)],
            }],
            ..Default::default()
        };
        for name in ["cs", "p"].into_iter().chain(extra) {
            list.commands.push(CommandData {
                name: name.into(),
                description: "Hive".into(),
                permission_level: "any".into(),
                alias_enum: wire::I32LE(-1),
                overloads: vec![CommandOverload::default()],
                ..Default::default()
            });
        }
        let mut body = Vec::new();
        list.encode(&mut wire::Writer::new(&mut body)).unwrap();
        let mut batch = Vec::new();
        push_packet(&mut batch, AVAILABLE_COMMANDS, &body);
        batch
    }

    fn decode(packet: &[u8]) -> AvailableCommands {
        let mut reader = Reader::new(packet);
        assert_eq!(reader.var_u32(), Some(AVAILABLE_COMMANDS));
        AvailableCommands::decode(&mut wire::Reader::new(reader.rest())).unwrap()
    }

    #[test]
    fn autocs_is_added_with_map_and_action_suggestions() {
        let batch = hive_list(None);
        let packet = &split_batch(&batch).unwrap()[0];
        let list = decode(&with_autocs(packet.bytes).expect("rewritten"));
        let names: Vec<_> = list
            .commands
            .iter()
            .map(|c| c.name.to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["cs", "p", NAME]);
        let autocs = list.commands.last().unwrap();
        assert_eq!(autocs.overloads.len(), 3);
        assert!(autocs.overloads[0].parameter_data.is_empty());
        let values = |overload: usize| -> Vec<String> {
            let symbol = autocs.overloads[overload].parameter_data[0].parse_symbol.0;
            assert_eq!(symbol & 0x30_0000, 0x30_0000, "valid enum");
            let entry = &list.enum_data[(symbol & 0xF_FFFF) as usize];
            entry
                .values
                .iter()
                .map(|i| {
                    list.enum_values[i.0 as usize]
                        .to_string_lossy()
                        .into_owned()
                })
                .collect()
        };
        let maps = values(1);
        assert_eq!(maps.len(), MAPS.len());
        assert!(maps.contains(&"souls_end".to_string()) && maps.contains(&"temple".to_string()));
        assert_eq!(values(2), ACTIONS);
        // "temple" was already listed: reused, not repeated; Hive's own enum is untouched.
        assert_eq!(
            list.enum_values
                .iter()
                .filter(|v| v.to_string_lossy() == "temple")
                .count(),
            1
        );
        assert_eq!(list.enum_data[0].values, vec![wire::U32LE(1)]);
    }

    #[test]
    fn an_existing_autocs_or_unreadable_list_is_left_alone() {
        let batch = hive_list(Some(NAME));
        assert!(with_autocs(split_batch(&batch).unwrap()[0].bytes).is_none());
        let mut broken = Vec::new();
        push_packet(&mut broken, AVAILABLE_COMMANDS, &[0xff, 0xff]);
        assert!(with_autocs(split_batch(&broken).unwrap()[0].bytes).is_none());
    }

    #[test]
    fn slugs_are_single_typeable_words() {
        assert_eq!(slug("Soul's End"), "souls_end");
        assert_eq!(slug("Coven (Halloween)"), "coven_halloween");
        assert_eq!(slug("Hanging Gardens (Spring)"), "hanging_gardens_spring");
        assert_eq!(slug("O2"), "o2");
        for map in MAPS {
            let slug = slug(map);
            assert!(
                !slug.is_empty() && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "{slug}"
            );
        }
    }

    #[test]
    fn suggested_values_are_understood() {
        let mut h = Harness::new();
        let (kept, sent) = h.client(&command("/autocs souls_end"));
        assert!(!kept);
        assert!(sent.said("Map Soul's End"));
        let (_, sent) = h.client(&command("/autocs stop"));
        assert!(sent.said("Stopped"));
        let (_, sent) = h.client(&command("/autocs hanging_gardens_spring"));
        assert!(sent.said("Map Hanging Gardens (Spring)"));
        h.client(&command("/autocs stop"));
        let (kept, sent) = h.client(&command("/autocs teams"));
        assert!(!kept);
        assert!(sent.said("Teams are set during"));
    }
}

// ---- team pairings per map ----------------------------------------------------------------------

mod pairings {
    use super::*;
    use crate::autocs::{MAPS, opponent};

    #[test]
    fn every_map_has_the_listed_opponent() {
        let expected = |map: &str| match map {
            "Autumn"
            | "Lumora"
            | "Pirates"
            | "Prisma"
            | "Soul's End"
            | "Spring (Spring)"
            | "Summer (Summer)"
            | "Temple"
            | "Winter (Winterfest)"
            | "Cobalt"
            | "Orion"
            | "Quantum"
            | "Serrano" => Some("BLUE"),
            "Cyberspace"
            | "Hanging Gardens"
            | "Hanging Gardens (Spring)"
            | "Rome"
            | "Ruins"
            | "O2"
            | "Tropical" => Some("YELLOW"),
            "Coven (Halloween)"
            | "Gingerbread (Winterfest)"
            | "Grove"
            | "Summer Coast"
            | "Jotunheim"
            | "Monster Labs (Halloween)"
            | "Tesla Labs"
            | "Railway" => Some("GREEN"),
            "Random" => None,
            other => panic!("{other} has no expected pairing"),
        };
        for map in MAPS {
            assert_eq!(opponent(map), expected(map), "{map}");
        }
    }

    #[test]
    fn every_pairing_is_confirmed() {
        use crate::autocs::pairing;
        // Confirmed by the player in their own games on 2026-11-06.
        let tested = [
            ("Tropical", "YELLOW"),
            ("Cobalt", "BLUE"),
            ("Jotunheim", "GREEN"),
            ("Monster Labs (Halloween)", "GREEN"),
            ("O2", "YELLOW"),
            ("Orion", "BLUE"),
            ("Quantum", "BLUE"),
            ("Serrano", "BLUE"),
            ("Tesla Labs", "GREEN"),
            ("Railway", "GREEN"),
        ];
        for (map, team) in tested {
            assert_eq!(pairing(map), Some((team, true)), "{map}");
            assert_eq!(opponent(map), Some(team), "{map}: players are moved");
        }
        assert_eq!(pairing("Temple"), Some(("BLUE", true)));
        assert_eq!(pairing("Nowhere"), None);
    }

    /// A run on `map` up to the Set Roles list, with H0nri in the party.
    fn at_roles(map: &str) -> Harness {
        let mut h = Harness::new();
        h.create(&format!("!autocs {map}"), "Party: Boldworks, H0nri", 8);
        h.client(&chat("!teams"));
        h.press(&manage(5), 5, 2);
        h.press(&presets(6), 6, 4);
        h.press(&saved_presets(7), 7, 2);
        h.press(&load_preset(8), 8, 0);
        h.press(&presets(9), 9, 5);
        h.press(&manage(10), 10, 0);
        h.press(&game_settings(11), 11, 3);
        let names: Vec<(String, &str)> = ["Random", "Grove", "Orion", "Rome", "Temple"]
            .iter()
            .map(|name| (name.to_string(), "Select to equip"))
            .collect();
        let buttons: Vec<(&str, &str)> = names.iter().map(|(n, d)| (n.as_str(), *d)).collect();
        let index = buttons
            .iter()
            .position(|(name, _)| name.eq_ignore_ascii_case(map))
            .unwrap();
        h.press(&menu(12, "Map", "", &buttons), 12, index);
        h.press(&game_settings(13), 13, 2);
        h
    }

    /// Eight players: the user, their party member H0nri, and six others.
    fn eight(order: [&str; 3]) -> Vec<(String, String)> {
        let mut names: Vec<&str> = order.to_vec();
        names.extend(["P2", "P3", "P4", "P5", "P6"]);
        names
            .iter()
            .map(|name| (name.to_string(), "Player".to_owned()))
            .collect()
    }

    #[test]
    fn a_yellow_map_puts_the_others_on_yellow() {
        let mut h = at_roles("rome");
        let mut players = eight(["Boldworks", "Charlie", "H0nri"]);
        let (sent, _) = h.set_roles(20, &mut players);
        assert_eq!(players[1].1, "YELLOW");
        assert!(
            sent.said("RED Boldworks, H0nri | YELLOW Charlie"),
            "{:?}",
            sent.said
        );
    }

    #[test]
    fn a_green_map_puts_the_others_on_green() {
        let mut h = at_roles("grove");
        h.press(
            &roles(20, &[("Boldworks", "RED"), ("Charlie", "Player")]),
            20,
            1,
        );
        h.press(&team_choice(21, "Charlie"), 21, 3); // GREEN
    }

    #[test]
    fn random_leaves_the_others_to_hive() {
        // Hive picks a Random map later, so the team facing RED is unknown.
        let mut h = at_roles("random");
        let mut players = eight(["Charlie", "H0nri", "Boldworks"]);
        let (sent, _) = h.set_roles(20, &mut players);
        // Charlie is skipped: Hive places him.
        assert_eq!(players[0].1, "Player");
        assert!(
            sent.said("RED H0nri, Boldworks | Hive places Charlie"),
            "{:?}",
            sent.said
        );
    }
}

// ---- the map menu -------------------------------------------------------------------------------

/// The map menu's buttons: name, detail line and picture, formatting removed.
pub(super) fn picker_buttons(json: &str) -> Vec<(String, String, Option<String>)> {
    let form: serde_json::Value = serde_json::from_str(json).unwrap();
    form["buttons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|button| {
            let text = strip(button["text"].as_str().unwrap());
            let (name, detail) = text.split_once('\n').unwrap();
            let image = button
                .pointer("/image/data")
                .and_then(|url| url.as_str())
                .map(str::to_owned);
            (name.to_owned(), detail.to_owned(), image)
        })
        .collect()
}

mod map_menu {
    use super::*;
    use crate::autocs::{MAPS, order_by};

    fn open() -> Vec<(String, String, Option<String>)> {
        let mut h = Harness::new();
        let (_, sent) = h.client(&chat("!autocs"));
        picker_buttons(&sent.forms[0].1)
    }

    #[test]
    fn every_map_shows_the_team_faced() {
        let buttons = open();
        assert_eq!(buttons.len(), MAPS.len());
        assert_eq!(
            (buttons[0].0.as_str(), buttons[0].1.as_str()),
            ("Random", "Hive picks the map")
        );
        let detail = |name: &str| buttons.iter().find(|b| b.0 == name).unwrap().1.clone();
        assert_eq!(detail("Temple"), "vs BLUE");
        assert_eq!(detail("Rome"), "vs YELLOW");
        assert_eq!(detail("Grove"), "vs GREEN");
        assert_eq!(detail("Railway"), "vs GREEN");
        assert!(
            buttons[1..].iter().all(|b| b.1.starts_with("vs ")),
            "{buttons:?}"
        );
    }

    #[test]
    fn every_map_but_random_has_hives_picture() {
        for (name, _, image) in open() {
            assert_eq!(image.is_some(), name != "Random", "{name}");
        }
    }

    #[test]
    fn picking_uses_the_grouped_order() {
        let mut h = Harness::new();
        let (_, sent) = h.client(&chat("!autocs"));
        let buttons = picker_buttons(&sent.forms[0].1);
        let grove = buttons
            .iter()
            .position(|(name, _, _)| name == "Grove")
            .unwrap();
        let mut answer = Vec::new();
        packets::form_response(&mut answer, PICKER_FORM, &format!("{grove}\n"));
        let (_, sent) = h.client(&answer);
        assert!(sent.said("Map Grove"));
    }

    #[test]
    fn a_ranking_orders_the_whole_menu() {
        let maps: Vec<String> = MAPS.iter().map(|map| map.to_string()).collect();
        let ordered = order_by(&maps, &["Rome", "souls_end", "Temple"]);
        assert_eq!(&ordered[..4], ["Random", "Rome", "Soul's End", "Temple"]);
        // The rest keep Hive's order.
        assert_eq!(ordered[4], "Autumn");
    }

    #[test]
    fn a_new_map_from_hive_joins_the_menu() {
        let mut h = Harness::new();
        h.join_lobby("");
        h.press(&manage(5), 5, 2);
        h.press(&presets(6), 6, 4);
        h.press(&saved_presets(7), 7, 2);
        h.press(&load_preset(8), 8, 0);
        h.press(&presets(9), 9, 5);
        h.press(&manage(10), 10, 0);
        h.press(&game_settings(11), 11, 3);
        h.press(
            &menu(
                12,
                "Map",
                "",
                &[("Random", ""), ("Brand New", ""), ("Temple", "")],
            ),
            12,
            2,
        );
        h.client(&chat("!autocs stop"));
        let (_, sent) = h.client(&chat("!autocs"));
        let names: Vec<String> = picker_buttons(&sent.forms[0].1)
            .into_iter()
            .map(|b| b.0)
            .collect();
        assert_eq!(names, ["Random", "Temple", "Brand New"]);
    }
}

#[test]
fn the_players_list_orders_the_menu() {
    let mut h = Harness::new();
    let (_, sent) = h.client(&chat("!autocs"));
    let names: Vec<String> = picker_buttons(&sent.forms[0].1)
        .into_iter()
        .map(|b| b.0)
        .collect();
    let expected = [
        "Random",
        // The player's list, top to bottom.
        "Temple",
        "Rome",
        "Ruins",
        "Spring (Spring)",
        "Summer (Summer)",
        "Autumn",
        "Winter (Winterfest)",
        "Pirates",
        "Hanging Gardens",
        "Hanging Gardens (Spring)",
        "Tropical",
        "Cyberspace",
        "Prisma",
        "Soul's End",
        "Gingerbread (Winterfest)",
        "Grove",
        "O2",
        "Cobalt",
        "Jotunheim",
        "Coven (Halloween)",
        "Summer Coast",
        "Serrano",
        "Lumora",
        "Tesla Labs",
        "Monster Labs (Halloween)",
        "Orion",
        "Quantum",
        // Not listed.
        "Railway",
    ];
    assert_eq!(names, expected);
}
