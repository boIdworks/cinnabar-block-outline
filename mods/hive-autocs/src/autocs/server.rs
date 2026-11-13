//! What the server says: lobby and player messages, forms and the full-lobby check.

use super::*;

impl AutoCs {
    /// A packet from the server; true when it must not reach the player.
    pub fn from_server(&mut self, packet: &Packet<'_>) -> bool {
        match packet.id {
            crate::commands::AVAILABLE_COMMANDS
                if self.run.stage == Stage::Picking && self.changing.is_none() =>
            {
                self.picker_cancel_until = None;
                self.picker_reopen_at = Some(self.tick + 5);
                false
            }
            crate::commands::AVAILABLE_COMMANDS if self.replay_overview.is_some() => {
                self.replay_form = None;
                self.replay_reopen_at = Some(self.tick + 5);
                false
            }
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

    pub(super) fn server_text(&mut self, text: &packets::Text) {
        if text.kind == TEXT_CHAT {
            return;
        }
        let plain = strip(&text.message);
        if is_game_over(&plain)
            && self.run.started_by_autocs
            && self.run.stage == Stage::Lobby
            && !self.awaiting_overview
        {
            let overview = serde_json::json!({
                "type": "form", "title": "Game Over!", "content": "Game over. Choose the next map to replay.",
                "buttons": [{"text": "Replay CS\nChoose the next map"}, {"text": "Close Overview"}]
            }).to_string();
            self.awaiting_overview = true;
            self.replay_form = Some((REPLAY_FORM, 0));
            self.replay_overview = Some(overview.clone());
            packets::form_request(&mut self.to_client, REPLAY_FORM, &overview);
        }
        if let Some((by, name)) = invite_sent(&plain) {
            let by_mod = self
                .run
                .invited
                .iter()
                .any(|known| known.eq_ignore_ascii_case(name));
            if !by_mod
                && !name.eq_ignore_ascii_case(&self.me)
                && !self
                    .invites
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(name))
            {
                self.invites.push(name.to_owned());
                let from = by
                    .map(|by| format!(" (invited by §e{by}§r)"))
                    .unwrap_or_default();
                self.notes
                    .push(format!("AUTOCS remember invite {name:?} by={by:?}"));
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
                self.run
                    .requested
                    .retain(|spec| !spec.eq_ignore_ascii_case(name));
                self.run
                    .spectating
                    .retain(|spec| !spec.eq_ignore_ascii_case(name));
                self.notes
                    .push(format!("AUTOCS forget spec {name:?} role={role:?}"));
                self.say(&format!(
                    "§e{name}§r is playing now; I won't make them a spectator again."
                ));
                self.player_back();
            }
        }
        if let Stage::Party(_) = self.run.stage {
            self.run.party.push_str(&plain.to_lowercase());
            self.run.party.push('\n');
        }
        // Lobby chatter that names players must not read as party members.
        if self.run.party_refresh.is_some() && !is_chatter(&plain) {
            self.run.party_new.push_str(&plain.to_lowercase());
            self.run.party_new.push('\n');
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
            if let Some(name) = plain
                .split_once(" joined.")
                .map(|(before, _)| player_name(before))
            {
                if !self
                    .run
                    .present
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(name))
                {
                    self.run.present.push(name.to_owned());
                }
                if self.run.stage == Stage::Lobby
                    && self.is_spec(name)
                    && !name.eq_ignore_ascii_case(&self.me)
                {
                    self.say(&format!("§e{name}§r will spectate, as in your last game."));
                }
            }
            if let Some(name) = plain
                .split_once(" left.")
                .map(|(before, _)| player_name(before))
            {
                self.run
                    .present
                    .retain(|known| !known.eq_ignore_ascii_case(name));
                self.run
                    .spectating
                    .retain(|spec| !spec.eq_ignore_ascii_case(name));
            }
            self.lobby = count;
            self.check_full();
        }
    }

    /// Players in the server, spectators not counted.
    pub(super) fn players(&self) -> u32 {
        self.lobby.saturating_sub(self.spectators())
    }

    /// Spectators in the server: those Hive shows as Spectator, and remembered spectators who
    /// joined and will be set with the teams.
    pub(super) fn spectators(&self) -> u32 {
        let mut names: Vec<String> = self
            .run
            .spectating
            .iter()
            .map(|name| name.to_lowercase())
            .collect();
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
    pub(super) fn lobby_text(&self) -> String {
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
    pub(super) fn player_back(&mut self) {
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
        self.notes
            .push("AUTOCS lobby full after a spectator came back; setting the teams".into());
        self.say(&format!(
            "{}. Setting the teams now: your party §cRED§r, {} (if the menu closed, §eright-click the settings item§r).",
            self.lobby_text(),
            self.others()
        ));
    }

    /// What is wrong with a lobby that isn't exactly full, for chat.
    pub(super) fn not_full_text(&self) -> String {
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

    pub(super) fn check_full(&mut self) {
        if self.run.stage != Stage::Lobby || self.run.roles != Task::Pending {
            return;
        }
        // Spectators do not count: 7 players and 1 spectator wait for an eighth player, and 9
        // players are too many for the 4v4.
        if self.players() == FULL_LOBBY {
            if !self.run.roles_prompted {
                self.ready_for_teams();
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
}
