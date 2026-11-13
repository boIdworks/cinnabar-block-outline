//! The auto queue's host calls: reading the chat and the player's life state, and sending the queue
//! command and the party question. What to do with them is in [`crate::autoq`], which is tested
//! natively.
//!
//! The host traps (and so quarantines the mod) a callback that makes more than eight player-state
//! reads, and a frame with every feature on already spends about that many. So every such read in
//! the mod calls [`note_read`], this runs after the rest of the frame, and it makes at most one
//! read a frame, only when the frame has one left.

use super::*;
use crate::autoq::{self, Line};

/// The player-state reads the host allows one callback.
const MAX_READS: u32 = 8;

/// Seconds between this feature's reads. Chat and the player's life state take turns, so each is
/// read every quarter second.
const READ_EVERY: f32 = 0.125;

/// Fuel a frame must have left for the feature to read at all.
const QUEUE_RESERVE: u64 = 25_000;

/// Seconds between reads of the sidebar, which is where Hive marks a custom game.
const SIDEBAR_EVERY: f32 = 1.0;

thread_local! {
    /// The player-state reads made so far this frame, and the session the frame is in (0: none).
    static USED: StdCell<(u32, u64)> = const { StdCell::new((0, 0)) };
    /// The queue Hive last named and any command waiting.
    static QUEUE: StdCell<autoq::Queue> = const { StdCell::new(autoq::Queue::new()) };
    /// How the chat is being read.
    static READS: StdCell<Reads> = const { StdCell::new(Reads::new()) };
}

/// Where the chat reading stands.
#[derive(Clone, Copy)]
struct Reads {
    /// The newest chat line read.
    after: u64,
    /// The retained chat (32 lines) is being skipped, so a game over from before does not queue:
    /// one read a frame until it is all read.
    skipping: bool,
    /// Whether the skip has started (none of this runs before the switch is on).
    started: bool,
    /// When the next read is due, and whether it is chat's turn (else the life state's).
    next: f32,
    chat_turn: bool,
    /// When the sidebar is next read (it says whether the game is a custom one).
    sidebar_next: f32,
}

impl Reads {
    const fn new() -> Self {
        Self {
            after: 0,
            skipping: false,
            started: false,
            next: 0.0,
            chat_turn: true,
            sidebar_next: 0.0,
        }
    }
}

/// Starts a frame's count of player-state reads.
pub(super) fn new_frame() {
    USED.with(|used| used.set((0, used.get().1)));
}

/// Counts one player-state read made this frame. Call it before every such read.
pub(super) fn note_read() {
    USED.with(|used| {
        let (reads, session) = used.get();
        used.set((reads.saturating_add(1), session));
    });
}

/// Notes the session the frame is in (0 when there is no world).
pub(super) fn note_session(session: u64) {
    USED.with(|used| used.set((used.get().0, session)));
}

/// One frame of the auto queue: while it is on and in a world, read the chat or the player's life
/// state, and send the party question or the queue command once due. Off, it costs one setting
/// read.
pub(super) fn queue_frame() {
    if !settings_with(|s| s.auto_queue) {
        if READS.with(|reads| reads.get().started) {
            QUEUE.with(|queue| queue.set(autoq::Queue::new()));
            READS.with(|reads| reads.set(Reads::new()));
        }
        return;
    }
    let (used, session) = USED.with(StdCell::get);
    if session == 0 {
        return;
    }
    let now = SECONDS.with(StdCell::get);
    let mut reads = READS.with(StdCell::get);
    let mut queue = QUEUE.with(StdCell::get);
    if used < MAX_READS && headroom() >= QUEUE_RESERVE {
        if !reads.started {
            reads.started = true;
            reads.skipping = true;
        }
        if reads.skipping || now >= reads.next {
            read_once(&mut reads, &mut queue, now);
        }
    }
    if queue.ask_ready(now) {
        ask_party(&mut queue, now);
    }
    if queue.ready(now) {
        send(&mut queue, now);
    }
    READS.with(|stored| stored.set(reads));
    QUEUE.with(|stored| stored.set(queue));
}

/// Says so on the HUD when a line or a death has just scheduled the queue command.
fn announce(was_waiting: bool, queue: &autoq::Queue, why: &str) {
    if !was_waiting && queue.waiting() {
        show_label(&format!("Auto queue: {why}, queueing in a second"));
    }
}

/// Says so on the HUD when the game finished but no mode is known, so nothing can be queued.
fn unknown_mode(queue: &autoq::Queue) {
    if queue.mode.is_none() {
        show_label("Auto queue: I don't know the mode yet. Queue once with /q");
    }
}

/// Says so on the HUD when an elimination has just started the party question.
fn announce_question(was_asking: bool, queue: &autoq::Queue) {
    if !was_asking && queue.asking() {
        show_label("Auto queue: eliminated, checking your party");
    }
}

/// Makes this frame's one read: chat's turn, or the life state's.
fn read_once(reads: &mut Reads, queue: &mut autoq::Queue, now: f32) {
    if reads.skipping || reads.chat_turn {
        read_chat(reads, queue, now);
    } else {
        if now >= reads.sidebar_next {
            reads.sidebar_next = now + SIDEBAR_EVERY;
            read_sidebar(queue);
        } else {
            read_life(queue, now);
        }
        reads.chat_turn = true;
        reads.next = now + READ_EVERY;
    }
}

/// Reads Hive's sidebar for the "Custom" row of a custom game, which is never auto queued. After
/// the game the sidebar is the game summary, which leaves what was known as it was.
fn read_sidebar(queue: &mut autoq::Queue) {
    note_read();
    let Ok(Some(sidebar)) = player_state::read_sidebar() else {
        return;
    };
    let labels = sidebar.rows.iter().map(|row| row.label.as_str());
    let Some(custom) = autoq::custom_game(&sidebar.title, labels) else {
        return;
    };
    if custom && !queue.is_custom() {
        show_label("Auto queue: custom game, off for this one");
    }
    queue.set_custom(custom);
}

/// Reads whether the player is alive, so dying (with a "FINAL KILL!" line) is known. While a queue
/// is already waiting there is nothing left to find out, so the host call is skipped.
fn read_life(queue: &mut autoq::Queue, now: f32) {
    if queue.waiting() {
        return;
    }
    note_read();
    let alive = match player_state::read_world() {
        Ok(Some(world)) => Some(world.alive),
        _ => None,
    };
    let was_asking = queue.asking();
    let died = queue.alive(alive, now);
    if died && !queue.asking() {
        show_label("Auto queue: you died");
    }
    announce_question(was_asking, queue);
}

/// Reads up to four chat lines since the last: the queue Hive names is remembered; a game over, the
/// party out, or an answer that there is no party queues; an elimination starts the party
/// question. While the old chat is being skipped nothing is acted on (the queue named last is still
/// learned).
fn read_chat(reads: &mut Reads, queue: &mut autoq::Queue, now: f32) {
    note_read();
    let batch = match player_state::read_chat(reads.after, 4) {
        Ok(batch) => batch,
        Err(error) => {
            report_once(&format!(
                "Boldy Mods: auto queue cannot read chat ({error})"
            ));
            reads.next = now + 1.0;
            return;
        }
    };
    let skipping = reads.skipping;
    for message in &batch {
        reads.after = reads.after.max(message.sequence);
        let Some(line) = autoq::read_line(&message.text) else {
            continue;
        };
        if let Line::Queueing(mode) = line {
            let changed = queue.queueing(mode);
            if changed && !skipping {
                show_label(&format!("Auto queue: {}", mode.as_str()));
            }
            continue;
        }
        match line {
            Line::QueueingUnknown => {
                queue.queueing_unknown();
                continue;
            }
            Line::QueueingNamed(game) => {
                if let Some(mode) = queue.queueing_named(game)
                    && !skipping
                {
                    show_label(&format!("Auto queue: {}", mode.as_str()));
                }
                continue;
            }
            Line::Joined(who, max) => {
                if let Some(mode) = queue.joined(who, max)
                    && !skipping
                {
                    show_label(&format!("Auto queue: {}", mode.as_str()));
                }
                continue;
            }
            _ => {}
        }
        if skipping {
            continue;
        }
        let was = queue.waiting();
        let was_asking = queue.asking();
        match line {
            Line::GameOver => {
                queue.finished(now);
                announce(was, queue, "game over");
                unknown_mode(queue);
                // A game over that queues nothing says why, so a quiet failure can be told apart.
                if !was && !queue.waiting() && queue.mode.is_some() {
                    show_label("Auto queue: game over seen, but a queue was just sent");
                }
            }
            Line::PartyOut => {
                queue.finished(now);
                announce(was, queue, "your party is out");
                unknown_mode(queue);
            }
            Line::NotInParty => {
                queue.not_in_party(now);
                announce(was, queue, "no party");
            }
            Line::YouEliminated => {
                queue.eliminated(now);
                announce_question(was_asking, queue);
                unknown_mode(queue);
            }
            Line::SelfOut(who) => {
                queue.self_out(who, now);
                announce_question(was_asking, queue);
            }
            Line::YouWereKilled => {
                queue.killed(now);
                announce_question(was_asking, queue);
            }
            Line::FinalKill => {
                queue.final_kill(now);
                announce_question(was_asking, queue);
            }
            Line::Queueing(_)
            | Line::QueueingUnknown
            | Line::QueueingNamed(_)
            | Line::Joined(..) => {}
        }
    }
    if batch.len() >= 4 {
        // More waiting: read again next frame.
        return;
    }
    if skipping {
        reads.skipping = false;
        // Says it is on and reading: the first sign that the switch and the permission work.
        show_label("Auto queue: on, watching chat");
    }
    reads.chat_turn = false;
    reads.next = now + READ_EVERY;
}

/// Asks Hive whether the player has a party (`/p list`); its answer comes back as a chat line.
fn ask_party(queue: &mut autoq::Queue, now: f32) {
    match gameplay::request_command(autoq::PARTY_COMMAND) {
        Ok(()) => queue.asked(now),
        Err(error) => {
            queue.ask_refused(now);
            report_refusal(&error);
        }
    }
}

/// Asks Cinnabar to send the queue command. A refusal is tried again shortly (a screen may own
/// input for a moment).
fn send(queue: &mut autoq::Queue, now: f32) {
    let Some(mode) = queue.mode else {
        return;
    };
    let command = autoq::command(mode);
    match gameplay::request_command(&command) {
        Ok(()) => {
            queue.sent(now);
            show_label(&format!("Auto queue: {command}"));
        }
        Err(error) => {
            queue.refused(now);
            report_refusal(&error);
        }
    }
}

/// Shows, once, a refusal that says the permission is missing; other refusals are just retried.
fn report_refusal(error: &str) {
    if error.contains("denied") || error.contains("not granted") {
        report_once(&format!(
            "Boldy Mods: allow the Commands permission for the auto queue ({error})"
        ));
    } else {
        // Retried shortly, but said once, so a command that is never taken is not silent.
        report_once(&format!("Boldy Mods: auto queue command refused ({error})"));
    }
}
