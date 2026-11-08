//! The packet-mod exports: every batch goes through the recorder and the automatic setup, and
//! the recorder's lines go to standard output, which Cinnabar copies into `client.log`.

use std::cell::RefCell;

use mod_api::packet_mod::{
    Guest,
    cinnabar::extension::packet_relay::{Direction, Outbound, SessionIdentity, SessionPack},
};

use crate::autocs::AutoCs;
use crate::commands;
use crate::packets;
use crate::recorder::{Recorder, Way};
use crate::wire::{rewrite, split_batch};

mod stdout {
    wit_bindgen::generate!({ path: "wit", world: "boldy:autocs/logging", generate_all });
}

/// Ticks (50 ms each) after the session starts before greeting the player, so the message
/// arrives once the world is on screen.
const GREETING_TICKS: u32 = 200;

struct Session {
    recorder: Recorder,
    auto: AutoCs,
    ticks: u32,
    greeted: bool,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

fn flush(session: &mut Session) {
    for note in session.auto.take_notes() {
        session.recorder.note(note);
    }
    let lines = session.recorder.drain();
    if lines.is_empty() {
        return;
    }
    let stream = stdout::wasi::cli::stdout::get_stdout();
    for line in lines {
        let mut bytes = line.into_bytes();
        bytes.push(b'\n');
        if stream.blocking_write_and_flush(&bytes).is_err() {
            return;
        }
    }
}

/// Runs `f` on the session, if one has started.
fn with<T>(f: impl FnOnce(&mut Session) -> T) -> Option<T> {
    SESSION.with_borrow_mut(|session| session.as_mut().map(f))
}

struct Mod;

impl Guest for Mod {
    fn start(
        identity: SessionIdentity,
        _client_data: String,
        _startup: Vec<u8>,
    ) -> Result<Vec<SessionPack>, String> {
        let mut session = Session {
            recorder: Recorder::default(),
            auto: AutoCs::new(identity.display_name.clone()),
            ticks: 0,
            greeted: false,
        };
        session.recorder.note(format!(
            "session start player={:?} hive-autocs v{}",
            identity.display_name,
            env!("CARGO_PKG_VERSION")
        ));
        flush(&mut session);
        SESSION.with_borrow_mut(|slot| *slot = Some(session));
        Ok(Vec::new())
    }

    fn on_batch(direction: Direction, batch: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
        let way = match direction {
            Direction::FromServer => Way::FromServer,
            Direction::FromClient => Way::FromClient,
        };
        Ok(with(|session| {
            let Some(packets) = split_batch(&batch) else {
                session
                    .recorder
                    .note(format!("unreadable {way:?} batch of {} bytes", batch.len()));
                flush(session);
                return None;
            };
            session.recorder.batch(way, &packets);
            let replaced = rewrite(&packets, batch.len(), |packet| match way {
                Way::FromServer if session.auto.from_server(packet) => Some(None),
                Way::FromClient if session.auto.from_client(packet) => Some(None),
                Way::FromServer if packet.id == commands::AVAILABLE_COMMANDS => {
                    match commands::with_autocs(packet.bytes) {
                        Some(rewritten) => {
                            session.recorder.note("added /autocs to the command list");
                            Some(Some(rewritten))
                        }
                        None => {
                            session.recorder.note(
                                "command list left as sent (unreadable or already has /autocs)",
                            );
                            None
                        }
                    }
                }
                _ => None,
            });
            flush(session);
            replaced
        })
        .flatten())
    }

    fn tick() -> Result<Outbound, String> {
        Ok(with(|session| {
            session.ticks = session.ticks.saturating_add(1);
            let (mut to_client, to_server) = session.auto.tick();
            if !session.greeted && session.ticks >= GREETING_TICKS {
                session.greeted = true;
                packets::system_message(
                    &mut to_client,
                    "§d[Auto CS]§r Ready. Type §e/autocs§r (it fills in the map names) or §e!autocs§r for a BedWars Squads custom server.",
                );
            }
            flush(session);
            Outbound { to_client, to_server }
        })
        .unwrap_or(Outbound { to_client: Vec::new(), to_server: Vec::new() }))
    }

    fn stop() {
        with(|session| {
            let packets = session.recorder.packets();
            session
                .recorder
                .note(format!("session end after {packets} packets"));
            flush(session);
        });
    }
}

mod_api::packet_mod::export_packet_mod!(Mod with_types_in mod_api::packet_mod);
