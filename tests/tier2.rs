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
    state.make_mut().players[PlayerId::Player1.index()]
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
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .count();
    assert_eq!(count, 7);
    assert_eq!(state.world().zone(dead), Some(Zone::Graveyard));
}

// ============================================================
// Stage 3 奥秘批次
// ============================================================

#[test]
fn misdirection_redirects_attack_away_from_hero() {
    use orange_stone::cards::def::MISDIRECTION;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &MISDIRECTION);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    let other_enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
    let own_minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
    let mut state = builder.build();

    // Player1 打出误导
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // Player2 攻击 Player1 英雄
    state.set_active_player(PlayerId::Player2);
    let hero = state.player(PlayerId::Player1).hero;
    let enemy_hero = state.player(PlayerId::Player2).hero;
    // 记录攻击前各候选角色的生命值
    let before: Vec<i32> = [enemy_hero, other_enemy, own_minion]
        .iter()
        .map(|&e| state.world().health(e).unwrap().0)
        .collect();
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
        "misdirection should be revealed"
    );
    // 己方英雄不受伤害
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(30))
    );
    // 2 点伤害落到了另一个角色身上：敌方英雄 / 其他敌方随从 / 己方随从
    let damage_spread: i32 = [enemy_hero, other_enemy, own_minion]
        .iter()
        .zip(before.iter())
        .map(|(&e, &b)| (b - state.world().health(e).unwrap().0).max(0))
        .sum();
    assert_eq!(
        damage_spread, 2,
        "attack should hit exactly one other character for 2"
    );
}

#[test]
fn noble_sacrifice_summons_defender_as_new_target() {
    use orange_stone::cards::def::NOBLE_SACRIFICE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &NOBLE_SACRIFICE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 3, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // Player2 攻击 Player1 英雄 → 崇高牺牲召唤 2/1 防御者
    state.set_active_player(PlayerId::Player2);
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
        "noble sacrifice should be revealed"
    );
    // 己方英雄不受伤害
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(30))
    );
    // 防御者被召唤并承受了 3 点伤害（2/1 → 死亡）
    let defenders: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Graveyard, PlayerId::Player1)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "PALADIN_022t")
        })
        .collect();
    assert_eq!(
        defenders.len(),
        1,
        "defender should be summoned and die to the attack"
    );
    // 攻击者受到防御者的 2 点反击
    assert_eq!(
        state.world().health(attacker),
        Some(orange_stone::core::component::Health(1))
    );
}

#[test]
fn snipe_damages_played_minion() {
    use orange_stone::cards::def::SNIPE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SNIPE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    let mut state = builder.build();

    // Player1 打出狙击
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // Player2 打出一个 4/4 随从
    state.set_active_player(PlayerId::Player2);
    let played = builder_add_custom_hand(&mut state, PlayerId::Player2, 4, 4, 4);
    let log = engine
        .apply(&mut state, Action::PlayCard { card: played })
        .unwrap();

    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. })),
        "snipe should be revealed"
    );
    // 随从被 4 点伤害击杀
    assert_eq!(state.world().zone(played), Some(Zone::Graveyard));
}

/// 辅助：直接在已构建的状态中添加自定义随从到手牌（用于跨回合测试）。
fn builder_add_custom_hand(
    state: &mut orange_stone::core::state::GameState,
    player: PlayerId,
    atk: i32,
    hp: i32,
    cost: i32,
) -> Entity {
    let world = state.world_mut();
    let e = world.spawn();
    world.set_health(e, orange_stone::core::component::Health(hp));
    world.set_attack(e, orange_stone::core::component::Attack(atk));
    world.set_cost(e, orange_stone::core::component::Cost(cost));
    world.set_card_type(e, CardType::Minion);
    world.set_player(e, player);
    world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
    world.set_zone(e, Zone::Hand);
    world.zones_mut().insert(Zone::Hand, player, e);
    e
}

#[test]
fn snake_trap_summons_three_snakes() {
    use orange_stone::cards::def::SNAKE_TRAP;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SNAKE_TRAP);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let friendly = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 2, 2);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    let mut state = builder.build();

    // Player1 打出毒蛇陷阱
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // Player2 攻击 Player1 的随从 → 随从受伤，召唤三条蛇
    state.set_active_player(PlayerId::Player2);
    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: friendly,
            },
        )
        .unwrap();

    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. })),
        "snake trap should be revealed"
    );
    // Player1 场上应有 3 条 1/1 蛇 + 受伤的友方随从
    let snakes: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "HUNTER_019t")
        })
        .collect();
    assert_eq!(snakes.len(), 3, "three snakes should be summoned");
    for s in &snakes {
        assert_eq!(
            state.world().attack(*s),
            Some(orange_stone::core::component::Attack(1))
        );
        assert_eq!(
            state.world().health(*s),
            Some(orange_stone::core::component::Health(1))
        );
    }
    // 友方随从受到 2 点伤害
    assert_eq!(
        state.world().health(friendly),
        Some(orange_stone::core::component::Health(0))
    );
}

#[test]
fn spellbender_redirects_spell_damage_to_itself() {
    use orange_stone::cards::def::CardDef;
    use orange_stone::cards::def::SPELLBENDER;
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    // 测试用法术：对一个敌方随从造成 5 点伤害（目标必为场上唯一的己方随从）
    let test_spell = CardDef {
        id: "TEST_S1",
        name: "Test Bolt",
        card_type: CardType::Spell,
        cost: 1,
        attack: 0,
        health: 0,
        durability: 0,
        battlecry: Some(CardEffect::DealDamage {
            amount: 5,
            target: EffectTarget::AnyEnemyMinion,
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
    builder.add_minion_to_hand(PlayerId::Player1, &SPELLBENDER);
    builder.set_mana(PlayerId::Player1, 10, 10);
    builder.set_mana(PlayerId::Player2, 10, 10);
    let own_minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
    let mut state = builder.build();

    // Player1 打出法术扭曲者
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // Player2 施放法术（目标为 Player1 的随从）
    state.set_active_player(PlayerId::Player2);
    let spell = builder_add_custom_hand(&mut state, PlayerId::Player2, 0, 0, 1);
    {
        // 测试法术没有在注册表中，手动设置其类型和效果组件
        let world = state.world_mut();
        world.set_card_type(spell, CardType::Spell);
        world.set_battlecry(
            spell,
            orange_stone::core::component::Battlecry(test_spell.battlecry.unwrap()),
        );
    }
    let log = engine
        .apply(&mut state, Action::PlayCard { card: spell })
        .unwrap();

    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. })),
        "spellbender should be revealed"
    );
    // 1/3 法术扭曲者被召唤并承受了 5 点伤害
    let spellbenders: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Graveyard, PlayerId::Player1)
        .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "MAGE_019t"))
        .collect();
    assert_eq!(
        spellbenders.len(),
        1,
        "spellbender should be summoned and die"
    );
    // 原目标随从不受伤害
    assert_eq!(
        state.world().health(own_minion),
        Some(orange_stone::core::component::Health(3))
    );
}

// ============================================================
// Stage 4 费用减免批次
// ============================================================

#[test]
fn sorcerers_apprentice_reduces_spell_cost() {
    use orange_stone::cards::def::{FIREBALL, SORCERERS_APPRENTICE};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &SORCERERS_APPRENTICE);
    builder.add_minion_to_hand(PlayerId::Player1, &FIREBALL);
    // 火球术 4 费，学徒减免 1 → 3 费；只给 3 法力
    builder.set_mana(PlayerId::Player1, 3, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    // 有效费用为 3
    assert_eq!(
        state.world().effective_cost(card),
        Some(orange_stone::core::component::Cost(3))
    );

    // 3 法力即可打出
    let log = engine.apply(&mut state, Action::PlayCard { card }).unwrap();
    assert!(
        log.iter().any(|e| matches!(e, Event::SpellCast { .. })),
        "fireball should be cast with discounted cost"
    );
    // 法力扣减 3（不是 4）
    assert_eq!(state.player(PlayerId::Player1).current_mana, 0);
}

#[test]
fn summoning_portal_reduces_minion_cost_min_one() {
    use orange_stone::cards::def::{BOULDERFIST_OGRE, SUMMONING_PORTAL, WISP};

    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &SUMMONING_PORTAL);
    builder.add_minion_to_hand(PlayerId::Player1, &BOULDERFIST_OGRE);
    builder.add_minion_to_hand(PlayerId::Player1, &WISP);
    let state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let ogre = hand[0];
    let wisp = hand[1];

    // 6 费食人魔 → 4 费；0 费小精灵 → 至少 1 费
    assert_eq!(
        state.world().effective_cost(ogre),
        Some(orange_stone::core::component::Cost(4))
    );
    assert_eq!(
        state.world().effective_cost(wisp),
        Some(orange_stone::core::component::Cost(1))
    );
}

#[test]
fn kirin_tor_mage_makes_next_secret_free() {
    use orange_stone::cards::def::{EXPLOSIVE_TRAP, KIRIN_TOR_MAGE};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &KIRIN_TOR_MAGE);
    builder.add_minion_to_hand(PlayerId::Player1, &EXPLOSIVE_TRAP);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let ktm = hand[0];
    let trap = hand[1];

    // 打出肯瑞托法师（3 费）
    engine
        .apply(&mut state, Action::PlayCard { card: ktm })
        .unwrap();
    assert_eq!(state.player(PlayerId::Player1).current_mana, 7);
    assert!(state.player(PlayerId::Player1).next_secret_free);

    // 下一个奥秘免费
    engine
        .apply(&mut state, Action::PlayCard { card: trap })
        .unwrap();
    // 法力未扣减
    assert_eq!(state.player(PlayerId::Player1).current_mana, 7);
    // 一次性效果已消耗
    assert!(!state.player(PlayerId::Player1).next_secret_free);
    // 奥秘已挂载到 SetAside
    assert_eq!(state.world().zone(trap), Some(Zone::SetAside));
}

#[test]
fn far_sight_draws_card_with_reduced_cost() {
    use orange_stone::cards::def::{FAR_SIGHT, OGRE_MAGI};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &FAR_SIGHT);
    builder.add_minion_to_deck(PlayerId::Player1, &OGRE_MAGI);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    let log = engine.apply(&mut state, Action::PlayCard { card }).unwrap();
    assert!(log.iter().any(|e| matches!(e, Event::CardDrawn { .. })));

    // 抽到的奥术傀儡（4 费）费用减少 3 → 1
    let drawn: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(drawn.len(), 1);
    assert_eq!(
        state.world().cost(drawn[0]),
        Some(orange_stone::core::component::Cost(1))
    );
}

// ============================================================
// Stage 5 德鲁伊批次
// ============================================================

#[test]
fn cenarius_choose_one_buffs_or_summons_treants() {
    use orange_stone::cards::def::{CENARIUS, CENARIUS_TREANT};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &CENARIUS);
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

    // 抉择随机：要么所有随从 +2/+2，要么召唤两个 2/2 树人
    let treants: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == CENARIUS_TREANT.id)
        })
        .collect();
    if !treants.is_empty() {
        assert_eq!(treants.len(), 2, "buff branch: two treants expected");
        for t in &treants {
            assert_eq!(
                state.world().attack(*t),
                Some(orange_stone::core::component::Attack(2))
            );
            assert_eq!(
                state.world().health(*t),
                Some(orange_stone::core::component::Health(2))
            );
        }
    } else {
        // +2/+2 分支：友方随从 2/3 → 4/5
        assert_eq!(
            state.world().attack(ally),
            Some(orange_stone::core::component::Attack(4))
        );
        assert_eq!(
            state.world().health(ally),
            Some(orange_stone::core::component::Health(5))
        );
    }
}

#[test]
fn keeper_of_the_grove_choose_one_damage_or_silence() {
    use orange_stone::cards::def::{GOLDSHIRE_FOOTMAN, KEEPER_OF_THE_GROVE};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &KEEPER_OF_THE_GROVE);
    builder.add_minion_to_board(PlayerId::Player2, &GOLDSHIRE_FOOTMAN);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    // 找到嘲讽随从（闪金镇步兵）
    let footman: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player2)
        .find(|&e| state.world().taunt(e).is_some())
        .expect("precondition: goldshire footman has taunt");

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 抉择随机：2 点伤害（打在嘲讽随从或敌方英雄）或沉默嘲讽随从
    let enemy_hero = state.player(PlayerId::Player2).hero;
    let footman_hp = state.world().health(footman).unwrap().0;
    let hero_hp = state.world().health(enemy_hero).unwrap().0;
    if state.world().taunt(footman).is_none() {
        // 沉默分支：嘲讽被移除
    } else {
        // 伤害分支：嘲讽随从（1/2，受 2 伤死亡）或英雄受到 2 点伤害
        assert!(
            footman_hp <= 0 || hero_hp == 28,
            "damage branch should hit the footman (1/2) or the enemy hero, got footman={footman_hp} hero={hero_hp}"
        );
    }
}

#[test]
fn soul_of_the_forest_grants_deathrattle_summoning_treant() {
    use orange_stone::cards::def::{CENARIUS_TREANT, SOUL_OF_THE_FOREST};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SOUL_OF_THE_FOREST);
    let ally = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 2, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 友方随从获得亡语：召唤 2/2 树人
    let dr = state.world().deathrattle(ally);
    assert!(
        matches!(
            dr,
            Some(orange_stone::core::component::Deathrattle(
                orange_stone::core::effect::CardEffect::SummonMinion {
                    card_id: "DRUID_023t"
                }
            ))
        ),
        "ally should have deathrattle summoning a treant"
    );

    // 杀掉友方随从 → 亡语召唤树人
    let attacker = builder_add_custom_hand(&mut state, PlayerId::Player1, 0, 0, 0);
    // 直接用伤害事件杀死：通过攻击
    state.set_active_player(PlayerId::Player2);
    let enemy_attacker = builder_add_custom_hand(&mut state, PlayerId::Player2, 5, 5, 5);
    {
        let world = state.world_mut();
        world.set_zone(enemy_attacker, Zone::Play);
        world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player2, enemy_attacker);
    }
    let _ = attacker;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: enemy_attacker,
                defender: ally,
            },
        )
        .unwrap();

    assert_eq!(state.world().zone(ally), Some(Zone::Graveyard));
    // 亡语召唤了 2/2 树人
    let treants: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == CENARIUS_TREANT.id)
        })
        .collect();
    assert_eq!(treants.len(), 1, "deathrattle should summon one treant");
}

#[test]
fn king_mukla_gives_opponent_two_bananas() {
    use orange_stone::cards::def::KING_MUKLA;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &KING_MUKLA);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 对手手牌中有 2 张香蕉
    let bananas: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player2)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "NEUTRAL_T16t")
        })
        .collect();
    assert_eq!(bananas.len(), 2, "opponent should receive two bananas");
}

// ============================================================
// Stage 6 免疫批次
// ============================================================

#[test]
fn bestial_wrath_grants_attack_and_immune_until_turn_end() {
    use orange_stone::cards::def::{BESTIAL_WRATH, WISP};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BESTIAL_WRATH);
    // 场上唯一随从（小精灵）→ 必然成为目标
    builder.add_minion_to_board(PlayerId::Player1, &WISP);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 3, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 获得 +2 攻击力和免疫
    let beast: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .find(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .unwrap();
    assert_eq!(
        state.world().attack(beast),
        Some(orange_stone::core::component::Attack(3))
    );
    assert!(state.world().immune(beast).is_some());

    // 敌方攻击它：免疫忽略伤害（攻击者不反击它，它也不死）
    state.set_active_player(PlayerId::Player2);
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: beast,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().health(beast),
        Some(orange_stone::core::component::Health(1)),
        "immune minion should not take damage"
    );

    // 回合结束 → 免疫清除
    state.set_active_player(PlayerId::Player1);
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert!(
        state.world().immune(beast).is_none(),
        "immune should expire at end of turn"
    );
}

#[test]
fn gladiators_longbow_hero_immune_while_attacking() {
    use orange_stone::cards::def::GLADIATORS_LONGBOW;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId::Player1, &GLADIATORS_LONGBOW);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 4, 5, 4);
    let mut state = builder.build();

    let hero = state.player(PlayerId::Player1).hero;

    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender,
            },
        )
        .unwrap();

    // 英雄受到 0 伤害（免疫），防御者受到 5 点伤害
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(30))
    );
    assert_eq!(
        state.world().health(defender),
        Some(orange_stone::core::component::Health(0))
    );
}

#[test]
fn icicle_freezes_unfrozen_minion() {
    use orange_stone::cards::def::ICICLE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &ICICLE);
    let enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 敌方唯一随从被冻结（未受伤）
    assert!(state.world().freeze(enemy).is_some());
    assert_eq!(
        state.world().health(enemy),
        Some(orange_stone::core::component::Health(3))
    );
}

#[test]
fn icicle_damages_already_frozen_minion() {
    use orange_stone::cards::def::ICICLE;
    use orange_stone::core::component::Freeze;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &ICICLE);
    let enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();
    // 预冻结
    state.world_mut().set_freeze(enemy, Freeze);

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 已冻结 → 造成 2 点伤害
    assert_eq!(
        state.world().health(enemy),
        Some(orange_stone::core::component::Health(1))
    );
}

#[test]
fn natalie_seline_destroys_minion_and_gains_health() {
    use orange_stone::cards::def::NATALIE_SELINE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &NATALIE_SELINE);
    builder.add_custom_minion_to_board(PlayerId::Player2, 1, 6, 1);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 敌方 6 HP 随从被消灭，娜塔莉获得 6 点生命值（4/5 → 4/11）
    let enemy_dead = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player2)
        .filter(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .count();
    assert_eq!(enemy_dead, 0, "enemy minion should be destroyed");
    assert_eq!(
        state.world().health(card),
        Some(orange_stone::core::component::Health(11))
    );
}

// ============================================================
// Stage 7 控制/腐蚀/命令怒吼/过载/变形批次
// ============================================================

#[test]
fn shadow_madness_takes_control_until_end_of_turn() {
    use orange_stone::cards::def::SHADOW_MADNESS;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SHADOW_MADNESS);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 敌方随从被控制（属于 Player1）
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player1));
    assert!(
        state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .any(|e| e == enemy_minion)
    );

    // 回合结束 → 归还
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player2));
}

#[test]
fn shadow_madness_ignores_high_attack_minions() {
    use orange_stone::cards::def::SHADOW_MADNESS;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SHADOW_MADNESS);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 5, 5);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 5 攻击随从不受暗影狂乱影响
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player2));
}

#[test]
fn mind_control_permanently_steals_minion() {
    use orange_stone::cards::def::MIND_CONTROL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &MIND_CONTROL);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 5, 5);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player1));

    // 回合结束后仍属于 Player1
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player1));
}

#[test]
fn corruption_destroys_minion_at_start_of_turn() {
    use orange_stone::cards::def::CORRUPTION;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &CORRUPTION);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();
    assert_eq!(state.world().zone(enemy_minion), Some(Zone::Play));

    // P1 结束 → P2 回合 → P2 结束 → P1 回合开始时被腐蚀的随从死亡
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zone(enemy_minion), Some(Zone::Graveyard));
}

#[test]
fn commanding_shout_prevents_minion_death() {
    use orange_stone::cards::def::COMMANDING_SHOUT;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &COMMANDING_SHOUT);
    let ally = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 2, 1);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 3, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();
    assert_eq!(state.player(PlayerId::Player1).minion_min_health, 1);

    // 敌方 3 攻随从攻击 1/2 随从 → 生命值钳制在 1（不死）
    state.set_active_player(PlayerId::Player2);
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: ally,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().health(ally),
        Some(orange_stone::core::component::Health(1))
    );
    assert_eq!(state.world().zone(ally), Some(Zone::Play));

    // 回合结束后效果清除
    state.set_active_player(PlayerId::Player1);
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.player(PlayerId::Player1).minion_min_health, 0);
}

#[test]
fn unbound_elemental_gains_stats_when_overload_played() {
    use orange_stone::cards::def::{LIGHTNING_BOLT, UNBOUND_ELEMENTAL};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &LIGHTNING_BOLT);
    builder.add_minion_to_board(PlayerId::Player1, &UNBOUND_ELEMENTAL);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    // 打出带过载的闪电箭
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 无羁元素获得 +1/+1（2/4 → 3/5）
    let elemental: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == UNBOUND_ELEMENTAL.id)
        })
        .unwrap();
    assert_eq!(
        state.world().attack(elemental),
        Some(orange_stone::core::component::Attack(3))
    );
    assert_eq!(
        state.world().health(elemental),
        Some(orange_stone::core::component::Health(5))
    );
}

#[test]
fn tinkmaster_transforms_enemy_minion() {
    use orange_stone::cards::def::TINKMASTER_OVERSPARK;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &TINKMASTER_OVERSPARK);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 4, 4, 4);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 敌方随从被变形为 5/5 暴龙或 1/1 松鼠
    let atk = state.world().attack(enemy_minion).unwrap().0;
    let hp = state.world().health(enemy_minion).unwrap().0;
    assert!(
        (atk, hp) == (5, 5) || (atk, hp) == (1, 1),
        "transformed stats should be 5/5 or 1/1, got {atk}/{hp}"
    );
    assert_eq!(
        state.world().card_id(enemy_minion).unwrap().0,
        if (atk, hp) == (5, 5) {
            "NEUTRAL_T17a"
        } else {
            "NEUTRAL_T17b"
        }
    );
    // 效果组件被清除（无战吼/嘲讽等）
    assert!(state.world().battlecry(enemy_minion).is_none());
}

// ============================================================
// Stage 8 Tier 3 随机卡池批次
// ============================================================

#[test]
fn brightwing_adds_random_legendary_to_hand() {
    use orange_stone::cards::def::BRIGHTWING;
    use orange_stone::cards::sets::LEGENDARY_CLASSIC;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BRIGHTWING);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 手牌中新增一张传说随从
    let gained: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(gained.len(), 1);
    let id = state.world().card_id(gained[0]).unwrap().0;
    assert!(
        LEGENDARY_CLASSIC.iter().any(|l| l.id == id),
        "added card {id} should be a classic legendary"
    );
}

#[test]
fn nozdormu_is_vanilla_8_8() {
    use orange_stone::cards::def::NOZDORMU;

    assert_eq!(NOZDORMU.cost, 9);
    assert_eq!(NOZDORMU.attack, 8);
    assert_eq!(NOZDORMU.health, 8);
    assert!(NOZDORMU.battlecry.is_none());
}

#[test]
fn xavius_end_turn_adds_shadow_spell() {
    use orange_stone::cards::def::XAVIUS;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &XAVIUS);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 回合结束时生成一张暗影法术
    engine.apply(&mut state, Action::EndTurn).unwrap();
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(hand.len(), 1, "one shadow spell should be added");
    let name = state
        .world()
        .card_id(hand[0])
        .and_then(|c| orange_stone::cards::def::card_by_id(c.0))
        .map(|d| d.name)
        .unwrap();
    assert!(
        name.contains("Shadow"),
        "added card should be a shadow spell, got {name}"
    );
}

#[test]
fn ysera_end_turn_adds_dream_card() {
    use orange_stone::cards::def::YSERA;
    use orange_stone::cards::pool::DREAM_POOL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &YSERA);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    engine.apply(&mut state, Action::EndTurn).unwrap();
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(hand.len(), 1, "one dream card should be added");
    let id = state.world().card_id(hand[0]).unwrap().0;
    assert!(
        DREAM_POOL.contains(&id),
        "added card {id} should be a dream card"
    );
}

#[test]
fn barrens_stablehand_summons_random_beast() {
    use orange_stone::cards::def::BARRENS_STABLEHAND;
    use orange_stone::cards::pool::BEAST_POOL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BARRENS_STABLEHAND);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 场上：驯马师 + 一个随机野兽
    let minions: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .collect();
    assert_eq!(minions.len(), 2, "stablehand + one beast expected");
    let beast = minions
        .iter()
        .find(|&&e| e != card)
        .copied()
        .expect("beast should exist");
    let beast_id = state.world().card_id(beast).unwrap().0;
    assert!(
        BEAST_POOL.contains(&beast_id),
        "summoned minion {beast_id} should be a beast"
    );
}

#[test]
fn animal_companion_summons_one_of_three() {
    use orange_stone::cards::def::ANIMAL_COMPANION;
    use orange_stone::cards::pool::COMPANION_POOL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &ANIMAL_COMPANION);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    let companions: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| COMPANION_POOL.contains(&c.0))
        })
        .collect();
    assert_eq!(companions.len(), 1, "exactly one companion expected");
}

#[test]
fn tome_of_intellect_adds_mage_spell() {
    use orange_stone::cards::def::TOME_OF_INTELLECT;
    use orange_stone::cards::sets::MAGE_CLASSIC;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &TOME_OF_INTELLECT);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(hand.len(), 1);
    let id = state.world().card_id(hand[0]).unwrap().0;
    let def = orange_stone::cards::def::card_by_id(id).unwrap();
    assert_eq!(
        def.card_type,
        orange_stone::core::component::CardType::Spell,
        "added card should be a spell"
    );
    assert!(
        MAGE_CLASSIC.iter().any(|m| m.id == id),
        "added card {id} should be a mage spell"
    );
}

#[test]
fn antonidas_adds_fireball_on_spell_cast() {
    use orange_stone::cards::def::{ARCHMAGE_ANTONIDAS, MOONFIRE};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &ARCHMAGE_ANTONIDAS);
    builder.add_minion_to_hand(PlayerId::Player1, &MOONFIRE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let antonidas = hand[0];
    let moonfire = hand[1];
    engine
        .apply(&mut state, Action::PlayCard { card: antonidas })
        .unwrap();

    // 施放月光术 → 火球术入手
    engine
        .apply(&mut state, Action::PlayCard { card: moonfire })
        .unwrap();
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(hand.len(), 1, "one fireball should be added");
    assert_eq!(state.world().card_id(hand[0]).unwrap().0, "MAGE_005");
}

#[test]
fn pilfer_adds_non_rogue_card() {
    use orange_stone::cards::def::PILFER;
    use orange_stone::cards::sets::ROGUE_CLASSIC;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &PILFER);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(hand.len(), 1);
    let id = state.world().card_id(hand[0]).unwrap().0;
    assert!(
        !ROGUE_CLASSIC.iter().any(|r| r.id == id),
        "pilfered card {id} should not be a rogue card"
    );
}

#[test]
fn call_of_the_void_adds_demon() {
    use orange_stone::cards::def::CALL_OF_THE_VOID;
    use orange_stone::cards::pool::DEMON_POOL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &CALL_OF_THE_VOID);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(hand.len(), 1);
    let id = state.world().card_id(hand[0]).unwrap().0;
    assert!(
        DEMON_POOL.contains(&id),
        "added card {id} should be a demon"
    );
}

#[test]
fn bane_of_doom_damages_and_summons_demon_if_killed() {
    use orange_stone::cards::def::BANE_OF_DOOM;
    use orange_stone::cards::pool::DEMON_POOL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BANE_OF_DOOM);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    engine.apply(&mut state, Action::PlayCard { card }).unwrap();

    // 随机目标：1/1 随从（死亡 → 召唤恶魔）或敌方英雄（仅受 2 点伤害）
    let enemy_hero = state.player(PlayerId::Player2).hero;
    let minion_dead = state.world().zone(enemy_minion) == Some(Zone::Graveyard);
    let hero_hp = state.world().health(enemy_hero).unwrap().0;
    if minion_dead {
        // 随从死亡 → 召唤一个随机恶魔（恶魔可能死于自身战吼的简化实现，如烈焰小鬼）
        let demons: Vec<Entity> = [Zone::Play, Zone::Graveyard]
            .iter()
            .flat_map(|&z| state.world().zones().iter(z, PlayerId::Player1))
            .filter(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| DEMON_POOL.contains(&c.0))
            })
            .collect();
        assert_eq!(
            demons.len(),
            1,
            "a demon should be summoned after the kill (may die to its own battlecry)"
        );
    } else {
        // 打到英雄：英雄受 2 点伤害，无召唤
        assert_eq!(hero_hp, 28, "enemy hero should take 2 damage");
        assert_eq!(state.world().zone(enemy_minion), Some(Zone::Play));
    }
}

// ============================================================
// Milestone B — unified attack pipeline verification
// ============================================================

/// 崇高牺牲：攻击重定向后，被召唤的防御者自动反击（统一管线按当前状态计算反击）。
#[test]
fn noble_sacrifice_attacker_takes_defender_retaliation() {
    use orange_stone::cards::def::NOBLE_SACRIFICE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &NOBLE_SACRIFICE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 3, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    engine
        .apply(&mut state, Action::PlayCard { card: hand[0] })
        .unwrap();

    // Player2 攻击 Player1 英雄 → 崇高牺牲召唤 2/1 防御者
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

    // 防御者承受 3 点伤害死亡，同时向攻击者反击 2 点（同时结算）
    assert_eq!(
        state.world().health(hero),
        Some(orange_stone::core::component::Health(30)),
        "hero should take no damage"
    );
    assert_eq!(
        state.world().health(attacker),
        Some(orange_stone::core::component::Health(1)),
        "attacker should take 2 retaliation from the defender"
    );
}

/// 武器在攻击中碎裂：攻击伤害仍包含武器加成（伤害在入队时已确定）。
#[test]
fn attack_with_breaking_weapon_deals_full_damage() {
    use orange_stone::cards::def::GOREHOWL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId::Player1, &GOREHOWL);
    let mut state = builder.build();

    let hero = state.player(PlayerId::Player1).hero;
    let enemy_hero = state.player(PlayerId::Player2).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: enemy_hero,
            },
        )
        .unwrap();

    // 7 点武器伤害全部生效，即使武器因这次攻击碎裂
    assert_eq!(
        state.world().health(enemy_hero),
        Some(orange_stone::core::component::Health(23))
    );
    // 武器被摧毁
    assert!(state.player(PlayerId::Player1).weapon.is_none());
}

/// 防御方随从死亡后仍然反击（炉石的同时结算语义）。
#[test]
fn dead_defender_still_retaliates() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 4, 5, 3);
    let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 2, 2);
    let mut state = builder.build();

    engine
        .apply(&mut state, Action::Attack { attacker, defender })
        .unwrap();

    // 防御者被 4 点伤害击杀（生命值降为负，进入坟墓场），但它的 2 点反击仍然生效
    assert_eq!(
        state.world().zone(defender),
        Some(Zone::Graveyard),
        "defender should be dead"
    );
    assert_eq!(
        state.world().health(attacker),
        Some(orange_stone::core::component::Health(3)),
        "attacker should take 2 retaliation even though defender died"
    );
}
