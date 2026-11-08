//! What the player types: the autocs command, map picks, begin and end, spectating and teams.

use super::*;

impl AutoCs {
    /// Chat meant for this mod; true when it was.
    pub(super) fn typed(&mut self, message: &str) -> bool {
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

    pub(super) fn autocs(&mut self, argument: &str) {
        if ["stop", "cancel"]
            .iter()
            .any(|stop| argument.eq_ignore_ascii_case(stop))
        {
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
        if let Some(hive) = self.changing.take() {
            self.cancel(hive);
        }
        if argument.is_empty() {
            self.run.stage = Stage::Picking;
            self.run.picker = menu_order(&self.maps);
            let form = picker_form(&self.run.picker, &self.images, NEW_SERVER_TEXT);
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

    pub(super) fn picked(&mut self, response: Option<&str>) {
        if let Some(hive) = self.changing.take() {
            self.changed(hive, response);
            return;
        }
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
    pub(super) fn begin(&mut self, map: String) {
        self.notes.push(format!(
            "AUTOCS start map={map:?} invites={:?}",
            self.invites
        ));
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

    /// Ends a set of games: stops any run and forgets the players to invite again.
    pub(super) fn end(&mut self) {
        if let Some(hive) = self.changing.take() {
            self.cancel(hive);
        }
        self.reset();
        let names = std::mem::take(&mut self.invites);
        let specs = std::mem::take(&mut self.specs);
        self.run.to_invite.clear();
        self.run.requested.clear();
        self.run.spec_pending = false;
        self.notes
            .push(format!("AUTOCS end invites={names:?} specs={specs:?}"));
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
            self.say(&format!(
                "Game over. {}; the next game starts fresh.",
                parts.join(", and ")
            ));
        }
    }

    /// `@Name spec`: make the player a spectator at the next right-click.
    pub(super) fn spectate(&mut self, player: &str) {
        if player.eq_ignore_ascii_case(&self.me) {
            self.say("You can't make yourself a spectator from here; use Set Roles.");
            return;
        }
        self.notes.push(format!("AUTOCS spectate {player:?}"));
        self.remember_spec(player);
        if !self
            .run
            .requested
            .iter()
            .any(|known| known.eq_ignore_ascii_case(player))
        {
            self.run.requested.push(player.to_owned());
        }
        self.run.spec_pending = true;
        if self.run.stage == Stage::Idle {
            // A server made by hand: answer its menus too.
            self.run.stage = Stage::Lobby;
        }
        if self.run.stage == Stage::Lobby {
            self.say(&format!(
                "§eRight-click the settings item§r to make §e{player}§r a spectator."
            ));
        } else {
            self.say(&format!(
                "I'll make §e{player}§r a spectator once you're in the server."
            ));
        }
    }

    pub(super) fn remember_spec(&mut self, player: &str) {
        if !self
            .specs
            .iter()
            .any(|known| known.eq_ignore_ascii_case(player))
        {
            self.specs.push(player.to_owned());
        }
    }

    pub(super) fn is_spec(&self, player: &str) -> bool {
        self.specs
            .iter()
            .any(|spec| spec.eq_ignore_ascii_case(player))
    }

    pub(super) fn teams(&mut self) {
        if self.run.stage != Stage::Lobby || self.run.roles != Task::Pending {
            self.say("Teams are set during an §e!autocs§r run, once you're in the server.");
            return;
        }
        self.ready_for_teams();
        self.run.roles_prompted = true;
        self.say("Right-click the §esettings item§r to set the teams.");
    }

    // ---- server packets -----------------------------------------------------------------------
}
