//! Adds `/autocs` to the command list the server sends, so the game suggests and fills it in as
//! it is typed: `/autocs <map>` with every map name, and `/autocs stop` / `/autocs teams`. The
//! server never sees the command; [`crate::autocs`] catches it.

use bedrock_protocol::packets::AvailableCommands;
use bedrock_protocol::types::{CommandData, CommandEnum, CommandOverload, CommandParameter};
use bedrock_protocol::wire::{self, Decode, Encode};

use crate::autocs::MAPS;
use crate::wire::{Reader, write_var_u32};

pub const AVAILABLE_COMMANDS: u32 = AvailableCommands::ID;
pub const NAME: &str = "autocs";
/// The actions besides a map.
pub const ACTIONS: [&str; 3] = ["stop", "teams", "end"];

/// Command parameter flags (Bedrock `CommandArgFlag`).
const ARG_VALID: u32 = 0x10_0000;
const ARG_ENUM: u32 = 0x20_0000;

/// A map name as one typeable word: lowercase letters and digits, words joined by `_`
/// ("Soul's End" → `souls_end`, "Coven (Halloween)" → `coven_halloween`).
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for word in name.split(|c: char| !c.is_alphanumeric() && c != '\'') {
        let word: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
        if !word.is_empty() {
            if !out.is_empty() {
                out.push('_');
            }
            out.push_str(&word.to_lowercase());
        }
    }
    out
}

/// The AvailableCommands packet (header included) with `/autocs` added; `None` when it cannot be
/// read or already has the command, so the original is forwarded.
pub fn with_autocs(packet: &[u8]) -> Option<Vec<u8>> {
    let mut header = Reader::new(packet);
    let header_value = header.var_u32()?;
    let body = header.rest();
    let mut commands = AvailableCommands::decode(&mut wire::Reader::new(body)).ok()?;
    if commands.commands.iter().any(|command| command.name.to_string_lossy() == NAME) {
        return None;
    }
    // Only rewrite a list the codec reproduces byte for byte, so nothing of Hive's is lost.
    let mut same = Vec::with_capacity(body.len());
    commands.encode(&mut wire::Writer::new(&mut same)).ok()?;
    if same != body {
        return None;
    }
    let maps = add_enum(&mut commands, "AutoCsMap", MAPS.iter().map(|map| slug(map)));
    let actions = add_enum(&mut commands, "AutoCsAction", ACTIONS.iter().map(|action| action.to_string()));
    let parameter = |name: &str, index: u32| CommandParameter {
        name: name.into(),
        parse_symbol: wire::U32LE(ARG_VALID | ARG_ENUM | index),
        is_optional: false,
        options: wire::U8(0),
    };
    let overload = |parameter_data| CommandOverload { is_chaining: false, parameter_data };
    commands.commands.push(CommandData {
        name: NAME.into(),
        description: "Auto CS: make a BedWars Squads custom server".into(),
        flags: wire::U16LE(0),
        permission_level: "any".into(),
        alias_enum: wire::I32LE(-1),
        command_data_chained_subcommand_indexes: Vec::new(),
        overloads: vec![
            overload(Vec::new()),
            overload(vec![parameter("map", maps)]),
            overload(vec![parameter("action", actions)]),
        ],
    });
    let mut out = Vec::with_capacity(packet.len() + 1024);
    write_var_u32(&mut out, header_value);
    commands.encode(&mut wire::Writer::new(&mut out)).ok()?;
    Some(out)
}

/// Adds an enum of `values`, reusing values already listed; returns its index.
fn add_enum(commands: &mut AvailableCommands, name: &str, values: impl Iterator<Item = String>) -> u32 {
    let indexes = values
        .map(|value| {
            let index = match commands.enum_values.iter().position(|known| known.to_string_lossy() == value) {
                Some(index) => index,
                None => {
                    commands.enum_values.push(value.as_str().into());
                    commands.enum_values.len() - 1
                }
            };
            wire::U32LE(index as u32)
        })
        .collect();
    commands.enum_data.push(CommandEnum { name: name.into(), values: indexes });
    (commands.enum_data.len() - 1) as u32
}
