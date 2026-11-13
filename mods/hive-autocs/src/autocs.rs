//! The automatic Bedwars Squads custom server on The Hive.
//!
//! `!autocs` shows a map menu (or takes the map from `!autocs <map>`), reads the party with
//! `/p list`, then creates the server with `/cs` and answers Hive's menus: Create Server, BedWars,
//! Squads. In the server, the player right-clicks the settings item once; the mod then loads the
//! preset, sets the map and closes the menu. When the lobby reaches eight players (or on `!teams`)
//! it asks for one more right-click and puts everyone outside the party on the map's
//! other team; on non-BLUE pairings it also puts the party on RED (see [`opponent`]); on maps without a known pairing Hive places everyone else.
//!
//! Changing the map: when the player opens Hive's Map menu by hand in the server (say, to fix a
//! misclick), the mod shows its own map menu instead and presses the picked map in Hive's. If the
//! teams were already set and the new map faces RED with another team, they are set again.
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

const REPLAY_FORM: u32 = 2_000_000_002;

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
    /// Hive's Map menu the player opened, held while the mod's map menu stands in for it.
    changing: Option<u32>,
    replay_form: Option<(u32, usize)>,
    replay_overview: Option<String>,
    replay_reopen_at: Option<u64>,
    awaiting_overview: bool,
    picker_cancel_until: Option<u64>,
    picker_reopen_at: Option<u64>,
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
    started_by_autocs: bool,
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
    /// Until the tick: the party is being read again, because players join it after `!autocs`
    /// (the first reading only knows the party as it was then).
    party_refresh: Option<u64>,
    /// The reply to the second reading, replacing `party` once it is complete.
    party_new: String,
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

mod control;
mod input;
mod server;
mod setup;
mod teams;

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
            images: IMAGES
                .iter()
                .map(|(map, url)| (map.to_string(), url.to_string()))
                .collect(),
            changing: None,
            replay_form: None,
            replay_overview: None,
            replay_reopen_at: None,
            awaiting_overview: false,
            picker_cancel_until: None,
            picker_reopen_at: None,
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
                    match autocs_command(line) {
                        Some(rest) => self.typed(&format!("!autocs {}", rest.trim())),
                        None => false,
                    }
                }
                None => false,
            },
            packets::MODAL_FORM_RESPONSE => match packets::read_form_response(packet) {
                Some((PICKER_FORM, response, _)) => {
                    self.picked(response.as_deref());
                    true
                }
                Some((form, response, _)) if self.replay_form.is_some_and(|(id, _)| id == form) => {
                    self.replay_form = None;
                    match response
                        .as_deref()
                        .and_then(|s| s.trim().parse::<usize>().ok())
                    {
                        Some(0) => {
                            if form != REPLAY_FORM {
                                self.cancel(form);
                            }
                            self.autocs("");
                            true
                        }
                        Some(index) if form != REPLAY_FORM => {
                            self.answer(form, index - 1);
                            true
                        }
                        _ => form == REPLAY_FORM,
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }
}

mod data;
mod menu;
mod text;

pub(crate) use data::*;
use menu::*;
pub(crate) use text::*;
