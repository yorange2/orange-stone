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
