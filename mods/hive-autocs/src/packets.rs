//! The Bedrock 1.26.60 packets this mod reads or sends, with their fields decoded.

use crate::wire::{Packet, Reader, push_packet, write_string, write_var_u32};

pub const TEXT: u32 = 9;
pub const CONTAINER_OPEN: u32 = 46;
pub const COMMAND_REQUEST: u32 = 77;
pub const MODAL_FORM_REQUEST: u32 = 100;
pub const MODAL_FORM_RESPONSE: u32 = 101;
pub const SERVER_SETTINGS_RESPONSE: u32 = 103;
pub const NPC_DIALOGUE: u32 = 169;
pub const CLOSE_FORM: u32 = 310;
pub const DATA_STORE_TO_CLIENT: u32 = 330;
pub const DATA_STORE_TO_SERVER: u32 = 332;
pub const DDUI_SHOW_SCREEN: u32 = 333;
pub const DDUI_CLOSE_SCREEN: u32 = 334;
pub const DDUI_RELOAD: u32 = 335;
pub const DDUI_SCREEN_CLOSED: u32 = 343;

/// Text packet message types (`TextPacketType`).
pub const TEXT_RAW: u8 = 0;
pub const TEXT_CHAT: u8 = 1;

/// A Text packet's message.
#[derive(Debug, PartialEq)]
pub struct Text {
    pub kind: u8,
    /// The author, for chat and whispers.
    pub author: Option<String>,
    pub message: String,
    /// Translation parameters.
    pub params: Vec<String>,
}

pub fn read_text(packet: &Packet<'_>) -> Option<Text> {
    let mut body = packet.body();
    body.bool()?;
    let shape = body.var_u32()?;
    let kind = body.u8()?;
    Some(match shape {
        0 => Text { kind, author: None, message: body.string()?, params: Vec::new() },
        1 => {
            let author = body.string()?;
            Text { kind, author: Some(author), message: body.string()?, params: Vec::new() }
        }
        2 => {
            let message = body.string()?;
            let count = body.var_u32()?.min(16);
            let params = (0..count).map(|_| body.string()).collect::<Option<_>>()?;
            Text { kind, author: None, message, params }
        }
        _ => return None,
    })
}

/// A server form: its ID and JSON.
pub fn read_form_request(packet: &Packet<'_>) -> Option<(u32, String)> {
    let mut body = packet.body();
    Some((body.var_u32()?, body.string()?))
}

/// A form answer: the form ID, the JSON response (none when cancelled) and the cancel reason.
pub fn read_form_response(packet: &Packet<'_>) -> Option<(u32, Option<String>, Option<u8>)> {
    let mut body = packet.body();
    let form = body.var_u32()?;
    let response = if body.bool()? { Some(body.string()?) } else { None };
    let reason = if body.bool()? { Some(body.u8()?) } else { None };
    Some((form, response, reason))
}

/// The command line of a CommandRequest.
pub fn read_command(packet: &Packet<'_>) -> Option<String> {
    packet.body().string()
}

/// A data-driven screen to show: its screen ID, form instance and data instance.
pub fn read_show_screen(packet: &Packet<'_>) -> Option<(String, u32, Option<u32>)> {
    let mut body = packet.body();
    let screen = body.string()?;
    let form = body.u32_le()?;
    let data = if body.bool()? { Some(body.u32_le()?) } else { None };
    Some((screen, form, data))
}

/// A system message shown only to this player, as a packet body for the client.
pub fn system_message(batch: &mut Vec<u8>, message: &str) {
    let mut body = Vec::new();
    body.push(0); // not localized
    write_var_u32(&mut body, 0); // message only
    body.push(TEXT_RAW);
    write_string(&mut body, message);
    write_string(&mut body, ""); // sender XUID
    write_string(&mut body, ""); // platform ID
    body.push(0); // no filtered message
    push_packet(batch, TEXT, &body);
}

/// A server form to show the player.
pub fn form_request(batch: &mut Vec<u8>, form: u32, json: &str) {
    let mut body = Vec::new();
    write_var_u32(&mut body, form);
    write_string(&mut body, json);
    push_packet(batch, MODAL_FORM_REQUEST, &body);
}

/// A form answer to send the server: `response` is the JSON value, as the client would send it.
pub fn form_response(batch: &mut Vec<u8>, form: u32, response: &str) {
    let mut body = Vec::new();
    write_var_u32(&mut body, form);
    body.push(1);
    write_string(&mut body, response);
    body.push(0); // no cancel reason
    push_packet(batch, MODAL_FORM_RESPONSE, &body);
}

/// Closes a form, as the client does when the player presses X.
pub fn form_cancel(batch: &mut Vec<u8>, form: u32) {
    let mut body = Vec::new();
    write_var_u32(&mut body, form);
    body.push(0); // no response
    body.push(1);
    body.push(0); // closed by the user
    push_packet(batch, MODAL_FORM_RESPONSE, &body);
}

/// A player command, encoded as Cinnabar's own client sends one; `seed` makes its request UUID.
pub fn command_request(batch: &mut Vec<u8>, line: &str, seed: u64) {
    let mut body = Vec::new();
    write_string(&mut body, line);
    write_string(&mut body, "player"); // origin type
    let mut uuid = [0u8; 16];
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    for chunk in uuid.chunks_mut(8) {
        // SplitMix64: any unique-looking value will do for a request UUID.
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        chunk.copy_from_slice(&(z ^ (z >> 31)).to_le_bytes());
    }
    uuid[6] = (uuid[6] & 0x0f) | 0x40; // version 4
    uuid[8] = (uuid[8] & 0x3f) | 0x80; // RFC 4122 variant
    body.extend_from_slice(&uuid);
    write_string(&mut body, ""); // request ID
    body.extend_from_slice(&0i64.to_le_bytes()); // player ID
    body.push(0); // not internal
    write_string(&mut body, "latest");
    push_packet(batch, COMMAND_REQUEST, &body);
}

/// Readable runs of text in a packet whose layout this mod does not decode: every
/// length-prefixed string of at least three printable characters.
pub fn strings(bytes: &[u8]) -> Vec<String> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let mut reader = Reader::new(&bytes[at..]);
        if let Some(length) = reader.var_u32()
            && (3..=4096).contains(&length)
            && let Some(text) = reader.take(length as usize)
            && let Ok(text) = std::str::from_utf8(text)
            && text.chars().all(|c| !c.is_control() || c == '\n')
        {
            found.push(text.to_owned());
            at = bytes.len() - reader.rest().len();
            continue;
        }
        at += 1;
    }
    found
}

pub fn hex(bytes: &[u8], limit: usize) -> String {
    let mut out: String = bytes.iter().take(limit).map(|byte| format!("{byte:02x}")).collect();
    if bytes.len() > limit {
        out.push_str(&format!("..(+{} bytes)", bytes.len() - limit));
    }
    out
}
