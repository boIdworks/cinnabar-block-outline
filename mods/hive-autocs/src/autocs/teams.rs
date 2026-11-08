//! Team and role assignment, and the spectator and team reports.

use super::*;

impl AutoCs {
    /// Lets the teams be set at the next right-click, reading the party again first: players
    /// join the party after `!autocs`, and one the mod doesn't know is put on the other team.
    pub(super) fn ready_for_teams(&mut self) {
        if !self.run.roles_ready && self.run.party_refresh.is_none() {
            self.run.party_refresh = Some(self.tick + PARTY_TICKS);
            self.run.party_new.clear();
            self.command(PARTY_COMMAND);
        }
        self.run.roles_ready = true;
    }

    /// Whether the teams are to be set now.
    pub(super) fn teams_now(&self) -> bool {
        self.run.roles == Task::Pending && self.run.roles_ready
    }

    /// The team the player belongs on; `None` leaves them for Hive to place.
    pub(super) fn team_of(&self, player: &str) -> Option<&'static str> {
        if self.in_party(player) {
            Some("RED")
        } else {
            opponent(&self.run.map)
        }
    }

    /// The role the mod gives the player now: Spectator for `@Name spec`, their team while the
    /// teams are being set, else none (left as they are).
    pub(super) fn role_of(&self, player: &str) -> Option<&'static str> {
        if self.is_spec(player) {
            Some("Spectator")
        } else if self.teams_now() {
            self.team_of(player)
        } else {
            None
        }
    }

    /// Where everyone outside the party goes, for chat.
    pub(super) fn others(&self) -> String {
        match pairing(&self.run.map) {
            Some((team, true)) => format!("everyone else {}{team}§r", colour(team)),
            Some((team, false)) => format!(
                "everyone else placed by Hive (§b{}§r is probably {}{team}§r, not confirmed)",
                self.run.map,
                colour(team)
            ),
            None => format!(
                "everyone else placed by Hive (§b{}§r has no team pairing set)",
                self.run.map
            ),
        }
    }

    pub(super) fn in_party(&self, player: &str) -> bool {
        player.eq_ignore_ascii_case(&self.me)
            || contains_word(&self.run.party, &player.to_lowercase())
    }

    /// The first listed player not yet on their team, counting a try for them.
    pub(super) fn next_player(&mut self, menu: &Menu) -> Option<usize> {
        for (index, button) in menu.buttons.iter().enumerate() {
            let Some(team) = self.role_of(&button.first) else {
                continue;
            };
            if button.second.eq_ignore_ascii_case(team) {
                continue;
            }
            let tries = match self
                .run
                .role_tries
                .iter_mut()
                .find(|(name, _)| *name == button.first)
            {
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
    pub(super) fn report_specs(&mut self, menu: &Menu) {
        let requested = std::mem::take(&mut self.run.requested);
        let (here, missing): (Vec<String>, Vec<String>) = requested.into_iter().partition(|spec| {
            menu.buttons
                .iter()
                .any(|button| button.first.eq_ignore_ascii_case(spec))
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

    pub(super) fn report_teams(&mut self, menu: &Menu) {
        let names = |team: &str| -> String {
            let list: Vec<&str> = menu
                .buttons
                .iter()
                .filter(|button| button.second.eq_ignore_ascii_case(team))
                .map(|button| button.first.as_str())
                .collect();
            if list.is_empty() {
                "-".into()
            } else {
                list.join(", ")
            }
        };
        let red = names("RED");
        let (other, others) = match opponent(&self.run.map) {
            Some(team) => (format!("{}{team}§r", colour(team)), names(team)),
            None => ("§7Hive places§r".to_string(), names("Player")),
        };
        self.notes
            .push(format!("AUTOCS teams red={red:?} other={others:?}"));
        self.say(&format!("Teams: §cRED§r {red} | {other} {others}"));
    }

    // ---- output -------------------------------------------------------------------------------
}
