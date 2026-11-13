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

    /// Whether dying in this game is final: Skywars (`SKY`, `SW`) and Survival Games (`SG`), with
    /// any size. Bedwars and other games that respawn are not, and an unknown game is not.
    pub fn no_respawns(&self) -> bool {
        let name = self.as_str();
        ["SKY", "SW", "SG"].iter().any(|game| {
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
    /// "Game OVER!".
    GameOver,
    /// "All members of your party are now spectating.": the whole party is out.
    PartyOut,
    /// "You were eliminated": the player is out for good.
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
    if bare.starts_with("you were eliminated") {
        return Some(Line::YouEliminated);
    }
    if bare.starts_with("you were killed by") {
        return Some(Line::YouWereKilled);
    }
    if lower.contains("final kill") {
        return Some(Line::FinalKill);
    }
    let at = lower.find(FINDING)? + FINDING.len();
    // The name is read from the lowercased copy; `Mode` makes it capitals again.
    let name: String = lower[at..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    Mode::new(&name).map(Line::Queueing)
}

/// The command that queues `mode`. (Hive has no `/rq`: it answers "Unknown command", so a game
/// whose mode is not known is not queued at all.)
pub fn command(mode: Mode) -> String {
    format!("/q {}", mode.as_str())
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
        }
    }

    /// Hive said the player is queueing for `mode`. True when that is a different one.
    pub fn queueing(&mut self, mode: Mode) -> bool {
        let changed = self.mode != Some(mode);
        self.mode = Some(mode);
        changed
    }

    /// The player's game is finished at `now` (Game OVER, the party out, or eliminated with no
    /// party): queue the next one soon, unless one was just queued. True when it was scheduled.
    pub fn finished(&mut self, now: f32) -> bool {
        if self.mode.is_none() || self.due.is_some() || now < self.quiet_until {
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
        if self.mode.is_none()
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
