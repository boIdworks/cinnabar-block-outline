//! Invites, spectators and auto-spec.

use super::*;

mod invites_and_spectators {
    use super::*;

    /// Hive's invite and spectator menus, recorded on 2026-11-06 with WBSever hosting and
    /// Boldworks as the guest, by the form ID they had then.
    fn recorded(form: u32) -> Vec<u8> {
        let line = include_str!("../fixtures/hive-forms-invite-spec.tsv")
            .lines()
            .find(|line| line.split('\t').next() == Some(&form.to_string()))
            .unwrap_or_else(|| panic!("form {form} not recorded"));
        let mut batch = Vec::new();
        packets::form_request(&mut batch, form, line.split_once('\t').unwrap().1);
        batch
    }

    fn host() -> Harness {
        Harness::named("WBSever")
    }

    /// `!autocs temple` as WBSever up to arriving in the server.
    fn new_server(h: &mut Harness) -> Sent {
        h.create(
            "!autocs temple",
            "You have to be in a party to run this command.",
            1,
        )
    }

    #[test]
    fn spec_messages_are_read() {
        let mut h = host();
        for (typed, caught) in [
            ("@Boldworks spec", true),
            ("@boldworks SPEC", true),
            ("  @Some Name spec  ", true),
            ("@Boldworks specs", false),
            ("@ spec", false),
            ("hi @Boldworks spec", false),
            ("@Boldworks", false),
        ] {
            let (kept, _) = h.client(&chat(typed));
            assert_eq!(!kept, caught, "{typed}");
        }
    }

    #[test]
    fn a_hand_invite_is_remembered_and_sent_again_next_game() {
        let mut h = host();
        // First game: the player invites Boldworks by hand.
        let (kept, sent) = h.server(&system("§a§l» §rInvite sent to §eboldworks"));
        assert!(kept, "Hive's message still shows");
        assert!(sent.said("I'll invite boldworks to your next games too"));
        // The same invite again is not announced twice.
        assert!(
            h.server(&system("§a§l» §rInvite sent to §eboldworks"))
                .1
                .said
                .is_empty()
        );

        // Next game: preset, map, then the invite, through Hive's real menus.
        let sent = new_server(&mut h);
        assert!(sent.said("and invite boldworks"));
        h.press(&manage(5), 5, 2);
        h.press(&presets(6), 6, 4);
        h.press(&saved_presets(7), 7, 2);
        h.press(&load_preset(8), 8, 0);
        h.press(&presets(9), 9, 5);
        h.press(&manage(10), 10, 0);
        h.press(&game_settings(11), 11, 3);
        h.press(&maps(12), 12, 3);
        h.press(&game_settings(13), 13, 5); // Go Back to Manage
        h.press(&manage(14), 14, 1); // Access & Permissions
        h.press(&access_permissions(30), 30, 2); // Change Access Mode
        h.press(&access_mode(31), 31, 0); // Set To Open to All Mode
        h.press(&access_permissions_open(32), 32, 0); // Invite Player
        h.press(&recorded(9), 9, 1); // Invite Username
        let (kept, sent) = h.server(&recorded(10));
        assert!(!kept);
        assert_eq!(
            sent.answers,
            vec![(10, Some("[null,\"boldworks\"]\n".into()))]
        );
        // Hive confirms; that is the mod's own invite, not a new one to remember.
        assert!(
            h.server(&system("§a§l» §rInvite sent to §eboldworks"))
                .1
                .said
                .is_empty()
        );
        h.press(&recorded(11), 11, 2); // Invite Player again: Go Back
        h.press(&access_permissions_open(33), 33, 3); // Access & Permissions: Go Back
        let sent = h.close(&manage(15), 15);
        assert!(sent.said("Set preset new scrim rules and map Temple"));
        assert!(sent.said("Invited boldworks"));
        assert!(
            sent.said("Opened the server to everyone"),
            "{:?}",
            sent.said
        );
        assert_eq!(sent.commands, vec![WARP_COMMAND.to_string()]);
    }

    #[test]
    fn an_invite_by_another_player_is_remembered_too() {
        let mut h = host();
        // Hive's message when a co-host invites someone (recorded 2026-11-06).
        let (kept, sent) = h.server(&system("§a§l» §r§eSuperHason §rsent an invite to §eKeizYT"));
        assert!(kept, "Hive's message still shows");
        assert!(
            sent.said("I'll invite KeizYT (invited by SuperHason) to your next games too"),
            "{:?}",
            sent.said
        );
        assert_eq!(
            h.auto.take_notes().last().unwrap(),
            "AUTOCS remember invite \"KeizYT\" by=Some(\"SuperHason\")"
        );
        // Not announced twice, and not added again when the host invites the same player.
        assert!(
            h.server(&system("§a§l» §r§eSuperHason §rsent an invite to §eKeizYT"))
                .1
                .said
                .is_empty()
        );
        assert!(
            h.server(&system("§a§l» §rInvite sent to §ekeizyt"))
                .1
                .said
                .is_empty()
        );
        // The next game invites them.
        let sent = new_server(&mut h);
        assert!(sent.said("and invite KeizYT"), "{:?}", sent.said);
    }

    #[test]
    fn end_works_with_slips_of_the_command() {
        for typed in ["/auto end", "/autocsto end", "/AutoCS END", "/autocs  end"] {
            let mut h = host();
            h.server(&system("Invite sent to H0nri"));
            let (kept, sent) = h.client(&command(typed));
            assert!(!kept, "{typed}");
            assert!(
                sent.said("I won't invite H0nri again"),
                "{typed}: {:?}",
                sent.said
            );
        }
    }

    #[test]
    fn end_stops_a_run_in_the_server() {
        let mut h = Harness::new();
        h.join_lobby("");
        let (_, sent) = h.client(&command("/autocs end"));
        assert!(sent.said("Game over"), "{:?}", sent.said);
        // The settings menus are the player's again.
        assert!(h.server(&manage(5)).0);
    }

    #[test]
    fn end_forgets_the_invites() {
        let mut h = host();
        h.server(&system("Invite sent to boldworks"));
        h.server(&system("Invite sent to H0nri"));
        let (kept, sent) = h.client(&command("/autocs end"));
        assert!(!kept);
        assert!(sent.said("I won't invite boldworks, H0nri again"));
        let sent = new_server(&mut h);
        assert!(!sent.said("and invite"));
        // With nobody to invite, Access & Permissions is opened only to open the server.
        h.press(&manage(5), 5, 2);
        h.press(&presets(6), 6, 4);
        h.press(&saved_presets(7), 7, 2);
        h.press(&load_preset(8), 8, 0);
        h.press(&presets(9), 9, 5);
        h.press(&manage(10), 10, 0);
        h.press(&game_settings(11), 11, 3);
        h.press(&maps(12), 12, 3);
        h.press(&game_settings(13), 13, 5);
        h.open_to_all(&manage(14), 14, 15);
    }

    #[test]
    fn invites_survive_a_stopped_run() {
        let mut h = host();
        h.server(&system("Invite sent to boldworks"));
        h.client(&chat("!autocs temple"));
        h.client(&chat("!autocs stop"));
        let sent = new_server(&mut h);
        assert!(sent.said("and invite boldworks"));
    }

    #[test]
    fn spec_makes_the_player_a_spectator_through_hives_menus() {
        let mut h = host();
        // A server made by hand: the mod is idle until the spec request.
        let (kept, sent) = h.client(&chat("@Boldworks spec"));
        assert!(!kept, "the request is not sent to Hive's chat");
        assert!(sent.said("Right-click the settings item to make Boldworks a spectator"));
        h.press(&manage(5), 5, 0); // Game Settings
        h.press(&game_settings(6), 6, 2); // Set Roles
        h.press(&recorded(15), 15, 0); // Boldworks
        h.press(&recorded(16), 16, 4); // Spectator
        let sent = h.close(&recorded(17), 17); // Boldworks / Spectator: done
        assert!(sent.said("Boldworks is spectating"));
        // Hive shows Game Settings and then Manage Your Server as the menu closes.
        h.press(&game_settings(18), 18, 5); // Go Back
        h.close(&manage(19), 19);
        // Nothing left: the next right-click is the player's.
        assert!(h.server(&manage(20)).0);
    }

    #[test]
    fn spec_for_someone_not_here_is_reported() {
        let mut h = host();
        h.client(&chat("@Nobody spec"));
        h.press(&manage(5), 5, 0);
        h.press(&game_settings(6), 6, 2);
        let sent = h.close(&recorded(15), 15);
        assert!(sent.said("Nobody isn't in the server"));
    }

    #[test]
    fn spectators_stay_spectators_when_teams_are_set() {
        let mut h = Harness::new();
        h.join_lobby("Party: Boldworks, H0nri");
        h.preset_and_map();
        h.client(&chat("@H0nri spec"));
        h.client(&chat("!teams"));
        h.press(&manage(20), 20, 0);
        h.press(&game_settings(21), 21, 2);
        // H0nri is in the party, but asked to spectate: Spectator, not RED. The other eight play.
        let mut players: Vec<(String, String)> = [
            "Boldworks",
            "H0nri",
            "Charlie",
            "P2",
            "P3",
            "P4",
            "P5",
            "P6",
            "P7",
        ]
        .iter()
        .map(|name| (name.to_string(), "Player".to_owned()))
        .collect();
        let (sent, _) = h.set_roles(22, &mut players);
        assert_eq!(players[0].1, "Player");
        assert_eq!(players[1].1, "Spectator");
        assert_eq!(players[2].1, "BLUE");
        assert!(sent.said("H0nri is spectating"), "{:?}", sent.said);
        assert!(
            sent.said("party roles left unchanged | BLUE Charlie"),
            "{:?}",
            sent.said
        );
    }
}

#[test]
fn a_spectator_leaving_stops_counting() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.preset_and_map();
    h.client(&chat("@Watcher spec"));
    h.press(&manage(20), 20, 0);
    h.press(&game_settings(21), 21, 2);
    h.press(
        &roles(22, &[("Boldworks", "Player"), ("Watcher", "Player")]),
        22,
        1,
    );
    h.press(&team_choice(23, "Watcher"), 23, 4);
    h.close(
        &roles(24, &[("Boldworks", "Player"), ("Watcher", "Spectator")]),
        24,
    );
    h.server(&system("§a§l» §r§7Watcher left. §8[1/16]"));
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7P2 joined. §8[{FULL_LOBBY}/16]"
    )));
    assert!(
        sent.said(&format!("{FULL_LOBBY}/{FULL_LOBBY} players are in.")),
        "{:?}",
        sent.said
    );
}

#[test]
fn a_spectator_set_with_the_teams_holds_the_start() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.preset_and_map();
    // The lobby fills, and one of them asks to spectate in the same right-click.
    h.server(&system(&format!(
        "§a§l» §r§7Watcher joined. §8[{FULL_LOBBY}/16]"
    )));
    h.client(&chat("@Watcher spec"));
    h.press(&manage(20), 20, 0);
    h.press(&game_settings(21), 21, 2);
    let mut players: Vec<(String, &str)> = vec![("Boldworks".into(), "RED")];
    players.extend((2..FULL_LOBBY).map(|n| (format!("P{n}"), "BLUE")));
    players.push(("Watcher".into(), "Player"));
    fn list<'a>(players: &'a [(String, &'a str)]) -> Vec<(&'a str, &'a str)> {
        players
            .iter()
            .map(|(name, role)| (name.as_str(), *role))
            .collect()
    }
    let last = players.len() - 1;
    h.press(&roles(22, &list(&players)), 22, last);
    h.press(&team_choice(23, "Watcher"), 23, 4);
    players[last].1 = "Spectator";
    let sent = h.close(&roles(24, &list(&players)), 24);
    assert!(
        sent.said("waiting for 1 more player, so the teams aren't finished"),
        "{:?}",
        sent.said
    );
    // Said once: no "Teams:" line for unfinished teams, and no second status line.
    assert!(!sent.said("Teams:"), "{:?}", sent.said);
    assert!(!sent.said("When "), "{:?}", sent.said);
    // Back up the menus: nothing is started.
    h.press(&game_settings(25), 25, 5);
    let sent = h.close(&manage(26), 26);
    assert!(!sent.said("Starting"));
    assert!(
        sent.said.is_empty(),
        "the wait was said once: {:?}",
        sent.said
    );
    // The missing player arrives: right-click again, they get a team, and the game starts.
    let (_, sent) = h.server(&system(&format!(
        "§a§l» §r§7Newcomer joined. §8[{}/16]",
        FULL_LOBBY + 1
    )));
    assert!(
        sent.said("players are in (+1 spectating)"),
        "{:?}",
        sent.said
    );
    h.press(&manage(27), 27, 0);
    h.press(&game_settings(28), 28, 2);
    players.push(("Newcomer".into(), "Player"));
    h.press(&roles(29, &list(&players)), 29, players.len() - 1);
    h.press(&team_choice(30, "Newcomer"), 30, 1);
    players.last_mut().unwrap().1 = "BLUE";
    h.close(&roles(31, &list(&players)), 31);
    h.press(&game_settings(32), 32, 5);
    if START_AFTER_TEAMS {
        let (_, sent) = h.server(&manage(33));
        assert!(sent.said("Starting the game"));
    }
}

// ---- spectators remembered across games ---------------------------------------------------------

mod auto_spec {
    use super::*;

    /// `@Watcher spec` in a first server, applied through Set Roles.
    fn spectated_once() -> Harness {
        let mut h = Harness::new();
        h.join_lobby("");
        h.preset_and_map();
        h.client(&chat("@Watcher spec"));
        h.press(&manage(20), 20, 0);
        h.press(&game_settings(21), 21, 2);
        h.press(
            &roles(22, &[("Boldworks", "Player"), ("Watcher", "Player")]),
            22,
            1,
        );
        h.press(&team_choice(23, "Watcher"), 23, 4);
        h.server(&system("§a§l» §rSet §eWatcher's§r role to §bSpectator"));
        h.close(
            &roles(24, &[("Boldworks", "Player"), ("Watcher", "Spectator")]),
            24,
        );
        h.press(&game_settings(25), 25, 5);
        h.close(&manage(26), 26);
        h
    }

    /// A new `!autocs` server, joined.
    fn next_server(h: &mut Harness) {
        h.create(
            "!autocs temple",
            "You have to be in a party to run this command.",
            1,
        );
        h.preset_and_map();
    }

    #[test]
    fn a_spectator_spectates_again_in_the_next_server() {
        let mut h = spectated_once();
        next_server(&mut h);
        // Watcher joins: announced, and not counted as a player.
        let (_, sent) = h.server(&system("§a§l» §r§7Watcher joined. §8[2/16]"));
        assert!(
            sent.said("Watcher will spectate, as in your last game"),
            "{:?}",
            sent.said
        );
        let full = FULL_LOBBY + 1;
        for n in 3..full {
            let (_, sent) = h.server(&system(&format!("§a§l» §r§7P{n} joined. §8[{n}/16]")));
            assert!(!sent.said("players are in"), "{n}");
        }
        let (_, sent) = h.server(&system(&format!("§a§l» §r§7Last joined. §8[{full}/16]")));
        assert!(
            sent.said(&format!(
                "{FULL_LOBBY}/{FULL_LOBBY} players are in (+1 spectating)"
            )),
            "{:?}",
            sent.said
        );
        // The teams pass makes Watcher a spectator with no @ request.
        h.press(&manage(30), 30, 0);
        h.press(&game_settings(31), 31, 2);
        let mut players: Vec<(String, &str)> =
            vec![("Boldworks".into(), "RED"), ("Watcher".into(), "Player")];
        players.extend((3..=full).map(|n| {
            (
                if n == full {
                    "Last".to_string()
                } else {
                    format!("P{n}")
                },
                "BLUE",
            )
        }));
        let list = |players: &[(String, &str)]| -> Vec<(String, String)> {
            players
                .iter()
                .map(|(name, role)| (name.clone(), role.to_string()))
                .collect()
        };
        let owned = list(&players);
        let borrowed: Vec<(&str, &str)> = owned
            .iter()
            .map(|(n, r)| (n.as_str(), r.as_str()))
            .collect();
        h.press(&roles(32, &borrowed), 32, 1);
        h.press(&team_choice(33, "Watcher"), 33, 4);
        players[1].1 = "Spectator";
        let owned = list(&players);
        let borrowed: Vec<(&str, &str)> = owned
            .iter()
            .map(|(n, r)| (n.as_str(), r.as_str()))
            .collect();
        let sent = h.close(&roles(34, &borrowed), 34);
        assert!(!sent.said("aren't finished"), "{:?}", sent.said);
    }

    #[test]
    fn set_back_to_a_player_by_hand_they_play() {
        let mut h = spectated_once();
        let (_, sent) = h.server(&system("§a§l» §rSet §eWatcher's§r role to §bRED"));
        assert!(sent.said("Watcher is playing now"));
        next_server(&mut h);
        let (_, sent) = h.server(&system("§a§l» §r§7Watcher joined. §8[2/16]"));
        assert!(!sent.said("will spectate"));
        assert!(
            !sent.said("spectating"),
            "Watcher counts as a player: {:?}",
            sent.said
        );
    }

    #[test]
    fn a_spectator_set_by_hand_is_remembered() {
        let mut h = Harness::new();
        let (_, sent) = h.server(&system("§a§l» §rSet §eViewer's§r role to §bSpectator"));
        assert!(sent.said("I'll keep Viewer spectating in your next games"));
        next_server(&mut h);
        let (_, sent) = h.server(&system("§a§l» §r§7Viewer joined. §8[2/16]"));
        assert!(sent.said("Viewer will spectate"));
    }

    #[test]
    fn end_forgets_the_spectators() {
        let mut h = spectated_once();
        h.server(&system("Invite sent to Pal"));
        let (_, sent) = h.client(&chat("!autocs end"));
        assert!(
            sent.said("I won't invite Pal again, and Watcher won't be made spectators"),
            "{:?}",
            sent.said
        );
        next_server(&mut h);
        let (_, sent) = h.server(&system("§a§l» §r§7Watcher joined. §8[2/16]"));
        assert!(!sent.said("will spectate"));
    }

    #[test]
    fn role_messages_are_read() {
        let mut h = Harness::new();
        // Map messages look alike but are not role changes.
        let (_, sent) = h.server(&system("§a§l» §rSet §eMap §rto §bTemple"));
        assert!(sent.said.is_empty());
    }
}

#[test]
fn a_spectator_back_to_playing_fills_the_lobby_and_the_mod_takes_over() {
    let mut h = Harness::new();
    h.join_lobby("");
    h.preset_and_map();
    // FULL_LOBBY in the server, one of them a spectator: the teams wait.
    h.server(&system(&format!(
        "§a§l» §r§7Watcher joined. §8[{FULL_LOBBY}/16]"
    )));
    h.client(&chat("@Watcher spec"));
    h.press(&manage(20), 20, 0);
    h.press(&game_settings(21), 21, 2);
    let mut players: Vec<(String, &str)> = vec![("Boldworks".into(), "RED")];
    players.extend((2..FULL_LOBBY).map(|n| (format!("P{n}"), "BLUE")));
    players.push(("Watcher".into(), "Player"));
    fn list<'a>(players: &'a [(String, &'a str)]) -> Vec<(&'a str, &'a str)> {
        players
            .iter()
            .map(|(name, role)| (name.as_str(), *role))
            .collect()
    }
    let watcher = players.len() - 1;
    h.press(&roles(22, &list(&players)), 22, watcher);
    h.press(&team_choice(23, "Watcher"), 23, 4);
    h.server(&system("§a§l» §rSet §eWatcher's§r role to §bSpectator"));
    players[watcher].1 = "Spectator";
    assert!(
        h.close(&roles(24, &list(&players)), 24)
            .said("aren't finished")
    );
    h.press(&game_settings(25), 25, 5);
    h.close(&manage(26), 26);

    // The player opens the menus by hand and sets Watcher back to RED.
    assert!(h.server(&manage(27)).0);
    assert!(h.server(&game_settings(28)).0);
    assert!(h.server(&roles(29, &list(&players))).0);
    assert!(h.server(&team_choice(30, "Watcher")).0);
    players[watcher].1 = "RED";
    // Hive confirms and shows the list again: now the mod's.
    let mut batch = system("§a§l» §rSet §eWatcher's§r role to §bRED");
    batch.extend(roles(31, &list(&players)));
    let (kept, sent) = h.server(&batch);
    assert!(!kept, "the list is the mod's");
    assert!(sent.said("Watcher is playing now"));
    assert!(
        sent.said(&format!(
            "{FULL_LOBBY}/{FULL_LOBBY} players are in. Setting the teams now"
        )),
        "{:?}",
        sent.said
    );
    // Watcher is not in the party: moved to the map's team (BLUE on Temple).
    assert_eq!(sent.answers, vec![(31, Some(format!("{watcher}\n")))]);
    h.press(&team_choice(32, "Watcher"), 32, 1);
    players[watcher].1 = "BLUE";
    h.close(&roles(33, &list(&players)), 33);
    h.press(&game_settings(34), 34, 5);
    if START_AFTER_TEAMS {
        let (_, sent) = h.server(&manage(35));
        assert_eq!(sent.answers, vec![(35, Some("3\n".into()))]);
        assert!(sent.said("Starting the game"));
    }
}
