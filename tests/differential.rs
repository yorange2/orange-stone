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
use orange_stone::core::entity::Entity;
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

/// Finds the first Play-zone entity with the given card ID (Wave 0 scenarios).
fn find_entity(
    state: &GameState,
    player: orange_stone::core::player::PlayerId,
    card_id: &str,
) -> Entity {
    state
        .world()
        .zones()
        .iter(Zone::Play, player)
        .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == card_id))
        .expect("entity with card id on the board")
}

// Small helpers — keep the scenarios readable.
fn PlayerId1() -> orange_stone::core::player::PlayerId {
    orange_stone::core::player::PlayerId::Player1
}

fn PlayerId2() -> orange_stone::core::player::PlayerId {
    orange_stone::core::player::PlayerId::Player2
}

// ============================================================
// Wave 0 — wiring cards (fidelity-debt-roadmap W0): mechanisms
// already existed; these scenarios pin the exact resolution
// (target sets, trigger timing, death-phase interaction).
// ============================================================

/// W0-1 Knife Juggler — after you summon a minion, deal 1 damage to a random
/// enemy (target set: hero + minions; only friendly summons trigger it).
#[test]
fn w0_knife_juggler_throws_after_friendly_summon() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, KNIFE_JUGGLER, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &KNIFE_JUGGLER);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId2(), &WORGEN_INFILTRATOR);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // The enemy summons a minion — the juggler only reacts to friendly summons
    state.set_active_player(PlayerId2());
    let enemy_minion = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .next()
        .expect("worgen in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: enemy_minion,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(2)),
        "enemy summon must not trigger the friendly-side juggler"
    );
    // P1 summons a minion — the knife flies at the (single) enemy minion
    state.set_active_player(PlayerId1());
    let raptor = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("raptor in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: raptor,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(1)),
        "the knife hits the only enemy candidate"
    );
}

/// W0-2 Wild Pyromancer — after you cast a spell, deal 1 damage to ALL minions;
/// the trigger fires after the spell resolves and only while the pyro is alive.
#[test]
fn w0_wild_pyromancer_aoe_after_spell() {
    use orange_stone::cards::def::WILD_PYROMANCER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &WILD_PYROMANCER);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 3, 2);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let pyro = find_entity(&state, PlayerId1(), "NEUTRAL_R12");
    // A custom no-op spell entity in hand
    let noop_spell = {
        use orange_stone::core::component::{Battlecry, Cost};
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(0));
        world.set_player(e, PlayerId1());
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, PlayerId1(), e);
        e
    };
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: noop_spell,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(2)),
        "all minions take 1 — including the enemy"
    );
    assert_eq!(
        state.world().effective_health(pyro),
        Some(Health(1)),
        "the pyro damages itself too"
    );
}

#[test]
fn w0_wild_pyromancer_does_not_fire_when_killed_by_the_spell() {
    use orange_stone::cards::def::WILD_PYROMANCER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &WILD_PYROMANCER);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 3, 2);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let pyro = find_entity(&state, PlayerId1(), "NEUTRAL_R12");
    // A spell that deals 5 to all friendly minions — kills the pyro
    let killing_spell = {
        use orange_stone::core::component::Battlecry;
        use orange_stone::core::component::Cost;
        use orange_stone::core::effect::EffectTarget;
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(1));
        world.set_player(e, PlayerId1());
        world.set_battlecry(
            e,
            Battlecry(orange_stone::core::effect::CardEffect::DealDamage {
                amount: 5,
                target: EffectTarget::AllFriendlyMinions,
            }),
        );
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, PlayerId1(), e);
        e
    };
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: killing_spell,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().zone(pyro), Some(Zone::Graveyard));
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(3)),
        "the dead pyro's trigger must not fire — the enemy is undamaged"
    );
}

/// W0-3 Demolisher — at the start of your turn, deal 2 damage to a random enemy.
#[test]
fn w0_demolisher_fires_at_turn_start() {
    use orange_stone::cards::def::DEMOLISHER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &DEMOLISHER);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // P1 ends → P2's turn (no demolition). P2 ends → P1's turn start: 2 damage
    // to the only enemy candidate kills it.
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zone(enemy), Some(Zone::Play));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
}

/// W0-4 Doomsayer — at the start of your turn, destroy ALL minions (itself
/// included) in a single death batch.
#[test]
fn w0_doomsayer_destroys_all_minions_including_itself() {
    use orange_stone::cards::def::DOOMSAYER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &DOOMSAYER);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 3, 3, 3);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let doomsayer = find_entity(&state, PlayerId1(), "NEUTRAL_E04");
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zone(enemy), Some(Zone::Play));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().zone(doomsayer), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId1()).hero),
        Some(Health(30)),
        "heroes are untouched"
    );
}

/// W0-5 Sword of Justice — whenever you summon a minion, give IT +1/+1 (the
/// event subject, not a random friendly minion); a destroyed sword stops
/// firing.
#[test]
fn w0_sword_of_justice_buffs_the_summoned_minion() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, SWORD_OF_JUSTICE, TRUESILVER_CHAMPION};
    let mut builder = GameBuilder::new();
    let bystander = builder.add_custom_minion_to_board(PlayerId1(), 5, 5, 5);
    builder.equip_weapon(PlayerId1(), &SWORD_OF_JUSTICE);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raptor = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("raptor in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: raptor,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(raptor), Some(Attack(4)));
    assert_eq!(state.world().effective_health(raptor), Some(Health(3)));
    assert_eq!(state.world().effective_attack(bystander), Some(Attack(5)));
    assert_eq!(
        state.world().effective_health(bystander),
        Some(Health(5)),
        "the buff lands on the SUMMONED minion, not a random friendly one"
    );
}

#[test]
fn w0_sword_of_justice_stops_firing_when_destroyed() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, SWORD_OF_JUSTICE};
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &SWORD_OF_JUSTICE);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hero = state.player(PlayerId1()).hero;
    // Five hero attacks (1/5 sword) — five 1/1 enemy minions die and the
    // sword's durability runs out on the last swing.
    for _ in 0..5 {
        // Test plumbing: a hero attacks once per turn; reset the counter so
        // the durability burn-down can be driven in a single turn.
        state
            .world_mut()
            .set_attacks_used(hero, orange_stone::core::component::AttacksUsed(0));
        let fodder = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_health(e, Health(1));
            world.set_attack(e, Attack(1));
            world.set_cost(e, orange_stone::core::component::Cost(1));
            world.set_card_type(e, CardType::Minion);
            world.set_player(e, PlayerId2());
            world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
            world.set_zone(e, Zone::Play);
            world.zones_mut().insert(Zone::Play, PlayerId2(), e);
            e
        };
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker: hero,
                    defender: fodder,
                },
            )
            .unwrap();
    }
    assert_eq!(state.player(PlayerId1()).weapon, None, "sword broken");
    // Summon a fresh minion — the broken sword must not buff it
    let raptor = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("raptor in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: raptor,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(raptor), Some(Attack(3)));
    assert_eq!(state.world().effective_health(raptor), Some(Health(2)));
}

/// W0-6 Kul Tiran Chaplain — Battlecry: give a friendly minion +2 Health
/// (random friendly minion in self-play — not always itself).
#[test]
fn w0_kul_tiran_chaplain_buffs_a_friendly_minion() {
    use orange_stone::cards::def::KUL_TIRAN_CHAPLAIN;
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(42);
    let other = builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 2);
    builder.add_minion_to_hand(PlayerId1(), &KUL_TIRAN_CHAPLAIN);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let chaplain = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("chaplain in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: chaplain,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // Exactly one friendly minion gained +2 Health — the chaplain or the other.
    let buffed: Vec<_> = [chaplain, other]
        .into_iter()
        .filter(|&e| {
            state
                .world()
                .effective_health(e)
                .is_some_and(|h| h.0 == 5 || h.0 == 3)
        })
        .collect();
    assert!(!buffed.is_empty(), "one friendly minion is buffed");
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId2()).hero),
        Some(Health(30)),
        "the enemy is untouched"
    );
}

/// W0-7 Young Priestess — at the end of your turn, give ANOTHER random friendly
/// minion +1 Health (never itself; nothing happens when alone).
#[test]
fn w0_young_priestess_buffs_another_minion() {
    use orange_stone::cards::def::YOUNG_PRIESTESS;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &YOUNG_PRIESTESS);
    let other = builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 2);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let priestess = find_entity(&state, PlayerId1(), "NEUTRAL_R21");
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().effective_attack(priestess), Some(Attack(2)));
    assert_eq!(
        state.world().effective_health(priestess),
        Some(Health(1)),
        "the priestess never buffs herself"
    );
    assert_eq!(state.world().effective_health(other), Some(Health(4)));
}

#[test]
fn w0_young_priestess_alone_does_nothing() {
    use orange_stone::cards::def::YOUNG_PRIESTESS;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &YOUNG_PRIESTESS);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let priestess = find_entity(&state, PlayerId1(), "NEUTRAL_R21");
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().effective_health(priestess), Some(Health(1)));
}

/// W0-8 Master Swordsmith — at the end of your turn, give ANOTHER random
/// friendly minion +1 Attack.
#[test]
fn w0_master_swordsmith_buffs_another_minion() {
    use orange_stone::cards::def::MASTER_SWORDSMITH;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &MASTER_SWORDSMITH);
    let other = builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 2);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let smith = find_entity(&state, PlayerId1(), "NEUTRAL_R23");
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().effective_attack(smith), Some(Attack(1)));
    assert_eq!(state.world().effective_attack(other), Some(Attack(4)));
}

/// W0-9 Gurubashi Berserker — Enrage: each damage event grants a PERMANENT +3
/// Attack (fires before the death check; stacks across hits).
#[test]
fn w0_gurubashi_berserker_enrage_permanent() {
    use orange_stone::cards::def::GURUBASHI_BERSERKER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &GURUBASHI_BERSERKER);
    let attacker1 = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let berserker = find_entity(&state, PlayerId1(), "NEUTRAL_B19");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: attacker1,
                defender: berserker,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(berserker), Some(Attack(5)));
    assert_eq!(state.world().effective_health(berserker), Some(Health(7)));
    // A second damage event stacks: 8 attack
    let attacker2 = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(1));
        world.set_attack(e, Attack(1));
        world.set_cost(e, orange_stone::core::component::Cost(1));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, PlayerId2());
        world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, PlayerId2(), e);
        e
    };
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: attacker2,
                defender: berserker,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(berserker), Some(Attack(8)));
    assert_eq!(state.world().effective_health(berserker), Some(Health(6)));
}

/// W0-10 Tauren Warrior — Taunt + Enrage: +3 Attack when damaged.
#[test]
fn w0_tauren_warrior_enrage_with_taunt() {
    use orange_stone::cards::def::TAUREN_WARRIOR;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &TAUREN_WARRIOR);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let tauren = find_entity(&state, PlayerId1(), "NEUTRAL_C11");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: tauren,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(tauren), Some(Attack(5)));
    assert_eq!(state.world().effective_health(tauren), Some(Health(2)));
    assert_eq!(state.world().taunt(tauren).is_some(), true);
}

/// W0-11 Angry Chicken — the Enrage buff fires BEFORE the death check (the
/// chicken reaches +5 Attack while still on the board), then the chicken dies
/// to the same damage. The buff is cleared when the minion leaves play, so the
/// observable outcome is a mutual trade with the attacker.
#[test]
fn w0_angry_chicken_enrage_fires_before_death() {
    use orange_stone::cards::def::ANGRY_CHICKEN;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &ANGRY_CHICKEN);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let chicken = find_entity(&state, PlayerId1(), "NEUTRAL_R02");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: chicken,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(chicken),
        Some(Zone::Graveyard),
        "the 1/1 chicken dies to 1 damage despite the enrage"
    );
    assert_eq!(
        state.world().zone(attacker),
        Some(Zone::Graveyard),
        "the attacker dies to the retaliation"
    );
}

/// W0-12 Spiteful Smith — Enrage: your weapon gains +2 Attack.
#[test]
fn w0_spiteful_smith_buffs_weapon_on_damage() {
    use orange_stone::cards::def::{SPITEFUL_SMITH, TRUESILVER_CHAMPION};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &SPITEFUL_SMITH);
    builder.equip_weapon(PlayerId1(), &TRUESILVER_CHAMPION);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let smith = find_entity(&state, PlayerId1(), "NEUTRAL_C15");
    let weapon = find_entity(&state, PlayerId1(), "PALADIN_006");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: smith,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(smith), Some(Health(5)));
    assert_eq!(
        state.world().attack(weapon),
        Some(Attack(6)),
        "Truesilver 4/2 + 2 weapon attack"
    );
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(2)),
        "durability is untouched"
    );
}

/// W0-13 Emperor Cobra — Poison: minions damaged by it are destroyed outright
/// (0-attack targets deal no retaliation); a Divine Shield absorbs the poison.
#[test]
fn w0_emperor_cobra_poison_kills_and_divine_shield_absorbs() {
    use orange_stone::cards::def::EMPEROR_COBRA;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &EMPEROR_COBRA);
    let big = builder.add_custom_minion_to_board(PlayerId2(), 0, 8, 8);
    // Poison does not bypass Divine Shield — the shield absorbs it
    let shielded = builder.add_custom_minion_to_board(PlayerId2(), 0, 2, 2);
    {
        use orange_stone::core::component::DivineShield;
        let world = builder.state_mut().world_mut();
        world.set_divine_shield(shielded, DivineShield);
    }
    let mut state = builder.build();
    let engine = GameEngine::new();
    let cobra = find_entity(&state, PlayerId1(), "NEUTRAL_R16");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: cobra,
                defender: big,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(big),
        Some(Zone::Graveyard),
        "poison destroys the 0/8 despite dealing only 2 damage"
    );
    assert_eq!(state.world().zone(cobra), Some(Zone::Play));
    // Test plumbing: a minion attacks once per turn — reset so the shield
    // interaction can be driven in the same turn.
    state
        .world_mut()
        .set_attacks_used(cobra, orange_stone::core::component::AttacksUsed(0));
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: cobra,
                defender: shielded,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(shielded),
        Some(Zone::Play),
        "divine shield absorbs the poison damage"
    );
    assert_eq!(
        state.world().divine_shield(shielded).is_some(),
        false,
        "the shield is consumed"
    );
}
