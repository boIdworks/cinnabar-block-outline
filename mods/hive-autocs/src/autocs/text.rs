//! Reading and ordering the Hive's text: menus, names, chat and commands.

use super::*;

/// The map menu's order: Random, then [`RANKING`] top to bottom, then any other map in Hive's
/// order. Each button still shows the team faced there.
pub fn menu_order(maps: &[String]) -> Vec<String> {
    order_by(maps, RANKING)
}

/// [`menu_order`] with `ranking` in place of [`RANKING`].
pub fn order_by(maps: &[String], ranking: &[&str]) -> Vec<String> {
    let first = |map: &str| if map == "Random" { 0 } else { 1 };
    let rank = |map: &str| position(ranking, map, |ranked| ranked).unwrap_or(usize::MAX);
    let mut ordered: Vec<(usize, &String)> = maps.iter().enumerate().collect();
    ordered.sort_by_key(|(index, map)| (first(map), rank(map), *index));
    ordered.into_iter().map(|(_, map)| map.clone()).collect()
}

/// The map menu: each map with the team the party faces there and Hive's picture.
pub(crate) fn picker_form(maps: &[String], images: &[(String, String)], content: &str) -> Value {
    let buttons: Vec<Value> = maps
        .iter()
        .map(|map| {
            let detail = match (map.as_str(), pairing(map)) {
                ("Random", _) => "§8Hive picks the map".to_string(),
                (_, Some((team, true))) => format!("§8vs {}{team}", colour(team)),
                (_, Some((team, false))) => format!("§8vs {}{team} §7(unconfirmed)", colour(team)),
                _ => "§8Hive sets the other team".to_string(),
            };
            let mut button = serde_json::json!({ "text": format!("§l{map}§r\n{detail}") });
            if let Some((_, url)) = images.iter().find(|(name, _)| name == map) {
                button["image"] = serde_json::json!({ "type": "url", "data": url });
            }
            button
        })
        .collect();
    serde_json::json!({
        "type": "form",
        "title": "Auto CS",
        "content": content,
        "buttons": buttons,
    })
}

/// The map menu's text for a new server, and for changing the map in one.
pub(crate) const NEW_SERVER_TEXT: &str = "Pick the map for your BedWars Squads custom server. Your party plays §cRED§r; each map shows the team you face.";

pub(crate) const CHANGE_TEXT: &str =
    "Pick the new map. Your party plays §cRED§r; each map shows the team you face.";

/// Removes Minecraft formatting codes (`§` and the character after it).
pub fn strip(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// The player named by `@Name spec` (case and spacing aside), if `message` is that.
pub(crate) fn spec_target(message: &str) -> Option<&str> {
    let rest = message.trim().strip_prefix('@')?;
    let at = rest.len().checked_sub(" spec".len())?;
    let (name, word) = (rest.get(..at)?, rest.get(at..)?);
    (word.eq_ignore_ascii_case(" spec") && !name.trim().is_empty()).then(|| name.trim())
}

/// Lines of the lobby's own chatter that must not be read as the party list: joins, leaves,
/// invites and role changes name players who may not be in the party.
pub(crate) fn is_chatter(plain: &str) -> bool {
    plain.contains(" joined.")
        || plain.contains(" left.")
        || plain.contains("nvite")
        || role_set(plain).is_some()
}

/// Whether `plain` (formatting removed) is Hive's end-of-game line, `» Game OVER!`. Whatever
/// symbol Hive puts in front of it does not matter.
pub(crate) fn is_game_over(plain: &str) -> bool {
    plain
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim()
        .eq_ignore_ascii_case("Game OVER!")
}

/// The player's name at the end of the text before " joined." or " left." ("» Name").
pub(crate) fn player_name(before: &str) -> &str {
    before.rsplit("» ").next().unwrap_or(before).trim()
}

/// The player and role in Hive's "Set NAME's role to ROLE" message.
pub(crate) fn role_set(plain: &str) -> Option<(&str, &str)> {
    let (_, rest) = plain.split_once("Set ")?;
    let (name, role) = rest.split_once("'s role to ")?;
    let (name, role) = (name.trim(), role.trim());
    (!name.is_empty() && !role.is_empty()).then_some((name, role))
}

/// Who was invited in Hive's invite messages, and by whom if not by the player: "Invite sent to
/// NAME" (the player's own) and "OTHER sent an invite to NAME" (another player with the right
/// to invite, recorded 2026-11-06).
pub(crate) fn invite_sent(plain: &str) -> Option<(Option<&str>, &str)> {
    let (by, name) = match plain.split_once(" sent an invite to ") {
        Some((before, name)) => (Some(player_name(before)), name),
        None => (None, plain.split_once("Invite sent to ")?.1),
    };
    let name = name.trim();
    (!name.is_empty()).then_some((by, name))
}

/// What follows `name` when `line` starts with it as a whole word, ignoring case.
pub(crate) fn word<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let head = line.get(..name.len())?;
    let rest = &line[name.len()..];
    (head.eq_ignore_ascii_case(name) && rest.chars().next().is_none_or(char::is_whitespace))
        .then_some(rest)
}

/// The rest of a slash command meant for this mod: `autocs` and slips of it, any command word
/// starting with `auto` (`/auto end`, `/autocsto end`). Hive has no such command, so nothing of
/// Hive's is caught.
pub(crate) fn autocs_command(line: &str) -> Option<&str> {
    let end = line.find(char::is_whitespace).unwrap_or(line.len());
    let (name, rest) = line.split_at(end);
    name.get(..4)
        .is_some_and(|head| head.eq_ignore_ascii_case("auto"))
        .then_some(rest)
}

/// Whether `text` holds `name` with no letter, digit or underscore on either side.
pub(crate) fn contains_word(text: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let part = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    text.match_indices(name).any(|(at, _)| {
        !part(text[..at].chars().next_back()) && !part(text[at + name.len()..].chars().next())
    })
}

/// The `n` of a join or leave message's `[n/m]`.
pub(crate) fn lobby_count(text: &str) -> Option<u32> {
    let open = text.rfind('[')?;
    let (count, _) = text[open + 1..].split_once('/')?;
    count.trim().parse().ok()
}

/// The map named `wanted`, compared as typeable words (`souls_end`, or "Soul's End"): an exact
/// match, else the only one starting with it.
pub(crate) fn find_map(maps: &[String], wanted: &str) -> Option<String> {
    let wanted = slug(wanted);
    if wanted.is_empty() {
        return None;
    }
    if let Some(map) = maps.iter().find(|map| slug(map) == wanted) {
        return Some(map.clone());
    }
    let mut matches = maps.iter().filter(|map| slug(map).starts_with(&wanted));
    match (matches.next(), matches.next()) {
        (Some(map), None) => Some(map.clone()),
        _ => None,
    }
}
