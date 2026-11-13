use super::*;

fn mode(name: &str) -> Mode {
    Mode::new(name).expect("a valid mode")
}

#[test]
fn hives_queue_line_names_the_mode() {
    assert_eq!(
        read_line("Finding you a game of BED-DUOS..."),
        Some(Line::Queueing(mode("BED-DUOS")))
    );
    assert_eq!(
        read_line("Finding you a game of BED..."),
        Some(Line::Queueing(mode("BED")))
    );
    // Colours and a symbol in front do not matter, nor does the case.
    assert_eq!(
        read_line("§b» §rFinding you a game of §eSW-SQUADS§r..."),
        Some(Line::Queueing(mode("SW-SQUADS")))
    );
    assert_eq!(
        read_line("finding you a game of bed-mega..."),
        Some(Line::Queueing(mode("BED-MEGA")))
    );
}

#[test]
fn the_mode_is_stored_in_capitals() {
    assert_eq!(mode("bed-duos").as_str(), "BED-DUOS");
    assert_eq!(command(mode("sky-squads")), "/q SKY-SQUADS");
}

#[test]
fn hives_end_of_game_line_is_found_through_its_symbol_and_colours() {
    assert_eq!(read_line("§c§l» §r§c§lGame OVER!"), Some(Line::GameOver));
    assert_eq!(read_line("» Game OVER!"), Some(Line::GameOver));
    assert_eq!(read_line("Game over!"), Some(Line::GameOver));
}

#[test]
fn the_party_and_final_kill_lines_are_read() {
    assert_eq!(
        read_line("§a» All members of your party are now spectating."),
        Some(Line::PartyOut)
    );
    assert_eq!(
        read_line("§c§l» §r§c§lFINAL KILL! §r§aBoldworks §7eliminated"),
        Some(Line::FinalKill)
    );
    assert_eq!(
        read_line("» FINAL KILL! qxuf2 killed Boldworks!"),
        Some(Line::FinalKill)
    );
    assert_eq!(
        read_line("§c§l» §r§7§lYou were eliminated"),
        Some(Line::YouEliminated)
    );
    assert_eq!(
        read_line("You have to be in a party to run this command."),
        Some(Line::NotInParty)
    );
    assert_eq!(
        read_line("You were killed by Ok dqrk. They had 9 hearts"),
        Some(Line::YouWereKilled)
    );
    assert_eq!(
        read_line("§c» §rYou were killed by §9Ok dqrk§r. They had §c9 hearts"),
        Some(Line::YouWereKilled)
    );
}

#[test]
fn other_chat_is_not_acted_on() {
    assert_eq!(read_line("§6§l» §r§dMagenta Team are the WINNERS!"), None);
    assert_eq!(read_line("» Game OVER! Really?"), None);
    assert_eq!(read_line("Finding you a game of ..."), None);
    assert_eq!(read_line("Finding you a game"), None);
    assert_eq!(read_line(""), None);
    // A long line is never one of them.
    let long = format!("{}Game OVER!", " ".repeat(100));
    assert_eq!(read_line(&long), None);
}

#[test]
fn a_mode_cannot_carry_anything_but_letters_digits_and_dashes() {
    assert!(Mode::new("").is_none());
    assert!(Mode::new("BED DUOS").is_none());
    assert!(Mode::new("BED;op").is_none());
    assert!(Mode::new("/BED").is_none());
    assert!(Mode::new(&"A".repeat(25)).is_none());
    assert!(Mode::new(&"A".repeat(24)).is_some());
}

#[test]
fn the_command_is_q_with_the_mode() {
    assert_eq!(command(mode("BED-SQUADS")), "/q BED-SQUADS");
}

#[test]
fn a_game_over_queues_after_the_delay_once() {
    let mut queue = known();
    assert!(queue.finished(100.0));
    assert!(queue.waiting());
    assert!(!queue.ready(100.5), "too early");
    assert!(!queue.finished(100.6), "still the same game");
    assert!(queue.ready(101.0));
    queue.sent(101.0);
    assert!(!queue.waiting());
    assert!(!queue.ready(102.0));
}

#[test]
fn a_game_over_just_after_a_queue_is_ignored_but_the_next_game_counts() {
    let mut queue = known();
    queue.finished(0.0);
    queue.sent(1.0);
    assert!(!queue.finished(5.0));
    assert!(!queue.finished(20.9));
    assert!(queue.finished(21.1));
}

#[test]
fn a_refused_command_is_retried_for_a_while_then_dropped() {
    let mut queue = known();
    queue.queueing(mode("BED"));
    queue.finished(0.0);
    assert!(queue.ready(1.0));
    queue.refused(1.0);
    assert!(!queue.ready(1.1), "waits between tries");
    assert!(queue.ready(1.3));
    queue.refused(1.3);
    assert!(!queue.ready(DELAY + RETRY_FOR + 1.0), "gave up");
    assert!(!queue.waiting());
    // Nothing was sent, so the mode is kept and the next game over is not held back.
    assert_eq!(queue.mode, Some(mode("BED")));
    assert!(queue.finished(40.0));
}

#[test]
fn queueing_says_whether_the_mode_changed() {
    let mut queue = Queue::default();
    assert!(queue.queueing(mode("BED")));
    assert!(!queue.queueing(mode("bed")));
    assert!(queue.queueing(mode("BED-DUOS")));
    assert_eq!(queue.mode, Some(mode("BED-DUOS")));
}

#[test]
fn the_mode_is_kept_after_its_queue_is_sent() {
    // Hive may put the player straight into a game with no "Finding you a game" line, so the next
    // game over queues the same mode again.
    let mut queue = Queue::default();
    queue.queueing(mode("SKY"));
    assert!(queue.finished(0.0));
    queue.sent(1.0);
    assert_eq!(queue.mode, Some(mode("SKY")));
    assert!(queue.finished(100.0));
    assert!(queue.ready(101.0));
    assert_eq!(command(queue.mode.unwrap()), "/q SKY");
    // A queue the player starts by hand names its own mode.
    queue.sent(101.0);
    queue.queueing(mode("BED-DUOS"));
    assert_eq!(queue.mode, Some(mode("BED-DUOS")));
}

#[test]
fn nothing_is_queued_when_the_mode_is_not_known() {
    // Hive has no /rq, so there is nothing to fall back on.
    let mut queue = Queue::default();
    assert!(!queue.finished(0.0));
    assert!(!queue.waiting());
    assert!(!queue.eliminated(0.0));
    assert!(!queue.asking());
}

/// A queue that knows its mode.
fn known() -> Queue {
    let mut queue = Queue::default();
    queue.queueing(mode("BED"));
    queue
}

/// A queue for `name` with the player alive.
fn playing(name: &str) -> Queue {
    let mut queue = Queue::default();
    queue.queueing(mode(name));
    queue.alive(Some(true), 0.0);
    queue
}

/// Runs the party question to its answer: asked at `at`, answered "no party" a moment later.
fn ask_and_hear_no_party(queue: &mut Queue, at: f32) -> bool {
    assert!(queue.ask_ready(at), "the question is due");
    queue.asked(at);
    assert!(!queue.ask_ready(at + 0.1), "asked once");
    queue.not_in_party(at + 0.4)
}

#[test]
fn eliminated_with_no_party_queues_in_any_mode() {
    for name in ["SKY", "SKY-DUOS", "BED-SQUADS", "BED"] {
        let mut queue = playing(name);
        assert!(queue.alive(Some(false), 10.0), "a death");
        assert!(!queue.asking(), "not final yet");
        assert!(queue.final_kill(10.2));
        assert!(queue.asking(), "asks about the party first");
        assert!(!queue.waiting(), "does not queue before the answer");
        assert!(ask_and_hear_no_party(&mut queue, 10.3), "{name}");
        assert!(queue.waiting());
    }
}

#[test]
fn the_final_kill_may_come_before_the_death() {
    let mut queue = playing("SKY");
    assert!(!queue.final_kill(10.0), "someone else's, so far");
    assert!(queue.alive(Some(false), 10.5));
    assert!(queue.asking());
}

#[test]
fn hives_own_you_were_eliminated_starts_the_question_without_a_death_reading() {
    let mut queue = known();
    assert!(queue.eliminated(5.0));
    assert!(ask_and_hear_no_party(&mut queue, 5.0));
}

#[test]
fn a_party_keeps_waiting_for_its_own_line() {
    let mut queue = playing("SKY-DUOS");
    queue.alive(Some(false), 10.0);
    queue.final_kill(10.1);
    assert!(queue.ask_ready(10.1));
    queue.asked(10.1);
    // Hive answered with the party's list, or said nothing we know: nothing is queued.
    assert!(!queue.not_in_party(20.0), "too late to be the answer");
    assert!(!queue.waiting());
    // The party line comes when everyone is out.
    assert!(queue.finished(40.0));
}

#[test]
fn a_no_party_line_with_no_question_asked_is_not_acted_on() {
    // The player typed /p list themselves, or it is old.
    let mut queue = playing("SKY");
    assert!(!queue.not_in_party(10.0));
    assert!(!queue.waiting());
}

#[test]
fn a_refused_question_is_retried_then_dropped() {
    let mut queue = known();
    queue.eliminated(0.0);
    assert!(queue.ask_ready(0.0));
    queue.ask_refused(0.0);
    assert!(!queue.ask_ready(0.1), "waits between tries");
    assert!(queue.ask_ready(0.3));
    queue.ask_refused(0.3);
    assert!(!queue.ask_ready(ASK_FOR + 1.0), "gave up");
    assert!(!queue.asking());
}

#[test]
fn a_death_with_no_final_kill_does_nothing() {
    // Bedwars with the bed still standing: you respawn.
    let mut queue = playing("BED");
    assert!(queue.alive(Some(false), 10.0));
    queue.alive(Some(true), 15.0);
    assert!(!queue.asking() && !queue.waiting());
    // A final kill of somebody else well before or after is not this death.
    let mut queue = playing("SKY");
    queue.final_kill(1.0);
    queue.alive(Some(false), 10.0);
    assert!(!queue.asking());
    queue.final_kill(20.0);
    assert!(!queue.asking());
}

#[test]
fn only_alive_to_dead_is_a_death() {
    let mut queue = playing("SKY");
    assert!(!queue.alive(None, 1.0), "unknown is not a death");
    assert!(!queue.alive(Some(false), 2.0), "alive was lost sight of");
    let mut queue = Queue::default();
    assert!(!queue.alive(Some(false), 1.0), "never seen alive");
    assert!(!queue.alive(Some(false), 2.0), "still dead");
    assert!(!queue.alive(Some(true), 3.0));
    assert!(queue.alive(Some(false), 4.0));
}

#[test]
fn nothing_starts_while_a_queue_is_waiting_or_just_sent() {
    let mut queue = playing("SKY");
    queue.finished(10.0);
    assert!(!queue.eliminated(10.5), "a queue is already waiting");
    queue.sent(11.0);
    assert!(!queue.eliminated(12.0), "just queued");
    assert!(queue.eliminated(11.0 + QUIET + 0.1), "the next game");
}

#[test]
fn only_games_with_no_respawns_are_final() {
    for name in ["SKY", "SKY-DUOS", "SW", "SW-SQUADS", "SG", "SG-DUOS"] {
        assert!(mode(name).no_respawns(), "{name}");
    }
    for name in [
        "BED", "BED-DUOS", "BW", "SWITCH", "SGX", "DROP", "MURDER", "BUILD",
    ] {
        assert!(!mode(name).no_respawns(), "{name}");
    }
}

#[test]
fn being_killed_ends_the_game_only_where_there_are_no_respawns() {
    // Skywars: you are out.
    let mut queue = playing("SKY-DUOS");
    assert!(queue.killed(10.0));
    assert!(queue.asking());
    assert!(ask_and_hear_no_party(&mut queue, 10.0));
    // Bedwars: the bed may still stand, so being killed is not the end.
    let mut queue = playing("BED");
    assert!(!queue.killed(10.0));
    assert!(!queue.asking());
    // A game that was not seen being queued is not guessed at.
    let mut queue = Queue::default();
    assert!(!queue.killed(10.0));
    assert!(!queue.asking());
}
