//! Batch rewriting and changing the map.

use super::*;

mod batch_rewrite {
    use super::*;
    use crate::wire::rewrite;

    fn three() -> Vec<u8> {
        let mut batch = Vec::new();
        for id in [1, 2, 3] {
            push_packet(&mut batch, id, &[id as u8; 3]);
        }
        batch
    }

    fn ids(batch: &[u8]) -> Vec<u32> {
        split_batch(batch)
            .unwrap()
            .iter()
            .map(|packet| packet.id)
            .collect()
    }

    #[test]
    fn an_unchanged_batch_is_not_rebuilt() {
        let batch = three();
        assert_eq!(
            rewrite(&split_batch(&batch).unwrap(), batch.len(), |_| None),
            None
        );
    }

    #[test]
    fn packets_are_dropped_and_replaced_in_place() {
        let batch = three();
        let packets = split_batch(&batch).unwrap();
        let mut nine = Vec::new();
        crate::wire::push_packet(&mut nine, 9, &[]);
        let nine = split_batch(&nine).unwrap()[0].bytes.to_vec();
        // Drop the first, replace the last: the middle one is copied after the change began.
        let out = rewrite(&packets, batch.len(), |packet| match packet.id {
            1 => Some(None),
            3 => Some(Some(nine.clone())),
            _ => None,
        });
        assert_eq!(ids(&out.unwrap()), [2, 9]);
        // A change in the middle keeps the packets before it.
        let out = rewrite(&packets, batch.len(), |packet| {
            (packet.id == 2).then_some(None)
        });
        assert_eq!(ids(&out.unwrap()), [1, 3]);
    }
}

// ---- changing the map in the server -------------------------------------------------------------

mod change_map {
    use super::*;

    /// Hive's Map menu with Rome added (YELLOW faces RED there), Temple selected.
    fn hive_maps(form: u32) -> Vec<u8> {
        menu(
            form,
            "Map",
            "This menu allows you to select any available map.",
            &[
                ("Random", "Select to equip"),
                ("Autumn", "Select to equip"),
                ("Rome", "Select to equip"),
                ("Temple", "» Selected «"),
            ],
        )
    }

    /// Opens Hive's Map menu by hand; returns the mod's menu in its place.
    fn open(h: &mut Harness, form: u32) -> Vec<(String, String, Option<String>)> {
        let (kept, sent) = h.server(&hive_maps(form));
        assert!(!kept, "Hive's Map menu should be replaced");
        let (id, json) = &sent.forms[0];
        assert_eq!(*id, PICKER_FORM);
        assert!(json.contains("Now: §bTemple"), "{json}");
        picker_buttons(json)
    }

    fn pick(h: &mut Harness, buttons: &[(String, String, Option<String>)], map: &str) -> Sent {
        let index = buttons.iter().position(|(name, _, _)| name == map).unwrap();
        let mut answer = Vec::new();
        packets::form_response(&mut answer, PICKER_FORM, &format!("{index}\n"));
        let (kept, sent) = h.client(&answer);
        assert!(!kept);
        sent
    }

    #[test]
    fn opening_hives_map_menu_shows_the_mods_and_presses_the_pick() {
        let mut h = Harness::new();
        h.join_lobby("");
        h.preset_and_map();
        let buttons = open(&mut h, 30);
        let sent = pick(&mut h, &buttons, "Rome");
        assert_eq!(sent.answers, vec![(30, Some("2\n".into()))]);
        assert!(sent.said("Map changed to Rome"), "{:?}", sent.said);
        assert!(sent.said("YELLOW"), "{:?}", sent.said);
        // Hive's Game Settings after the pick stays the player's.
        assert!(h.server(&game_settings(31)).0);
    }

    #[test]
    fn closing_the_mods_menu_closes_hives() {
        let mut h = Harness::new();
        h.join_lobby("");
        h.preset_and_map();
        open(&mut h, 30);
        let mut answer = Vec::new();
        packets::form_cancel(&mut answer, PICKER_FORM);
        let (_, sent) = h.client(&answer);
        assert_eq!(sent.answers, vec![(30, None)]);
        assert!(sent.commands.is_empty());
    }

    #[test]
    fn a_server_made_by_hand_gets_the_menu_too() {
        let mut h = Harness::new();
        let buttons = open(&mut h, 7);
        let sent = pick(&mut h, &buttons, "Autumn");
        assert_eq!(sent.answers, vec![(7, Some("1\n".into()))]);
        assert!(sent.said("Map changed to Autumn"));
    }

    /// A full lobby on Temple with the teams set; returns the players as Hive lists them.
    fn teams_on_temple(h: &mut Harness) -> Vec<(String, String)> {
        h.join_lobby("Party: Boldworks, H0nri");
        h.preset_and_map();
        for (joined, name) in
            (2..=FULL_LOBBY).zip(["Charlie", "H0nri", "P2", "P3", "P4", "P5", "P6"])
        {
            h.server(&system(&format!(
                "§a§l» §r§7{name} joined. §8[{joined}/16]"
            )));
        }
        h.press(&manage(20), 20, 0);
        h.press(&game_settings(21), 21, 2);
        let mut players: Vec<(String, String)> = [
            "Boldworks",
            "Charlie",
            "H0nri",
            "P2",
            "P3",
            "P4",
            "P5",
            "P6",
        ]
        .iter()
        .map(|name| (name.to_string(), "Player".to_owned()))
        .collect();
        let (_, next) = h.set_roles(22, &mut players);
        assert_eq!(players[1].1, "BLUE");
        h.press(&game_settings(next), next, 5);
        h.close(&manage(next + 1), next + 1);
        players
    }

    #[test]
    fn a_new_opponent_sets_the_teams_again() {
        let mut h = Harness::new();
        let mut players = teams_on_temple(&mut h);

        // Misclicked Temple: open Hive's Map menu and pick Rome, where YELLOW faces RED.
        let buttons = open(&mut h, 60);
        let sent = pick(&mut h, &buttons, "Rome");
        assert!(sent.said("Setting the teams again"), "{:?}", sent.said);
        // Hive shows Game Settings next: on to Set Roles, everyone else to YELLOW.
        h.press(&game_settings(61), 61, 2);
        let (sent, next) = h.set_roles(62, &mut players);
        let teams: Vec<&str> = players.iter().map(|(_, role)| role.as_str()).collect();
        assert_eq!(
            teams,
            [
                "RED", "YELLOW", "RED", "YELLOW", "YELLOW", "YELLOW", "YELLOW", "YELLOW"
            ]
        );
        assert!(
            sent.said("RED Boldworks, H0nri | YELLOW Charlie"),
            "{:?}",
            sent.said
        );
        h.press(&game_settings(next), next, 5);
        h.close(&manage(next + 1), next + 1);
        assert!(h.server(&manage(next + 2)).0);
    }

    #[test]
    fn the_same_opponent_keeps_the_teams() {
        let mut h = Harness::new();
        teams_on_temple(&mut h);
        let buttons = open(&mut h, 60);
        // Autumn faces BLUE like Temple: nothing to move, and Game Settings is the player's.
        let sent = pick(&mut h, &buttons, "Autumn");
        assert_eq!(
            sent.answers,
            vec![(
                60,
                Some(
                    "1
"
                    .into()
                )
            )]
        );
        assert!(!sent.said("Setting the teams again"), "{:?}", sent.said);
        assert!(h.server(&game_settings(61)).0);
    }
}
