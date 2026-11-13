//! Whole-run tests: the map menu, creating the party, teams, presets and timeouts.

use super::*;

#[test]
fn autocs_shows_the_map_menu_and_starts_on_a_pick() {
    let mut h = Harness::new();
    let (kept, sent) = h.client(&chat("!autocs"));
    assert!(!kept, "!autocs must not reach Hive");
    let (form, json) = &sent.forms[0];
    assert_eq!(*form, PICKER_FORM);
    let temple = picker_buttons(json)
        .iter()
        .position(|(name, _, _)| name == "Temple")
        .unwrap();
    let mut answer = Vec::new();
    packets::form_response(&mut answer, PICKER_FORM, &format!("{temple}\n"));
    let (kept, sent) = h.client(&answer);
    assert!(
        !kept,
        "the answer to the mod's own menu must not reach Hive"
    );
    assert!(sent.said("Map Temple"));
    assert_eq!(sent.commands, vec![PARTY_COMMAND.to_string()]);
}

#[test]
fn closing_the_map_menu_cancels() {
    let mut h = Harness::new();
    h.client(&chat("!autocs"));
    let mut answer = Vec::new();
    packets::form_cancel(&mut answer, PICKER_FORM);
    h.client(&answer);
    let sent = h.run(100);
    assert!(sent.said("Cancelled"));
    assert!(sent.commands.is_empty());
}

#[test]
fn autocs_command_and_unknown_map() {
    let mut h = Harness::new();
    let (kept, sent) = h.client(&command("/autocs nowhere"));
    assert!(!kept);
    assert!(sent.said("No map called nowhere"));
    // An ambiguous prefix is refused too ("Hanging Gardens" has two variants upstream).
    let (_, sent) = h.client(&chat("!autocs hanging"));
    assert!(sent.said("No map called hanging"));
    let (_, sent) = h.client(&chat("!autocs soul"));
    assert!(sent.said("Soul's End"));
}

#[test]
fn look_alike_chat_and_commands_pass_through() {
    let mut h = Harness::new();
    for typed in ["!autocsx", "autocs", "hi !autocs", "!markers", "gg"] {
        assert!(h.client(&chat(typed)).0, "{typed}");
    }
    assert!(h.client(&command("/cs")).0);
    // Hive's menus are the player's while no run is going.
    assert!(h.server(&manage(5)).0);
}

#[test]
fn full_run_creates_sets_preset_map_and_teams() {
    let mut h = Harness::new();
    h.join_lobby("Party members (4): Boldworks, H0nri, Alpha_1, Bravo");
    let sent = h.preset_and_map();
    assert!(sent.said(&format!("Set preset {PRESET} and map Temple")));
    assert!(sent.said(&format!("When {FULL_LOBBY} players are in")));

    // Players join; the one that fills the lobby brings the reminder, no one before it.
    let others = ["Charlie", "P2", "P3", "P4", "P5", "P6", "P7"];
    for (joined, name) in (2..FULL_LOBBY).zip(others) {
        let (_, sent) = h.server(&system(&format!(
            "§a§l» §r§7{name} joined. §8[{joined}/16]"
        )));
        assert!(sent.said.is_empty(), "{:?}", sent.said);
    }
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7P7 joined. §8[{FULL_LOBBY}/16]"
    )));
    assert!(sent.said(&format!("{FULL_LOBBY}/{FULL_LOBBY} players are in")));

    // Right-click: Manage → Game Settings → Set Roles.
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
    assert_eq!(players.len() as u32, FULL_LOBBY);
    let (sent, next) = h.set_roles(22, &mut players);
    let teams: Vec<&str> = players.iter().map(|(_, role)| role.as_str()).collect();
    // Party members stay as they are; everyone else is set to BLUE.
    assert_eq!(
        teams,
        [
            "Player", "BLUE", "Player", "BLUE", "BLUE", "BLUE", "BLUE", "BLUE"
        ]
    );
    assert!(
        sent.said("party roles left unchanged | BLUE Charlie, P2"),
        "{:?}",
        sent.said
    );
    if !START_AFTER_TEAMS {
        assert!(
            sent.said("Teams are set. Start the game when you're ready"),
            "{:?}",
            sent.said
        );
    }
    // Closing Set Roles shows Game Settings, then Manage Your Server: back up, then start.
    h.press(&game_settings(next), next, 5); // Go Back
    if START_AFTER_TEAMS {
        let (kept, sent) = h.server(&manage(next + 1));
        assert!(!kept);
        assert_eq!(
            sent.answers,
            vec![(next + 1, Some("3\n".into()))],
            "Start Game"
        );
        assert!(sent.said("Starting the game"));
    } else {
        // Nothing more to say on the final close: "Teams are set" came with Set Roles.
        assert!(h.close(&manage(next + 1), next + 1).said.is_empty());
    }
    // Afterwards the menus are the player's again.
    assert!(h.server(&manage(next + 2)).0);
}

#[test]
fn players_who_join_the_party_after_autocs_are_not_put_on_blue() {
    let mut h = Harness::new();
    // Created before the party is full: only the user was in it.
    h.join_lobby("Party members (1): Boldworks");
    h.preset_and_map();

    // H0nri joins the party and the server later; seven others fill the lobby.
    let others = ["Charlie", "H0nri", "P2", "P3", "P4", "P5"];
    for (joined, name) in (2..FULL_LOBBY).zip(others) {
        h.server(&system(&format!(
            "§a§l» §r§7{name} joined. §8[{joined}/16]"
        )));
    }
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7P6 joined. §8[{FULL_LOBBY}/16]"
    )));
    // The lobby is full: the party is read again.
    assert_eq!(
        sent.commands,
        vec![PARTY_COMMAND.to_string()],
        "{:?}",
        sent.commands
    );
    // Join messages arriving meanwhile do not make anyone a party member.
    h.server(&system("§a§l» §r§7Charlie joined. §8[8/16]"));
    h.server(&system("Party members (2): Boldworks, H0nri"));
    h.run(40);

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
    let (sent, _) = h.set_roles(22, &mut players);
    let teams: Vec<&str> = players.iter().map(|(_, role)| role.as_str()).collect();
    assert_eq!(
        teams,
        [
            "Player", "BLUE", "Player", "BLUE", "BLUE", "BLUE", "BLUE", "BLUE"
        ]
    );
    assert!(
        sent.said("party roles left unchanged | BLUE Charlie, P2"),
        "{:?}",
        sent.said
    );
}

#[test]
fn an_access_menu_without_a_public_button_is_skipped_without_a_warp() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.press(&manage(5), 5, 2);
    h.press(&presets(6), 6, 4);
    h.press(&saved_presets(7), 7, 2);
    h.press(&load_preset(8), 8, 0);
    h.press(&presets(9), 9, 5);
    h.press(&manage(10), 10, 0);
    h.press(&game_settings(11), 11, 3);
    h.press(&maps(12), 12, 3);
    h.press(&game_settings(13), 13, 5);
    h.press(&manage(14), 14, 1);
    h.press(&access_permissions(15), 15, 2);
    // Hive words it some other way: go back, say so, and warp nobody into a closed server.
    let (kept, sent) = h.server(&menu(
        16,
        "Change Access Mode",
        "",
        &[("Set To Friends-Only Mode", ""), ("Go Back", "")],
    ));
    assert!(!kept);
    assert_eq!(
        sent.answers,
        vec![(
            16,
            Some(
                "1
"
                .into()
            )
        )]
    );
    assert!(
        sent.said("couldn't find the button that opens the server"),
        "{:?}",
        sent.said
    );
    h.press(&access_permissions(17), 17, 4);
    let sent = h.close(&manage(18), 18);
    assert!(sent.commands.is_empty(), "{:?}", sent.commands);
}

#[test]
fn more_than_a_full_lobby_does_not_start() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.preset_and_map();
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7Eighth joined. §8[{FULL_LOBBY}/16]"
    )));
    assert!(
        sent.said("Right-click the settings item"),
        "{:?}",
        sent.said
    );

    // A ninth player: the go-ahead is taken back and the right-click is the player's.
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7Ninth joined. §8[{}/16]",
        FULL_LOBBY + 1
    )));
    assert!(sent.said("too many to start"), "{:?}", sent.said);
    assert!(
        !sent.said("Right-click the settings item"),
        "{:?}",
        sent.said
    );
    assert!(
        h.server(&manage(20)).0,
        "no teams are set with {} players",
        FULL_LOBBY + 1
    );

    // One leaves: exactly full again, so it is asked for once more.
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7Ninth left. §8[{FULL_LOBBY}/16]"
    )));
    assert!(
        sent.said("Right-click the settings item"),
        "{:?}",
        sent.said
    );
}

#[test]
fn a_closed_menu_after_the_teams_is_the_players() {
    // If Hive does not show the parent menu soon after, the next menu is the player's.
    let mut h = Harness::new();
    h.join_lobby("");
    h.preset_and_map();
    h.client(&chat("!teams"));
    h.press(&manage(20), 20, 0);
    h.press(&game_settings(21), 21, 2);
    h.close(&roles(22, &[("Boldworks", "RED")]), 22);
    h.run(40);
    let (kept, _) = h.server(&game_settings(23));
    assert!(kept);
}

#[test]
fn party_names_match_whole_words_only() {
    let mut h = Harness::new();
    h.join_lobby("Party: Boldworks, Al");
    h.preset_and_map();
    h.client(&chat("!teams"));
    h.press(&manage(20), 20, 0);
    h.press(&game_settings(21), 21, 2);
    // "Alex" contains "al" but is not in the party.
    h.press(&roles(22, &[("Alex", "Player"), ("Al", "RED")]), 22, 0);
    h.press(&team_choice(23, "Alex"), 23, 1);
}

#[test]
fn a_refused_move_is_not_retried_forever() {
    let mut h = Harness::new();
    h.join_lobby("not in a party");
    h.preset_and_map();
    h.client(&chat("!teams"));
    h.press(&manage(20), 20, 0);
    h.press(&game_settings(21), 21, 2);
    let stuck = [("Charlie", "Player")];
    h.press(&roles(22, &stuck), 22, 0);
    h.press(&team_choice(23, "Charlie"), 23, 1);
    h.press(&roles(24, &stuck), 24, 0);
    h.press(&team_choice(25, "Charlie"), 25, 1);
    // Hive kept refusing: stop and close.
    h.close(&roles(26, &stuck), 26);
}

#[test]
fn a_missing_preset_is_skipped() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.press(&manage(5), 5, 2);
    h.press(&presets(6), 6, 4);
    let (_, sent) = h.server(&menu(
        7,
        "Saved Presets",
        "",
        &[("scrim prac", ""), ("Go Back", "")],
    ));
    assert!(sent.said("No saved preset called"));
    assert_eq!(sent.answers, vec![(7, Some("1\n".into()))]);
}

#[test]
fn an_unknown_menu_is_handed_to_the_player() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.press(&manage(5), 5, 2);
    let (kept, sent) = h.server(&menu(6, "Something New", "", &[("A", "")]));
    assert!(kept);
    assert!(sent.said("Something New"));
}

#[test]
fn a_party_menu_reply_is_read_and_closed() {
    let mut h = Harness::new();
    h.client(&chat("!autocs temple"));
    let (kept, sent) = h.server(&menu(9, "Your Party", "Members: H0nri", &[("Leave", "")]));
    assert!(!kept);
    assert_eq!(sent.answers, vec![(9, None)]);
}

#[test]
fn creation_times_out() {
    let mut h = Harness::new();
    h.client(&chat("!autocs temple"));
    let sent = h.run(40 + 301);
    assert!(sent.said("didn't open the custom server menu"));
}

#[test]
fn stop_cancels_a_run() {
    let mut h = Harness::new();
    h.client(&chat("!autocs temple"));
    let (kept, sent) = h.client(&chat("!autocs stop"));
    assert!(!kept);
    assert!(sent.said("Stopped"));
    assert!(h.run(60).commands.is_empty());
}

#[test]
fn mark_adds_a_note() {
    let mut h = Harness::new();
    let (kept, sent) = h.client(&chat("!mark picking the map"));
    assert!(!kept);
    assert!(sent.said("Note added"));
    assert_eq!(
        h.auto.take_notes(),
        vec!["MARK picking the map".to_string()]
    );
}

#[test]
fn game_over_replay_opens_map_picker_and_preserves_other_buttons() {
    let mut h = Harness::new();
    h.create("!autocs temple", "Party: Boldworks", 1);
    let overview = menu(
        900,
        "§5§lGame Over!",
        "Overview",
        &[("Close Overview", ""), ("Exit to Hub", "")],
    );
    let (kept, sent) = h.server(&overview);
    assert!(!kept);
    let displayed: serde_json::Value = serde_json::from_str(&sent.forms[0].1).unwrap();
    assert_eq!(displayed["buttons"].as_array().unwrap().len(), 3);
    let mut response = Vec::new();
    packets::form_response(&mut response, 900, "0\n");
    let (kept, sent) = h.client(&response);
    assert!(!kept);
    assert_eq!(sent.answers, vec![(900, None)]);
    assert_eq!(sent.forms[0].0, PICKER_FORM);
}

#[test]
fn picker_reopens_after_server_change_even_if_closed_during_transfer() {
    let mut h = Harness::new();
    h.client(&chat("!autocs"));
    let mut cancel = Vec::new();
    packets::form_cancel(&mut cancel, PICKER_FORM);
    h.client(&cancel);
    let mut batch = Vec::new();
    push_packet(&mut batch, crate::commands::AVAILABLE_COMMANDS, &[]);
    let (_, sent) = h.server(&batch);
    assert_eq!(sent.forms[0].0, PICKER_FORM);
    assert!(!h.run(100).said("Cancelled"));
}

#[test]
fn replay_is_absent_in_manually_managed_games_and_other_buttons_keep_their_meaning() {
    let overview = menu(
        900,
        "Game Over!",
        "Overview",
        &[("Close Overview", ""), ("Exit to Hub", "")],
    );
    let mut manual = Harness::new();
    manual.client(&chat("@Someone spec"));
    assert!(manual.server(&overview).0);
    let mut auto = Harness::new();
    auto.create("!autocs temple", "Party: Boldworks", 1);
    let (_, sent) = auto.server(&overview);
    let displayed: serde_json::Value = serde_json::from_str(&sent.forms[0].1).unwrap();
    assert!(
        displayed["buttons"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("Replay CS")
    );
    let mut response = Vec::new();
    packets::form_response(&mut response, 900, "2\n");
    let (kept, sent) = auto.client(&response);
    assert!(!kept);
    assert_eq!(sent.answers, vec![(900, Some("1\n".into()))]);
}

#[test]
fn game_over_screen_reopens_after_transfer_and_replay_opens_picker() {
    let mut h = Harness::new();
    h.create("!autocs temple", "Party: Boldworks", 1);
    h.server(&menu(
        900,
        "Game Over!",
        "Overview",
        &[("Close Overview", ""), ("Exit to Hub", "")],
    ));
    let mut batch = Vec::new();
    push_packet(&mut batch, crate::commands::AVAILABLE_COMMANDS, &[]);
    let (_, sent) = h.server(&batch);
    let (id, json) = &sent.forms[0];
    let displayed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(
        displayed["buttons"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("Replay CS")
    );
    let mut response = Vec::new();
    packets::form_response(&mut response, *id, "0\n");
    let (kept, sent) = h.client(&response);
    assert!(!kept);
    assert!(sent.answers.is_empty());
    assert_eq!(sent.forms[0].0, PICKER_FORM);
}

#[test]
fn game_over_message_immediately_offers_replay_before_hives_overview() {
    let mut h = Harness::new();
    h.create("!autocs temple", "Party: Boldworks", 1);
    let (_, sent) = h.server(&system("Game OVER!"));
    let (id, json) = &sent.forms[0];
    let displayed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(
        displayed["buttons"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("Replay CS")
    );
    let mut answer = Vec::new();
    packets::form_response(&mut answer, *id, "0\n");
    let (_, sent) = h.client(&answer);
    assert_eq!(sent.forms[0].0, PICKER_FORM);
    let (kept, _) = h.server(&menu(
        900,
        "Game Over!",
        "Overview",
        &[("Close Overview", "")],
    ));
    assert!(!kept);
}

/// Hive's end-of-game line as recorded in the log, with its `»` and colours.
const GAME_OVER_LINE: &str = "§c§l» §r§c§lGame OVER!";

#[test]
fn game_over_line_is_read_with_hives_symbol() {
    assert!(is_game_over(&strip(GAME_OVER_LINE)));
    assert!(is_game_over("Game OVER!"));
    assert!(is_game_over("? Game over!"));
    assert!(!is_game_over("» Magenta Team are the WINNERS!"));
    assert!(!is_game_over("» Game OVER! Really?"));
}

#[test]
fn hives_real_game_over_line_offers_the_replay_menu() {
    let mut h = Harness::new();
    h.create("!autocs temple", "Party: Boldworks", 1);
    let (_, sent) = h.server(&system(GAME_OVER_LINE));
    assert_eq!(sent.forms.len(), 1, "the replay menu");
    assert!(h.run(60).commands.is_empty());
}
