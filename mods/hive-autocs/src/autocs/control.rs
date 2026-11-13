//! Driving the run: ticks, answers, commands, failure and reset.

use super::*;

impl AutoCs {
    /// Advances one tick and returns the batches to send: to the client, then to the server.
    pub fn tick(&mut self) -> (Vec<u8>, Vec<u8>) {
        self.tick += 1;
        if self.replay_reopen_at.is_some_and(|at| self.tick >= at) {
            self.replay_reopen_at = None;
            if let Some(json) = &self.replay_overview {
                self.replay_form = Some((REPLAY_FORM, 0));
                packets::form_request(&mut self.to_client, REPLAY_FORM, json);
            }
        }
        if self.picker_reopen_at.is_some_and(|at| self.tick >= at) {
            self.picker_reopen_at = None;
            if self.run.stage == Stage::Picking {
                self.show_picker();
            }
        }
        if self.picker_cancel_until.is_some_and(|at| self.tick >= at) {
            self.picker_cancel_until = None;
            if self.run.stage == Stage::Picking {
                self.reset();
                self.say("Cancelled.");
            }
        }
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
        if let Some(until) = self.run.party_refresh
            && self.tick >= until
        {
            self.run.party_refresh = None;
            let fresh = std::mem::take(&mut self.run.party_new);
            if !fresh.trim().is_empty() {
                let party = fresh.trim().replace('\n', " | ");
                self.notes.push(format!("AUTOCS party refreshed {party:?}"));
                self.run.party = fresh;
            }
        }
        let now = self.tick;
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut self.answers)
            .into_iter()
            .partition(|(at, _)| *at <= now);
        self.answers = later;
        for (_, batch) in due {
            self.to_server.extend(batch);
        }
        (
            std::mem::take(&mut self.to_client),
            std::mem::take(&mut self.to_server),
        )
    }

    /// Lines for the recorder's log.
    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }

    /// Presses a button of a menu.
    pub(super) fn answer(&mut self, form: u32, index: usize) {
        self.notes
            .push(format!("AUTOCS answer form={form} button={index}"));
        self.reply(|batch| packets::form_response(batch, form, &format!("{index}\n")));
    }

    /// Answers a custom form with its JSON response.
    pub(super) fn answer_with(&mut self, form: u32, response: &str) {
        self.notes
            .push(format!("AUTOCS answer form={form} response={response:?}"));
        self.reply(|batch| packets::form_response(batch, form, response));
    }

    pub(super) fn cancel(&mut self, form: u32) {
        self.notes.push(format!("AUTOCS close form={form}"));
        self.reply(|batch| packets::form_cancel(batch, form));
    }

    /// Queues a form answer, sent after [`ANSWER_TICKS`].
    pub(super) fn reply(&mut self, encode: impl FnOnce(&mut Vec<u8>)) {
        let mut batch = Vec::new();
        encode(&mut batch);
        self.answers.push((self.tick + ANSWER_TICKS, batch));
    }

    pub(super) fn command(&mut self, line: &str) {
        self.commands += 1;
        self.notes.push(format!("AUTOCS command {line:?}"));
        packets::command_request(&mut self.to_server, line, self.commands ^ self.tick << 20);
    }

    pub(super) fn say(&mut self, message: &str) {
        packets::system_message(&mut self.to_client, &format!("{TAG} {message}"));
    }

    pub(super) fn fail(&mut self, message: &str) {
        self.notes.push(format!("AUTOCS failed {message:?}"));
        self.say(message);
        self.reset();
    }

    /// Ends the current run; what lasts the session (invites, spectators, maps) stays.
    pub(super) fn reset(&mut self) {
        self.run = Run::default();
        self.replay_overview = None;
        self.replay_reopen_at = None;
        self.replay_form = None;
        self.picker_cancel_until = None;
        self.picker_reopen_at = None;
    }
}
