use super::*;

fn mode(name: &str) -> Mode {
    Mode::new(name).expect("a valid mode")
}

fn who(name: &str) -> Name {
    Name::new(name).expect("a valid name")
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
fn a_game_named_for_people_is_not_a_queue_id() {
    // Recorded from Hive when The Bridge is queued from the menu: it must not become `/q THE`.
    assert_eq!(
        read_line("§b§l» §r§7§7Finding you a game of The Bridge..."),
        Some(Line::QueueingNamed(Named::BySize(&[
            (2, "BRIDGE"),
            (4, "BRIDGE-DUOS")
        ])))
    );
    // Recorded from Hive's menus: SkyWars is one word but still not a code (`/q SKYWARS` is not
    // one), and Build Battle is two.
    assert_eq!(
        read_line("§b§l» §r§7§7Finding you a game of SkyWars..."),
        Some(Line::QueueingNamed(Named::BySize(&[(12, "SKY")])))
    );
    assert_eq!(
        read_line("§b§l» §r§7§7Finding you a game of Build Battle..."),
        Some(Line::QueueingNamed(Named::BySize(&[(12, "BUILD")])))
    );
    assert_eq!(
        read_line("§b§l» §r§7§7Finding you a game of Survival Games..."),
        Some(Line::QueueingNamed(Named::BySize(&[(24, "SG")])))
    );
    assert_eq!(
        read_line("Finding you a game of Capture The Flag..."),
        Some(Line::QueueingNamed(Named::Code("CTF")))
    );
    assert_eq!(
        read_line("Finding you a game of Ground Wars..."),
        Some(Line::QueueingNamed(Named::Code("GROUND")))
    );
    assert_eq!(
        read_line("Finding you a game of Murder Mystery..."),
        Some(Line::QueueingNamed(Named::Code("MURDER")))
    );
    assert_eq!(
        read_line("Finding you a game of Some New Game..."),
        Some(Line::QueueingUnknown)
    );
}

#[test]
fn the_lobby_size_is_read_from_the_joined_line() {
    // Recorded from Hive.
    assert_eq!(
        read_line("§a§l» §r§7Boldworks joined. §8[1/2]"),
        Some(Line::Joined(who("boldworks"), 2))
    );
    assert_eq!(
        read_line("§a§l» §r§7Chomik4696 joined. §8[8/8]"),
        Some(Line::Joined(who("chomik4696"), 8))
    );
    assert_eq!(read_line("Boldworks joined."), None);
}

#[test]
fn the_bridge_lobby_size_tells_solos_from_duos() {
    let bridge = Named::BySize(&[(2, "BRIDGE"), (4, "BRIDGE-DUOS")]);
    let me = who("boldworks");
    let mut queue = Queue::default();
    queue.queueing(mode("BED-DUOS"));
    // A lobby joined with no such game queued teaches nothing.
    assert_eq!(queue.joined(me, 2), None);
    assert_eq!(queue.mode, Some(mode("BED-DUOS")));

    assert_eq!(
        queue.queueing_named(bridge),
        None,
        "the size is not seen yet"
    );
    assert_eq!(queue.mode, None, "Bedwars is forgotten");
    assert_eq!(queue.joined(me, 2), Some(mode("BRIDGE")));
    assert_eq!(queue.joined(me, 4), None, "the size is read once");

    queue.queueing_named(bridge);
    assert_eq!(queue.joined(me, 4), Some(mode("BRIDGE-DUOS")));

    // Any other size is not a Bridge lobby, and ends the wait.
    queue.queueing_named(bridge);
    assert_eq!(queue.joined(me, 8), None);
    assert_eq!(queue.mode, None);
    assert_eq!(queue.joined(me, 2), None);
}

#[test]
fn a_game_with_one_code_is_that_mode_at_once() {
    let mut queue = Queue::default();
    queue.queueing(mode("BED-DUOS"));
    assert_eq!(queue.queueing_named(Named::Code("CTF")), Some(mode("CTF")));
    assert_eq!(command(queue.mode.unwrap()), "/q CTF");
}

#[test]
fn every_code_in_hives_command_list_is_queued() {
    // Hive's list (support.playhive.com/commands), aliases included.
    let codes = [
        "BED",
        "BW",
        "BED-DUOS",
        "BW-DUOS",
        "BED-SQUADS",
        "BW-SQUADS",
        "BED-MEGA",
        "BW-MEGA",
        "BED-DUELS",
        "BW-DUELS",
        "BED-DUELS-DUOS",
        "BW-DUELS-DUOS",
        "HUB-BED",
        "DROP",
        "PARTY",
        "BP",
        "BRIDGE",
        "BRIDGE-DUOS",
        "BUILD",
        "BB",
        "BUILD-DUOS",
        "BB-DUOS",
        "BUILDX",
        "BUILD-DUOSX",
        "BB-DUOSX",
        "BUILD-SPEED",
        "BB-SPEED",
        "CTF",
        "DR",
        "GI",
        "GRAV",
        "GROUND",
        "GW",
        "HIDE",
        "HNS",
        "MOB",
        "MURDER",
        "MM",
        "HUB-PARKOUR",
        "REPLAY",
        "SKY",
        "SW",
        "SKY-DUOS",
        "SW-DUOS",
        "SKY-SQUADS",
        "SW-SQUADS",
        "SKY-MEGA",
        "SW-MEGA",
        "SG",
        "SG-DUOS",
    ];
    for code in codes {
        // Said as the queue starts, so the mode is learned and sent back as `/q CODE`.
        let line = format!("§b§l» §r§7§7Finding you a game of {code}...");
        assert_eq!(read_line(&line), Some(Line::Queueing(mode(code))), "{code}");
        assert_eq!(command(mode(code)), format!("/q {code}"));
    }
    // And the codes the menu names for people are real codes in that list.
    for game in [
        "block drop",
        "block party",
        "capture the flag",
        "deathrun",
        "ghost invasion",
        "gravity",
        "ground wars",
        "hide and seek",
        "mobgame",
        "murder mystery",
        "the bridge",
        "skywars",
        "build battle",
        "survival games",
    ] {
        let Some(Line::QueueingNamed(found)) =
            read_line(&format!("Finding you a game of {game}..."))
        else {
            panic!("{game} is not known");
        };
        let named_codes: Vec<&str> = match found {
            Named::Code(code) => vec![code],
            Named::BySize(sizes) => sizes.iter().map(|(_, code)| *code).collect(),
        };
        for code in named_codes {
            assert!(codes.contains(&code), "{game}: {code}");
        }
    }
}

#[test]
fn an_unknown_queue_forgets_the_last_mode() {
    // Bedwars was queued, then The Bridge from the menu: its game over must not queue Bedwars.
    let mut queue = Queue::default();
    queue.queueing(mode("BED-DUOS"));
    assert!(queue.queueing_unknown());
    assert!(!queue.finished(0.0), "no mode, so nothing is queued");
    assert!(!queue.queueing_unknown(), "nothing left to forget");
    // Queueing by hand names the mode again.
    queue.queueing(mode("BRIDGE-DUOS"));
    assert!(queue.finished(1.0));
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
    // Recorded from Hive Deathrun: finishing the run ends the player's game before the game ends.
    assert_eq!(
        read_line("§a§l» §r§e You finished in §f5th §eplace! §7[01:41.782]"),
        Some(Line::YouEliminated)
    );
    assert_eq!(
        read_line("§a§l» §r§7Roselily2 has finished in 6th place! [01:42.384]"),
        None,
        "other players finishing is not the player"
    );
    // Recorded from Hive Gravity: all maps done is the end; one map done is not.
    assert_eq!(
        read_line("§a§l» §r§e You finished all maps and came in §f5th §eplace! §7[01:08.4]"),
        Some(Line::YouEliminated)
    );
    assert_eq!(
        read_line("§a§l» §r§e You finished §fDimensions§e! §7[00:10.565] [+00:01.28]"),
        None
    );
    assert_eq!(
        read_line("§a§l» §r§7EnviousPuma6122 finished all maps and came in 2nd place! [01:0"),
        None
    );
    // Recorded from Hive Bedwars.
    assert_eq!(
        read_line("§c§l» §r§iYou have been §celiminated §ifrom the game! §8[No bed]"),
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
    for name in [
        "SKY",
        "SKY-DUOS",
        "SW",
        "SW-SQUADS",
        "SG",
        "SG-DUOS",
        "PARTY",
        "BP",
        "DROP",
    ] {
        assert!(mode(name).no_respawns(), "{name}");
    }
    for name in [
        "BED", "BED-DUOS", "BW", "SWITCH", "SGX", "DROPX", "MURDER", "BUILD",
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

#[test]
fn a_death_by_ones_own_doing_ends_a_no_respawn_game() {
    // Recorded from Hive SkyWars (solos): falling into the void prints only these two lines, then
    // the "You died!" title. There is no kill line, and the `alive` flag stays on while spectating.
    assert_eq!(
        read_line("§c§l» §r§7Boldworks §cdid an oopsie!"),
        Some(Line::SelfOut(who("boldworks")))
    );
    // A chat message that imitates it is not a name.
    assert_eq!(
        read_line("§7qz x7q2§8 [§iU§8] §7§l» §rboldworks did an oopsie!"),
        None
    );

    let mut queue = Queue::default();
    queue.queueing(mode("SKY"));
    // The first join line after the queue starts is the player's own.
    queue.joined(who("boldworks"), 12);
    queue.joined(who("someone else"), 12);
    assert!(!queue.self_out(who("someone else"), 5.0), "not the player");
    assert!(!queue.asking());
    assert!(queue.self_out(who("boldworks"), 6.0));
    assert!(
        queue.asking(),
        "the party question is asked, then the queue"
    );
}

#[test]
fn an_oopsie_only_ends_a_game_that_has_no_respawns() {
    let mut queue = Queue::default();
    queue.queueing(mode("BED-DUOS"));
    queue.joined(who("boldworks"), 8);
    assert!(!queue.self_out(who("boldworks"), 5.0), "Bedwars respawns");

    // With the mode unknown nothing is queued either.
    let mut queue = Queue::default();
    queue.queueing(mode("SKY"));
    queue.joined(who("boldworks"), 12);
    queue.queueing_unknown();
    assert!(!queue.self_out(who("boldworks"), 5.0));
}

#[test]
fn the_players_name_is_learned_again_at_each_queue() {
    let mut queue = Queue::default();
    queue.queueing(mode("SKY"));
    queue.joined(who("first name"), 12);
    queue.joined(who("boldworks"), 12);
    assert!(
        !queue.self_out(who("boldworks"), 1.0),
        "the second to join is not the player"
    );
    queue.queueing(mode("SKY"));
    queue.joined(who("boldworks"), 12);
    assert!(queue.self_out(who("boldworks"), 2.0));
}

#[test]
fn survival_games_from_the_menu_is_queued_again_after_a_kill() {
    // Recorded from Hive: Survival Games has 24 players; being killed ends the game for good.
    let mut queue = Queue::default();
    assert_eq!(
        queue.queueing_named(Named::BySize(&[(24, "SG")])),
        None,
        "the size comes next"
    );
    assert_eq!(queue.joined(who("boldworks"), 24), Some(mode("SG")));
    assert!(
        queue.killed(50.0),
        "no respawns: out, so the party is asked about"
    );
    assert!(queue.ask_ready(50.0));
    queue.asked(50.0);
    assert!(queue.not_in_party(50.5), "Hive says there is no party");
    assert!(queue.ready(51.6));
    assert_eq!(command(queue.mode.unwrap()), "/q SG");
}

#[test]
fn finishing_a_deathrun_queues_deathrun_again() {
    // "Finding you a game of DeathRun..." is the menu's name for it; finishing is the player
    // being out, so the party is asked about and, with none, `/q DR` follows.
    let Some(Line::QueueingNamed(game)) =
        read_line("§b§l» §r§7§7Finding you a game of DeathRun...")
    else {
        panic!("DeathRun is not known");
    };
    let mut queue = Queue::default();
    assert_eq!(queue.queueing_named(game), Some(mode("DR")));
    assert!(
        queue.eliminated(100.0),
        "finished: the party is asked about"
    );
    assert!(queue.ask_ready(100.0));
    queue.asked(100.0);
    assert!(queue.not_in_party(100.5));
    assert!(queue.ready(101.6));
    assert_eq!(command(queue.mode.unwrap()), "/q DR");
}

#[test]
fn block_party_out_lines_end_the_players_game() {
    // Recorded from Hive Block Party: the player's own line has no [+2XP]; the others' have.
    assert_eq!(
        read_line("§c§l» §r§fBoldworks §cmade their last dance move"),
        Some(Line::SelfOut(who("boldworks")))
    );
    assert_eq!(
        read_line("§c§l» §r§fCardPoem4674 §cmade their last dance move §7[+2XP]"),
        Some(Line::SelfOut(who("cardpoem4674")))
    );
    assert_eq!(
        read_line("§c§l» §r§fWorkedTuna5207 §cfell off §7[+2XP]"),
        Some(Line::SelfOut(who("workedtuna5207")))
    );

    let Some(Line::QueueingNamed(game)) =
        read_line("§b§l» §r§7§7Finding you a game of Block Party...")
    else {
        panic!("Block Party is not known");
    };
    let mut queue = Queue::default();
    assert_eq!(queue.queueing_named(game), Some(mode("PARTY")));
    queue.joined(who("boldworks"), 14);
    // Somebody else falling off is not the player.
    assert!(!queue.self_out(who("cardpoem4674"), 20.0));
    assert!(queue.self_out(who("boldworks"), 30.0));
    assert!(queue.ask_ready(30.0));
    queue.asked(30.0);
    assert!(queue.not_in_party(30.5));
    assert!(queue.ready(31.6));
    assert_eq!(command(queue.mode.unwrap()), "/q PARTY");
}

#[test]
fn block_drop_falling_off_the_map_ends_the_players_game() {
    // Recorded from Hive Block Drop: the others' lines end with [5 XP], the player's does not.
    assert_eq!(
        read_line("§c§l» §r§fBoldworks §cfell off the map!"),
        Some(Line::SelfOut(who("boldworks")))
    );
    assert_eq!(
        read_line("§c§l» §r§fAx0catt972 §cfell off the map! §7[5 XP]"),
        Some(Line::SelfOut(who("ax0catt972")))
    );
    let Some(Line::QueueingNamed(game)) =
        read_line("§b§l» §r§7§7Finding you a game of Block Drop...")
    else {
        panic!("Block Drop is not known");
    };
    let mut queue = Queue::default();
    assert_eq!(queue.queueing_named(game), Some(mode("DROP")));
    queue.joined(who("boldworks"), 12);
    assert!(!queue.self_out(who("ax0catt972"), 90.0), "somebody else");
    assert!(queue.self_out(who("boldworks"), 100.0));
    assert!(queue.ask_ready(100.0));
    queue.asked(100.0);
    assert!(queue.not_in_party(100.5));
    assert!(queue.ready(101.6));
    assert_eq!(command(queue.mode.unwrap()), "/q DROP");
}

#[test]
fn a_custom_row_in_the_sidebar_marks_a_custom_game() {
    // Recorded from Hive: the top right of a custom game says "Custom" in a sidebar row.
    let title = "support.playhive.com/ui";
    let custom = ["§7Custom", "§f24:56", "§aLevel §f§f50§7/100"];
    assert_eq!(custom_game(title, custom.into_iter()), Some(true));
    assert_eq!(custom_game(title, ["custom "].into_iter()), Some(true));
    let normal = ["§7Now Playing", "§f24:56", "§aLevel §f§f50§7/100"];
    assert_eq!(custom_game(title, normal.into_iter()), Some(false));
    assert_eq!(custom_game(title, [].into_iter()), Some(false));
    // Other words that only contain it are not it.
    assert_eq!(custom_game(title, ["Customs"].into_iter()), Some(false));
    assert_eq!(
        custom_game(title, ["§7Custom game"].into_iter()),
        Some(false)
    );
    // After the game the sidebar is the summary: it says nothing, so what was known stays.
    assert_eq!(custom_game("Game Summary", normal.into_iter()), None);
}

#[test]
fn a_custom_game_is_never_queued() {
    let mut queue = Queue::default();
    queue.queueing(mode("SKY"));
    queue.set_custom(true);
    assert!(queue.is_custom());
    assert!(!queue.finished(100.0), "game over");
    assert!(!queue.waiting());
    assert!(!queue.eliminated(100.0), "eliminated");
    assert!(!queue.asking());
    assert!(!queue.ready(110.0));
    // A normal game after it queues again.
    queue.set_custom(false);
    assert!(queue.finished(120.0));
    assert!(queue.ready(121.5));
}

#[test]
fn a_game_found_to_be_custom_drops_what_was_scheduled() {
    let mut queue = Queue::default();
    queue.queueing(mode("SKY"));
    assert!(queue.eliminated(10.0));
    queue.set_custom(true);
    assert!(!queue.asking());
    assert!(!queue.ask_ready(10.5));
    assert!(!queue.finished(0.0));
    queue.set_custom(false);
    assert!(queue.finished(30.0));
    queue.set_custom(true);
    assert!(!queue.waiting());
    assert!(!queue.ready(32.0));
}
