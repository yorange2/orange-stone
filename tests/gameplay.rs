//! 集成测试 — 完整对局流程。
//!
//! 测试覆盖 Phase 1 所有功能：
//! - 出牌（白板随从）
//! - 随从交易
//! - 攻击英雄
//! - 回合切换
//! - 游戏结束
//! - 确定性回放

use orange_stone::cards::def::{CHILLWIND_YETI, RIVER_CROCOLISK, WISP};
use orange_stone::core::action::Action;
use orange_stone::core::entity::Entity;
use orange_stone::core::event::Event;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::{GameState, Phase};
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::engine::rules::EngineError;
use orange_stone::sim::game::GameBuilder;

// ============================================================
// 出牌测试
// ============================================================

#[test]
fn play_minion_moves_from_hand_to_board() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &CHILLWIND_YETI);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    let log = engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 检查事件
    assert_eq!(log.len(), 2);
    assert!(matches!(log[0], Event::CardPlayed { .. }));
    assert!(matches!(log[1], Event::MinionSummoned { .. }));

    // 卡牌应在战场上
    assert_eq!(state.world().zone(card), Some(Zone::Play));
    assert!(
        state
            .world()
            .zones()
            .is_empty(Zone::Hand, PlayerId::Player1)
    );
}

#[test]
fn not_enough_mana_rejected() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &CHILLWIND_YETI);
    // 不设置法力，默认 0
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    let result = engine.apply(&mut state, Action::PlayCard { card });
    assert_eq!(result, Err(EngineError::NotEnoughMana));
}

#[test]
fn play_card_when_board_has_7_minions_fails() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    // 填满 7 个随从
    for _ in 0..7 {
        builder.add_minion_to_board(PlayerId::Player1, &WISP);
    }
    builder.add_minion_to_hand(PlayerId::Player1, &CHILLWIND_YETI);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    let result = engine.apply(&mut state, Action::PlayCard { card });
    assert_eq!(result, Err(EngineError::BoardFull));
}

#[test]
fn play_card_not_your_turn() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player2, &RIVER_CROCOLISK);
    builder.active_player(PlayerId::Player1);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player2)
        .collect();
    let card = hand[0];

    let result = engine.apply(&mut state, Action::PlayCard { card });
    assert_eq!(result, Err(EngineError::NotYourCard));
}

// ============================================================
// 攻击测试
// ============================================================

#[test]
fn attack_trade_deals_damage_both_ways() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 4, 5, 3);
    let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    let state = builder.build();

    let mut state_copy = state.clone();

    // 执行攻击
    let _log = engine
        .apply(&mut state_copy, Action::Attack { attacker, defender })
        .unwrap();

    // defender 应受到 4 伤害: 3 - 4 = -1 → 死亡
    assert_eq!(
        state_copy.world().health(defender),
        Some(orange_stone::core::component::Health(-1))
    );
    assert_eq!(state_copy.world().zone(defender), Some(Zone::Graveyard));

    // attacker 应受到 2 伤害: 5 - 2 = 3
    assert_eq!(
        state_copy.world().health(attacker),
        Some(orange_stone::core::component::Health(3))
    );
    assert_eq!(state_copy.world().zone(attacker), Some(Zone::Play));
}

#[test]
fn attacker_dies_in_trade_still_deals_damage() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 4, 1, 3);
    let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 10, 2);
    let mut state = builder.build();

    let log = engine
        .apply(&mut state, Action::Attack { attacker, defender })
        .unwrap();

    // defender 受到 4 伤害: 10 → 6，存活
    assert_eq!(
        state.world().health(defender),
        Some(orange_stone::core::component::Health(6))
    );
    // attacker 受到 3 伤害: 1 → -2，死亡
    assert_eq!(
        state.world().health(attacker),
        Some(orange_stone::core::component::Health(-2))
    );
    assert_eq!(state.world().zone(attacker), Some(Zone::Graveyard));
    // event log 包含 MinionDied
    assert!(log.iter().any(|e| matches!(e, Event::MinionDied { .. })));
}

#[test]
fn attack_hero_deals_damage_one_way() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 5, 3, 3);
    let state = builder.build();
    let hero = state.player(PlayerId::Player2).hero;

    let mut state = state;
    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    // 英雄受到 5 伤害
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(25))
    );
    // 攻击者不受伤害（英雄不还手）
    assert_eq!(
        state.world().health(attacker),
        Some(orange_stone::core::component::Health(3))
    );
    // DamageDealt 事件只有一个（只有攻击者→英雄）
    let damage_count = log
        .iter()
        .filter(|e| matches!(e, Event::DamageDealt { .. }))
        .count();
    assert_eq!(damage_count, 1);
}

#[test]
fn hero_death_ends_game() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 30, 3, 3);
    let state = builder.build();
    let hero = state.player(PlayerId::Player2).hero;

    let mut state = state;
    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    assert_eq!(
        state.phase(),
        Phase::GameOver {
            winner: PlayerId::Player1
        }
    );
    assert!(log.iter().any(|e| matches!(e, Event::GameOver { .. })));
}

#[test]
fn cannot_attack_twice_in_one_turn() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 5, 3);
    let defender1 = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 5, 1);
    let defender2 = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 5, 1);
    let mut state = builder.build();

    // 第一次攻击
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: defender1,
            },
        )
        .unwrap();

    // 第二次攻击应该失败
    let result = engine.apply(
        &mut state,
        Action::Attack {
            attacker,
            defender: defender2,
        },
    );
    assert_eq!(result, Err(EngineError::AttacksExhausted));
}

#[test]
fn attacks_reset_after_end_turn() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 5, 3);
    let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 20, 1);
    let mut state = builder.build();

    // 攻击
    engine
        .apply(&mut state, Action::Attack { attacker, defender })
        .unwrap();

    assert_eq!(
        state.world().attacks_used(attacker),
        Some(orange_stone::core::component::AttacksUsed(1))
    );

    // Player1 结束回合 → Player2 的回合
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Player2 结束回合 → 回到 Player1
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // Player1 的随从攻击次数应已重置
    assert_eq!(
        state.world().attacks_used(attacker),
        Some(orange_stone::core::component::AttacksUsed(0))
    );
}

#[test]
fn cannot_attack_own_minions() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 5, 3);
    let friendly = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 5, 1);
    let mut state = builder.build();

    let result = engine.apply(
        &mut state,
        Action::Attack {
            attacker,
            defender: friendly,
        },
    );
    assert_eq!(result, Err(EngineError::InvalidTarget));
}

#[test]
fn cannot_attack_with_stale_entity() {
    let engine = GameEngine::new();
    let mut state = GameState::new();
    let hero = state.player(PlayerId::Player2).hero;
    let stale = Entity::new(999, 0);

    let result = engine.apply(
        &mut state,
        Action::Attack {
            attacker: stale,
            defender: hero,
        },
    );
    assert!(matches!(result, Err(EngineError::EntityGone(_))));
}

// ============================================================
// 回合切换测试
// ============================================================

#[test]
fn end_turn_switches_active_player() {
    let engine = GameEngine::new();
    let mut state = GameState::new();

    let log = engine.apply(&mut state, Action::EndTurn).unwrap();

    assert_eq!(state.active_player(), PlayerId::Player2);
    assert_eq!(state.turn(), 2);
    assert_eq!(state.phase(), Phase::Main);
    assert!(log.iter().any(|e| matches!(e, Event::TurnEnded { .. })));
    assert!(log.iter().any(|e| matches!(e, Event::TurnStarted { .. })));
}

#[test]
fn game_over_rejects_all_actions() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 30, 3, 3);
    let state = builder.build();
    let hero = state.player(PlayerId::Player2).hero;

    let mut state = state;
    // 先结束游戏
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    // 所有操作都应该拒绝
    assert_eq!(
        engine.apply(&mut state, Action::EndTurn),
        Err(EngineError::GameAlreadyOver)
    );
}

// ============================================================
// 确定性测试
// ============================================================

#[test]
fn same_actions_produce_identical_results() {
    let engine = GameEngine::new();

    let build_game = || {
        let mut builder = GameBuilder::new();
        let card1 = builder.add_custom_minion_to_hand(PlayerId::Player1, 3, 4, 3);
        let card2 = builder.add_custom_minion_to_hand(PlayerId::Player1, 2, 3, 2);
        let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 10, 1);
        builder.set_mana(PlayerId::Player1, 10, 10);
        (builder.build(), card1, card2, defender)
    };

    let (mut state1, card1a, card2a, defender_a) = build_game();
    let (mut state2, card1b, card2b, defender_b) = build_game();

    // 执行相同的操作序列
    let log1 = engine
        .apply(&mut state1, Action::PlayCard { card: card1a })
        .unwrap();
    let log2 = engine
        .apply(&mut state2, Action::PlayCard { card: card1b })
        .unwrap();
    assert_eq!(log1, log2);

    let log1 = engine
        .apply(&mut state1, Action::PlayCard { card: card2a })
        .unwrap();
    let log2 = engine
        .apply(&mut state2, Action::PlayCard { card: card2b })
        .unwrap();
    assert_eq!(log1, log2);

    let log1 = engine.apply(&mut state1, Action::EndTurn).unwrap();
    let log2 = engine.apply(&mut state2, Action::EndTurn).unwrap();
    assert_eq!(log1, log2);

    // Player2 攻击（现在 Player2 是 active player）
    let log1 = engine
        .apply(
            &mut state1,
            Action::Attack {
                attacker: defender_a,
                defender: card1a,
            },
        )
        .unwrap();
    let log2 = engine
        .apply(
            &mut state2,
            Action::Attack {
                attacker: defender_b,
                defender: card1b,
            },
        )
        .unwrap();
    assert_eq!(log1, log2);

    // 最终状态应该一致
    assert_eq!(state1.world().health(card1a), state2.world().health(card1b));
    assert_eq!(
        state1.world().health(defender_a),
        state2.world().health(defender_b)
    );
}

#[test]
fn full_game_scenario() {
    let engine = GameEngine::new();

    let mut builder = GameBuilder::new();
    let yeti = builder.add_custom_minion_to_hand(PlayerId::Player1, 4, 5, 4);
    let croc = builder.add_custom_minion_to_hand(PlayerId::Player1, 2, 3, 2);
    let ogre = builder.add_custom_minion_to_board(PlayerId::Player2, 6, 7, 6);
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    let mut state = builder.build();

    // Turn 1: Player1 打出 Yeti (4/5)
    let log = engine
        .apply(&mut state, Action::PlayCard { card: yeti })
        .unwrap();
    assert!(log.iter().any(|e| matches!(e, Event::CardPlayed { .. })));
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::MinionSummoned { .. }))
    );

    // Player1 打出 Crocolisk (2/3)
    let log = engine
        .apply(&mut state, Action::PlayCard { card: croc })
        .unwrap();
    assert_eq!(log.len(), 2);

    // 尝试用 Croc (2/3) 攻击 Ogre (6/7) — 自杀式攻击但合法
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: croc,
                defender: ogre,
            },
        )
        .unwrap();
    // Croc 受到 6 伤害: 3 → -3 死亡; Ogre 受到 2 伤害: 7 → 5
    assert_eq!(state.world().zone(croc), Some(Zone::Graveyard));
    assert_eq!(
        state.world().health(ogre),
        Some(orange_stone::core::component::Health(5))
    );

    // 用 Yeti (4/5) 攻击受伤的 Ogre (6/5)
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: yeti,
                defender: ogre,
            },
        )
        .unwrap();
    // Yeti 受到 6 伤害: 5 → -1 死亡; Ogre 受到 4 伤害: 5 → 1 存活
    assert_eq!(state.world().zone(yeti), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(ogre), Some(Zone::Play));
    assert_eq!(
        state.world().health(ogre),
        Some(orange_stone::core::component::Health(1))
    );

    // Player1 结束回合
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // 现在 Player2 的回合，Ogre 可以攻击; 先获取英雄实体
    let hero = state.player(PlayerId::Player1).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: ogre,
                defender: hero,
            },
        )
        .unwrap();

    // 英雄受到 6 伤害
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(24))
    );
}
