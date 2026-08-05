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

/// 构建一个光环密集的战场模板：
/// P1: 4 个光环源（突袭队长、暴风城勇士、恐狼前锋、巫师学徒）+ 3 个白板
/// P2: 7 个白板；双方手牌各 2 张带光环的牌（非活跃光环源）
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

/// 全战场有效属性查询（模拟 bot 决策时的全盘扫描）。
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

/// 连续打出 5 张随从牌（含 3 张光环牌），走完整的事件循环。
fn bench_play_minions(c: &mut Criterion) {
    let engine = GameEngine::new();
    let mut template = GameBuilder::new();
    template.set_mana(PlayerId::Player1, 10, 10);
    // 全部 2 费卡牌，5 张合计 10 费（2 张恐狼前锋 + 巫师学徒 + 2 张白板）
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
                        .apply(&mut state, Action::PlayCard { card, target: None })
                        .expect("play should succeed");
                }
                black_box(state.turn());
            },
            BatchSize::SmallInput,
        )
    });
}

/// 一轮 7 次随从交换攻击（双方各 7 个随从，含伤害结算与死亡检查）。
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
