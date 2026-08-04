//! 集成测试 — 完整对局流程。
//!
//! 测试覆盖 Phase 1 所有功能：
//! - 出牌（白板随从）
//! - 随从交易
//! - 攻击英雄
//! - 回合切换
//! - 游戏结束
//! - 确定性回放

use orange_stone::cards::def::{BLOODFEN_RAPTOR, MURLOC_RAIDER, OGRE_MAGI};
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
    builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
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
    builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
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
        builder.add_minion_to_board(PlayerId::Player1, &MURLOC_RAIDER);
    }
    builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
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
    builder.add_minion_to_hand(PlayerId::Player2, &BLOODFEN_RAPTOR);
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

    // 召唤失调：刚打出的随从不能攻击，先过两个回合让召唤失调消失
    // Player1 结束回合 → Player2 的回合
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Player2 结束回合 → 回到 Player1 的回合
    engine.apply(&mut state, Action::EndTurn).unwrap();

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

// ============================================================
// Phase 3: 武器测试
// ============================================================

#[test]
fn equip_weapon_gives_hero_attack() {
    use orange_stone::cards::def::EAGLEHORN_BOW;

    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
    let state = builder.build();

    let player = state.player(PlayerId::Player1);
    assert!(player.weapon.is_some());
    let weapon = player.weapon.unwrap();
    assert_eq!(
        state.world().attack(weapon),
        Some(orange_stone::core::component::Attack(3))
    );
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(2))
    );
}

#[test]
fn hero_attack_with_weapon_consumes_durability() {
    use orange_stone::cards::def::EAGLEHORN_BOW;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let state = builder.build();

    let hero = state.player(PlayerId::Player1).hero;
    let defender_hero = state.player(PlayerId::Player2).hero;

    let mut state = state;
    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: defender_hero,
            },
        )
        .unwrap();

    // 武器耐久应从 2 减到 1
    let weapon = state.player(PlayerId::Player1).weapon.unwrap();
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(1))
    );
    // 敌方英雄应受 3 伤害（武器攻击力）
    assert_eq!(
        state.world().health(defender_hero),
        Some(orange_stone::core::component::Health(27))
    );
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::AttackDeclared { .. }))
    );
}

#[test]
fn weapon_breaks_when_durability_reaches_zero() {
    use orange_stone::cards::def::EAGLEHORN_BOW;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    let state = builder.build();
    let hero = state.player(PlayerId::Player1).hero;
    let enemy = state.player(PlayerId::Player2).hero;

    let mut state = state;
    // 第一次攻击：耐久 2 → 1
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: enemy,
            },
        )
        .unwrap();
    assert!(state.player(PlayerId::Player1).weapon.is_some());

    // 需要结束两个回合才能再次攻击（英雄每回合只能攻击一次）
    engine.apply(&mut state, Action::EndTurn).unwrap(); // Player2's turn
    engine.apply(&mut state, Action::EndTurn).unwrap(); // Back to Player1

    // 第二次攻击：耐久 1 → 0，武器应被摧毁
    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: enemy,
            },
        )
        .unwrap();

    assert!(state.player(PlayerId::Player1).weapon.is_none());
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::WeaponDestroyed { .. }))
    );
}

#[test]
fn hero_cannot_attack_without_weapon() {
    let engine = GameEngine::new();
    let mut state = GameState::new();
    let hero = state.player(PlayerId::Player1).hero;
    let enemy = state.player(PlayerId::Player2).hero;

    let result = engine.apply(
        &mut state,
        Action::Attack {
            attacker: hero,
            defender: enemy,
        },
    );
    assert_eq!(result, Err(EngineError::InvalidTarget));
}

// ============================================================
// Phase 3: 护甲测试
// ============================================================

#[test]
fn armor_absorbs_damage_before_health() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
    builder.hero_armor(PlayerId::Player2, 5);
    let state = builder.build();
    let hero = state.player(PlayerId::Player2).hero;

    let mut state = state;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    // 英雄受到 3 伤害，护甲应该吸收全部：5 → 2
    assert_eq!(state.player(PlayerId::Player2).armor, 2);
    // 生命值不应减少（全部被护甲吸收）
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(30))
    );
}

#[test]
fn damage_spills_over_armor_to_health() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 10, 3, 3);
    builder.hero_armor(PlayerId::Player2, 3);
    let state = builder.build();
    let hero = state.player(PlayerId::Player2).hero;

    let mut state = state;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    // 护甲 3 被全部消耗
    assert_eq!(state.player(PlayerId::Player2).armor, 0);
    // 剩余 7 伤害穿透到生命值：30 → 23
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(23))
    );
}

// ============================================================
// Phase 3: 英雄技能测试
// ============================================================

#[test]
fn hero_power_use_once_per_turn() {
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_hero_power(
        PlayerId::Player1,
        2,
        CardEffect::DealDamage {
            amount: 1,
            target: EffectTarget::AnyEnemy,
        },
    );
    let state = builder.build();
    let hero = state.player(PlayerId::Player1).hero;

    let mut state = state;
    // 第一次使用：成功
    let log = engine
        .apply(&mut state, Action::HeroPower { hero })
        .unwrap();
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::HeroPowerActivated { .. }))
    );
    // 法力应减少 2
    assert_eq!(state.player(PlayerId::Player1).current_mana, 8);

    // 第二次使用：应失败（本回合已使用）
    let result = engine.apply(&mut state, Action::HeroPower { hero });
    assert_eq!(result, Err(EngineError::HeroPowerAlreadyUsed));
}

#[test]
fn hero_power_resets_after_turn() {
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    builder.set_hero_power(
        PlayerId::Player1,
        2,
        CardEffect::DealDamage {
            amount: 1,
            target: EffectTarget::AnyEnemy,
        },
    );
    let state = builder.build();
    let hero = state.player(PlayerId::Player1).hero;

    let mut state = state;
    // 使用英雄技能
    engine
        .apply(&mut state, Action::HeroPower { hero })
        .unwrap();
    assert_eq!(
        state.world().hero_power_used(hero),
        Some(orange_stone::core::component::HeroPowerUsed(true))
    );

    // Player1 结束回合 → Player2
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Player2 结束回合 → 回到 Player1（TurnStarted 重置）
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // 英雄技能应已重置
    assert_eq!(
        state.world().hero_power_used(hero),
        Some(orange_stone::core::component::HeroPowerUsed(false))
    );
    // 可以再次使用
    let result = engine.apply(&mut state, Action::HeroPower { hero });
    assert!(result.is_ok());
}

// ============================================================
// Phase 3: 光环测试
// ============================================================

#[test]
fn aura_buffs_other_friendly_minions() {
    use orange_stone::cards::def::GRIMSCALE_ORACLE;

    let mut builder = GameBuilder::new();
    // 团队领袖：其他友方随从 +1 攻击力
    builder.add_minion_to_board(PlayerId::Player1, &GRIMSCALE_ORACLE);
    let croc = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 3, 2);
    let state = builder.build();

    // 鳄鱼的有效攻击力应该从 2 变为 3（光环 +1）
    assert_eq!(
        state.world().effective_attack(croc),
        Some(orange_stone::core::component::Attack(3))
    );
    // 团队领袖自身不应受加成（OtherFriendlyMinions）
    let leader: Vec<_> = state
        .world()
        .zones()
        .iter(orange_stone::core::zone::Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .collect();
    // leader 的有效攻击力应包括自身基础攻击力 + 其他光环
    // GRIMSCALE_ORACLE 有 1 攻击力（自身），不给自己加
    assert_eq!(
        state.world().effective_attack(leader[0]),
        Some(orange_stone::core::component::Attack(1))
    );
}

#[test]
fn aura_bonus_disappears_when_source_dies() {
    use orange_stone::cards::def::GRIMSCALE_ORACLE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &GRIMSCALE_ORACLE);
    let croc = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 3, 2);
    let enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 1, 3);
    builder.active_player(PlayerId::Player2);
    let state = builder.build();

    let mut state = state;
    // 团队领袖被敌方随从攻击致死
    let leader: Vec<_> = state
        .world()
        .zones()
        .iter(orange_stone::core::zone::Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .collect();
    let leader_entity = leader[0];

    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: enemy,
                defender: leader_entity,
            },
        )
        .unwrap();

    // 团队领袖应死亡（1 HP vs 5 ATK → -4）
    assert_eq!(
        state.world().zone(leader_entity),
        Some(orange_stone::core::zone::Zone::Graveyard)
    );
    // 鳄鱼的有效攻击力应恢复到 2（无光环加成）
    assert_eq!(
        state.world().effective_attack(croc),
        Some(orange_stone::core::component::Attack(2))
    );
}

#[test]
fn stormwind_champion_buffs_attack_and_health() {
    use orange_stone::cards::def::MURLOC_WARLEADER;

    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &MURLOC_WARLEADER);
    let ally = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 3, 2);
    let state = builder.build();

    // 友方随从应获得 +2/+1
    assert_eq!(
        state.world().effective_attack(ally),
        Some(orange_stone::core::component::Attack(4))
    );
    assert_eq!(
        state.world().effective_health(ally),
        Some(orange_stone::core::component::Health(4))
    );
}

// ============================================================
// Phase 3: 奥秘测试
// ============================================================

#[test]
fn secret_triggers_on_enemy_hero_attack() {
    use orange_stone::core::component::{CardType, Secret, SecretTrigger};
    use orange_stone::core::effect::{CardEffect, EffectTarget};
    use orange_stone::core::zone::Zone;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    builder.active_player(PlayerId::Player2);
    let mut state = builder.build();

    // 给 Player2 装备武器
    {
        let inner = state.make_mut();
        let weapon = inner.world.spawn();
        inner
            .world
            .set_attack(weapon, orange_stone::core::component::Attack(3));
        inner
            .world
            .set_durability(weapon, orange_stone::core::component::Durability(2));
        inner.world.set_card_type(weapon, CardType::Weapon);
        inner.world.set_player(weapon, PlayerId::Player2);
        inner.world.set_zone(weapon, Zone::Play);
        inner
            .world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player2, weapon);
        inner.players[PlayerId::Player2.index()].weapon = Some(weapon);
    }

    // 在 Player1 的 SetAside 创建奥秘
    {
        let world = state.world_mut();
        let secret_entity = world.spawn();
        world.set_card_type(secret_entity, CardType::Spell);
        world.set_player(secret_entity, PlayerId::Player1);
        world.set_zone(secret_entity, Zone::SetAside);
        world.set_secret(
            secret_entity,
            Secret {
                trigger: SecretTrigger::AfterEnemyHeroAttacks,
                effect: CardEffect::DealDamage {
                    amount: 2,
                    target: EffectTarget::AllEnemyMinions,
                },
            },
        );
        world
            .zones_mut()
            .insert(Zone::SetAside, PlayerId::Player1, secret_entity);
    }

    let attacker_hero = state.player(PlayerId::Player2).hero;
    let defender_hero = state.player(PlayerId::Player1).hero;

    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: attacker_hero,
                defender: defender_hero,
            },
        )
        .unwrap();

    // 奥秘应该被触发
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. }))
    );
}

// ============================================================
// Phase 3: 复杂时序测试
// ============================================================

#[test]
fn deathrattle_triggers_before_zone_transfer() {
    use orange_stone::core::component::{CardType, Deathrattle};
    use orange_stone::core::effect::CardEffect;
    use orange_stone::core::zone::Zone;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 5, 2, 3);
    let minion = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
    let mut state = builder.build();

    // 给 minion 添加亡语：抽一张牌
    state
        .world_mut()
        .set_deathrattle(minion, Deathrattle(CardEffect::DrawCard { count: 1 }));

    // 把一张牌放入 Player2 的牌库
    let card_in_deck = {
        let world = state.world_mut();
        let card = world.spawn();
        world.set_card_type(card, CardType::Minion);
        world.set_player(card, PlayerId::Player2);
        world.set_zone(card, Zone::Deck);
        world.set_health(card, orange_stone::core::component::Health(1));
        world.set_attack(card, orange_stone::core::component::Attack(1));
        world.set_cost(card, orange_stone::core::component::Cost(1));
        world
            .zones_mut()
            .insert(Zone::Deck, PlayerId::Player2, card);
        card
    };

    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: minion,
            },
        )
        .unwrap();

    // minion 应被杀死并触发亡语抽牌
    assert_eq!(state.world().zone(minion), Some(Zone::Graveyard));
    // 亡语抽的牌应在手牌中
    assert!(log.iter().any(|e| matches!(e, Event::CardDrawn { .. })));
    assert_eq!(state.world().zone(card_in_deck), Some(Zone::Hand));
}

#[test]
fn multiple_secrets_trigger_in_order() {
    use orange_stone::core::component::{CardType, Secret, SecretTrigger};
    use orange_stone::core::effect::{CardEffect, EffectTarget};
    use orange_stone::core::zone::Zone;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    builder.active_player(PlayerId::Player2);
    let mut state = builder.build();

    // 在 Player1 的 SetAside 创建两个奥秘
    {
        let world = state.world_mut();
        let s1 = world.spawn();
        world.set_card_type(s1, CardType::Spell);
        world.set_player(s1, PlayerId::Player1);
        world.set_zone(s1, Zone::SetAside);
        world.set_secret(
            s1,
            Secret {
                trigger: SecretTrigger::AfterEnemyHeroAttacks,
                effect: CardEffect::DealDamage {
                    amount: 1,
                    target: EffectTarget::AnyEnemy,
                },
            },
        );
        world
            .zones_mut()
            .insert(Zone::SetAside, PlayerId::Player1, s1);

        let s2 = world.spawn();
        world.set_card_type(s2, CardType::Spell);
        world.set_player(s2, PlayerId::Player1);
        world.set_zone(s2, Zone::SetAside);
        world.set_secret(
            s2,
            Secret {
                trigger: SecretTrigger::AfterEnemyHeroAttacks,
                effect: CardEffect::GainArmor {
                    amount: 5,
                    target: EffectTarget::Self_,
                },
            },
        );
        world
            .zones_mut()
            .insert(Zone::SetAside, PlayerId::Player1, s2);
    }

    // 给 Player2 装备武器
    {
        let inner = state.make_mut();
        let weapon = inner.world.spawn();
        inner
            .world
            .set_attack(weapon, orange_stone::core::component::Attack(3));
        inner
            .world
            .set_durability(weapon, orange_stone::core::component::Durability(2));
        inner.world.set_card_type(weapon, CardType::Weapon);
        inner.world.set_player(weapon, PlayerId::Player2);
        inner.world.set_zone(weapon, Zone::Play);
        inner
            .world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player2, weapon);
        inner.players[PlayerId::Player2.index()].weapon = Some(weapon);
    }

    let attacker = state.player(PlayerId::Player2).hero;
    let defender = state.player(PlayerId::Player1).hero;

    let log = engine
        .apply(&mut state, Action::Attack { attacker, defender })
        .unwrap();

    // 两个奥秘都应被触发
    let secret_count = log
        .iter()
        .filter(|e| matches!(e, Event::SecretRevealed { .. }))
        .count();
    assert_eq!(secret_count, 2, "Both secrets should trigger");
}

// ============================================================
// Phase 4: Hero Attack 测试 (Heroic Strike, Claw, Bite)
// ============================================================

#[test]
fn heroic_strike_gives_hero_attack_this_turn() {
    use orange_stone::cards::def::HEROIC_STRIKE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &HEROIC_STRIKE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hero = state.player(PlayerId::Player1).hero;

    // 英雄初始攻击力应为 0
    assert_eq!(
        state.world().attack(hero),
        Some(orange_stone::core::component::Attack(0))
    );

    // 找到手牌中的 Heroic Strike
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    // 打出 Heroic Strike
    let log = engine
        .apply(&mut state, Action::PlayCard { card })
        .unwrap();

    // 检查事件
    assert!(log.iter().any(|e| matches!(e, Event::CardPlayed { .. })));

    // 英雄攻击力应为 4
    assert_eq!(
        state.world().attack(hero),
        Some(orange_stone::core::component::Attack(4))
    );
    assert_eq!(state.player(PlayerId::Player1).temp_attack_bonus, 4);

    // 法术牌应进入坟场
    assert_eq!(state.world().zone(card), Some(Zone::Graveyard));

    // 结束回合 → 英雄攻击力应恢复为 0
    engine.apply(&mut state, Action::EndTurn).unwrap();

    assert_eq!(
        state.world().attack(hero),
        Some(orange_stone::core::component::Attack(0))
    );
    assert_eq!(state.player(PlayerId::Player1).temp_attack_bonus, 0);
}

#[test]
fn hero_with_temp_attack_can_attack() {
    use orange_stone::cards::def::HEROIC_STRIKE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &HEROIC_STRIKE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hero = state.player(PlayerId::Player1).hero;
    let enemy_hero = state.player(PlayerId::Player2).hero;

    // 打 Heroic Strike
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine
        .apply(&mut state, Action::PlayCard { card })
        .unwrap();

    // 现在英雄有 4 攻击力，可以攻击（无需武器）
    let _log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: enemy_hero,
            },
        )
        .unwrap();

    // 敌方英雄应受到 4 点伤害
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(26))
    );
}
