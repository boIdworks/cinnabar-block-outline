//! Auto queue: when a Hive game is finished for the player, queue the next game of the same mode.
//!
//! The facts come from chat ([`read_line`]). Hive says `Finding you a game of BED-DUOS...` when
//! you queue, which names the queue exactly as `/q` takes it; the mod keeps it, and after it
//! queues a game itself the same mode stays the current one. With no mode known nothing is
//! queued (Hive has no `/rq`). The game is finished for the player
//! when Hive says `» Game OVER!`, or `All members of your party are now spectating.`, or when the
//! player is eliminated and is **not in a party**: then nobody is left to wait for. Being
//! eliminated is Hive's own `You were eliminated`; or, in a game with no respawns (Skywars), `You
//! were killed by ...`; or the player's `alive` flag turning off with a `FINAL KILL!` line right
//! with it (so dying with a bed still standing does not count, and no player name is needed). Whether the player has a party is asked with `/p list`: Hive's
//! reply `You have to be in a party to run this command.` means no party; anything else, or no
//! reply, is taken as a party, and the mod waits for the party line instead.
//!
//! [`Queue`] is the little state machine between those facts and the commands: it remembers the
//! mode, waits a moment, and keeps asking until the host takes a command. Nothing here calls the
//! host, so it is all tested natively.

/// Seconds from the game being finished to the queue command, so the end-of-game messages land
/// first.
pub const DELAY: f32 = 1.0;

/// How long after that a command the host refuses (a screen owns input, say) is still retried.
pub const RETRY_FOR: f32 = 6.0;

/// Seconds between those retries.
const RETRY_EVERY: f32 = 0.25;

/// A second finish this soon after a queue belongs to the same game.
pub const QUIET: f32 = 20.0;

/// How far apart (seconds) the player's `alive` flag turning off and a `FINAL KILL!` line may be
/// and still be the same death. Both come at the moment of death, read a quarter second apart.
const DEATH_WINDOW: f32 = 1.0;

/// How long the party question is retried if the host refuses it, and how long Hive's answer is
/// waited for once it is asked.
pub const ASK_FOR: f32 = 3.0;
pub const ANSWER_WAIT: f32 = 3.0;

/// The party question.
pub const PARTY_COMMAND: &str = "/p list";

/// The longest chat line looked at: the lines this reads are short, and chat is read often, so a
/// longer one is skipped without any work.
const LONGEST_LINE: usize = 96;

/// The longest queue name (`BUILD-DUOSX` is the longest Hive has; this leaves room).
const LONGEST_MODE: usize = 24;

/// What Hive says when a queue starts, before the queue's name.
const FINDING: &str = "finding you a game of ";

/// A queue name as `/q` takes it, such as `BED-DUOS`: capital letters, digits and dashes. Small and
/// `Copy`, so it can live in a `thread_local` cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode {
    bytes: [u8; LONGEST_MODE],
    len: u8,
}

impl Mode {
    /// The mode named `name`, in capitals; none when it is empty, too long or has anything but
    /// letters, digits and dashes (so it can never smuggle another command in).
    pub fn new(name: &str) -> Option<Self> {
        let name = name.as_bytes();
        if name.is_empty()
            || name.len() > LONGEST_MODE
            || !name.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            return None;
        }
        let mut bytes = [0; LONGEST_MODE];
        for (slot, byte) in bytes.iter_mut().zip(name) {
            *slot = byte.to_ascii_uppercase();
        }
        Some(Self {
            bytes,
            len: name.len() as u8,
        })
    }

    /// Whether being out in this game is final: Skywars (`SKY`, `SW`), Survival Games (`SG`) and
    /// Block Party (`PARTY`, `BP`) and Block Drop (`DROP`), with any size. Bedwars and other games that respawn are not,
    /// and an unknown game is not.
    pub fn no_respawns(&self) -> bool {
        let name = self.as_str();
        ["SKY", "SW", "SG", "PARTY", "BP", "DROP"]
            .iter()
            .any(|game| {
                name.strip_prefix(game)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
            })
    }

    pub fn as_str(&self) -> &str {
        // Only ASCII letters, digits and dashes are ever stored.
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }
}

/// What a chat line means to the auto queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    /// "Finding you a game of BED-DUOS...": the queue the player is in now.
    Queueing(Mode),
    /// "Finding you a game of The Bridge...": a queue named for people, not by an ID `/q` takes.
    QueueingUnknown,
    /// "Finding you a game of The Bridge...": a game named for people that is in [`named`].
    QueueingNamed(Named),
    /// "Boldworks joined. [1/2]": this player joined a lobby of this many players. After a queue
    /// starts, the first of these is the local player's own.
    Joined(Name, u32),
    /// "Boldworks did an oopsie!" (SkyWars, the void), "Boldworks made their last dance move" or
    /// "Boldworks fell off" (Block Party), "Boldworks fell off the map!" (Block Drop): this player is out by their own doing. Hive prints no
    /// kill line for it, so this is the only chat sign of it.
    SelfOut(Name),
    /// "Game OVER!".
    GameOver,
    /// "All members of your party are now spectating.": the whole party is out.
    PartyOut,
    /// "You were eliminated", or "You finished in 5th place!": the player is out for good.
    YouEliminated,
    /// "You were killed by X. They had 9 hearts": the player died (for good only where there are
    /// no respawns).
    YouWereKilled,
    /// A "FINAL KILL!" line: somebody was eliminated for good.
    FinalKill,
    /// "You have to be in a party to run this command.": Hive's answer to `/p list` with no party.
    NotInParty,
}

/// Removes `§` formatting codes (the `§` and the character after it) and lowercases.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// What a chat line says, if it is one of the lines the auto queue acts on, whatever symbol and
/// colours surround it.
pub fn read_line(text: &str) -> Option<Line> {
    if text.len() > LONGEST_LINE {
        return None;
    }
    let lower = plain(text);
    let bare = lower.trim_start_matches(|c: char| !c.is_alphanumeric());
    if bare.trim() == "game over!" {
        return Some(Line::GameOver);
    }
    if lower.contains("all members of your party are now spectating") {
        return Some(Line::PartyOut);
    }
    if lower.contains("you have to be in a party") {
        return Some(Line::NotInParty);
    }
    // Hive's real wording is "You have been eliminated from the game! [No bed]" (seen in the
    // autocs recording); "You were eliminated" is kept in case another game says it.
    if bare.starts_with("you have been eliminated") || bare.starts_with("you were eliminated") {
        return Some(Line::YouEliminated);
    }
    // A race is over for the player at "You finished in 5th place! [01:41.782]" (Deathrun) or
    // "You finished all maps and came in 5th place! [01:08.4]" (Gravity), a minute or more before
    // the game is over for everyone: the same as being out. A map done on the way ("You finished
    // Dimensions! [00:10.565]") is not it.
    if bare.starts_with("you finished in ") || bare.starts_with("you finished all maps") {
        return Some(Line::YouEliminated);
    }
    if bare.starts_with("you were killed by") {
        return Some(Line::YouWereKilled);
    }
    if lower.contains("final kill") {
        return Some(Line::FinalKill);
    }
    if let Some((who, max)) = joined(bare) {
        return Some(Line::Joined(who, max));
    }
    if let Some(who) = SELF_OUT
        .iter()
        .find_map(|verb| bare.find(verb))
        .and_then(|at| Name::new(&bare[..at]))
    {
        return Some(Line::SelfOut(who));
    }
    let at = lower.find(FINDING)? + FINDING.len();
    // Up to the "..." Hive ends the line with. A queue ID (`BED-DUOS`) is one word; a game
    // queued from the menu is named for people instead ("The Bridge"), which is not something
    // `/q` takes and does not say solos or duos either.
    let name = lower[at..].split('.').next().unwrap_or("").trim();
    if name.is_empty() {
        return None;
    }
    if let Some(game) = named(name) {
        return Some(Line::QueueingNamed(game));
    }
    // The name is read from the lowercased copy; `Mode` makes it capitals again.
    Some(Mode::new(name).map_or(Line::QueueingUnknown, Line::Queueing))
}

/// A game Hive names for people ("The Bridge") instead of by its queue code, and how to find the
/// code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Named {
    /// The game has one code.
    Code(&'static str),
    /// The code depends on the lobby size, read from the `[1/2]` of the next join line: pairs of
    /// (players in the lobby, code).
    BySize(&'static [(u32, &'static str)]),
}

/// The game a lowercase menu name stands for, from Hive's command list
/// (support.playhive.com/commands). Only Hive's own wording of the name is matched, and a name
/// that is not here is "unknown": the mod queues nothing until a `/q` by hand names the mode.
/// Bedwars, Skywars and the other games Hive names by code need no entry. The Hive menu wording is
/// confirmed for The Bridge only; the other names are Hive's game names.
fn named(name: &str) -> Option<Named> {
    Some(match name {
        "the bridge" => Named::BySize(&[(2, "BRIDGE"), (4, "BRIDGE-DUOS")]),
        // Recorded in the game: the menu's solo SkyWars and Build Battle both have 12 players. The
        // bigger lobbies (duos, squads, mega) are not recorded, so they are left out: with no
        // mode known nothing is queued, which beats queueing the wrong one.
        "skywars" => Named::BySize(&[(12, "SKY")]),
        "build battle" => Named::BySize(&[(12, "BUILD")]),
        // Recorded: the menu's Survival Games lobbies have 24 players.
        "survival games" => Named::BySize(&[(24, "SG")]),
        "block drop" => Named::Code("DROP"),
        "block party" => Named::Code("PARTY"),
        "capture the flag" => Named::Code("CTF"),
        "deathrun" => Named::Code("DR"),
        "ghost invasion" => Named::Code("GI"),
        "gravity" => Named::Code("GRAV"),
        "ground wars" => Named::Code("GROUND"),
        "hide and seek" => Named::Code("HIDE"),
        "mobgame" => Named::Code("MOB"),
        "murder mystery" => Named::Code("MURDER"),
        _ => return None,
    })
}

/// What Hive says after a player's name when they are out by their own doing. Other players' lines
/// end with `[+2XP]`; the local player's does not, which does not matter here.
const SELF_OUT: [&str; 3] = [" did an oopsie", " made their last dance move", " fell off"];

/// Who joined, and the lobby size, in Hive's "Name joined. [1/2]" line: the number after the slash.
fn joined(bare: &str) -> Option<(Name, u32)> {
    let at = bare.find(" joined.")?;
    let rest = &bare[at + " joined.".len()..];
    let count = &rest[rest.find('[')? + 1..];
    let (_, max) = count[..count.find(']')?].split_once('/')?;
    Some((Name::new(&bare[..at])?, max.trim().parse().ok()?))
}

/// The longest player name kept (Hive names are shorter).
const LONGEST_NAME: usize = 24;

/// A player's name, lowercased: letters, digits, spaces, `_` and `-`. A line with a rank, a symbol
/// or a chat message in front of a name never makes one, so what a player types cannot pass for
/// Hive's own lines. Small and `Copy`, like [`Mode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Name {
    bytes: [u8; LONGEST_NAME],
    len: u8,
}

impl Name {
    /// The name `text` (already lowercase) is, if it is one.
    pub fn new(text: &str) -> Option<Self> {
        let text = text.as_bytes();
        if text.is_empty()
            || text.len() > LONGEST_NAME
            || !text
                .iter()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b' ' | b'_' | b'-'))
            || text.first() == Some(&b' ')
            || text.last() == Some(&b' ')
        {
            return None;
        }
        let mut bytes = [0; LONGEST_NAME];
        for (slot, byte) in bytes.iter_mut().zip(text) {
            *slot = byte.to_ascii_lowercase();
        }
        Some(Self {
            bytes,
            len: text.len() as u8,
        })
    }
}

/// The command that queues `mode`. (Hive has no `/rq`: it answers "Unknown command", so a game
/// whose mode is not known is not queued at all.)
pub fn command(mode: Mode) -> String {
    format!("/q {}", mode.as_str())
}

/// Whether a sidebar row label is Hive's "Custom" tag (colour codes ignored).
fn custom_label(label: &str) -> bool {
    let mut plain = [0u8; 8];
    let mut len = 0;
    let mut chars = label.chars();
    while let Some(c) = chars.next() {
        if c == '\u{a7}' {
            chars.next();
            continue;
        }
        if !c.is_ascii() || len == plain.len() {
            return false;
        }
        plain[len] = c as u8;
        len += 1;
    }
    plain[..len].trim_ascii().eq_ignore_ascii_case(b"custom")
}

/// Whether the sidebar says the game is a Hive custom game: a "Custom" row in it. `None` when the
/// sidebar is the game summary after a game, which says nothing about the kind of game (what was
/// known stays).
pub fn custom_game<'a>(title: &str, mut labels: impl Iterator<Item = &'a str>) -> Option<bool> {
    if title.to_ascii_lowercase().contains("game summary") {
        return None;
    }
    Some(labels.any(custom_label))
}

/// What the auto queue knows and is waiting to do. Small and `Copy`, so a frame reads it cheaply.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Queue {
    /// The queue Hive last said it was finding a game in.
    pub mode: Option<Mode>,
    /// A queue command to send: from when, and until when to keep trying.
    due: Option<(f32, f32)>,
    /// No new queue is made for a finished game before this time.
    quiet_until: f32,
    /// The party question to send: from when, and until when to keep trying.
    ask: Option<(f32, f32)>,
    /// Until when Hive's answer to the party question is waited for.
    answer_until: Option<f32>,
    /// When the player's `alive` flag last turned off, and when a `FINAL KILL!` line last came.
    died_at: Option<f32>,
    kill_at: Option<f32>,
    /// Whether the player was alive at the last reading (none: not known).
    was_alive: Option<bool>,
    /// The local player's name: the one in the first join line after a queue starts.
    me: Option<Name>,
    /// Whether the next join line is the local player's own.
    learn_me: bool,
    /// A game whose code depends on the lobby size was just queued, and the size has not been seen
    /// yet: (players, code) pairs.
    sizes: Option<&'static [(u32, &'static str)]>,
    /// The game is a Hive custom game (the sidebar says "Custom"): nothing is queued for it.
    custom: bool,
}

impl Queue {
    /// Knows nothing and waits for nothing (a `const` form of `default`).
    pub const fn new() -> Self {
        Self {
            mode: None,
            due: None,
            quiet_until: 0.0,
            ask: None,
            answer_until: None,
            died_at: None,
            kill_at: None,
            was_alive: None,
            me: None,
            learn_me: false,
            sizes: None,
            custom: false,
        }
    }

    /// Whether the current game is a custom game, which the auto queue leaves alone.
    pub fn is_custom(&self) -> bool {
        self.custom
    }

    /// The sidebar says whether the game is a custom one (see [`custom_game`]). A custom game
    /// drops anything already scheduled.
    pub fn set_custom(&mut self, custom: bool) {
        self.custom = custom;
        if custom {
            self.due = None;
            self.ask = None;
            self.answer_until = None;
            self.died_at = None;
            self.kill_at = None;
        }
    }

    /// Hive said the player is queueing for `mode`. True when that is a different one.
    pub fn queueing(&mut self, mode: Mode) -> bool {
        let changed = self.mode != Some(mode);
        self.mode = Some(mode);
        self.sizes = None;
        self.learn_me = true;
        changed
    }

    /// Hive said the player is queueing for a game whose queue ID is not known, so the last mode
    /// is no longer the current one: queueing it would put the player in the wrong game. True when
    /// a mode was forgotten.
    pub fn queueing_unknown(&mut self) -> bool {
        self.sizes = None;
        self.learn_me = true;
        self.mode.take().is_some()
    }

    /// Hive said the player is queueing for a game it names for people ([`named`]). The old mode
    /// is forgotten. A game with one code is that mode at once (returned); one whose code depends
    /// on the lobby size waits for it ([`Queue::joined`]).
    pub fn queueing_named(&mut self, game: Named) -> Option<Mode> {
        self.queueing_unknown();
        match game {
            Named::Code(code) => {
                let mode = Mode::new(code)?;
                self.mode = Some(mode);
                Some(mode)
            }
            Named::BySize(sizes) => {
                self.sizes = Some(sizes);
                None
            }
        }
    }

    /// `who` joined a lobby of `max` players. After a queue starts the first to join is the local
    /// player, whose name is kept. Right after a game whose code depends on the lobby
    /// size was queued, that size picks the code (The Bridge: 2 is `BRIDGE`, 4 is `BRIDGE-DUOS`);
    /// any other size ends the wait with no mode. The mode is returned when it was just learned.
    pub fn joined(&mut self, who: Name, max: u32) -> Option<Mode> {
        if self.learn_me {
            self.me = Some(who);
            self.learn_me = false;
        }
        let sizes = self.sizes.take()?;
        let (_, code) = sizes.iter().find(|(players, _)| *players == max)?;
        let mode = Mode::new(code)?;
        self.mode = Some(mode);
        Some(mode)
    }

    /// `who` is out by their own doing at `now` ("did an oopsie!", "made their last dance move"). In a game with no respawns (and
    /// a known one) that is the end of the local player's game, when `who` is them: find out
    /// whether a party is waiting. True when the party question was started.
    pub fn self_out(&mut self, who: Name, now: f32) -> bool {
        self.me == Some(who)
            && self.mode.is_some_and(|mode| mode.no_respawns())
            && self.eliminated(now)
    }

    /// The player's game is finished at `now` (Game OVER, the party out, or eliminated with no
    /// party): queue the next one soon, unless one was just queued. True when it was scheduled.
    pub fn finished(&mut self, now: f32) -> bool {
        if self.custom || self.mode.is_none() || self.due.is_some() || now < self.quiet_until {
            return false;
        }
        self.ask = None;
        self.answer_until = None;
        self.due = Some((now + DELAY, now + DELAY + RETRY_FOR));
        true
    }

    /// The player is out for good at `now` (Hive said so): find out whether a party is waiting.
    /// True when the question was started.
    pub fn eliminated(&mut self, now: f32) -> bool {
        if self.custom
            || self.mode.is_none()
            || self.due.is_some()
            || self.ask.is_some()
            || self.answer_until.is_some_and(|until| now < until)
            || now < self.quiet_until
        {
            return false;
        }
        self.died_at = None;
        self.kill_at = None;
        self.ask = Some((now, now + ASK_FOR));
        true
    }

    /// Hive said the player was killed at `now`. That is the end of their game only in a game with
    /// no respawns (and a known one). True when the party question was started.
    pub fn killed(&mut self, now: f32) -> bool {
        self.mode.is_some_and(|mode| mode.no_respawns()) && self.eliminated(now)
    }

    /// A "FINAL KILL!" line came at `now`. True when that, with the player's death, started the
    /// party question.
    pub fn final_kill(&mut self, now: f32) -> bool {
        self.kill_at = Some(now);
        self.death_was_final(now)
    }

    /// The player's life state at `now` (none: not known, such as while loading). Only a change
    /// from alive to dead is a death. True when the player has just died.
    pub fn alive(&mut self, alive: Option<bool>, now: f32) -> bool {
        let died = self.was_alive == Some(true) && alive == Some(false);
        self.was_alive = alive;
        if died {
            self.died_at = Some(now);
            self.death_was_final(now);
        }
        died
    }

    /// A death and a final kill close together are the player's elimination.
    fn death_was_final(&mut self, now: f32) -> bool {
        let (Some(died), Some(kill)) = (self.died_at, self.kill_at) else {
            return false;
        };
        (died - kill).abs() <= DEATH_WINDOW && self.eliminated(now)
    }

    /// Whether the party question should be sent now. Gives up once the time to keep trying has
    /// passed.
    pub fn ask_ready(&mut self, now: f32) -> bool {
        match self.ask {
            Some((_, until)) if now >= until => {
                self.ask = None;
                false
            }
            Some((at, _)) => now >= at,
            None => false,
        }
    }

    /// The host took the party question: its answer is waited for.
    pub fn asked(&mut self, now: f32) {
        self.ask = None;
        self.answer_until = Some(now + ANSWER_WAIT);
    }

    /// The host refused the party question: ask again a moment later.
    pub fn ask_refused(&mut self, now: f32) {
        if let Some((_, until)) = self.ask {
            self.ask = Some((now + RETRY_EVERY, until));
        }
    }

    /// Hive said there is no party. If that answers a question just asked, nobody is left to wait
    /// for: queue. True when it was scheduled.
    pub fn not_in_party(&mut self, now: f32) -> bool {
        if self.answer_until.is_some_and(|until| now <= until) {
            self.answer_until = None;
            return self.finished(now);
        }
        false
    }

    /// Whether the party question is being asked or answered.
    pub fn asking(&self) -> bool {
        self.ask.is_some() || self.answer_until.is_some()
    }

    /// Whether the command should be asked for now. Gives up (and forgets it) once the time to keep
    /// trying has passed.
    pub fn ready(&mut self, now: f32) -> bool {
        match self.due {
            Some((_, until)) if now >= until => {
                self.due = None;
                false
            }
            Some((at, _)) => now >= at,
            None => false,
        }
    }

    /// The host took the queue command. The mode is kept: the game Hive puts the player in next is
    /// the one just asked for, and Hive does not always say so (a game found at once gets no
    /// "Finding you a game" line). A queue the player starts by hand names its own mode.
    pub fn sent(&mut self, now: f32) {
        self.due = None;
        self.learn_me = true;
        self.quiet_until = now + QUIET;
    }

    /// The host refused it: ask again a moment later.
    pub fn refused(&mut self, now: f32) {
        if let Some((_, until)) = self.due {
            self.due = Some((now + RETRY_EVERY, until));
        }
    }

    /// Whether a queue command is waiting to go out.
    pub fn waiting(&self) -> bool {
        self.due.is_some()
    }
}

#[cfg(test)]
mod tests;
