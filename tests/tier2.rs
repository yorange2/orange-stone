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

// ============================================================
// Stage 2 盗贼批次
// ============================================================

#[test]
fn headcrack_no_combo_goes_to_graveyard() {
    use orange_stone::cards::def::HEADCRACK;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &HEADCRACK);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 非连击：造成 2 点伤害，进入坟墓场
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(28))
    );
    assert_eq!(state.world().zone(card), Some(Zone::Graveyard));
}

#[test]
fn headcrack_combo_returns_to_hand() {
    use orange_stone::cards::def::{HEADCRACK, WISP};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &WISP);
    builder.add_minion_to_hand(PlayerId::Player1, &HEADCRACK);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let wisp = hand[0];
    let headcrack = hand[1];

    // 先出一张牌激活连击
    engine
        .apply(&mut state, Action::PlayCard { card: wisp })
        .unwrap();
    engine
        .apply(&mut state, Action::PlayCard { card: headcrack })
        .unwrap();

    // 连击：造成 2 点伤害，牌回到手牌
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(28))
    );
    assert_eq!(state.world().zone(headcrack), Some(Zone::Hand));
}

#[test]
fn kidnapper_combo_returns_enemy_minion() {
    use orange_stone::cards::def::{KIDNAPPER, WISP};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &WISP);
    builder.add_minion_to_hand(PlayerId::Player1, &KIDNAPPER);
    let enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 2, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let wisp = hand[0];
    let kidnapper = hand[1];

    engine
        .apply(&mut state, Action::PlayCard { card: wisp })
        .unwrap();
    engine
        .apply(&mut state, Action::PlayCard { card: kidnapper })
        .unwrap();

    // 连击：敌方随从回到其拥有者的手牌
    assert_eq!(state.world().zone(enemy), Some(Zone::Hand));
}

#[test]
fn shadowstep_returns_friendly_and_reduces_cost() {
    use orange_stone::cards::def::SHADOWSTEP;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SHADOWSTEP);
    let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 友方随从回到手牌，费用 3 → 1
    assert_eq!(state.world().zone(minion), Some(Zone::Hand));
    assert_eq!(
        state.world().cost(minion),
        Some(orange_stone::core::component::Cost(1))
    );
}

#[test]
fn betrayal_damages_adjacent_minions() {
    use orange_stone::cards::def::BETRAYAL;
    use orange_stone::core::component::Stealth;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BETRAYAL);
    // 两侧随从潜行（不可被单目标指定），中间的 5/5 是唯一目标
    let left = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
    let middle = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 5, 5);
    let right = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 2, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();
    // 测试辅助：手动给两侧随从挂上潜行
    state.world_mut().set_stealth(left, Stealth);
    state.world_mut().set_stealth(right, Stealth);

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 中间的 5/5 被选中：两侧各受 5 点伤害
    assert_eq!(
        state.world().health(left),
        Some(orange_stone::core::component::Health(-4))
    );
    assert_eq!(
        state.world().health(right),
        Some(orange_stone::core::component::Health(-3))
    );
    // 中间的目标不受伤害
    assert_eq!(
        state.world().health(middle),
        Some(orange_stone::core::component::Health(5))
    );
}

#[test]
fn blade_flurry_destroys_weapon_and_damages_all_enemies() {
    use orange_stone::cards::def::{BLADE_FLURRY, FIERY_WAR_AXE};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BLADE_FLURRY);
    builder.equip_weapon(PlayerId::Player1, &FIERY_WAR_AXE);
    let enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 武器被摧毁
    assert!(state.player(PlayerId::Player1).weapon.is_none());
    // 所有敌人（英雄 + 随从）受到武器攻击力 (3) 点伤害
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(27))
    );
    assert_eq!(
        state.world().health(enemy),
        Some(orange_stone::core::component::Health(2))
    );
}

#[test]
fn patient_assassin_poison_destroys_minion() {
    use orange_stone::cards::def::PATIENT_ASSASSIN;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &PATIENT_ASSASSIN);
    let enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 3, 3);
    let mut state = builder.build();

    let assassin: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .find(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .unwrap();

    // 潜行组件已通过 apply_card_keywords 挂载
    assert!(state.world().stealth(assassin).is_some());
    assert!(state.world().poison(assassin).is_some());

    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: assassin,
                defender: enemy,
            },
        )
        .unwrap();

    // 剧毒：3/3 随从被 1 点伤害直接消灭
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
    // 刺客受到 3 点反击伤害死亡
    assert_eq!(state.world().zone(assassin), Some(Zone::Graveyard));
}

#[test]
fn stealth_minion_cannot_be_attacked() {
    use orange_stone::cards::def::PATIENT_ASSASSIN;
    use orange_stone::engine::rules::EngineError;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.active_player(PlayerId::Player2);
    builder.add_minion_to_board(PlayerId::Player1, &PATIENT_ASSASSIN);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 3, 3);
    let mut state = builder.build();

    let assassin: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .find(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .unwrap();

    let result = engine.apply(
        &mut state,
        Action::Attack {
            attacker,
            defender: assassin,
        },
    );
    assert_eq!(result, Err(EngineError::InvalidTarget));
}

#[test]
fn perdition_blade_battlecry_on_play() {
    use orange_stone::cards::def::PERDITIONS_BLADE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &PERDITIONS_BLADE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 武器已装备（2/2）
    assert_eq!(state.player(PlayerId::Player1).weapon, Some(card));
    assert_eq!(
        state.world().attack(card),
        Some(orange_stone::core::component::Attack(2))
    );
    assert_eq!(
        state.world().durability(card),
        Some(orange_stone::core::component::Durability(2))
    );
    // 战吼造成 1 点伤害
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(29))
    );
}

#[test]
fn master_of_disguise_grants_stealth() {
    use orange_stone::cards::def::MASTER_OF_DISGUISE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &MASTER_OF_DISGUISE);
    let ally = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 3, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 友方随从获得潜行；伪装大师自身不获得（不能指定自己）
    assert!(state.world().stealth(ally).is_some());
    assert!(state.world().stealth(card).is_none());
}

// ============================================================
// Stage 2 回归：复活不超战场上限
// ============================================================

#[test]
fn resurrect_skipped_when_board_full() {
    use orange_stone::cards::def::HIGH_INQUISITOR_WHITEMANE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    // 6 个随从占场
    for _ in 0..6 {
        builder.add_custom_minion_to_board(PlayerId::Player1, 1, 1, 1);
    }
    // 一个死亡待复活的随从
    let dead = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 2, 2);
    builder.add_minion_to_hand(PlayerId::Player1, &HIGH_INQUISITOR_WHITEMANE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();
    // 让 dead 死亡并记录到本回合死亡列表
    state
        .world_mut()
        .move_to_zone(dead, Zone::Graveyard)
        .unwrap();
    state
        .make_mut()
        .players[PlayerId::Player1.index()]
        .died_this_turn
        .push(dead);

    // 打出怀特迈恩（第 7 个随从），战吼试图复活
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 战场仍是 7 个随从（复活被跳过），尸体留在坟墓场
    let count = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state
                .world()
                .card_type(e)
                == Some(orange_stone::core::component::CardType::Minion)
        })
        .count();
    assert_eq!(count, 7);
    assert_eq!(state.world().zone(dead), Some(Zone::Graveyard));
}
