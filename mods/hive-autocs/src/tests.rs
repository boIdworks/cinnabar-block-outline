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

mod menus;
use menus::picker_buttons;
mod protocol;
mod rewrite;
mod run;
mod spectators;
