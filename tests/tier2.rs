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
