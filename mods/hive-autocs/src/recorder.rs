//! The recorder: writes every form, data-driven screen, chat line and command of a session to
//! Cinnabar's log, so a custom-server setup done by hand can be replayed by the mod.

use std::collections::{BTreeSet, VecDeque};

use crate::packets;
use crate::wire::Packet;

/// Marks the recorder's lines in `client.log`.
pub const TAG: &str = "[autocs]";
/// Marks the continuation of a line split to fit the host's 1024-byte log lines.
pub const MORE: &str = "[autocs]+";
/// Text per log line, below the host's 1024-byte limit with room for the tag.
const CHUNK: usize = 900;
/// Log bytes written per callback; the host allows 16 KiB and drops the rest.
const BYTES_PER_CALL: usize = 12 * 1024;
/// Hex shown for packets kept as raw bytes.
const HEX_LIMIT: usize = 1024;

#[derive(Default)]
pub struct Recorder {
    pending: VecDeque<String>,
    seen: [BTreeSet<u32>; 2],
    /// Packets of either direction since the session began.
    count: u64,
}

/// Which way a batch travels.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Way {
    FromServer,
    FromClient,
}

impl Recorder {
    pub fn note(&mut self, text: impl Into<String>) {
        let text: String = text.into().replace(['\r', '\n'], "\\n");
        let mut rest = text.as_str();
        let mut first = true;
        loop {
            let mut cut = rest.len().min(CHUNK);
            while !rest.is_char_boundary(cut) {
                cut -= 1;
            }
            let (head, tail) = rest.split_at(cut);
            let tag = if first { TAG } else { MORE };
            self.pending.push_back(format!("{tag} {head}"));
            first = false;
            rest = tail;
            if rest.is_empty() {
                break;
            }
        }
    }

    /// Logs every packet of a batch.
    pub fn batch(&mut self, way: Way, packets: &[Packet<'_>]) {
        for packet in packets {
            self.count += 1;
            self.packet(way, packet);
        }
    }

    /// Logs one packet.
    fn packet(&mut self, way: Way, packet: &Packet<'_>) {
        let side = if way == Way::FromServer { "S>C" } else { "C>S" };
        if self.seen[way as usize].insert(packet.id) {
            self.note(format!("{side} first packet id {} ({} bytes)", packet.id, packet.bytes.len()));
        }
        let id = packet.id;
        match (way, id) {
            (_, packets::TEXT) => {
                let Some(text) = packets::read_text(packet) else {
                    self.raw(side, packet);
                    return;
                };
                let author = text.author.as_deref().unwrap_or("");
                self.note(format!(
                    "{side} text type={} author={author:?} msg={:?} params={:?}",
                    text.kind, text.message, text.params
                ));
            }
            (_, packets::MODAL_FORM_REQUEST | packets::SERVER_SETTINGS_RESPONSE) => {
                match packets::read_form_request(packet) {
                    Some((form, json)) => self.note(format!("{side} form id={id} form={form} json={json}")),
                    None => self.raw(side, packet),
                }
            }
            (_, packets::MODAL_FORM_RESPONSE) => match packets::read_form_response(packet) {
                Some((form, response, reason)) => self.note(format!(
                    "{side} form-response form={form} response={response:?} cancel={reason:?}"
                )),
                None => self.raw(side, packet),
            },
            (_, packets::COMMAND_REQUEST) => match packets::read_command(packet) {
                Some(command) => self.note(format!("{side} command {command:?}")),
                None => self.raw(side, packet),
            },
            (_, packets::DDUI_SHOW_SCREEN) => match packets::read_show_screen(packet) {
                Some((screen, form, data)) => {
                    self.note(format!("{side} ddui-show screen={screen:?} form={form} data={data:?}"))
                }
                None => self.raw(side, packet),
            },
            (
                _,
                packets::CONTAINER_OPEN
                | packets::NPC_DIALOGUE
                | packets::CLOSE_FORM
                | packets::DATA_STORE_TO_CLIENT
                | packets::DATA_STORE_TO_SERVER
                | packets::DDUI_CLOSE_SCREEN
                | packets::DDUI_RELOAD
                | packets::DDUI_SCREEN_CLOSED,
            ) => self.raw(side, packet),
            _ => {}
        }
    }

    /// Logs a packet as its readable strings and hex.
    fn raw(&mut self, side: &str, packet: &Packet<'_>) {
        let strings = packets::strings(packet.bytes);
        let hex = packets::hex(packet.bytes, HEX_LIMIT);
        self.note(format!("{side} packet id={} strings={strings:?} hex={hex}", packet.id));
    }

    /// Takes the lines to write in this callback, keeping the rest for later ones.
    pub fn drain(&mut self) -> Vec<String> {
        let mut budget = BYTES_PER_CALL;
        let mut lines = Vec::new();
        while let Some(line) = self.pending.front() {
            if line.len() + 1 > budget {
                break;
            }
            budget -= line.len() + 1;
            lines.extend(self.pending.pop_front());
        }
        lines
    }

    pub fn packets(&self) -> u64 {
        self.count
    }
}

