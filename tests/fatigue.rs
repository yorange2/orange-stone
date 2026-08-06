//! Integration tests — fatigue (empty-deck draw damage, official HS rule).
//!
//! Every draw attempt on an empty deck funnels through the draw choke point
//! (`draw_card_no_queue`), which deals damage to the drawing hero equal to the
//! player's 1-based fatigue counter and then increments the counter (1, 2, 3,
//! …). Fatigue damage goes through the unified `DamageDealt` pipeline: armor
//! absorbs it, lethal ends the game with the correct winner, and non-empty
//! decks never fatigue.

use orange_stone::cards::def::OGRE_MAGI;
use orange_stone::core::action::Action;
use orange_stone::core::effect::CardEffect;
use orange_stone::core::entity::Entity;
use orange_stone::core::event::Event;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::{GameState, Step};
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::game::GameBuilder;

/// Effective hero health (base − accumulated damage).
fn hero_hp(state: &GameState, player: PlayerId) -> i32 {
    state
        .world()
        .effective_health(state.player(player).hero)
        .unwrap()
        .0
}

/// The fatigue damage events dealt to a hero (self-damage by that hero's
/// owner), in order.
fn fatigue_hits(log: &[Event], hero: Entity) -> Vec<i32> {
    log.iter()
        .filter_map(|e| match e {
            Event::DamageDealt {
                source,
                target,
                amount,
            } if *target == hero && *source == hero => Some(*amount),
            _ => None,
        })
        .collect()
}

#[test]
fn first_empty_deck_draw_deals_1_second_deals_2() {
    let engine = GameEngine::new();
    let mut state = GameBuilder::new().build(); // both decks empty
    let p2_hero = state.player(PlayerId::Player2).hero;

    // EndTurn → P2's turn-start DrawStep hits the empty deck: fatigue 1
    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(fatigue_hits(&log, p2_hero), vec![1]);
    assert_eq!(hero_hp(&state, PlayerId::Player2), 29);
    assert_eq!(state.player(PlayerId::Player2).fatigue, 2);

    // P1's turn fatigues too (1) — then P2's second attempt deals 2
    engine.apply(&mut state, Action::EndTurn).unwrap();
    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(fatigue_hits(&log, p2_hero), vec![2]);
    assert_eq!(hero_hp(&state, PlayerId::Player2), 27);
    assert_eq!(state.player(PlayerId::Player2).fatigue, 3);
}

#[test]
fn multi_draw_on_empty_deck_hits_once_per_attempt() {
    // A draw-2 effect on an empty deck deals 1 + 2 = 3 (official rule 2)
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.set_mana(PlayerId::Player1, 2, 2);
    let hero = builder.state_mut().player(PlayerId::Player1).hero;
    builder.set_hero_power(PlayerId::Player1, 1, CardEffect::DrawCard { count: 2 });
    let mut state = builder.build();

    let log = engine
        .apply(&mut state, Action::HeroPower { hero, target: None })
        .unwrap();

    assert_eq!(fatigue_hits(&log, hero), vec![1, 2]);
    assert_eq!(hero_hp(&state, PlayerId::Player1), 27);
    assert_eq!(state.player(PlayerId::Player1).fatigue, 3);
}

#[test]
fn turn_start_draw_step_fatigues() {
    // The DrawStep of the turn-start sequence fatigues on an empty deck — the
    // DamageDealt follows the TurnStarted event in the EndTurn resolution
    let engine = GameEngine::new();
    let mut state = GameBuilder::new().build();
    let p2_hero = state.player(PlayerId::Player2).hero;

    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    let turn_started = log
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::TurnStarted {
                    player: PlayerId::Player2
                }
            )
        })
        .expect("turn start should appear in the log");
    assert!(
        log[turn_started..].iter().any(
            |e| matches!(e, Event::DamageDealt { target, amount: 1, .. } if *target == p2_hero)
        )
    );
}

#[test]
fn armor_absorbs_fatigue() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.hero_armor(PlayerId::Player2, 5);
    let mut state = builder.build();
    let p2_hero = state.player(PlayerId::Player2).hero;

    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(fatigue_hits(&log, p2_hero), vec![1]);
    assert_eq!(hero_hp(&state, PlayerId::Player2), 30);
    assert_eq!(state.player(PlayerId::Player2).armor, 4);
    assert_eq!(state.player(PlayerId::Player2).fatigue, 2);
}

#[test]
fn fatigue_lethal_ends_game_with_correct_winner() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.hero_health(PlayerId::Player2, 1);
    let mut state = builder.build();

    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    assert!(log.iter().any(|e| matches!(
        e,
        Event::GameOver {
            winner: PlayerId::Player1
        }
    )));
    assert_eq!(
        state.step(),
        Step::GameOver {
            winner: PlayerId::Player1
        }
    );
}

#[test]
fn draw_from_non_empty_deck_has_no_fatigue() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_deck(PlayerId::Player2, &OGRE_MAGI);
    let mut state = builder.build();

    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    assert!(!log.iter().any(|e| matches!(e, Event::DamageDealt { .. })));
    // The card moved deck → hand, no damage, counter untouched (still 1)
    assert_eq!(
        state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .count(),
        1
    );
    assert_eq!(hero_hp(&state, PlayerId::Player2), 30);
    assert_eq!(state.player(PlayerId::Player2).fatigue, 1);
}

#[test]
fn opening_and_mulligan_draws_never_fatigue() {
    // Opening/mulligan draws run through the queue-less helper on a provably
    // full deck — no damage, counters untouched (official rule 5)
    let mut builder = GameBuilder::new();
    for _ in 0..30 {
        builder.add_minion_to_deck(PlayerId::Player1, &OGRE_MAGI);
    }
    for _ in 0..30 {
        builder.add_minion_to_deck(PlayerId::Player2, &OGRE_MAGI);
    }
    let mut state = builder.build();
    state.begin_game();

    assert_eq!(
        state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .count(),
        3
    );
    // 4 cards + The Coin for the second player
    assert_eq!(
        state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .count(),
        5
    );
    assert_eq!(state.player(PlayerId::Player1).fatigue, 1);
    assert_eq!(state.player(PlayerId::Player2).fatigue, 1);
}

#[test]
fn ice_block_prevents_fatigue_lethal_and_game_continues() {
    use orange_stone::core::component::{Secret, SecretTrigger};
    use orange_stone::core::effect::CardEffect;
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.hero_health(PlayerId::Player2, 1);
    let p2_hero = builder.state_mut().player(PlayerId::Player2).hero;
    // A played secret sits in SetAside — attach the component to a spawned
    // entity and move it there (the secret system only scans SetAside)
    let secret_entity = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
    builder.set_secret_on_entity(
        secret_entity,
        Secret {
            trigger: SecretTrigger::WhenFriendlyHeroFatallyDamaged,
            effect: Some(CardEffect::PreventFatalDamageAndImmune),
        },
    );
    {
        let world = builder.state_mut().world_mut();
        let _ = world.move_to_zone(secret_entity, Zone::SetAside);
    }
    let mut state = builder.build();

    // EndTurn → P2's DrawStep fatigues for 1 — lethal, but Ice Block saves her
    let log = engine.apply(&mut state, Action::EndTurn).unwrap();
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. }))
    );
    assert!(!log.iter().any(|e| matches!(e, Event::GameOver { .. })));
    assert_eq!(hero_hp(&state, PlayerId::Player2), 1);
    assert!(
        state.world().immune(p2_hero).is_some(),
        "the hero becomes Immune"
    );
    assert_eq!(state.step(), Step::Main);

    // The secret is spent — the next fatigue hit kills the hero
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(
        state.step(),
        Step::GameOver {
            winner: PlayerId::Player1
        }
    );
}

#[test]
fn battle_with_tiny_decks_ends_with_a_fatigue_winner() {
    use orange_stone::sim::battle::{BattleRunner, BotType};
    // Deck 1: the opening deal empties the decks, then every turn draw
    // fatigues — the battle must end with a real winner well before the
    // 60-turn cap (previously a deck-draining battle stalled forever)
    let mut runner = BattleRunner::new(BotType::Smart, 7);
    let result = runner.run_battle(1);
    assert!(
        result.winner.is_some(),
        "a deck-draining battle must end with a real winner, not the turn cap"
    );
    assert!(
        result.turns < 60,
        "fatigue ends the battle well before the turn cap (turns = {})",
        result.turns
    );
}

#[test]
fn fatigue_is_deterministic_across_same_seed_replays() {
    // Fatigue uses no RNG — two identical same-seed states replay
    // byte-identically through the empty-deck draws
    let engine = GameEngine::new();
    let build = |seed: u64| {
        let mut builder = GameBuilder::new();
        builder.with_rng_seed(seed);
        builder.build()
    };
    let mut s1 = build(42);
    let mut s2 = build(42);
    let mut logs1 = Vec::new();
    let mut logs2 = Vec::new();
    for _ in 0..6 {
        logs1.push(engine.apply(&mut s1, Action::EndTurn).unwrap());
        logs2.push(engine.apply(&mut s2, Action::EndTurn).unwrap());
    }
    assert_eq!(logs1, logs2, "the fatigue replay must be identical");
    assert_eq!(
        hero_hp(&s1, PlayerId::Player2),
        hero_hp(&s2, PlayerId::Player2)
    );
    assert_eq!(
        s1.player(PlayerId::Player2).fatigue,
        s2.player(PlayerId::Player2).fatigue
    );
}
