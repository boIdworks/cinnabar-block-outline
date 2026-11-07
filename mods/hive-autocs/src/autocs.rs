//! The automatic Bedwars Squads custom server on The Hive.
//!
//! `!autocs` shows a map menu (or takes the map from `!autocs <map>`), reads the party with
//! `/p list`, then creates the server with `/cs` and answers Hive's menus: Create Server, BedWars,
//! Squads. In the server, the player right-clicks the settings item once; the mod then loads the
//! preset, sets the map and closes the menu. When the lobby reaches eight players (or on `!teams`)
//! it asks for one more right-click and puts the party on RED and everyone else on the map's
//! other team (see [`opponent`]); on maps without a known pairing Hive places everyone else.
//!
//! Invites: every player the player invites by hand (Hive's "Invite sent to ..." message) is
//! remembered and invited again to each later `!autocs` server, until `!autocs end`. `@Name spec`
//! in chat makes that player a spectator at the next right-click, and again in every later
//! `!autocs` server they join (set with the teams), until they are set back to a player in Hive's
//! Set Roles menu or `!autocs end`.
//!
//! Every Hive menu the mod answers is kept from the player's screen, and buttons are chosen by
//! their text, so a reordered menu still works. A menu the mod does not know is passed to the
//! player untouched.

use serde_json::Value;

use crate::commands::slug;
use crate::packets::{self, TEXT_CHAT};
use crate::wire::Packet;

/// The saved preset loaded on every automatic server.
pub const PRESET: &str = "new scrim rules";
/// The command that lists the party.
pub const PARTY_COMMAND: &str = "/p list";
/// The command that brings the party to the new server, sent once it is open to everyone.
pub const WARP_COMMAND: &str = "/p warp";
/// Words in the Access Mode button that opens the server to everyone (Hive's is "Set To Open to
/// All Mode", recorded 2026-11-06; the others are for a reworded menu), and in the buttons that
/// must never be pressed for it.
const OPEN_WORDS: [&str; 5] = ["public", "open", "everyone", "anyone", "all players"];
const NOT_OPEN_WORDS: [&str; 4] = ["invite", "private", "closed", "back"];
/// Players (spectators not counted) the teams are set for: exactly this many, no more, no fewer.
pub const FULL_LOBBY: u32 = 8;
/// Press Start Game in the same right-click, once the teams are set. Off: the player starts the
/// game themselves when they are ready (2026-11-06).
pub const START_AFTER_TEAMS: bool = false;
/// Form ID of the mod's own map menu, far above the small IDs Hive counts up from.
pub const PICKER_FORM: u32 = 2_000_000_001;
/// The Squads maps Hive offered when the menus were recorded (2026-11-05); the menu uses the
/// live list once Hive has shown it this session.
pub const MAPS: [&str; 29] = [
    "Random", "Autumn", "Cobalt", "Coven (Halloween)", "Cyberspace", "Gingerbread (Winterfest)",
    "Grove", "Hanging Gardens", "Hanging Gardens (Spring)", "Jotunheim", "Lumora",
    "Monster Labs (Halloween)", "O2", "Orion", "Pirates", "Prisma", "Quantum", "Railway", "Rome",
    "Ruins", "Serrano", "Soul's End", "Spring (Spring)", "Summer (Summer)", "Summer Coast",
    "Temple", "Tesla Labs", "Tropical", "Winter (Winterfest)",
];

/// The map menu's order, top to bottom, as the player listed it (2026-11-06): most likely
/// played first. Names may leave out the event tag ("Spring" for "Spring (Spring)"). Random
/// comes before them, and maps not listed (Railway) after them in Hive's order.
pub const RANKING: &[&str] = &[
    "Temple", "Rome", "Ruins", "Spring", "Summer", "Autumn", "Winter", "Pirates", "Hanging Gardens",
    "Hanging Gardens (Spring)", "Tropical", "Cyberspace", "Prisma", "Soul's End", "Gingerbread",
    "Grove", "O2", "Cobalt", "Jotunheim", "Coven", "Summer Coast", "Serrano", "Lumora",
    "Tesla Labs", "Monster Labs", "Orion", "Quantum",
];

/// Hive's picture for each map, as its Map menu showed them on 2026-11-05; the menu uses the
/// live pictures once Hive has shown them this session.
const IMAGES: [(&str, &str); 28] = [
    ("Autumn", "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.png"),
    ("Cobalt", "https://cdn.playhive.com//icons/map/BED_SQUAD_Cobalt.png"),
    ("Coven (Halloween)", "https://cdn.playhive.com//icons/map/BED_SQUAD_Coven.halloween.png"),
    ("Cyberspace", "https://cdn.playhive.com//icons/map/BED_SQUAD_Cyberspace.png"),
    ("Gingerbread (Winterfest)", "https://cdn.playhive.com//icons/map/BED_SQUAD_Gingerbread.winterfest.png"),
    ("Grove", "https://cdn.playhive.com//icons/map/BED_SQUAD_Grove.png"),
    ("Hanging Gardens", "https://cdn.playhive.com//icons/map/BED_SQUAD_HangingGardens.png"),
    ("Hanging Gardens (Spring)", "https://cdn.playhive.com//icons/map/BED_SQUAD_HangingGardens.spring.png"),
    ("Jotunheim", "https://cdn.playhive.com//icons/map/BED_SQUAD_Jotunheim.png"),
    ("Lumora", "https://cdn.playhive.com//icons/map/BED_SQUAD_Lumora.png"),
    ("Monster Labs (Halloween)", "https://cdn.playhive.com//icons/map/BED_SQUAD_TeslaLabs.halloween.png"),
    ("O2", "https://cdn.playhive.com//icons/map/BED_SQUAD_O2.png"),
    ("Orion", "https://cdn.playhive.com//icons/map/BED_SQUAD_Orion.png"),
    ("Pirates", "https://cdn.playhive.com//icons/map/BED_SQUAD_Pirates.png"),
    ("Prisma", "https://cdn.playhive.com//icons/map/BED_SQUAD_Prisma.png"),
    ("Quantum", "https://cdn.playhive.com//icons/map/BED_SQUAD_Quantum.png"),
    ("Railway", "https://cdn.playhive.com//icons/map/BED_SQUAD_Railway.png"),
    ("Rome", "https://cdn.playhive.com//icons/map/BED_SQUAD_Rome.png"),
    ("Ruins", "https://cdn.playhive.com//icons/map/BED_SQUAD_Ruins.png"),
    ("Serrano", "https://cdn.playhive.com//icons/map/BED_SQUAD_Serrano.png"),
    ("Soul's End", "https://cdn.playhive.com//icons/map/BED_SQUAD_SoulsEnd.png"),
    ("Spring (Spring)", "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.spring.png"),
    ("Summer (Summer)", "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.summer.png"),
    ("Summer Coast", "https://cdn.playhive.com//icons/map/BED_SQUAD_SummerCoast.png"),
    ("Temple", "https://cdn.playhive.com//icons/map/BED_SQUAD_Temple.png"),
    ("Tesla Labs", "https://cdn.playhive.com//icons/map/BED_SQUAD_TeslaLabs.png"),
    ("Tropical", "https://cdn.playhive.com//icons/map/BED_SQUAD_Tropical.png"),
    ("Winter (Winterfest)", "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.winterfest.png"),
];

/// The team facing RED on each map, by its name without the event tag ("Coven" for "Coven
/// (Halloween)"), and whether the player has confirmed it. All were confirmed by the player
/// in their own games on 2026-11-06. An unconfirmed pairing (`false`) only sorts and labels the
/// map menu: the party goes on RED and Hive places everyone else, so a guess cannot send
/// players to a team the map lacks. Maps not listed, and Random, are the same, without a menu
/// group.
const OPPONENTS: [(&str, &str, bool); 27] = [
    ("Autumn", "BLUE", true), ("Lumora", "BLUE", true), ("Pirates", "BLUE", true),
    ("Prisma", "BLUE", true), ("Soul's End", "BLUE", true), ("Spring", "BLUE", true),
    ("Summer", "BLUE", true), ("Temple", "BLUE", true), ("Winter", "BLUE", true),
    ("Cobalt", "BLUE", true), ("Orion", "BLUE", true), ("Quantum", "BLUE", true),
    ("Serrano", "BLUE", true),
    ("Cyberspace", "YELLOW", true), ("Hanging Gardens", "YELLOW", true), ("Rome", "YELLOW", true),
    ("Ruins", "YELLOW", true), ("O2", "YELLOW", true), ("Tropical", "YELLOW", true),
    ("Coven", "GREEN", true), ("Gingerbread", "GREEN", true), ("Grove", "GREEN", true),
    ("Summer Coast", "GREEN", true), ("Jotunheim", "GREEN", true), ("Monster Labs", "GREEN", true),
    ("Tesla Labs", "GREEN", true), ("Railway", "GREEN", true),
];

/// The team facing RED on `map` and whether that is confirmed, if it is listed.
pub fn pairing(map: &str) -> Option<(&'static str, bool)> {
    lookup(&OPPONENTS, map, |(name, _, _)| name).map(|(_, team, confirmed)| (*team, *confirmed))
}

/// The confirmed team facing RED on `map`: the team the mod moves everyone else to.
pub fn opponent(map: &str) -> Option<&'static str> {
    pairing(map).filter(|(_, confirmed)| *confirmed).map(|(team, _)| team)
}

/// A map's name without its event tag: "Spring" for "Spring (Spring)".
fn base(map: &str) -> &str {
    match map.find(" (") {
        Some(at) if map.ends_with(')') => &map[..at],
        _ => map,
    }
}

/// The entry of `list` naming `map`: its exact name first, else its name without the event tag.
fn lookup<'a, T>(list: &'a [T], map: &str, name: impl Fn(&T) -> &str) -> Option<&'a T> {
    position(list, map, name).map(|index| &list[index])
}

/// Where [`lookup`] finds `map` in `list`.
fn position<T>(list: &[T], map: &str, name: impl Fn(&T) -> &str) -> Option<usize> {
    let find = |wanted: &str| list.iter().position(|entry| slug(name(entry)) == slug(wanted));
    find(map).or_else(|| find(base(map)))
}

/// How a team name is coloured in chat.
fn colour(team: &str) -> &'static str {
    match team {
        "RED" => "§c",
        "BLUE" => "§9",
        "YELLOW" => "§e",
        "GREEN" => "§a",
        _ => "§7",
    }
}

/// Ticks (50 ms) spent collecting the reply to the party command.
const PARTY_TICKS: u64 = 40;
/// Ticks to wait for each creation menu before giving up.
const CREATE_TICKS: u64 = 300;
/// Ticks to wait for the custom server to be joined.
const JOIN_TICKS: u64 = 1200;
/// Ticks within which Hive shows the parent menu after the mod closes a nested one (closing
/// Set Roles shows Game Settings, closing that shows Manage Your Server).
const PARENT_TICKS: u64 = 20;
/// Ticks between receiving a menu and answering it.
const ANSWER_TICKS: u64 = 3;
/// Menus the mod may answer in one run before it assumes something is wrong.
const MAX_STEPS: u32 = 120;
/// Times the mod tries to move one player before leaving them as they are.
const MAX_ROLE_TRIES: u8 = 2;

const TAG: &str = "§d[Auto CS]§r";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Stage {
    #[default]
    Idle,
    /// The map menu is open.
    Picking,
    /// Waiting for the party command's reply until the tick.
    Party(u64),
    /// Answering the creation menus; gives up at the tick.
    Creating(u64),
    /// Waiting to arrive in the server; gives up at the tick.
    Joining(u64),
    /// In the server; the settings menus are answered when they open.
    Lobby,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Task {
    Pending,
    /// Also the state of a task no run has asked for.
    #[default]
    Done,
    /// Done and already reported to the player.
    Reported,
    Failed,
}

/// Everything that lasts a whole session: the player, Hive's map list, the players to invite
/// and keep spectating across servers, and the packets waiting to go out.
pub struct AutoCs {
    me: String,
    tick: u64,
    /// The current run; a new `!autocs`, `!autocs stop` or a failure starts it afresh.
    run: Run,
    /// Players in the server, from the last join or leave message.
    lobby: u32,
    /// Players invited by hand this set of games; invited again to every new server until
    /// `!autocs end`.
    invites: Vec<String>,
    /// Players to keep as spectators, in this and every later server until they are set back to
    /// a player or `!autocs end`.
    specs: Vec<String>,
    /// The live map list from Hive's Map menu.
    maps: Vec<String>,
    /// Each map's picture, by name.
    images: Vec<(String, String)>,
    answers: Vec<(u64, Vec<u8>)>,
    to_client: Vec<u8>,
    to_server: Vec<u8>,
    notes: Vec<String>,
    commands: u64,
}

/// One run of `!autocs`: from the map menu to the teams in that server.
#[derive(Default)]
struct Run {
    stage: Stage,
    map: String,
    preset: Task,
    map_task: Task,
    /// Opening the server to everyone (Access & Permissions → Change Access Mode), after which
    /// the party is warped in.
    access: Task,
    roles: Task,
    /// Teams may be set: the lobby is full or `!teams` was typed.
    roles_ready: bool,
    /// The full-lobby reminder was shown.
    roles_prompted: bool,
    /// Whether a settings menu is being answered right now.
    navigating: bool,
    /// After closing a nested menu: the tick until which its parent menu is the mod's to close.
    closing_until: Option<u64>,
    /// Pressing Start Game once the teams are set.
    start: Task,
    /// The status after a right-click was said while closing a nested menu (or with the wait for
    /// players), so closing its parent does not say it again.
    told: bool,
    /// Menus answered this run.
    steps: u32,
    /// The reply to the party command, formatting removed and lowercased.
    party: String,
    role_tries: Vec<(String, u8)>,
    /// Players still to invite to the current server.
    to_invite: Vec<String>,
    /// Players invited to the current server by the mod.
    invited: Vec<String>,
    /// `@Name spec` requests for the current server, and whether they are still to do.
    requested: Vec<String>,
    spec_pending: bool,
    /// Players in the current server, from Hive's join and leave messages.
    present: Vec<String>,
    /// Players Hive's Set Roles list last showed as spectators; they do not count toward the
    /// lobby.
    spectating: Vec<String>,
    /// The maps in the order the map menu showed them.
    picker: Vec<String>,
}

impl AutoCs {
    pub fn new(me: impl Into<String>) -> Self {
        Self {
            me: me.into(),
            tick: 0,
            run: Run::default(),
            lobby: 0,
            invites: Vec::new(),
            specs: Vec::new(),
            maps: MAPS.iter().map(|map| map.to_string()).collect(),
            images: IMAGES.iter().map(|(map, url)| (map.to_string(), url.to_string())).collect(),
            answers: Vec::new(),
            to_client: Vec::new(),
            to_server: Vec::new(),
            notes: Vec::new(),
            commands: 0,
        }
    }

    // ---- player input -------------------------------------------------------------------------

    /// A packet from the player's client; true when it must not reach the server.
    pub fn from_client(&mut self, packet: &Packet<'_>) -> bool {
        match packet.id {
            packets::TEXT => match packets::read_text(packet) {
                Some(text) => self.typed(&text.message),
                None => false,
            },
            packets::COMMAND_REQUEST => match packets::read_command(packet) {
                Some(command) => {
                    let line = command.trim();
                    let line = line.strip_prefix('/').unwrap_or(line);
                    word(line, "autocs").is_some() && self.typed(&format!("!{line}"))
                }
                None => false,
            },
            packets::MODAL_FORM_RESPONSE => match packets::read_form_response(packet) {
                Some((PICKER_FORM, response, _)) => {
                    self.picked(response.as_deref());
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Chat meant for this mod; true when it was.
    fn typed(&mut self, message: &str) -> bool {
        if let Some(player) = spec_target(message) {
            self.spectate(player);
            return true;
        }
        let Some(line) = message.trim().strip_prefix('!') else {
            return false;
        };
        if let Some(rest) = word(line, "autocs") {
            self.autocs(rest.trim());
        } else if word(line, "teams").is_some() {
            self.teams();
        } else if let Some(note) = word(line, "mark") {
            self.notes.push(format!("MARK {}", note.trim()));
            self.say("Note added to the log.");
        } else {
            return false;
        }
        true
    }

    fn autocs(&mut self, argument: &str) {
        if ["stop", "cancel"].iter().any(|stop| argument.eq_ignore_ascii_case(stop)) {
            if self.run.stage == Stage::Idle {
                self.say("Nothing to stop.");
            } else {
                self.reset();
                self.say("Stopped.");
            }
            return;
        }
        if argument.eq_ignore_ascii_case("teams") {
            self.teams();
            return;
        }
        if argument.eq_ignore_ascii_case("end") {
            self.end();
            return;
        }
        if self.run.stage != Stage::Idle && self.run.stage != Stage::Lobby {
            self.say("Already setting up a server. Type §e!autocs stop§r to cancel it.");
            return;
        }
        self.reset();
        if argument.is_empty() {
            self.run.stage = Stage::Picking;
            self.run.picker = menu_order(&self.maps);
            let form = picker_form(&self.run.picker, &self.images);
            packets::form_request(&mut self.to_client, PICKER_FORM, &form.to_string());
        } else {
            match find_map(&self.maps, argument) {
                Some(map) => self.begin(map),
                None => self.say(&format!(
                    "No map called §e{argument}§r. Type §e!autocs§r to pick from the list."
                )),
            }
        }
    }

    fn picked(&mut self, response: Option<&str>) {
        if self.run.stage != Stage::Picking {
            return;
        }
        let map = response
            .and_then(|response| response.trim().parse::<usize>().ok())
            .and_then(|index| self.run.picker.get(index).cloned());
        match map {
            Some(map) => self.begin(map),
            None => {
                self.reset();
                self.say("Cancelled.");
            }
        }
    }

    /// Starts a run for `map`: the party command first, then the server.
    fn begin(&mut self, map: String) {
        self.notes.push(format!("AUTOCS start map={map:?} invites={:?}", self.invites));
        self.say(&format!("Map §b{map}§r. Checking your party..."));
        self.run.to_invite = self.invites.clone();
        self.run.map = map;
        self.run.preset = Task::Pending;
        self.run.map_task = Task::Pending;
        self.run.access = Task::Pending;
        self.run.roles = Task::Pending;
        if START_AFTER_TEAMS {
            self.run.start = Task::Pending;
        }
        self.run.stage = Stage::Party(self.tick + PARTY_TICKS);
        self.command(PARTY_COMMAND);
    }

    /// Ends a set of games: forgets the players to invite again.
    fn end(&mut self) {
        let names = std::mem::take(&mut self.invites);
        let specs = std::mem::take(&mut self.specs);
        self.run.to_invite.clear();
        self.run.requested.clear();
        self.run.spec_pending = false;
        self.notes.push(format!("AUTOCS end invites={names:?} specs={specs:?}"));
        let mut parts = Vec::new();
        if !names.is_empty() {
            parts.push(format!("I won't invite §e{}§r again", names.join(", ")));
        }
        if !specs.is_empty() {
            parts.push(format!("§e{}§r won't be made spectators", specs.join(", ")));
        }
        if parts.is_empty() {
            self.say("Game over. Nothing to forget; the next game starts fresh.");
        } else {
            self.say(&format!("Game over. {}; the next game starts fresh.", parts.join(", and ")));
        }
    }

    /// `@Name spec`: make the player a spectator at the next right-click.
    fn spectate(&mut self, player: &str) {
        if player.eq_ignore_ascii_case(&self.me) {
            self.say("You can't make yourself a spectator from here; use Set Roles.");
            return;
        }
        self.notes.push(format!("AUTOCS spectate {player:?}"));
        self.remember_spec(player);
        if !self.run.requested.iter().any(|known| known.eq_ignore_ascii_case(player)) {
            self.run.requested.push(player.to_owned());
        }
        self.run.spec_pending = true;
        if self.run.stage == Stage::Idle {
            // A server made by hand: answer its menus too.
            self.run.stage = Stage::Lobby;
        }
        if self.run.stage == Stage::Lobby {
            self.say(&format!("§eRight-click the settings item§r to make §e{player}§r a spectator."));
        } else {
            self.say(&format!("I'll make §e{player}§r a spectator once you're in the server."));
        }
    }

    fn remember_spec(&mut self, player: &str) {
        if !self.specs.iter().any(|known| known.eq_ignore_ascii_case(player)) {
            self.specs.push(player.to_owned());
        }
    }

    fn is_spec(&self, player: &str) -> bool {
        self.specs.iter().any(|spec| spec.eq_ignore_ascii_case(player))
    }

    fn teams(&mut self) {
        if self.run.stage != Stage::Lobby || self.run.roles != Task::Pending {
            self.say("Teams are set during an §e!autocs§r run, once you're in the server.");
            return;
        }
        self.run.roles_ready = true;
        self.run.roles_prompted = true;
        self.say("Right-click the §esettings item§r to set the teams.");
    }

    // ---- server packets -----------------------------------------------------------------------

    /// A packet from the server; true when it must not reach the player.
    pub fn from_server(&mut self, packet: &Packet<'_>) -> bool {
        match packet.id {
            packets::TEXT => {
                if let Some(text) = packets::read_text(packet) {
                    self.server_text(&text);
                }
                false
            }
            packets::MODAL_FORM_REQUEST => match packets::read_form_request(packet) {
                Some((form, json)) => self.form(form, &json),
                None => false,
            },
            _ => false,
        }
    }

    fn server_text(&mut self, text: &packets::Text) {
        if text.kind == TEXT_CHAT {
            return;
        }
        let plain = strip(&text.message);
        if let Some((by, name)) = invite_sent(&plain) {
            let by_mod = self.run.invited.iter().any(|known| known.eq_ignore_ascii_case(name));
            if !by_mod
                && !name.eq_ignore_ascii_case(&self.me)
                && !self.invites.iter().any(|known| known.eq_ignore_ascii_case(name))
            {
                self.invites.push(name.to_owned());
                let from = by.map(|by| format!(" (invited by §e{by}§r)")).unwrap_or_default();
                self.notes.push(format!("AUTOCS remember invite {name:?} by={by:?}"));
                self.say(&format!(
                    "I'll invite §e{name}§r{from} to your next games too (§e/autocs end§r when this group is done)."
                ));
            }
        }
        if let Some((name, role)) = role_set(&plain) {
            if role.eq_ignore_ascii_case("Spectator") {
                if !self.is_spec(name) {
                    self.remember_spec(name);
                    self.notes.push(format!("AUTOCS remember spec {name:?}"));
                    self.say(&format!(
                        "I'll keep §e{name}§r spectating in your next games (set them back to a player to stop)."
                    ));
                }
            } else if self.is_spec(name) {
                // The mod never moves a spectator, so this was done by hand: they play now.
                self.specs.retain(|spec| !spec.eq_ignore_ascii_case(name));
                self.run.requested.retain(|spec| !spec.eq_ignore_ascii_case(name));
                self.run.spectating.retain(|spec| !spec.eq_ignore_ascii_case(name));
                self.notes.push(format!("AUTOCS forget spec {name:?} role={role:?}"));
                self.say(&format!("§e{name}§r is playing now; I won't make them a spectator again."));
                self.player_back();
            }
        }
        if let Stage::Party(_) = self.run.stage {
            self.run.party.push_str(&plain.to_lowercase());
            self.run.party.push('\n');
        }
        if (plain.contains(" joined.") || plain.contains(" left."))
            && let Some(count) = lobby_count(&plain)
        {
            if let Stage::Joining(_) = self.run.stage
                && plain.contains(&format!("{} joined.", self.me))
            {
                self.run.stage = Stage::Lobby;
                self.run.present.clear();
                self.run.spectating.clear();
                let invites = if self.run.to_invite.is_empty() {
                    String::new()
                } else {
                    format!(", and invite §e{}§r", self.run.to_invite.join(", "))
                };
                self.say(&format!(
                    "You're in! §eRight-click the settings item§r and I'll load the preset, set the map{invites}."
                ));
            }
            if let Some(name) = plain.split_once(" joined.").map(|(before, _)| player_name(before)) {
                if !self.run.present.iter().any(|known| known.eq_ignore_ascii_case(name)) {
                    self.run.present.push(name.to_owned());
                }
                if self.run.stage == Stage::Lobby && self.is_spec(name) && !name.eq_ignore_ascii_case(&self.me) {
                    self.say(&format!("§e{name}§r will spectate, as in your last game."));
                }
            }
            if let Some(name) = plain.split_once(" left.").map(|(before, _)| player_name(before)) {
                self.run.present.retain(|known| !known.eq_ignore_ascii_case(name));
                self.run.spectating.retain(|spec| !spec.eq_ignore_ascii_case(name));
            }
            self.lobby = count;
            self.check_full();
        }
    }

    /// Players in the server, spectators not counted.
    fn players(&self) -> u32 {
        self.lobby.saturating_sub(self.spectators())
    }

    /// Spectators in the server: those Hive shows as Spectator, and remembered spectators who
    /// joined and will be set with the teams.
    fn spectators(&self) -> u32 {
        let mut names: Vec<String> = self.run.spectating.iter().map(|name| name.to_lowercase()).collect();
        for name in &self.run.present {
            if self.is_spec(name) && !name.eq_ignore_ascii_case(&self.me) {
                names.push(name.to_lowercase());
            }
        }
        names.sort();
        names.dedup();
        names.len() as u32
    }

    /// The lobby count for chat, with the spectators if any.
    fn lobby_text(&self) -> String {
        match self.spectators() {
            0 => format!("{}/{FULL_LOBBY} players are in", self.players()),
            n => format!(
                "{}/{FULL_LOBBY} players are in (+{n} spectating)",
                self.players()
            ),
        }
    }

    /// A spectator was set back to a player by hand. If that fills the lobby, set the teams at
    /// once: the player is in Set Roles, so the list Hive shows next is the mod's to finish (and
    /// then on to Start Game); if the menu has closed, the next right-click does it.
    fn player_back(&mut self) {
        if self.run.stage != Stage::Lobby || self.run.roles != Task::Pending {
            return;
        }
        if self.players() != FULL_LOBBY {
            self.check_full();
            return;
        }
        self.run.roles_ready = true;
        self.run.roles_prompted = true;
        self.run.closing_until = Some(self.tick + PARENT_TICKS);
        self.notes.push("AUTOCS lobby full after a spectator came back; setting the teams".into());
        self.say(&format!(
            "{}. Setting the teams now: your party §cRED§r, {} (if the menu closed, §eright-click the settings item§r).",
            self.lobby_text(),
            self.others()
        ));
    }

    /// What is wrong with a lobby that isn't exactly full, for chat.
    fn not_full_text(&self) -> String {
        let players = self.players();
        if players > FULL_LOBBY {
            let over = players - FULL_LOBBY;
            format!(
                "{}: too many to start. Make {over} of them spectate (§e@Name spec§r) or remove {}; I'll set the teams at exactly {FULL_LOBBY}",
                self.lobby_text(),
                if over == 1 { "them" } else { "some" }
            )
        } else {
            let missing = FULL_LOBBY - players;
            format!(
                "{}: waiting for {missing} more player{}",
                self.lobby_text(),
                if missing == 1 { "" } else { "s" }
            )
        }
    }

    fn check_full(&mut self) {
        if self.run.stage != Stage::Lobby || self.run.roles != Task::Pending {
            return;
        }
        // Spectators do not count: 7 players and 1 spectator wait for an eighth player, and 9
        // players are too many for the 4v4.
        if self.players() == FULL_LOBBY {
            if !self.run.roles_prompted {
                self.run.roles_ready = true;
                self.run.roles_prompted = true;
                self.say(&format!(
                    "{}. §eRight-click the settings item§r to set the teams: your party §cRED§r, {}.",
                    self.lobby_text(),
                    self.others()
                ));
            }
        } else if self.players() > FULL_LOBBY {
            // It was full and someone else joined: take the go-ahead back.
            self.run.roles_ready = false;
            self.run.roles_prompted = false;
            self.say(&format!("{}.", self.not_full_text()));
        }
    }

    /// A Hive menu; true when the mod answers it instead of the player.
    fn form(&mut self, form: u32, json: &str) -> bool {
        let Ok(value) = serde_json::from_str::<Value>(json) else {
            return false;
        };
        let menu = Menu::new(&value);
        match self.run.stage {
            Stage::Party(_) => {
                // A party menu instead of a chat reply: read it and close it.
                self.run.party.push_str(&strip(json).to_lowercase());
                self.cancel(form);
                true
            }
            Stage::Creating(_) => self.creating(form, &menu),
            Stage::Lobby => self.settings(form, &menu),
            _ => false,
        }
    }

    fn creating(&mut self, form: u32, menu: &Menu) -> bool {
        let choice = match menu.title.as_str() {
            "Custom Server Menu" => menu.button("Create Server"),
            "Create Server" => menu.button("BedWars").or_else(|| {
                let squads = menu.button("Squads");
                if squads.is_some() {
                    self.run.stage = Stage::Joining(self.tick + JOIN_TICKS);
                    self.say("Creating the server...");
                }
                squads
            }),
            _ => None,
        };
        match choice {
            Some(index) => {
                if let Stage::Creating(_) = self.run.stage {
                    self.run.stage = Stage::Creating(self.tick + CREATE_TICKS);
                }
                self.answer(form, index);
                true
            }
            None => {
                self.fail(&format!("Hive showed §e{}§r, which I didn't expect. It's yours now.", menu.title));
                false
            }
        }
    }

    /// The server's settings menus.
    fn settings(&mut self, form: u32, menu: &Menu) -> bool {
        let roles = self.teams_now() || self.run.spec_pending;
        let invites = !self.run.to_invite.is_empty();
        let access = self.run.access == Task::Pending;
        // Start Game waits for the teams.
        let start = self.run.start == Task::Pending && self.run.roles == Task::Done;
        let work = self.run.preset == Task::Pending
            || self.run.map_task == Task::Pending
            || roles
            || invites
            || access
            || start;
        let parent = self.run.closing_until.take().is_some_and(|until| self.tick <= until);
        if !self.run.navigating {
            if !parent && (menu.title != "Manage Your Server" || !work) {
                return false;
            }
            self.run.navigating = true;
        }
        self.run.steps += 1;
        if self.run.steps > MAX_STEPS {
            self.run.navigating = false;
            self.fail("Too many menus; I stopped so nothing goes wrong.");
            return false;
        }
        let choice = match menu.title.as_str() {
            "Manage Your Server" => {
                if self.run.preset == Task::Pending {
                    menu.button("Manage Presets")
                } else if self.run.map_task == Task::Pending || roles {
                    menu.button("Game Settings")
                } else if invites || access {
                    menu.button("Access & Permissions")
                } else if start {
                    self.run.start = Task::Done;
                    self.run.navigating = false;
                    self.run.stage = Stage::Idle;
                    self.notes.push("AUTOCS start game".into());
                    self.say("Starting the game. Good luck!");
                    menu.button("Start Game")
                } else {
                    return self.finish(form, false);
                }
            }
            "Preset Settings" => {
                if self.run.preset == Task::Pending {
                    menu.button("Saved Presets")
                } else {
                    menu.button("Go Back")
                }
            }
            "Saved Presets" => {
                if self.run.preset == Task::Pending
                    && let Some(index) = menu.button(PRESET)
                {
                    Some(index)
                } else {
                    if self.run.preset == Task::Pending {
                        self.run.preset = Task::Failed;
                        self.say(&format!("No saved preset called §e{PRESET}§r; skipping it."));
                    }
                    menu.button("Go Back")
                }
            }
            "Load Preset?" => {
                if self.run.preset == Task::Pending && menu.content.to_lowercase().contains(PRESET) {
                    self.run.preset = Task::Done;
                    self.notes.push("AUTOCS preset loaded".into());
                    menu.button("Confirm")
                } else {
                    menu.button("Go Back")
                }
            }
            "Game Settings" => {
                if self.run.map_task == Task::Pending {
                    menu.button("Map")
                } else if roles {
                    menu.button("Set Roles")
                } else {
                    menu.button("Go Back")
                }
            }
            "Access & Permissions" => {
                if access {
                    menu.button("Change Access Mode")
                } else if invites {
                    menu.button("Invite Player")
                } else {
                    menu.button("Go Back")
                }
            }
            title if title.to_lowercase().contains("access mode") => {
                let open = if access { menu.button_with(&OPEN_WORDS, &NOT_OPEN_WORDS) } else { None };
                if let Some(index) = open {
                    self.run.access = Task::Done;
                    self.notes.push(format!("AUTOCS access mode {:?}", menu.buttons[index].first));
                    Some(index)
                } else {
                    if access {
                        self.run.access = Task::Failed;
                        self.say("I couldn't find the button that opens the server to everyone; do it by hand, then §e/p warp§r.");
                    }
                    menu.button("Go Back")
                }
            }
            "Invite Player" if menu.text_input => match self.run.to_invite.first().cloned() {
                Some(name) => {
                    self.run.to_invite.remove(0);
                    self.run.invited.push(name.clone());
                    self.notes.push(format!("AUTOCS invite {name:?}"));
                    let response = serde_json::json!([null, name]).to_string();
                    self.answer_with(form, &format!("{response}\n"));
                    return true;
                }
                None => return self.finish(form, true),
            },
            "Invite Player" => {
                if invites { menu.button("Invite Username") } else { menu.button("Go Back") }
            }
            "Map" => {
                self.maps = menu.buttons.iter().map(|button| button.first.clone()).collect();
                for button in &menu.buttons {
                    if let Some(url) = &button.image {
                        match self.images.iter_mut().find(|(map, _)| *map == button.first) {
                            Some((_, known)) => known.clone_from(url),
                            None => self.images.push((button.first.clone(), url.clone())),
                        }
                    }
                }
                match menu.button(&self.run.map) {
                    Some(index) if self.run.map_task == Task::Pending => {
                        self.run.map_task = Task::Done;
                        self.notes.push(format!("AUTOCS map set {:?}", self.run.map));
                        Some(index)
                    }
                    _ => {
                        self.run.map_task = Task::Failed;
                        self.say(&format!("Hive has no map called §e{}§r now; pick it by hand.", self.run.map));
                        return self.finish(form, true);
                    }
                }
            }
            "Set Roles" if menu.content.starts_with("Setting role for ") => {
                let player = menu.content["Setting role for ".len()..].trim().trim_end_matches('.');
                // Players the mod leaves to Hive are never opened, but be safe: unassign.
                let role = self.role_of(player).unwrap_or("Player");
                menu.button(role)
            }
            "Set Roles" => match self.next_player(menu) {
                Some(index) => Some(index),
                None => {
                    self.run.spectating = menu
                        .buttons
                        .iter()
                        .filter(|button| button.second.eq_ignore_ascii_case("Spectator"))
                        .map(|button| button.first.clone())
                        .collect();
                    if self.run.spec_pending {
                        self.run.spec_pending = false;
                        self.report_specs(menu);
                    }
                    if self.teams_now() {
                        let players = menu.buttons.len() as u32 - self.run.spectating.len() as u32;
                        self.run.present = menu.buttons.iter().map(|button| button.first.clone()).collect();
                        if players == FULL_LOBBY {
                            self.run.roles = Task::Done;
                            self.report_teams(menu);
                        } else {
                            // Short of players (or too many): wait until it is exactly full,
                            // then set the teams.
                            self.run.roles_ready = false;
                            self.run.roles_prompted = false;
                            self.lobby = menu.buttons.len() as u32;
                            self.say(&format!(
                                "{}, so the teams aren't finished. I'll tell you when to right-click.",
                                self.not_full_text()
                            ));
                            self.run.told = true;
                        }
                    }
                    // Closing Set Roles shows Game Settings: go back up from there.
                    return self.finish(form, true);
                }
            },
            _ => {
                // Not a menu the mod knows: hand it to the player and stop answering.
                self.run.navigating = false;
                self.say(&format!("Hive showed §e{}§r; it's yours.", menu.title));
                return false;
            }
        };
        match choice {
            Some(index) => {
                self.answer(form, index);
                true
            }
            None => {
                self.run.navigating = false;
                self.say(&format!("Couldn't find the right button in §e{}§r; it's yours.", menu.title));
                false
            }
        }
    }

    /// Closes the open menu and reports what is left. Closing a `nested` menu makes Hive show
    /// its parent, which the mod answers too (on to Start Game, or closed).
    fn finish(&mut self, form: u32, nested: bool) -> bool {
        self.cancel(form);
        self.run.navigating = false;
        if nested {
            self.run.closing_until = Some(self.tick + PARENT_TICKS);
        }
        let mut done = Vec::new();
        if self.run.preset == Task::Done {
            done.push(format!("preset §b{PRESET}§r"));
            self.run.preset = Task::Reported;
        }
        if self.run.map_task == Task::Done {
            done.push(format!("map §b{}§r", self.run.map));
            self.run.map_task = Task::Reported;
        }
        if !done.is_empty() {
            self.say(&format!("Set {}.", done.join(" and ")));
        }
        if self.run.access == Task::Done {
            self.run.access = Task::Reported;
            self.say("Opened the server to everyone and sent §e/p warp§r.");
            self.command(WARP_COMMAND);
        }
        if !self.run.invited.is_empty() {
            self.say(&format!("Invited §e{}§r.", self.run.invited.join(", ")));
            self.run.invited.clear();
        }
        // Once per right-click: closing a nested menu and then its parent say it once.
        let say_status = !self.run.told;
        self.run.told = nested;
        if say_status {
            if self.run.roles == Task::Pending {
                if self.players() == FULL_LOBBY {
                    self.run.roles_ready = true;
                    self.say("Right-click the §esettings item§r again to set the teams.");
                } else if self.players() > FULL_LOBBY {
                    self.say(&format!("{}.", self.not_full_text()));
                } else {
                    self.say(&format!(
                        "When {FULL_LOBBY} players are in ({} now) I'll ask you to right-click again for the teams. Or type §e!teams§r.",
                        self.players().max(1)
                    ));
                }
            } else if self.run.roles == Task::Done && self.run.start != Task::Pending {
                self.say(
                    "Teams are set. Start the game when you're ready: §esettings item§r → §eStart Game§r.",
                );
            }
        }
        if self.run.roles == Task::Done && self.run.start != Task::Pending && !nested {
            self.run.stage = Stage::Idle;
        }
        true
    }

    /// Whether the teams are to be set now.
    fn teams_now(&self) -> bool {
        self.run.roles == Task::Pending && self.run.roles_ready
    }

    /// The team the player belongs on; `None` leaves them for Hive to place.
    fn team_of(&self, player: &str) -> Option<&'static str> {
        if self.in_party(player) { Some("RED") } else { opponent(&self.run.map) }
    }

    /// The role the mod gives the player now: Spectator for `@Name spec`, their team while the
    /// teams are being set, else none (left as they are).
    fn role_of(&self, player: &str) -> Option<&'static str> {
        if self.is_spec(player) {
            Some("Spectator")
        } else if self.teams_now() {
            self.team_of(player)
        } else {
            None
        }
    }

    /// Where everyone outside the party goes, for chat.
    fn others(&self) -> String {
        match pairing(&self.run.map) {
            Some((team, true)) => format!("everyone else {}{team}§r", colour(team)),
            Some((team, false)) => format!(
                "everyone else placed by Hive (§b{}§r is probably {}{team}§r, not confirmed)",
                self.run.map,
                colour(team)
            ),
            None => format!("everyone else placed by Hive (§b{}§r has no team pairing set)", self.run.map),
        }
    }

    fn in_party(&self, player: &str) -> bool {
        player.eq_ignore_ascii_case(&self.me) || contains_word(&self.run.party, &player.to_lowercase())
    }

    /// The first listed player not yet on their team, counting a try for them.
    fn next_player(&mut self, menu: &Menu) -> Option<usize> {
        for (index, button) in menu.buttons.iter().enumerate() {
            let Some(team) = self.role_of(&button.first) else {
                continue;
            };
            if button.second.eq_ignore_ascii_case(team) {
                continue;
            }
            let tries = match self.run.role_tries.iter_mut().find(|(name, _)| *name == button.first) {
                Some((_, tries)) => tries,
                None => {
                    self.run.role_tries.push((button.first.clone(), 0));
                    &mut self.run.role_tries.last_mut().unwrap().1
                }
            };
            if *tries < MAX_ROLE_TRIES {
                *tries += 1;
                return Some(index);
            }
        }
        None
    }

    /// Reports the spectators set, and those not in the server.
    fn report_specs(&mut self, menu: &Menu) {
        let requested = std::mem::take(&mut self.run.requested);
        let (here, missing): (Vec<String>, Vec<String>) = requested.into_iter().partition(|spec| {
            menu.buttons.iter().any(|button| button.first.eq_ignore_ascii_case(spec))
        });
        if !here.is_empty() {
            let verb = if here.len() == 1 { "is" } else { "are" };
            self.say(&format!("§e{}§r {verb} spectating.", here.join(", ")));
        }
        if !missing.is_empty() {
            self.say(&format!(
                "§e{}§r isn't in the server; they'll spectate if they join.",
                missing.join(", ")
            ));
        }
    }

    fn report_teams(&mut self, menu: &Menu) {
        let names = |team: &str| -> String {
            let list: Vec<&str> = menu
                .buttons
                .iter()
                .filter(|button| button.second.eq_ignore_ascii_case(team))
                .map(|button| button.first.as_str())
                .collect();
            if list.is_empty() { "-".into() } else { list.join(", ") }
        };
        let red = names("RED");
        let (other, others) = match opponent(&self.run.map) {
            Some(team) => (format!("{}{team}§r", colour(team)), names(team)),
            None => ("§7Hive places§r".to_string(), names("Player")),
        };
        self.notes.push(format!("AUTOCS teams red={red:?} other={others:?}"));
        self.say(&format!("Teams: §cRED§r {red} | {other} {others}"));
    }

    // ---- output -------------------------------------------------------------------------------

    /// Advances one tick and returns the batches to send: to the client, then to the server.
    pub fn tick(&mut self) -> (Vec<u8>, Vec<u8>) {
        self.tick += 1;
        match self.run.stage {
            Stage::Party(until) if self.tick >= until => {
                let party = self.run.party.trim().replace('\n', " | ");
                self.notes.push(format!("AUTOCS party reply {party:?}"));
                self.run.stage = Stage::Creating(self.tick + CREATE_TICKS);
                self.command("/cs");
            }
            Stage::Creating(until) if self.tick >= until => {
                self.fail("Hive didn't open the custom server menu. Try again in a moment.");
            }
            Stage::Joining(until) if self.tick >= until => {
                self.fail("The server didn't start in time (maybe your daily limit?).");
            }
            _ => {}
        }
        let now = self.tick;
        let (due, later): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.answers).into_iter().partition(|(at, _)| *at <= now);
        self.answers = later;
        for (_, batch) in due {
            self.to_server.extend(batch);
        }
        (std::mem::take(&mut self.to_client), std::mem::take(&mut self.to_server))
    }

    /// Lines for the recorder's log.
    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }

    /// Presses a button of a menu.
    fn answer(&mut self, form: u32, index: usize) {
        self.notes.push(format!("AUTOCS answer form={form} button={index}"));
        self.reply(|batch| packets::form_response(batch, form, &format!("{index}\n")));
    }

    /// Answers a custom form with its JSON response.
    fn answer_with(&mut self, form: u32, response: &str) {
        self.notes.push(format!("AUTOCS answer form={form} response={response:?}"));
        self.reply(|batch| packets::form_response(batch, form, response));
    }

    fn cancel(&mut self, form: u32) {
        self.notes.push(format!("AUTOCS close form={form}"));
        self.reply(|batch| packets::form_cancel(batch, form));
    }

    /// Queues a form answer, sent after [`ANSWER_TICKS`].
    fn reply(&mut self, encode: impl FnOnce(&mut Vec<u8>)) {
        let mut batch = Vec::new();
        encode(&mut batch);
        self.answers.push((self.tick + ANSWER_TICKS, batch));
    }

    fn command(&mut self, line: &str) {
        self.commands += 1;
        self.notes.push(format!("AUTOCS command {line:?}"));
        packets::command_request(&mut self.to_server, line, self.commands ^ self.tick << 20);
    }

    fn say(&mut self, message: &str) {
        packets::system_message(&mut self.to_client, &format!("{TAG} {message}"));
    }

    fn fail(&mut self, message: &str) {
        self.notes.push(format!("AUTOCS failed {message:?}"));
        self.say(message);
        self.reset();
    }

    /// Ends the current run; what lasts the session (invites, spectators, maps) stays.
    fn reset(&mut self) {
        self.run = Run::default();
    }
}

/// A Hive button menu with its text cleaned of formatting.
struct Menu {
    title: String,
    content: String,
    buttons: Vec<Button>,
    /// A custom form with a text box (Hive's username field).
    text_input: bool,
}

/// A button's first line (its name), second line (its detail) and picture.
struct Button {
    first: String,
    second: String,
    image: Option<String>,
}

impl Menu {
    fn new(value: &Value) -> Self {
        let text = |value: Option<&Value>| strip(value.and_then(Value::as_str).unwrap_or(""));
        let buttons = value
            .get("buttons")
            .and_then(Value::as_array)
            .map(|buttons| {
                buttons
                    .iter()
                    .map(|button| {
                        let label = text(button.get("text"));
                        let mut lines = label.lines();
                        Button {
                            first: lines.next().unwrap_or("").trim().to_owned(),
                            second: lines.next().unwrap_or("").trim().to_owned(),
                            image: button
                                .pointer("/image/data")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let text_input = value.get("type").and_then(Value::as_str) == Some("custom_form")
            && value.get("content").and_then(Value::as_array).is_some_and(|elements| {
                elements.iter().any(|element| element.get("type").and_then(Value::as_str) == Some("input"))
            });
        Self { title: text(value.get("title")), content: text(value.get("content")), buttons, text_input }
    }

    /// The button whose name is `name`, ignoring case.
    fn button(&self, name: &str) -> Option<usize> {
        self.buttons.iter().position(|button| button.first.eq_ignore_ascii_case(name))
    }

    /// The first button whose name has one of the `wanted` words and none of the `unwanted`.
    fn button_with(&self, wanted: &[&str], unwanted: &[&str]) -> Option<usize> {
        self.buttons.iter().position(|button| {
            let name = button.first.to_lowercase();
            wanted.iter().any(|word| name.contains(word)) && !unwanted.iter().any(|word| name.contains(word))
        })
    }
}

/// The map menu's order: Random, then [`RANKING`] top to bottom, then any other map in Hive's
/// order. Each button still shows the team faced there.
pub fn menu_order(maps: &[String]) -> Vec<String> {
    order_by(maps, RANKING)
}

/// [`menu_order`] with `ranking` in place of [`RANKING`].
pub fn order_by(maps: &[String], ranking: &[&str]) -> Vec<String> {
    let first = |map: &str| if map == "Random" { 0 } else { 1 };
    let rank = |map: &str| position(ranking, map, |ranked| ranked).unwrap_or(usize::MAX);
    let mut ordered: Vec<(usize, &String)> = maps.iter().enumerate().collect();
    ordered.sort_by_key(|(index, map)| (first(map), rank(map), *index));
    ordered.into_iter().map(|(_, map)| map.clone()).collect()
}

/// The map menu: each map with the team the party faces there and Hive's picture.
fn picker_form(maps: &[String], images: &[(String, String)]) -> Value {
    let buttons: Vec<Value> = maps
        .iter()
        .map(|map| {
            let detail = match (map.as_str(), pairing(map)) {
                ("Random", _) => "§8Hive picks the map".to_string(),
                (_, Some((team, true))) => format!("§8vs {}{team}", colour(team)),
                (_, Some((team, false))) => format!("§8vs {}{team} §7(unconfirmed)", colour(team)),
                _ => "§8Hive sets the other team".to_string(),
            };
            let mut button = serde_json::json!({ "text": format!("§l{map}§r\n{detail}") });
            if let Some((_, url)) = images.iter().find(|(name, _)| name == map) {
                button["image"] = serde_json::json!({ "type": "url", "data": url });
            }
            button
        })
        .collect();
    serde_json::json!({
        "type": "form",
        "title": "Auto CS",
        "content": "Pick the map for your BedWars Squads custom server. Your party plays §cRED§r; each map shows the team you face.",
        "buttons": buttons,
    })
}

/// Removes Minecraft formatting codes (`§` and the character after it).
pub fn strip(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// The player named by `@Name spec` (case and spacing aside), if `message` is that.
fn spec_target(message: &str) -> Option<&str> {
    let rest = message.trim().strip_prefix('@')?;
    let at = rest.len().checked_sub(" spec".len())?;
    let (name, word) = (rest.get(..at)?, rest.get(at..)?);
    (word.eq_ignore_ascii_case(" spec") && !name.trim().is_empty()).then(|| name.trim())
}

/// The player's name at the end of the text before " joined." or " left." ("» Name").
fn player_name(before: &str) -> &str {
    before.rsplit("» ").next().unwrap_or(before).trim()
}

/// The player and role in Hive's "Set NAME's role to ROLE" message.
fn role_set(plain: &str) -> Option<(&str, &str)> {
    let (_, rest) = plain.split_once("Set ")?;
    let (name, role) = rest.split_once("'s role to ")?;
    let (name, role) = (name.trim(), role.trim());
    (!name.is_empty() && !role.is_empty()).then_some((name, role))
}

/// Who was invited in Hive's invite messages, and by whom if not by the player: "Invite sent to
/// NAME" (the player's own) and "OTHER sent an invite to NAME" (another player with the right
/// to invite, recorded 2026-11-06).
fn invite_sent(plain: &str) -> Option<(Option<&str>, &str)> {
    let (by, name) = match plain.split_once(" sent an invite to ") {
        Some((before, name)) => (Some(player_name(before)), name),
        None => (None, plain.split_once("Invite sent to ")?.1),
    };
    let name = name.trim();
    (!name.is_empty()).then_some((by, name))
}

/// What follows `name` when `line` starts with it as a whole word, ignoring case.
fn word<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let head = line.get(..name.len())?;
    let rest = &line[name.len()..];
    (head.eq_ignore_ascii_case(name) && rest.chars().next().is_none_or(char::is_whitespace))
        .then_some(rest)
}

/// Whether `text` holds `name` with no letter, digit or underscore on either side.
fn contains_word(text: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let part = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    text.match_indices(name).any(|(at, _)| {
        !part(text[..at].chars().next_back()) && !part(text[at + name.len()..].chars().next())
    })
}

/// The `n` of a join or leave message's `[n/m]`.
fn lobby_count(text: &str) -> Option<u32> {
    let open = text.rfind('[')?;
    let (count, _) = text[open + 1..].split_once('/')?;
    count.trim().parse().ok()
}

/// The map named `wanted`, compared as typeable words (`souls_end`, or "Soul's End"): an exact
/// match, else the only one starting with it.
fn find_map(maps: &[String], wanted: &str) -> Option<String> {
    let wanted = slug(wanted);
    if wanted.is_empty() {
        return None;
    }
    if let Some(map) = maps.iter().find(|map| slug(map) == wanted) {
        return Some(map.clone());
    }
    let mut matches = maps.iter().filter(|map| slug(map).starts_with(&wanted));
    match (matches.next(), matches.next()) {
        (Some(map), None) => Some(map.clone()),
        _ => None,
    }
}
