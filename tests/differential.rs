//! Differential validation harness (roadmap F5) — golden reference scenarios.
//!
//! Each scenario encodes a game state, an action sequence, and the EXACT
//! expected outcome, derived from Hearthstone / SabberStone semantics (the
//! reference implementations). These golden expectations are the machine-
//! checkable half of the differential contract; the other half runs the same
//! scenarios through SabberStone externally and diffs the outcomes — see
//! `tools/differential_sabberstone.md` for the protocol.
//!
//! The scenarios deliberately stress the resolution primitives added by
//! Milestones G1–G9: step ordering, play-order triggers, death batching,
//! enchantment expiry, counter-secret interception, and target re-validation.

use orange_stone::core::action::Action;
use orange_stone::core::component::{Attack, CardType, Cost, Damage, Health};
use orange_stone::core::event::Event;
use orange_stone::core::state::{ChoiceKind, GameState, Step};
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::{GameEngine, Resolution};
use orange_stone::sim::game::GameBuilder;

/// Reference scenario 1 — turn-start order (SabberStone MAIN_START_TRIGGERS →
/// mana refill → draw): the first player does not draw on turn 1, and a
/// start-of-turn effect fires before the turn's draw.
#[test]
fn scenario_turn_start_ordering() {
    use orange_stone::cards::def::BLOODFEN_RAPTOR;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_deck(PlayerId2(), &BLOODFEN_RAPTOR);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // First player's turn 1: no draw
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId1()), 1);
    // Second player's first turn: draws exactly one (turn 2 in the counter)
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId2()), 0);
    assert_eq!(state.world().zones().len(Zone::Hand, PlayerId2()), 1);
    // First player's second turn: draws (the counter is now 3)
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId1()), 0);
    assert_eq!(state.step(), Step::Main);
}

/// Reference scenario 2 — end-of-turn effects resolve before wrap-up
/// (SabberStone MAIN_END_TRIGGERS → MAIN_CLEANUP): an end-of-turn effect that
/// reads the hero's temporary attack sees the full buff.
#[test]
fn scenario_end_of_turn_before_wrap_up() {
    use orange_stone::core::component::{
        Enchantment, EnchantmentExpiry, Trigger, TriggerEvent, TriggerTiming,
    };
    let mut state = GameState::new();
    let hero = state.player(PlayerId1()).hero;
    state.world_mut().add_enchantment(
        hero,
        Enchantment {
            attack: 5,
            health: 0,
            cost: 0,
            expiry: EnchantmentExpiry::UntilEndOfTurn,
        },
    );
    state.world_mut().set_trigger(
        hero,
        Trigger {
            event: TriggerEvent::TurnEnd,
            timing: TriggerTiming::Whenever,
            effect: orange_stone::core::effect::CardEffect::DealHeroAttackDamage {
                target: orange_stone::core::effect::EffectTarget::AnyEnemy,
            },
        },
    );
    let enemy = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(4));
        world.set_attack(e, Attack(1));
        world.set_cost(e, Cost(1));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, PlayerId2());
        world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, PlayerId2(), e);
        e
    };
    let engine = GameEngine::new();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // The end-of-turn effect resolved at full strength (5 damage kills the 4-HP
    // minion) BEFORE the temporary enchantment expired
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
    assert_eq!(
        state.world().effective_attack(hero),
        Some(Attack(0)),
        "the until-end-of-turn enchantment expired at wrap-up"
    );
}

/// Reference scenario 3 — death batching with heal rescue (SabberStone
/// death phase): a minion healed above 0 before its death processes survives.
#[test]
fn scenario_death_batch_heal_rescue() {
    use orange_stone::core::component::Deathrattle;
    use orange_stone::core::effect::{CardEffect, EffectTarget};
    let mut state = GameState::new();
    // B (healer, played first) dies first; its deathrattle heals A
    let b = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(1));
        world.set_attack(e, Attack(0));
        world.set_cost(e, Cost(1));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, PlayerId2());
        world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
        world.set_deathrattle(
            e,
            Deathrattle(CardEffect::RestoreHealth {
                amount: 10,
                target: EffectTarget::AllFriendlyMinions,
            }),
        );
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, PlayerId2(), e);
        e
    };
    let a = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(1));
        world.set_attack(e, Attack(0));
        world.set_cost(e, Cost(1));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, PlayerId2());
        world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, PlayerId2(), e);
        e
    };
    // P1's AOE battlecry deals 1 to all enemy minions
    let aoe = {
        use orange_stone::core::component::Battlecry;
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(2));
        world.set_attack(e, Attack(1));
        world.set_cost(e, Cost(1));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, PlayerId1());
        world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
        world.set_battlecry(
            e,
            Battlecry(CardEffect::DealDamage {
                amount: 1,
                target: EffectTarget::AllEnemyMinions,
            }),
        );
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, PlayerId1(), e);
        e
    };
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: aoe,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // B died first and its deathrattle healed A back above 0 — A survives
    assert_eq!(state.world().zone(b), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(a), Some(Zone::Play));
    assert_eq!(state.world().effective_health(a), Some(Health(1)));
}

/// Reference scenario 4 — counter-secret interception (SabberStone: secrets
/// fire before the spell effect): Counterspell negates the enemy spell.
#[test]
fn scenario_counterspell_intercepts_before_effect() {
    use orange_stone::cards::def::COUNTERSPELL;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &COUNTERSPELL);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let minion = builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 3);
    let mut state = builder.build();
    let counterspell = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("counterspell in hand");
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: counterspell,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // P2 casts a damaging spell — negated before its effect resolves
    state.set_active_player(PlayerId2());
    let spell = {
        use orange_stone::core::component::Battlecry;
        use orange_stone::core::effect::{CardEffect, EffectTarget};
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(1));
        world.set_player(e, PlayerId2());
        world.set_battlecry(
            e,
            Battlecry(CardEffect::DealDamage {
                amount: 5,
                target: EffectTarget::AnyEnemyMinion,
            }),
        );
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, PlayerId2(), e);
        e
    };
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: spell,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(minion),
        Some(Health(3)),
        "the countered spell must not resolve"
    );
}

/// Reference scenario 5 — choose-one surfaces as a choice (SabberStone
/// ChoiceManager): the agent decides the branch via the G6 protocol.
#[test]
fn scenario_choose_one_choice_protocol() {
    use orange_stone::cards::def::CENARIUS;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &CENARIUS);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let cenarius = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("cenarius in hand");
    let engine = GameEngine::new();
    let resolution = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: cenarius,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let Resolution::NeedsChoice { choice } = resolution else {
        panic!("choose-one must surface a choice");
    };
    assert_eq!(choice.kind, ChoiceKind::ChooseOne);
    let resolution = engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 1, // summon two treants
            },
        )
        .unwrap();
    assert!(matches!(resolution, Resolution::Done(_)));
    let minions: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    assert_eq!(minions.len(), 3);
}

/// Reference scenario 6 — overload locks the next turn's mana (SabberStone
/// overload): playing an overload card reduces the owner's next-turn mana.
#[test]
fn scenario_overload_locks_next_turn() {
    use orange_stone::cards::def::LIGHTNING_BOLT;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &LIGHTNING_BOLT);
    let mut state = builder.build();
    let bolt = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("lightning bolt in hand");
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: bolt,
                target: None,
                position: None,
            },
        )
        .unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.player(PlayerId1()).mana_crystals, 2);
    assert_eq!(
        state.player(PlayerId1()).current_mana,
        1,
        "Lightning Bolt's overload (1) locks next-turn mana"
    );
}

/// Reference scenario 7 — attack trade resolves through the unified pipeline:
/// both minions deal damage simultaneously, deaths batch, and the event log
/// records the exact sequence (reference: SabberStone task queue order).
#[test]
fn scenario_attack_trade_event_sequence() {
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId1(), 4, 5, 3);
    let defender = builder.add_custom_minion_to_board(PlayerId2(), 2, 3, 2);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let log = engine
        .apply(&mut state, Action::Attack { attacker, defender })
        .unwrap();
    // Reference order: AttackDeclared → ResolveAttack → DamageDealt(→defender)
    // → MinionDied(defender) → DamageDealt(→attacker)
    let kinds: Vec<&str> = log
        .iter()
        .map(|e| match e {
            Event::AttackDeclared { .. } => "attack_declared",
            Event::ResolveAttack { .. } => "resolve_attack",
            Event::DamageDealt { .. } => "damage_dealt",
            Event::MinionDied { .. } => "minion_died",
            _ => "other",
        })
        .collect();
    assert!(
        kinds
            .windows(2)
            .any(|w| w == ["attack_declared", "resolve_attack"])
    );
    assert!(
        kinds
            .windows(2)
            .any(|w| w == ["resolve_attack", "damage_dealt"])
    );
    assert!(
        kinds
            .windows(2)
            .any(|w| w == ["damage_dealt", "minion_died"])
    );
    assert_eq!(state.world().zone(defender), Some(Zone::Graveyard));
    assert_eq!(state.world().effective_health(attacker), Some(Health(3)));
}

// Small helpers — keep the scenarios readable.
fn PlayerId1() -> orange_stone::core::player::PlayerId {
    orange_stone::core::player::PlayerId::Player1
}

fn PlayerId2() -> orange_stone::core::player::PlayerId {
    orange_stone::core::player::PlayerId::Player2
}
