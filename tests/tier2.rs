//! Tier 2 集成测试 — 怀旧系列新机制卡牌。
//!
//! 按机制分批组织：
//! - 基础框架：奥秘卡牌可打出、带战吼的武器
//! - 盗贼：连击/暗影步/背叛/剑刃乱舞/剧毒/潜行
//! - 奥秘重定向：误导/崇高牺牲/法术反制
//! - 费用减免、免疫、精神控制、变形等

use orange_stone::cards::def::{EXPLOSIVE_TRAP, FREEZING_TRAP, card_by_id};
use orange_stone::core::action::Action;
use orange_stone::core::component::CardType;
use orange_stone::core::entity::Entity;
use orange_stone::core::event::Event;
use orange_stone::core::player::PlayerId;
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::game::GameBuilder;

// ============================================================
// Stage 1 基础框架
// ============================================================

#[test]
fn playing_secret_card_moves_to_setaside_with_secret_component() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &EXPLOSIVE_TRAP);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 奥秘卡牌应进入 SetAside 区域，而不是坟墓场
    assert_eq!(state.world().zone(card), Some(Zone::SetAside));
    // 应挂载 Secret 组件
    let secret = state.world().secret(card);
    assert!(secret.is_some(), "secret card should have Secret component");
    let secret = secret.unwrap();
    assert_eq!(
        secret.trigger,
        orange_stone::core::component::SecretTrigger::WhenEnemyMinionAttacksHero
    );
    // 效果来自 battlecry 槽位（2 点伤害，所有敌人）
    assert!(matches!(
        secret.effect,
        orange_stone::core::effect::CardEffect::DealDamage { amount: 2, .. }
    ));
}

#[test]
fn played_secret_triggers_when_condition_met() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &EXPLOSIVE_TRAP);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    let mut state = builder.build();

    // Player1 打出爆炸陷阱
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();
    assert_eq!(state.world().zone(card), Some(Zone::SetAside));

    // 轮到 Player2 攻击
    state.set_active_player(PlayerId::Player2);

    // 敌方随从攻击 Player1 的英雄 → 奥秘触发
    let hero = state.player(PlayerId::Player1).hero;
    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. })),
        "secret should be revealed"
    );
    // 攻击随从受到 2 点伤害
    assert_eq!(
        state.world().health(attacker),
        Some(orange_stone::core::component::Health(1))
    );
    // 敌方英雄也受到 2 点伤害（所有敌人）
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(28))
    );
}

#[test]
fn freezing_trap_playable_and_triggers() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &FREEZING_TRAP);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 轮到 Player2 攻击
    state.set_active_player(PlayerId::Player2);
    let hero = state.player(PlayerId::Player1).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    // 攻击随从回到手牌，费用 +2
    assert_eq!(state.world().zone(attacker), Some(Zone::Hand));
    assert_eq!(
        state.world().cost(attacker),
        Some(orange_stone::core::component::Cost(5))
    );
}

/// 带战吼的武器：打出时先装备，再解析战吼。
#[test]
fn weapon_with_battlecry_resolves_on_play() {
    use orange_stone::cards::def::CardDef;
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    // 测试用武器：2/2 战吼造成 1 点伤害（毁灭之刃的原型）
    let weapon_def = CardDef {
        id: "TEST_W1",
        name: "Test Dagger",
        card_type: CardType::Weapon,
        cost: 2,
        attack: 2,
        health: 0,
        durability: 2,
        battlecry: Some(CardEffect::DealDamage {
            amount: 1,
            target: EffectTarget::AnyEnemy,
        }),
        deathrattle: None,
        taunt: false,
        hero_power: None,
        aura: None,
        secret: None,
        divine_shield: false,
        windfury: false,
        charge: false,
        spell_damage: 0,
        cant_attack: false,
        spell_effect: None,
        end_turn_effect: None,
        start_turn_effect: None,
        spell_trigger: None,
        death_trigger: None,
        summon_trigger: None,
        choose_one_effect: None,
        combo_effect: None,
        attack_equals_health: false,
    };

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &weapon_def);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 武器已装备
    assert_eq!(state.player(PlayerId::Player1).weapon, Some(card));
    // 战吼造成 1 点伤害 → 敌方英雄 29 HP
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(29))
    );
}

#[test]
fn card_id_registry_lookup() {
    // 所有注册的卡牌都能通过 id 反查
    assert!(card_by_id("HUNTER_T01").is_some());
    assert!(card_by_id("HUNTER_T02").is_some());
}
