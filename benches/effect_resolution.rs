//! Benchmarks for effect resolution (roadmap A3).
//!
//! Measures the two hot paths that A1/A2 were meant to accelerate:
//! - `effective_attack/health/cost` queries on an aura-heavy board (A2)
//! - full event-loop resolution of play-card and attack actions (A1 + damage)
//!
//! State templates are cloned per iteration — `GameState::clone` is an O(1)
//! Arc bump, so iteration overhead is negligible.

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use orange_stone::cards::def::{
    BLOODFEN_RAPTOR, DIRE_WOLF_ALPHA, RAID_LEADER, SORCERERS_APPRENTICE, STORMWIND_CHAMPION,
};
use orange_stone::core::action::Action;
use orange_stone::core::component::CardType;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::GameState;
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::game::GameBuilder;

/// Builds an aura-heavy board template:
/// P1: 4 aura sources (Raid Leader, Stormwind Champion, Dire Wolf Alpha, Sorcerer's Apprentice) + 3 vanilla minions
/// P2: 7 vanilla minions; both players hold 2 aura cards in hand (inactive aura sources)
fn aura_board_template() -> GameState {
    let mut b = GameBuilder::new();
    b.set_mana(PlayerId::Player1, 10, 10);
    b.set_mana(PlayerId::Player2, 10, 10);
    b.add_minion_to_board(PlayerId::Player1, &RAID_LEADER);
    b.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
    b.add_minion_to_board(PlayerId::Player1, &DIRE_WOLF_ALPHA);
    b.add_minion_to_board(PlayerId::Player1, &SORCERERS_APPRENTICE);
    for _ in 0..3 {
        b.add_minion_to_board(PlayerId::Player1, &BLOODFEN_RAPTOR);
        b.add_minion_to_board(PlayerId::Player2, &BLOODFEN_RAPTOR);
    }
    b.add_minion_to_board(PlayerId::Player2, &BLOODFEN_RAPTOR);
    b.add_minion_to_hand(PlayerId::Player1, &STORMWIND_CHAMPION);
    b.add_minion_to_hand(PlayerId::Player2, &RAID_LEADER);
    b.build()
}

/// Full-board effective stat queries (simulating a bot's full-board scan during decision making).
fn bench_effective_stats(c: &mut Criterion) {
    let template = aura_board_template();
    c.bench_function("effective_stats/aura_board_14_minions", |b| {
        b.iter_batched(
            || template.clone(),
            |state| {
                let world = state.world();
                let mut acc = 0i32;
                for player in [PlayerId::Player1, PlayerId::Player2] {
                    for e in world.zones().iter(Zone::Play, player) {
                        if world.card_type(e) == Some(CardType::Minion) {
                            acc += black_box(world.effective_attack(e)).map_or(0, |a| a.0);
                            acc += black_box(world.effective_health(e)).map_or(0, |h| h.0);
                        }
                    }
                    for e in world.zones().iter(Zone::Hand, player) {
                        acc += black_box(world.effective_cost(e)).map_or(0, |c| c.0);
                    }
                }
                black_box(acc);
            },
            BatchSize::SmallInput,
        )
    });
}

/// Plays 5 minion cards in a row (including 3 aura cards), going through the full event loop.
fn bench_play_minions(c: &mut Criterion) {
    let engine = GameEngine::new();
    let mut template = GameBuilder::new();
    template.set_mana(PlayerId::Player1, 10, 10);
    // All 2-cost cards, 5 cards totaling 10 mana (2x Dire Wolf Alpha + Sorcerer's Apprentice + 2 vanilla)
    template.add_minion_to_hand(PlayerId::Player1, &DIRE_WOLF_ALPHA);
    template.add_minion_to_hand(PlayerId::Player1, &SORCERERS_APPRENTICE);
    template.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
    template.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
    template.add_minion_to_hand(PlayerId::Player1, &DIRE_WOLF_ALPHA);
    let template = template.build();

    c.bench_function("effect_resolution/play_5_minions", |b| {
        b.iter_batched(
            || template.clone(),
            |mut state| {
                let cards: Vec<_> = state
                    .world()
                    .zones()
                    .entities(Zone::Hand, PlayerId::Player1);
                for card in cards {
                    engine
                        .apply(&mut state, Action::PlayCard {
                            card,
                            target: None,
                            position: None,
                        })
                        .expect("play should succeed");
                }
                black_box(state.turn());
            },
            BatchSize::SmallInput,
        )
    });
}

/// One round of 7 minion trades (7 minions per side, including damage resolution and death checks).
fn bench_attack_round(c: &mut Criterion) {
    let engine = GameEngine::new();
    let mut template = GameBuilder::new();
    let mut attackers = Vec::new();
    let mut defenders = Vec::new();
    for i in 0..7 {
        attackers.push(template.add_custom_minion_to_board(PlayerId::Player1, 4, 6, i + 1));
        defenders.push(template.add_custom_minion_to_board(PlayerId::Player2, 3, 5, i + 1));
    }
    let template = template.build();

    c.bench_function("effect_resolution/attack_round_7_trades", |b| {
        b.iter_batched(
            || template.clone(),
            |mut state| {
                let mut resolved = 0u32;
                for (attacker, defender) in attackers.iter().zip(&defenders) {
                    if engine
                        .apply(
                            &mut state,
                            Action::Attack {
                                attacker: *attacker,
                                defender: *defender,
                            },
                        )
                        .is_ok()
                    {
                        resolved += 1;
                    }
                }
                black_box(resolved);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_effective_stats,
    bench_play_minions,
    bench_attack_round
);
criterion_main!(benches);
