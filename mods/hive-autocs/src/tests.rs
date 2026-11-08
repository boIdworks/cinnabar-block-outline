use serde_json::json;

use crate::autocs::{
    AutoCs, FULL_LOBBY, PARTY_COMMAND, PICKER_FORM, PRESET, START_AFTER_TEAMS, WARP_COMMAND, strip,
};
use crate::packets::{self, TEXT, TEXT_CHAT, TEXT_RAW};
use crate::recorder::{MORE, Recorder, TAG, Way};
use crate::wire::{Reader, push_packet, split_batch, write_string, write_var_u32};

// ---- packet builders ----------------------------------------------------------------------------

fn text(kind: u8, author: Option<&str>, message: &str) -> Vec<u8> {
    let mut body = vec![0];
    write_var_u32(&mut body, if author.is_some() { 1 } else { 0 });
    body.push(kind);
    if let Some(author) = author {
        write_string(&mut body, author);
    }
    write_string(&mut body, message);
    write_string(&mut body, "");
    write_string(&mut body, "");
    body.push(0);
    let mut batch = Vec::new();
    push_packet(&mut batch, TEXT, &body);
    batch
}

fn chat(message: &str) -> Vec<u8> {
    text(TEXT_CHAT, Some("Boldworks"), message)
}

fn system(message: &str) -> Vec<u8> {
    text(TEXT_RAW, None, message)
}

fn command(line: &str) -> Vec<u8> {
    let mut batch = Vec::new();
    packets::command_request(&mut batch, line, 1);
    batch
}

/// A Hive button menu as Hive formats it: bold coloured names, a detail line.
fn menu(form: u32, title: &str, content: &str, buttons: &[(&str, &str)]) -> Vec<u8> {
    let buttons: Vec<_> = buttons
        .iter()
        .map(|(name, detail)| json!({ "text": format!("§5§l{name}\n§r§8{detail}") }))
        .collect();
    let json = json!({ "type": "form", "title": title, "content": content, "buttons": buttons });
    let mut batch = Vec::new();
    packets::form_request(&mut batch, form, &json.to_string());
    batch
}

fn manage(form: u32) -> Vec<u8> {
    menu(
        form,
        "Manage Your Server",
        "Select from the options below",
        &[
            ("Game Settings", "All customisation items"),
            ("Access & Permissions", ""),
            ("Manage Presets", "Load or save presets"),
            ("Start Game", "Start the game"),
            ("Reset Settings", ""),
            ("Kick Player", ""),
            ("Stop Server", ""),
        ],
    )
}

/// Hive's access menus as recorded on 2026-11-06, shown with the form ID `form`: Access &
/// Permissions as an invite-only server has it (145), Change Access Mode (146), and Access &
/// Permissions once the server is open to all (147), which has no Access List button.
fn access_recorded(recorded: u32, form: u32) -> Vec<u8> {
    let line = include_str!("fixtures/hive-forms-access.tsv")
        .lines()
        .find(|line| line.split('\t').next() == Some(&recorded.to_string()))
        .unwrap_or_else(|| panic!("form {recorded} not recorded"));
    let mut batch = Vec::new();
    packets::form_request(&mut batch, form, line.split_once('\t').unwrap().1);
    batch
}

fn access_permissions(form: u32) -> Vec<u8> {
    access_recorded(145, form)
}

fn access_mode(form: u32) -> Vec<u8> {
    access_recorded(146, form)
}

fn access_permissions_open(form: u32) -> Vec<u8> {
    access_recorded(147, form)
}

fn game_settings(form: u32) -> Vec<u8> {
    menu(
        form,
        "Game Settings",
        "Here you can customise your server!",
        &[
            ("Maximum Players", ""),
            ("World Time", ""),
            ("Set Roles", "Override selection"),
            ("Map", "Override map voting"),
            ("Bots", ""),
            ("Go Back", ""),
        ],
    )
}

fn presets(form: u32) -> Vec<u8> {
    menu(
        form,
        "Preset Settings",
        "Load, save or create a new preset!",
        &[
            ("Featured Presets", ""),
            ("Load Preset By Code", ""),
            ("Save Preset", ""),
            ("Recent Presets", ""),
            ("Saved Presets", ""),
            ("Go Back", ""),
        ],
    )
}

fn saved_presets(form: u32) -> Vec<u8> {
    menu(
        form,
        "Saved Presets",
        "View all your saved presets for BedWars.",
        &[
            ("midfight no regen", "AuUfBH"),
            ("midfight regen", "6msmhs"),
            ("new scrim rules", "k9DkoS"),
            ("scrim prac", "g2m0OS"),
            ("Go Back", ""),
        ],
    )
}

fn load_preset(form: u32) -> Vec<u8> {
    menu(
        form,
        "Load Preset?",
        "You are about to load the following preset:\n\n§d§lnew scrim rules\n§r§6Shop Settings",
        &[
            ("Confirm", "This will overwrite"),
            ("Delete Preset", ""),
            ("Go Back", ""),
        ],
    )
}

fn maps(form: u32) -> Vec<u8> {
    menu(
        form,
        "Map",
        "This menu allows you to select any available map.",
        &[
            ("Random", "Selected"),
            ("Autumn", "Select to equip"),
            ("Soul's End", "Select to equip"),
            ("Temple", "Select to equip"),
        ],
    )
}

/// The Set Roles player list: each player with their current role.
fn roles(form: u32, players: &[(&str, &str)]) -> Vec<u8> {
    let buttons: Vec<_> = players
        .iter()
        .map(|(name, role)| json!({ "text": format!("{name}§r\n§0{role}") }))
        .collect();
    let json = json!({
        "type": "form", "title": "Set Roles",
        "content": "You can manually assign players to roles and teams here.",
        "buttons": buttons,
    });
    let mut batch = Vec::new();
    packets::form_request(&mut batch, form, &json.to_string());
    batch
}

fn team_choice(form: u32, player: &str) -> Vec<u8> {
    // As Hive shows it with other players in the server: the teams, Spectator, Player.
    let buttons: Vec<_> = ["RED", "BLUE", "YELLOW", "GREEN"]
        .iter()
        .map(|team| json!({ "text": format!("§0{team}\n§8Click to assign") }))
        .chain([json!({ "text": "Spectator" }), json!({ "text": "Player" })])
        .collect();
    let json = json!({
        "type": "form", "title": "Set Roles",
        "content": format!("Setting role for {player}."),
        "buttons": buttons,
    });
    let mut batch = Vec::new();
    packets::form_request(&mut batch, form, &json.to_string());
    batch
}

// ---- driving the automation ---------------------------------------------------------------------

/// What one step sent: messages shown to the player and packets to the server.
#[derive(Default, Debug)]
struct Sent {
    said: Vec<String>,
    forms: Vec<(u32, String)>,
    commands: Vec<String>,
    /// Form answers: form, response (none: closed).
    answers: Vec<(u32, Option<String>)>,
}

impl Sent {
    fn said(&self, part: &str) -> bool {
        self.said.iter().any(|line| line.contains(part))
    }
}

struct Harness {
    auto: AutoCs,
    me: &'static str,
}

impl Harness {
    fn new() -> Self {
        Self::named("Boldworks")
    }

    fn named(me: &'static str) -> Self {
        Self {
            auto: AutoCs::new(me),
            me,
        }
    }

    /// Feeds a batch; returns whether each packet was kept, then runs ticks until the answers
    /// are out.
    fn client(&mut self, batch: &[u8]) -> (bool, Sent) {
        let packets = split_batch(batch).unwrap();
        let kept = packets.iter().all(|packet| !self.auto.from_client(packet));
        (kept, self.run(5))
    }

    fn server(&mut self, batch: &[u8]) -> (bool, Sent) {
        let packets = split_batch(batch).unwrap();
        let kept = packets.iter().all(|packet| !self.auto.from_server(packet));
        (kept, self.run(5))
    }

    fn run(&mut self, ticks: u32) -> Sent {
        let mut sent = Sent::default();
        for _ in 0..ticks {
            let (to_client, to_server) = self.auto.tick();
            for packet in split_batch(&to_client).unwrap() {
                match packet.id {
                    TEXT => sent
                        .said
                        .push(strip(&packets::read_text(&packet).unwrap().message)),
                    packets::MODAL_FORM_REQUEST => sent
                        .forms
                        .push(packets::read_form_request(&packet).unwrap()),
                    other => panic!("unexpected packet {other} to the client"),
                }
            }
            for packet in split_batch(&to_server).unwrap() {
                match packet.id {
                    packets::COMMAND_REQUEST => {
                        sent.commands.push(packets::read_command(&packet).unwrap())
                    }
                    packets::MODAL_FORM_RESPONSE => {
                        let (form, response, _) = packets::read_form_response(&packet).unwrap();
                        sent.answers.push((form, response));
                    }
                    other => panic!("unexpected packet {other} to the server"),
                }
            }
        }
        sent
    }

    /// A Hive menu the mod must answer by pressing `button`.
    fn press(&mut self, batch: &[u8], form: u32, button: usize) {
        let (kept, sent) = self.server(batch);
        assert!(!kept, "form {form} should be hidden from the player");
        assert_eq!(
            sent.answers,
            vec![(form, Some(format!("{button}\n")))],
            "form {form}"
        );
    }

    /// A Hive menu the mod must close.
    fn close(&mut self, batch: &[u8], form: u32) -> Sent {
        let (kept, sent) = self.server(batch);
        assert!(!kept, "form {form} should be hidden from the player");
        assert_eq!(sent.answers, vec![(form, None)], "form {form}");
        sent
    }

    /// Manage Your Server (`manage_menu`, with `form`) → Access & Permissions → Change Access
    /// Mode → Public → back out: the server is open to all and `/p warp` goes out. Uses form ids
    /// from `next`. Returns what the final close said.
    fn open_to_all(&mut self, manage_menu: &[u8], form: u32, next: u32) -> Sent {
        self.press(manage_menu, form, 1);
        self.press(&access_permissions(next), next, 2);
        self.press(&access_mode(next + 1), next + 1, 0); // Set To Open to All Mode
        self.press(&access_permissions_open(next + 2), next + 2, 3); // Go Back
        let sent = self.close(&manage(next + 3), next + 3);
        assert_eq!(sent.commands, vec![WARP_COMMAND.to_string()]);
        assert!(
            sent.said("Opened the server to everyone"),
            "{:?}",
            sent.said
        );
        sent
    }

    /// Plays Hive's Set Roles menus for `players` (name, role) until the mod closes the list,
    /// giving each button the mod presses its role. Form ids count up from `first`. Returns what
    /// closing the list said, and the next unused form id.
    fn set_roles(&mut self, first: u32, players: &mut [(String, String)]) -> (Sent, u32) {
        const ROLES: [&str; 6] = ["RED", "BLUE", "YELLOW", "GREEN", "Spectator", "Player"];
        let pressed = |sent: &Sent, form: u32| -> Option<usize> {
            match sent.answers.as_slice() {
                [(id, Some(answer))] if *id == form => Some(answer.trim().parse().unwrap()),
                [(id, None)] if *id == form => None,
                other => panic!("form {form}: {other:?}"),
            }
        };
        let mut form = first;
        loop {
            let list: Vec<(&str, &str)> = players
                .iter()
                .map(|(name, role)| (name.as_str(), role.as_str()))
                .collect();
            let (kept, sent) = self.server(&roles(form, &list));
            assert!(!kept, "form {form} should be hidden from the player");
            let Some(index) = pressed(&sent, form) else {
                return (sent, form + 1);
            };
            form += 1;
            let (kept, sent) = self.server(&team_choice(form, &players[index].0));
            assert!(!kept, "form {form} should be hidden from the player");
            players[index].1 = ROLES[pressed(&sent, form).unwrap()].to_owned();
            form += 1;
        }
    }

    /// Runs `!autocs temple` up to arriving in the server, with `party` as the party reply.
    /// Runs `typed` (an `!autocs` with a map) up to arriving in the new server with `lobby`
    /// players in it, `party` being the party command's reply. Returns what arriving said.
    fn create(&mut self, typed: &str, party: &str, lobby: u32) -> Sent {
        let (kept, sent) = self.client(&chat(typed));
        assert!(!kept);
        assert_eq!(sent.commands, vec![PARTY_COMMAND.to_string()]);
        self.server(&system(party));
        let sent = self.run(40);
        assert_eq!(sent.commands, vec!["/cs".to_string()]);
        self.press(
            &menu(
                2,
                "Custom Server Menu",
                "",
                &[("Create Server", "Choose a game"), ("Your Invites", "")],
            ),
            2,
            0,
        );
        self.press(
            &menu(
                3,
                "Create Server",
                "",
                &[("BedWars", ""), ("BlockDrop", ""), ("BedWars Hub", "")],
            ),
            3,
            0,
        );
        self.press(
            &menu(
                4,
                "Create Server",
                "",
                &[
                    ("Solos / Regular", ""),
                    ("Duos", ""),
                    ("Squads", ""),
                    ("Go back", ""),
                ],
            ),
            4,
            2,
        );
        let me = self.me;
        self.server(&system(&format!("§a§l» §r§7{me} joined. §8[{lobby}/16]")))
            .1
    }

    /// `!autocs temple` up to arriving, alone, in the server.
    fn join_lobby(&mut self, party: &str) {
        let sent = self.create("!autocs temple", party, 1);
        assert!(sent.said("Right-click the settings item"));
    }

    /// Answers the settings menus for the preset and map, as recorded.
    fn preset_and_map(&mut self) -> Sent {
        self.press(&manage(5), 5, 2);
        self.press(&presets(6), 6, 4);
        self.press(&saved_presets(7), 7, 2);
        self.press(&load_preset(8), 8, 0);
        self.press(&presets(9), 9, 5);
        self.press(&manage(10), 10, 0);
        self.press(&game_settings(11), 11, 3);
        self.press(&maps(12), 12, 3);
        self.press(&game_settings(13), 13, 5);
        self.open_to_all(&manage(14), 14, 15)
    }
}

// ---- wire ---------------------------------------------------------------------------------------

#[test]
fn var_u32_round_trips() {
    for value in [0, 1, 127, 128, 300, 16_383, 16_384, u32::MAX] {
        let mut out = Vec::new();
        write_var_u32(&mut out, value);
        assert_eq!(Reader::new(&out).var_u32(), Some(value));
    }
}

#[test]
fn system_message_decodes_as_text() {
    let mut batch = Vec::new();
    packets::system_message(&mut batch, "hello");
    let packets = split_batch(&batch).unwrap();
    assert_eq!(packets[0].id, TEXT);
    let text = packets::read_text(&packets[0]).unwrap();
    assert_eq!((text.message.as_str(), text.author), ("hello", None));
}

#[test]
fn form_packets_decode() {
    let mut batch = Vec::new();
    packets::form_request(&mut batch, 7, r#"{"type":"form"}"#);
    packets::form_response(&mut batch, 7, "2\n");
    packets::form_cancel(&mut batch, 7);
    let packets = split_batch(&batch).unwrap();
    assert_eq!(
        packets::read_form_request(&packets[0]),
        Some((7, r#"{"type":"form"}"#.into()))
    );
    assert_eq!(
        packets::read_form_response(&packets[1]),
        Some((7, Some("2\n".into()), None))
    );
    // As the recording shows for a closed menu: no response, reason 0.
    assert_eq!(
        packets::read_form_response(&packets[2]),
        Some((7, None, Some(0)))
    );
}

#[test]
fn command_request_layout() {
    let batch = command("/cs");
    let packets = split_batch(&batch).unwrap();
    let mut body = packets[0].body();
    assert_eq!(body.string().as_deref(), Some("/cs"));
    assert_eq!(body.string().as_deref(), Some("player"));
    let uuid = body.take(16).unwrap();
    assert_eq!(uuid[6] >> 4, 4);
    assert_eq!(body.string().as_deref(), Some(""));
    assert_eq!(body.take(8), Some(&[0u8; 8][..]));
    assert_eq!(body.bool(), Some(false));
    assert_eq!(body.string().as_deref(), Some("latest"));
    assert!(body.rest().is_empty());
}

#[test]
fn header_sub_client_bits_are_ignored() {
    let mut batch = Vec::new();
    push_packet(&mut batch, 100 | (1 << 10) | (2 << 12), &[]);
    assert_eq!(split_batch(&batch).unwrap()[0].id, 100);
}

#[test]
fn truncated_batch_is_unreadable() {
    let mut batch = chat("hi");
    batch.pop();
    assert!(split_batch(&batch).is_none());
}

// ---- recorder -----------------------------------------------------------------------------------

#[test]
fn recorder_logs_forms_and_chat() {
    let mut recorder = Recorder::default();
    let mut batch = chat("gg");
    batch.extend(manage(5));
    recorder.batch(Way::FromServer, &split_batch(&batch).unwrap());
    let lines = recorder.drain();
    assert!(lines.iter().any(|line| line.contains("msg=\"gg\"")));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("form=5") && line.contains("Manage Your Server"))
    );
}

#[test]
fn long_lines_split_on_char_boundaries() {
    let mut recorder = Recorder::default();
    recorder.note("é".repeat(1000));
    let lines = recorder.drain();
    assert!(lines.len() > 1);
    assert!(lines[0].starts_with(TAG) && lines[1].starts_with(MORE));
    assert!(lines.iter().all(|line| line.len() < 1024));
    let text: String = lines
        .iter()
        .map(|line| line.split_once(' ').unwrap().1)
        .collect();
    assert_eq!(text, "é".repeat(1000));
}

#[test]
fn drain_respects_the_log_allowance() {
    let mut recorder = Recorder::default();
    for _ in 0..100 {
        recorder.note("x".repeat(800));
    }
    let first = recorder.drain();
    assert!(first.iter().map(|line| line.len() + 1).sum::<usize>() <= 12 * 1024);
    assert!(!recorder.drain().is_empty());
}

#[test]
fn strings_finds_length_prefixed_text() {
    let mut bytes = vec![0xff, 0x02];
    write_string(&mut bytes, "hive:custom_server");
    bytes.push(0);
    assert_eq!(
        packets::strings(&bytes),
        vec!["hive:custom_server".to_string()]
    );
}

// ---- automatic custom server --------------------------------------------------------------------

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
    let (_, sent) = h.client(&answer);
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
    // RED: the user and H0nri, who is in the party; BLUE: everyone else.
    assert_eq!(
        teams,
        ["RED", "BLUE", "RED", "BLUE", "BLUE", "BLUE", "BLUE", "BLUE"]
    );
    assert!(
        sent.said("RED Boldworks, H0nri | BLUE Charlie, P2"),
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
fn players_who_join_the_party_after_autocs_are_put_on_red() {
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
        ["RED", "BLUE", "RED", "BLUE", "BLUE", "BLUE", "BLUE", "BLUE"]
    );
    assert!(
        sent.said("RED Boldworks, H0nri | BLUE Charlie, P2"),
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

// ---- Hive's real menus --------------------------------------------------------------------------

/// Hive's menus as recorded on 2026-11-05, by the form ID they had then.
fn recorded(form: u32) -> Vec<u8> {
    let line = include_str!("fixtures/hive-forms.tsv")
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
    h.press(&recorded(17), 17, 0);
    h.press(&recorded(18), 18, 0);
    let sent = h.close(&recorded(19), 19);
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
fn picker_buttons(json: &str) -> Vec<(String, String, Option<String>)> {
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

// ---- invites and spectators ---------------------------------------------------------------------

mod invites_and_spectators {
    use super::*;

    /// Hive's invite and spectator menus, recorded on 2026-11-06 with WBSever hosting and
    /// Boldworks as the guest, by the form ID they had then.
    fn recorded(form: u32) -> Vec<u8> {
        let line = include_str!("fixtures/hive-forms-invite-spec.tsv")
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
        assert_eq!(players[0].1, "RED");
        assert_eq!(players[1].1, "Spectator");
        assert_eq!(players[2].1, "BLUE");
        assert!(sent.said("H0nri is spectating"), "{:?}", sent.said);
        assert!(sent.said("RED Boldworks | BLUE Charlie"), "{:?}", sent.said);
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

// ---- batch rewriting ----------------------------------------------------------------------------

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
