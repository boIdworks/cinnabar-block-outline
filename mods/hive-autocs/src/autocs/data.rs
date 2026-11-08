//! The fixed Hive data: commands, words, maps, the ranking, images and opponents.

use super::*;

/// The saved preset loaded on every automatic server.
pub const PRESET: &str = "new scrim rules";

/// The command that lists the party.
pub const PARTY_COMMAND: &str = "/p list";

/// The command that brings the party to the new server, sent once it is open to everyone.
pub const WARP_COMMAND: &str = "/p warp";

/// Words in the Access Mode button that opens the server to everyone (Hive's is "Set To Open to
/// All Mode", recorded 2026-11-06; the others are for a reworded menu), and in the buttons that
/// must never be pressed for it.
pub(crate) const OPEN_WORDS: [&str; 5] = ["public", "open", "everyone", "anyone", "all players"];

pub(crate) const NOT_OPEN_WORDS: [&str; 4] = ["invite", "private", "closed", "back"];

/// Players (spectators not counted) the teams are set for: exactly this many, no more, no fewer.
pub const FULL_LOBBY: u32 = 8;

/// Press Start Game in the same right-click, once the teams are set. Off: the player starts the
/// game themselves when they are ready (2026-11-06).
pub const START_AFTER_TEAMS: bool = false;

/// Form ID of the mod's own map menu, far above the small IDs Hive counts up from.
pub const PICKER_FORM: u32 = 2_000_000_001;

/// The Squads maps Hive offered when the menus were recorded (2026-11-05); the menu uses the
/// live list once Hive has shown it this session.
pub const MAPS: [&str; 29] = [
    "Random",
    "Autumn",
    "Cobalt",
    "Coven (Halloween)",
    "Cyberspace",
    "Gingerbread (Winterfest)",
    "Grove",
    "Hanging Gardens",
    "Hanging Gardens (Spring)",
    "Jotunheim",
    "Lumora",
    "Monster Labs (Halloween)",
    "O2",
    "Orion",
    "Pirates",
    "Prisma",
    "Quantum",
    "Railway",
    "Rome",
    "Ruins",
    "Serrano",
    "Soul's End",
    "Spring (Spring)",
    "Summer (Summer)",
    "Summer Coast",
    "Temple",
    "Tesla Labs",
    "Tropical",
    "Winter (Winterfest)",
];

/// The map menu's order, top to bottom, as the player listed it (2026-11-06): most likely
/// played first. Names may leave out the event tag ("Spring" for "Spring (Spring)"). Random
/// comes before them, and maps not listed (Railway) after them in Hive's order.
pub const RANKING: &[&str] = &[
    "Temple",
    "Rome",
    "Ruins",
    "Spring",
    "Summer",
    "Autumn",
    "Winter",
    "Pirates",
    "Hanging Gardens",
    "Hanging Gardens (Spring)",
    "Tropical",
    "Cyberspace",
    "Prisma",
    "Soul's End",
    "Gingerbread",
    "Grove",
    "O2",
    "Cobalt",
    "Jotunheim",
    "Coven",
    "Summer Coast",
    "Serrano",
    "Lumora",
    "Tesla Labs",
    "Monster Labs",
    "Orion",
    "Quantum",
];

/// Hive's picture for each map, as its Map menu showed them on 2026-11-05; the menu uses the
/// live pictures once Hive has shown them this session.
pub(crate) const IMAGES: [(&str, &str); 28] = [
    (
        "Autumn",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.png",
    ),
    (
        "Cobalt",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Cobalt.png",
    ),
    (
        "Coven (Halloween)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Coven.halloween.png",
    ),
    (
        "Cyberspace",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Cyberspace.png",
    ),
    (
        "Gingerbread (Winterfest)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Gingerbread.winterfest.png",
    ),
    (
        "Grove",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Grove.png",
    ),
    (
        "Hanging Gardens",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_HangingGardens.png",
    ),
    (
        "Hanging Gardens (Spring)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_HangingGardens.spring.png",
    ),
    (
        "Jotunheim",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Jotunheim.png",
    ),
    (
        "Lumora",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Lumora.png",
    ),
    (
        "Monster Labs (Halloween)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_TeslaLabs.halloween.png",
    ),
    ("O2", "https://cdn.playhive.com//icons/map/BED_SQUAD_O2.png"),
    (
        "Orion",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Orion.png",
    ),
    (
        "Pirates",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Pirates.png",
    ),
    (
        "Prisma",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Prisma.png",
    ),
    (
        "Quantum",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Quantum.png",
    ),
    (
        "Railway",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Railway.png",
    ),
    (
        "Rome",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Rome.png",
    ),
    (
        "Ruins",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Ruins.png",
    ),
    (
        "Serrano",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Serrano.png",
    ),
    (
        "Soul's End",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_SoulsEnd.png",
    ),
    (
        "Spring (Spring)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.spring.png",
    ),
    (
        "Summer (Summer)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.summer.png",
    ),
    (
        "Summer Coast",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_SummerCoast.png",
    ),
    (
        "Temple",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Temple.png",
    ),
    (
        "Tesla Labs",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_TeslaLabs.png",
    ),
    (
        "Tropical",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Tropical.png",
    ),
    (
        "Winter (Winterfest)",
        "https://cdn.playhive.com//icons/map/BED_SQUAD_Autumn.winterfest.png",
    ),
];

/// The team facing RED on each map, by its name without the event tag ("Coven" for "Coven
/// (Halloween)"), and whether the player has confirmed it. All were confirmed by the player
/// in their own games on 2026-11-06. An unconfirmed pairing (`false`) only sorts and labels the
/// map menu: the party goes on RED and Hive places everyone else, so a guess cannot send
/// players to a team the map lacks. Maps not listed, and Random, are the same, without a menu
/// group.
pub(crate) const OPPONENTS: [(&str, &str, bool); 27] = [
    ("Autumn", "BLUE", true),
    ("Lumora", "BLUE", true),
    ("Pirates", "BLUE", true),
    ("Prisma", "BLUE", true),
    ("Soul's End", "BLUE", true),
    ("Spring", "BLUE", true),
    ("Summer", "BLUE", true),
    ("Temple", "BLUE", true),
    ("Winter", "BLUE", true),
    ("Cobalt", "BLUE", true),
    ("Orion", "BLUE", true),
    ("Quantum", "BLUE", true),
    ("Serrano", "BLUE", true),
    ("Cyberspace", "YELLOW", true),
    ("Hanging Gardens", "YELLOW", true),
    ("Rome", "YELLOW", true),
    ("Ruins", "YELLOW", true),
    ("O2", "YELLOW", true),
    ("Tropical", "YELLOW", true),
    ("Coven", "GREEN", true),
    ("Gingerbread", "GREEN", true),
    ("Grove", "GREEN", true),
    ("Summer Coast", "GREEN", true),
    ("Jotunheim", "GREEN", true),
    ("Monster Labs", "GREEN", true),
    ("Tesla Labs", "GREEN", true),
    ("Railway", "GREEN", true),
];

/// The team facing RED on `map` and whether that is confirmed, if it is listed.
pub fn pairing(map: &str) -> Option<(&'static str, bool)> {
    lookup(&OPPONENTS, map, |(name, _, _)| name).map(|(_, team, confirmed)| (*team, *confirmed))
}

/// The confirmed team facing RED on `map`: the team the mod moves everyone else to.
pub fn opponent(map: &str) -> Option<&'static str> {
    pairing(map)
        .filter(|(_, confirmed)| *confirmed)
        .map(|(team, _)| team)
}

/// A map's name without its event tag: "Spring" for "Spring (Spring)".
pub(crate) fn base(map: &str) -> &str {
    match map.find(" (") {
        Some(at) if map.ends_with(')') => &map[..at],
        _ => map,
    }
}

/// The entry of `list` naming `map`: its exact name first, else its name without the event tag.
pub(crate) fn lookup<'a, T>(list: &'a [T], map: &str, name: impl Fn(&T) -> &str) -> Option<&'a T> {
    position(list, map, name).map(|index| &list[index])
}

/// Where [`lookup`] finds `map` in `list`.
pub(crate) fn position<T>(list: &[T], map: &str, name: impl Fn(&T) -> &str) -> Option<usize> {
    let find = |wanted: &str| {
        list.iter()
            .position(|entry| slug(name(entry)) == slug(wanted))
    };
    find(map).or_else(|| find(base(map)))
}

/// How a team name is coloured in chat.
pub(crate) fn colour(team: &str) -> &'static str {
    match team {
        "RED" => "§c",
        "BLUE" => "§9",
        "YELLOW" => "§e",
        "GREEN" => "§a",
        _ => "§7",
    }
}
