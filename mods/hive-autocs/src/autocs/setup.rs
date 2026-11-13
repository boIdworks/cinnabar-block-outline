//! Creating the party and applying its settings, preset and map.

use super::*;

impl AutoCs {
    /// A Hive menu; true when the mod answers it instead of the player.
    pub(super) fn form(&mut self, form: u32, json: &str) -> bool {
        let Ok(value) = serde_json::from_str::<Value>(json) else {
            return false;
        };
        let menu = Menu::new(&value);
        if menu.title == "Game Over!" && self.awaiting_overview {
            self.awaiting_overview = false;
            if self.run.stage != Stage::Lobby {
                self.cancel(form);
                return true;
            }
        }
        if self.run.started_by_autocs
            && self.run.stage == Stage::Lobby
            && menu.title == "Game Over!"
        {
            let mut replay = value.clone();
            if let Some(buttons) = replay.get_mut("buttons").and_then(Value::as_array_mut) {
                buttons.insert(
                    0,
                    serde_json::json!({"text": "Replay CS\nChoose the next map"}),
                );
                self.replay_form = Some((form, 0));
                self.replay_overview = Some(replay.to_string());
                packets::form_request(&mut self.to_client, form, &replay.to_string());
                return true;
            }
        }
        if menu.title == "Map"
            && !self.run.navigating
            && matches!(self.run.stage, Stage::Idle | Stage::Lobby)
            && menu
                .buttons
                .iter()
                .any(|button| position(&MAPS, &button.first, |map| map).is_some())
        {
            return self.change_map(form, &menu);
        }
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

    pub(super) fn creating(&mut self, form: u32, menu: &Menu) -> bool {
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
                self.fail(&format!(
                    "Hive showed §e{}§r, which I didn't expect. It's yours now.",
                    menu.title
                ));
                false
            }
        }
    }

    /// The server's settings menus.
    pub(super) fn settings(&mut self, form: u32, menu: &Menu) -> bool {
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
        let parent = self
            .run
            .closing_until
            .take()
            .is_some_and(|until| self.tick <= until);
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
                        self.say(&format!(
                            "No saved preset called §e{PRESET}§r; skipping it."
                        ));
                    }
                    menu.button("Go Back")
                }
            }
            "Load Preset?" => {
                if self.run.preset == Task::Pending && menu.content.to_lowercase().contains(PRESET)
                {
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
                let open = if access {
                    menu.button_with(&OPEN_WORDS, &NOT_OPEN_WORDS)
                } else {
                    None
                };
                if let Some(index) = open {
                    self.run.access = Task::Done;
                    self.notes.push(format!(
                        "AUTOCS access mode {:?}",
                        menu.buttons[index].first
                    ));
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
                if invites {
                    menu.button("Invite Username")
                } else {
                    menu.button("Go Back")
                }
            }
            "Map" => {
                self.learn_maps(menu);
                match menu.button(&self.run.map) {
                    Some(index) if self.run.map_task == Task::Pending => {
                        self.run.map_task = Task::Done;
                        self.notes
                            .push(format!("AUTOCS map set {:?}", self.run.map));
                        Some(index)
                    }
                    _ => {
                        self.run.map_task = Task::Failed;
                        self.say(&format!(
                            "Hive has no map called §e{}§r now; pick it by hand.",
                            self.run.map
                        ));
                        return self.finish(form, true);
                    }
                }
            }
            "Set Roles" if menu.content.starts_with("Setting role for ") => {
                let player = menu.content["Setting role for ".len()..]
                    .trim()
                    .trim_end_matches('.');
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
                        self.run.present = menu
                            .buttons
                            .iter()
                            .map(|button| button.first.clone())
                            .collect();
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
                self.say(&format!(
                    "Couldn't find the right button in §e{}§r; it's yours.",
                    menu.title
                ));
                false
            }
        }
    }

    /// Keeps Hive's live map list and pictures from its Map menu.
    pub(super) fn learn_maps(&mut self, menu: &Menu) {
        self.maps = menu
            .buttons
            .iter()
            .map(|button| button.first.clone())
            .collect();
        for button in &menu.buttons {
            if let Some(url) = &button.image {
                match self.images.iter_mut().find(|(map, _)| *map == button.first) {
                    Some((_, known)) => known.clone_from(url),
                    None => self.images.push((button.first.clone(), url.clone())),
                }
            }
        }
    }

    /// Hive's Map menu, opened by the player: hold it and show the mod's map menu instead.
    pub(super) fn change_map(&mut self, form: u32, menu: &Menu) -> bool {
        self.learn_maps(menu);
        if let Some(stale) = self.changing.replace(form) {
            self.cancel(stale);
        }
        let current = menu
            .buttons
            .iter()
            .find(|button| button.second.to_lowercase().contains("selected"))
            .map(|button| button.first.clone());
        self.notes
            .push(format!("AUTOCS change map from={current:?}"));
        self.run.picker = menu_order(&self.maps);
        let content = match current {
            Some(map) => format!("Now: §b{map}§r. {CHANGE_TEXT}"),
            None => CHANGE_TEXT.to_string(),
        };
        let json = picker_form(&self.run.picker, &self.images, &content);
        packets::form_request(&mut self.to_client, PICKER_FORM, &json.to_string());
        true
    }

    /// The answer to the mod's map menu shown for Hive's Map menu `hive`: press the map there.
    pub(super) fn changed(&mut self, hive: u32, response: Option<&str>) {
        let map = response
            .and_then(|response| response.trim().parse::<usize>().ok())
            .and_then(|index| self.run.picker.get(index).cloned());
        let Some(map) = map else {
            // Closed: close Hive's too, which takes the player back to Game Settings.
            self.cancel(hive);
            return;
        };
        let Some(index) = self.maps.iter().position(|known| *known == map) else {
            self.cancel(hive);
            self.say(&format!(
                "Hive has no map called §e{map}§r now; pick it by hand."
            ));
            return;
        };
        self.answer(hive, index);
        self.notes.push(format!("AUTOCS map changed {map:?}"));
        let in_run = !self.run.map.is_empty();
        let new_teams =
            in_run && self.run.roles == Task::Done && opponent(&self.run.map) != opponent(&map);
        self.run.map = map.clone();
        if self.run.map_task == Task::Pending {
            // Set by hand before the mod got to it: keep the player's pick.
            self.run.map_task = Task::Reported;
        }
        if new_teams {
            // Hive shows Game Settings next: go on to Set Roles and move everyone.
            self.run.stage = Stage::Lobby;
            self.run.roles = Task::Pending;
            self.run.roles_ready = true;
            self.run.role_tries.clear();
            self.run.steps = 0;
            self.run.navigating = true;
            self.say(&format!(
                "Map changed to §b{map}§r. Setting the teams again: {}.",
                self.others()
            ));
        } else if in_run && self.run.roles == Task::Pending {
            self.say(&format!(
                "Map changed to §b{map}§r. The teams will be {}.",
                self.others()
            ));
        } else {
            self.say(&format!("Map changed to §b{map}§r."));
        }
    }

    /// Closes the open menu and reports what is left. Closing a `nested` menu makes Hive show
    /// its parent, which the mod answers too (on to Start Game, or closed).
    pub(super) fn finish(&mut self, form: u32, nested: bool) -> bool {
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
                    self.ready_for_teams();
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
}
