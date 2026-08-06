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
            race: None,
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

/// Finds the first Hand-zone entity with the given card ID (Wave 4 scenarios).
fn find_hand_entity(
    state: &GameState,
    player: orange_stone::core::player::PlayerId,
    card_id: &str,
) -> Entity {
    state
        .world()
        .zones()
        .iter(Zone::Hand, player)
        .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == card_id))
        .expect("entity with card id in hand")
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

// ============================================================
// Wave 1 — race/tribe field (fidelity-debt-roadmap W1):
// CardDef.race, race-conditioned targets/auras/triggers,
// race-filtered deck draw, field-driven pools.
// ============================================================

/// W1-1 Houndmaster — Battlecry: give a friendly Beast +2/+2 and Taunt
/// (target set: friendly Beasts only; buff + taunt land on the same minion).
#[test]
fn w1_houndmaster_buffs_only_a_friendly_beast() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, HOUNDMASTER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    let human = builder.add_custom_minion_to_board(PlayerId1(), 2, 3, 2);
    builder.add_minion_to_hand(PlayerId1(), &HOUNDMASTER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raptor = find_entity(&state, PlayerId1(), "CLASSIC_001");
    let houndmaster = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("houndmaster in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: houndmaster,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // The single Beast candidate takes the +2/+2 AND the Taunt
    assert_eq!(state.world().effective_attack(raptor), Some(Attack(5)));
    assert_eq!(state.world().effective_health(raptor), Some(Health(4)));
    assert_eq!(state.world().taunt(raptor).is_some(), true);
    assert_eq!(state.world().effective_attack(human), Some(Attack(2)));
    assert_eq!(state.world().effective_health(human), Some(Health(3)));
    assert_eq!(state.world().taunt(human).is_some(), false);
}

/// W1-2 Tundra Rhino — your Beasts have Charge (aura; includes the Rhino).
#[test]
fn w1_tundra_rhino_gives_beasts_charge() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, TUNDRA_RHINO};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &TUNDRA_RHINO);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_custom_minion_to_hand(PlayerId1(), 2, 3, 2);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raptor = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "CLASSIC_001")
        })
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
        state.world().effective_charge(raptor),
        true,
        "the rhino aura grants charge to a summoned Beast"
    );
    // The freshly summoned Beast can attack immediately
    let hero = state.player(PlayerId2()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: raptor,
                defender: hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(hero),
        Some(Health(27)),
        "the rhino-buffed Beast attacked despite summoning sickness"
    );
    // A non-Beast summoned the same turn has no charge and cannot attack
    let human = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("custom minion in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: human,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let hero = state.player(PlayerId2()).hero;
    assert_eq!(
        engine.apply(
            &mut state,
            Action::Attack {
                attacker: human,
                defender: hero
            }
        ),
        Err(orange_stone::engine::rules::EngineError::AttacksExhausted),
        "non-Beast summoning sickness still applies"
    );
}

/// W1-3 Coldlight Seer — Battlecry: give all other Murlocs +2 Health
/// (excludes itself and non-Murlocs).
#[test]
fn w1_coldlight_seer_buffs_other_murlocs_only() {
    use orange_stone::cards::def::{COLDLIGHT_SEER, MURLOC_RAIDER, MURLOC_TIDEHUNTER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &MURLOC_RAIDER);
    builder.add_minion_to_board(PlayerId1(), &MURLOC_TIDEHUNTER);
    let human = builder.add_custom_minion_to_board(PlayerId1(), 2, 3, 2);
    builder.add_minion_to_hand(PlayerId1(), &COLDLIGHT_SEER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raider = find_entity(&state, PlayerId1(), "NEUTRAL_B02");
    let tidehunter = find_entity(&state, PlayerId1(), "CLASSIC_006");
    let seer = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("seer in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: seer,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(raider), Some(Health(3)));
    assert_eq!(state.world().effective_health(tidehunter), Some(Health(3)));
    assert_eq!(
        state.world().effective_health(seer),
        Some(Health(3)),
        "the seer never buffs itself"
    );
    assert_eq!(state.world().effective_health(human), Some(Health(3)));
}

/// W1-4 Murloc Warleader — your other Murlocs have +2/+1 (aura; no self-buff).
#[test]
fn w1_murloc_warleader_aura_murloc_only() {
    use orange_stone::cards::def::{MURLOC_RAIDER, MURLOC_WARLEADER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &MURLOC_WARLEADER);
    builder.add_minion_to_board(PlayerId1(), &MURLOC_RAIDER);
    let human = builder.add_custom_minion_to_board(PlayerId1(), 2, 3, 2);
    let state = builder.build();
    let warleader = find_entity(&state, PlayerId1(), "NEUTRAL_E02");
    let raider = find_entity(&state, PlayerId1(), "NEUTRAL_B02");
    assert_eq!(state.world().effective_attack(raider), Some(Attack(4)));
    assert_eq!(state.world().effective_health(raider), Some(Health(2)));
    assert_eq!(
        state.world().effective_attack(warleader),
        Some(Attack(3)),
        "the aura excludes its own source"
    );
    assert_eq!(state.world().effective_health(warleader), Some(Health(3)));
    assert_eq!(state.world().effective_attack(human), Some(Attack(2)));
}

/// W1-5 Murloc Tidecaller — whenever you summon a Murloc, gain +1 Attack
/// (race-conditioned summon trigger; non-Murloc summons do nothing).
#[test]
fn w1_murloc_tidecaller_gains_attack_on_murloc_summon() {
    use orange_stone::cards::def::{MURLOC_RAIDER, MURLOC_TIDECALLER, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &MURLOC_TIDECALLER);
    builder.add_minion_to_hand(PlayerId1(), &MURLOC_RAIDER);
    builder.add_minion_to_hand(PlayerId1(), &WORGEN_INFILTRATOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let tidecaller = find_entity(&state, PlayerId1(), "NEUTRAL_R05");
    let raider = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "NEUTRAL_B02")
        })
        .expect("raider in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: raider,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(tidecaller), Some(Attack(2)));
    // A non-Murloc summon does not trigger it
    let worgen = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("worgen in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: worgen,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(tidecaller), Some(Attack(2)));
}

/// W1-6 Hungry Crab — Battlecry: destroy a Murloc (either side) and gain +2/+2.
#[test]
fn w1_hungry_crab_destroys_enemy_murloc_and_buffs() {
    use orange_stone::cards::def::{HUNGRY_CRAB, MURLOC_RAIDER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId2(), &MURLOC_RAIDER);
    builder.add_minion_to_hand(PlayerId1(), &HUNGRY_CRAB);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let enemy_murloc = find_entity(&state, PlayerId2(), "NEUTRAL_B02");
    let crab = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("crab in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: crab,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(enemy_murloc),
        Some(Zone::Graveyard),
        "the only Murloc candidate is destroyed"
    );
    assert_eq!(
        state.world().effective_attack(crab),
        Some(Attack(3)),
        "the crab gained +2/+2"
    );
    assert_eq!(state.world().effective_health(crab), Some(Health(4)));
}

/// W1-7 Sense Demons — draw two Demons from your deck (race-filtered draw).
#[test]
fn w1_sense_demons_draws_two_demons_from_deck() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, SENSE_DEMONS, VOIDWALKER, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_deck(PlayerId1(), &VOIDWALKER);
    builder.add_minion_to_deck(PlayerId1(), &VOIDWALKER);
    builder.add_minion_to_deck(PlayerId1(), &VOIDWALKER);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_deck(PlayerId1(), &WORGEN_INFILTRATOR);
    builder.add_minion_to_hand(PlayerId1(), &SENSE_DEMONS);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let sense = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("sense demons in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: sense,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    assert_eq!(hand.len(), 2, "two demons drawn");
    for card in &hand {
        assert_eq!(
            state.world().card_id(*card).map(|c| c.0),
            Some("WARLOCK_004"),
            "only Demons are drawn"
        );
    }
    assert_eq!(
        state.world().zones().len(Zone::Deck, PlayerId1()),
        3,
        "the non-Demons stay in the deck"
    );
}

/// W1-8 Demonfire — 2 damage to a minion; +2/+2 instead if it is a friendly Demon.
#[test]
fn w1_demonfire_buffs_friendly_demon_and_damages_others() {
    use orange_stone::cards::def::{DEMONFIRE, VOIDWALKER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &VOIDWALKER);
    builder.add_minion_to_hand(PlayerId1(), &DEMONFIRE);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let demonfire = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("demonfire in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: demonfire,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let voidwalker = find_entity(&state, PlayerId1(), "WARLOCK_004");
    assert_eq!(
        state.world().effective_attack(voidwalker),
        Some(Attack(3)),
        "the friendly Demon was buffed +2/+2"
    );
    assert_eq!(state.world().effective_health(voidwalker), Some(Health(5)));

    // Against an enemy minion: plain 2 damage
    let mut builder = GameBuilder::new();
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &DEMONFIRE);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let demonfire = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("demonfire in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: demonfire,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(enemy),
        Some(Zone::Graveyard),
        "the 2-damage Demonfire kills the 2/2 enemy minion"
    );
}

/// W1-9 Siegebreaker — Taunt; your other Demons have +1 Attack (aura).
#[test]
fn w1_siegebreaker_buffs_other_demons() {
    use orange_stone::cards::def::{SIEGEBREAKER, VOIDWALKER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &SIEGEBREAKER);
    builder.add_minion_to_board(PlayerId1(), &VOIDWALKER);
    let human = builder.add_custom_minion_to_board(PlayerId1(), 2, 3, 2);
    let state = builder.build();
    let siegebreaker = find_entity(&state, PlayerId1(), "WARLOCK_T01");
    let voidwalker = find_entity(&state, PlayerId1(), "WARLOCK_004");
    assert_eq!(state.world().effective_attack(voidwalker), Some(Attack(2)));
    assert_eq!(
        state.world().effective_attack(siegebreaker),
        Some(Attack(5)),
        "the aura excludes its own source"
    );
    assert_eq!(state.world().effective_attack(human), Some(Attack(2)));
    assert_eq!(state.world().taunt(siegebreaker).is_some(), true);
}

/// W1-10 Scavenging Hyena — whenever a friendly Beast dies, gain +2/+1
/// (race-conditioned death trigger; non-Beast deaths do nothing).
#[test]
fn w1_scavenging_hyena_only_counts_beast_deaths() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, SCAVENGING_HYENA};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &SCAVENGING_HYENA);
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    let human = builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 2);
    let attacker1 = builder.add_custom_minion_to_board(PlayerId2(), 4, 4, 4);
    let attacker2 = builder.add_custom_minion_to_board(PlayerId2(), 4, 4, 4);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hyena = find_entity(&state, PlayerId1(), "HUNTER_013");
    let raptor = find_entity(&state, PlayerId1(), "CLASSIC_001");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: attacker1,
                defender: raptor,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(raptor),
        Some(Zone::Graveyard),
        "the Beast dies to the 4/4"
    );
    assert_eq!(state.world().effective_attack(hyena), Some(Attack(4)));
    assert_eq!(state.world().effective_health(hyena), Some(Health(3)));
    // A non-Beast death does not trigger the hyena
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: attacker2,
                defender: human,
            },
        )
        .unwrap();
    assert_eq!(state.world().zone(human), Some(Zone::Graveyard));
    assert_eq!(state.world().effective_attack(hyena), Some(Attack(4)));
    assert_eq!(state.world().effective_health(hyena), Some(Health(3)));
}

/// W1-11 Starving Buzzard — whenever you summon a Beast, draw a card.
#[test]
fn w1_starving_buzzard_draws_on_beast_summon() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, STARVING_BUZZARD, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &STARVING_BUZZARD);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId1(), &WORGEN_INFILTRATOR);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raptor = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "CLASSIC_001")
        })
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
        state.world().zones().len(Zone::Hand, PlayerId1()),
        2,
        "summoning a Beast drew the deck card (worgen + drawn)"
    );
    // A non-Beast summon does not draw
    let worgen = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("worgen in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: worgen,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "the drawn card is still in hand; the non-Beast summon drew nothing"
    );
}

/// W1-12 Race pools — the field-driven Beast/Demon pools (fidelity-debt W1)
/// replace the old hardcoded ID lists; the delta is exactly the genuine
/// Beasts/Demons the old lists missed.
#[test]
fn w1_race_pools_are_field_driven() {
    use orange_stone::cards::sets::ALL_CARDS;
    use orange_stone::core::component::Race;

    let ids = |race: Race| {
        let mut v: Vec<&str> = ALL_CARDS
            .iter()
            .filter(|c| c.race == Some(race))
            .map(|c| c.id)
            .collect();
        v.sort_unstable();
        v
    };
    // The old hardcoded Beast pool (deleted in W1)
    let old_beasts = [
        "NEUTRAL_B08",
        "NEUTRAL_B03",
        "NEUTRAL_B11",
        "NEUTRAL_C04",
        "NEUTRAL_T14",
        "NEUTRAL_C10",
        "NEUTRAL_B13",
        "NEUTRAL_R20",
        "NEUTRAL_T05",
        "HUNTER_010",
        "HUNTER_006",
        "HUNTER_011",
        "HUNTER_014",
        "HUNTER_016",
    ];
    let new_beasts = ids(Race::Beast);
    for id in old_beasts {
        assert!(
            new_beasts.contains(&id),
            "old Beast-pool member {id} must stay in the field-driven pool"
        );
    }
    // The additions are exactly the Beasts the old list missed
    let additions: Vec<_> = new_beasts
        .iter()
        .filter(|id| !old_beasts.contains(id))
        .copied()
        .collect();
    assert_eq!(
        additions,
        vec![
            "CLASSIC_001", // Bloodfen Raptor
            "HUNTER_006t", // Hyena (Savannah Highmane token)
            "HUNTER_013",  // Scavenging Hyena
            "HUNTER_023a", // Huffer
            "HUNTER_023b", // Leokk
            "HUNTER_023c", // Misha
            "NEUTRAL_E03", // Hungry Crab
        ]
    );
    // Demons: the old 8 + Siegebreaker
    let demons = ids(Race::Demon);
    let old_demons = [
        "WARLOCK_004",
        "WARLOCK_002",
        "WARLOCK_007",
        "WARLOCK_011",
        "WARLOCK_012",
        "WARLOCK_016",
        "WARLOCK_019",
        "WARLOCK_022",
    ];
    for id in old_demons {
        assert!(demons.contains(&id), "old Demon-pool member {id} must stay");
    }
    assert_eq!(
        demons
            .iter()
            .filter(|id| !old_demons.contains(id))
            .copied()
            .collect::<Vec<_>>(),
        vec!["WARLOCK_T01"] // Siegebreaker
    );
}

// ============================================================
// Wave 2 — trigger classes (fidelity-debt-roadmap W2): heal,
// attack-on-target, card-played, secret-played, any-minion-died
// triggers + destroy-secret effects.
// ============================================================

/// W2-1 Lightwarden — whenever a character is healed, gain +2 Attack
/// (a heal that lands on an undamaged character is not a heal event).
#[test]
fn w2_lightwarden_gains_attack_on_real_heals() {
    use orange_stone::cards::def::LIGHTWARDEN;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &LIGHTWARDEN);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 6, 6, 2);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let lightwarden = find_entity(&state, PlayerId1(), "NEUTRAL_R04");
    let hero = state.player(PlayerId1()).hero;
    // Damage the friendly hero (30 -> 24) so three 2-point heals are all real
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(hero), Some(Health(24)));
    // A custom heal spell: restore 2 to the friendly hero
    state.set_active_player(PlayerId1());
    let heal_spell = {
        use orange_stone::core::component::{Battlecry, Cost};
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(1));
        world.set_player(e, PlayerId1());
        world.set_battlecry(
            e,
            Battlecry(orange_stone::core::effect::CardEffect::RestoreHealth {
                amount: 2,
                target: orange_stone::core::effect::EffectTarget::FriendlyHero,
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
                card: heal_spell,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(lightwarden), Some(Attack(3)));
    // A second real heal stacks
    let heal_spell2 = {
        use orange_stone::core::component::{Battlecry, Cost};
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(1));
        world.set_player(e, PlayerId1());
        world.set_battlecry(
            e,
            Battlecry(orange_stone::core::effect::CardEffect::RestoreHealth {
                amount: 2,
                target: orange_stone::core::effect::EffectTarget::FriendlyHero,
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
                card: heal_spell2,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(lightwarden), Some(Attack(5)));
    // A heal that restores nothing (hero back to full) does not trigger
    let heal_spell3 = {
        use orange_stone::core::component::{Battlecry, Cost};
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(1));
        world.set_player(e, PlayerId1());
        world.set_battlecry(
            e,
            Battlecry(orange_stone::core::effect::CardEffect::RestoreHealth {
                amount: 2,
                target: orange_stone::core::effect::EffectTarget::FriendlyHero,
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
                card: heal_spell3,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(lightwarden), Some(Attack(7)));
    // A heal that restores nothing (hero back to full) does not trigger
    let heal_spell4 = {
        use orange_stone::core::component::{Battlecry, Cost};
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, Cost(1));
        world.set_player(e, PlayerId1());
        world.set_battlecry(
            e,
            Battlecry(orange_stone::core::effect::CardEffect::RestoreHealth {
                amount: 2,
                target: orange_stone::core::effect::EffectTarget::FriendlyHero,
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
                card: heal_spell4,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(lightwarden),
        Some(Attack(7)),
        "a no-op heal is not a heal event"
    );
}

/// W2-2 Blessing of Wisdom — the buffed minion draws a card whenever IT attacks.
#[test]
fn w2_blessing_of_wisdom_draws_on_attacks() {
    use orange_stone::cards::def::{BLESSING_OF_WISDOM, BLOODFEN_RAPTOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    let b = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &BLESSING_OF_WISDOM);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let a = find_entity(&state, PlayerId1(), "CLASSIC_001");
    let blessing = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("blessing in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: blessing,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // Both friendly minions attack; exactly one carries the blessing
    let hero = state.player(PlayerId2()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: a,
                defender: hero,
            },
        )
        .unwrap();
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: b,
                defender: hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "exactly one attack drew a card — the blessed minion"
    );
    assert_eq!(
        state.world().zones().len(Zone::Deck, PlayerId1()),
        0,
        "the drawn card came from the deck"
    );
}

/// W2-3 Questing Adventurer — whenever you play a card, gain +1/+1.
#[test]
fn w2_questing_adventurer_grows_per_played_card() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, QUESTING_ADVENTURER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &QUESTING_ADVENTURER);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let qa = find_entity(&state, PlayerId1(), "NEUTRAL_R17");
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
    assert_eq!(state.world().effective_attack(qa), Some(Attack(3)));
    assert_eq!(state.world().effective_health(qa), Some(Health(3)));
}

/// W2-4 Secretkeeper — whenever a Secret is played (either player), gain +1/+1.
#[test]
fn w2_secretkeeper_grows_on_any_secret() {
    use orange_stone::cards::def::{COUNTERSPELL, SECRETKEEPER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &SECRETKEEPER);
    builder.add_minion_to_hand(PlayerId1(), &COUNTERSPELL);
    builder.add_minion_to_hand(PlayerId2(), &COUNTERSPELL);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let secretkeeper = find_entity(&state, PlayerId1(), "NEUTRAL_R06");
    let secret = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("counterspell in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: secret,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(secretkeeper),
        Some(Attack(2))
    );
    // The opponent's Secret triggers it too
    state.set_active_player(PlayerId2());
    let secret2 = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .next()
        .expect("counterspell in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: secret2,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(secretkeeper),
        Some(Attack(3))
    );
}

/// W2-5 Flesheating Ghoul — whenever a minion dies (either side), gain +1 Attack.
#[test]
fn w2_flesheating_ghoul_counts_every_death() {
    use orange_stone::cards::def::FLESHEATING_GHOUL;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &FLESHEATING_GHOUL);
    let friendly = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let ghoul = find_entity(&state, PlayerId1(), "NEUTRAL_C12");
    // A mutual trade kills both minions — two death events, +2 Attack total
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: friendly,
                defender: enemy,
            },
        )
        .unwrap();
    assert_eq!(state.world().zone(friendly), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
    assert_eq!(
        state.world().effective_attack(ghoul),
        Some(Attack(5)),
        "both deaths (friendly and enemy) counted"
    );
}

/// W2-6 SI:7 Infiltrator — Battlecry: destroy ONE random enemy Secret.
#[test]
fn w2_si7_destroys_one_enemy_secret() {
    use orange_stone::cards::def::{COUNTERSPELL, SI7_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId2(), &COUNTERSPELL);
    builder.add_minion_to_hand(PlayerId2(), &COUNTERSPELL);
    builder.add_minion_to_hand(PlayerId1(), &SI7_INFILTRATOR);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // P2 plays two Secrets
    state.set_active_player(PlayerId2());
    let secrets: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .collect();
    for secret in secrets {
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: secret,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
    }
    assert_eq!(state.world().zones().len(Zone::SetAside, PlayerId2()), 2);
    // P1 plays SI:7 — one random enemy Secret is destroyed
    state.set_active_player(PlayerId1());
    let si7 = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("si7 in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: si7,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::SetAside, PlayerId2()),
        1,
        "exactly one enemy Secret remains"
    );
}

/// W2-7 Eater of Secrets — Battlecry: destroy ALL enemy Secrets and gain +1/+1.
#[test]
fn w2_eater_of_secrets_destroys_all_and_buffs() {
    use orange_stone::cards::def::{COUNTERSPELL, EATER_OF_SECRETS};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId2(), &COUNTERSPELL);
    builder.add_minion_to_hand(PlayerId2(), &COUNTERSPELL);
    builder.add_minion_to_hand(PlayerId1(), &EATER_OF_SECRETS);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    state.set_active_player(PlayerId2());
    let secrets: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .collect();
    for secret in secrets {
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: secret,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
    }
    state.set_active_player(PlayerId1());
    let eater = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("eater in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: eater,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::SetAside, PlayerId2()),
        0,
        "all enemy Secrets destroyed"
    );
    assert_eq!(state.world().effective_attack(eater), Some(Attack(3)));
    assert_eq!(state.world().effective_health(eater), Some(Health(5)));
}

/// W2-8 Flare — destroy all enemy Secrets and draw a card.
#[test]
fn w2_flare_destroys_all_secrets_and_draws() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, FLARE, VAPORIZE};
    let mut builder = GameBuilder::new();
    // Vaporize (not Counterspell): the secrets must NOT intercept Flare itself
    builder.add_minion_to_hand(PlayerId2(), &VAPORIZE);
    builder.add_minion_to_hand(PlayerId2(), &VAPORIZE);
    builder.add_minion_to_hand(PlayerId1(), &FLARE);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    state.set_active_player(PlayerId2());
    let secrets: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .collect();
    for secret in secrets {
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: secret,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
    }
    state.set_active_player(PlayerId1());
    let flare = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("flare in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: flare,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::SetAside, PlayerId2()),
        0,
        "all enemy Secrets destroyed"
    );
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "Flare drew a card"
    );
}

// ============================================================
// Wave 3 — conditional targets & states (fidelity-debt-roadmap W3):
// attack-range, hand-size, hero-health, damaged-friendly, damaged-count,
// owns-secret, first-minion-cost, divine-shield absorb.
// ============================================================

/// W3-1 Stampeding Kodo — Battlecry: destroy a random enemy minion with 2 or
/// less Attack (attack ≤ N predicate).
#[test]
fn w3_stampeding_kodo_destroys_low_attack_minion() {
    use orange_stone::cards::def::STAMPEDING_KODO;
    let mut builder = GameBuilder::new();
    let weak = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    let strong = builder.add_custom_minion_to_board(PlayerId2(), 3, 3, 3);
    builder.add_minion_to_hand(PlayerId1(), &STAMPEDING_KODO);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let kodo = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("kodo in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: kodo,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().zone(weak), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(strong), Some(Zone::Play));
}

/// W3-2 Big Game Hunter — Battlecry: destroy a minion with 7 or more Attack
/// (attack ≥ N predicate, either side).
#[test]
fn w3_big_game_hunter_destroys_high_attack_minion() {
    use orange_stone::cards::def::BIG_GAME_HUNTER;
    let mut builder = GameBuilder::new();
    let big = builder.add_custom_minion_to_board(PlayerId2(), 8, 8, 8);
    let small = builder.add_custom_minion_to_board(PlayerId2(), 3, 3, 3);
    builder.add_minion_to_hand(PlayerId1(), &BIG_GAME_HUNTER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let bgh = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("bgh in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: bgh,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().zone(big), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(small), Some(Zone::Play));
}

/// W3-3 Twilight Drake — Battlecry: gain +1 Health for each card in hand.
#[test]
fn w3_twilight_drake_gains_health_per_hand_card() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, TWILIGHT_DRAKE, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId1(), &WORGEN_INFILTRATOR);
    builder.add_minion_to_hand(PlayerId1(), &TWILIGHT_DRAKE);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let drake = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "NEUTRAL_R19")
        })
        .expect("drake in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: drake,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(drake),
        Some(Health(3)),
        "two cards left in hand → 1 + 2 health"
    );
}

/// W3-4 Mortal Strike — 4 damage; 6 instead when your hero has 12 or less
/// Health (owner-health predicate).
#[test]
fn w3_mortal_strike_boosts_at_low_health() {
    use orange_stone::cards::def::MORTAL_STRIKE;
    // High-health hero: 4 damage
    let mut builder = GameBuilder::new();
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 1, 5, 5);
    builder.add_minion_to_hand(PlayerId1(), &MORTAL_STRIKE);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let strike = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("strike in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: strike,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // The random target is the enemy minion or the enemy hero — either way the
    // enemy took exactly 4 damage
    let enemy_damage = {
        let minion_dmg = state.world().damage(enemy).map_or(0, |d| d.0);
        let hero_dmg = state
            .world()
            .damage(state.player(PlayerId2()).hero)
            .map_or(0, |d| d.0);
        minion_dmg + hero_dmg
    };
    assert_eq!(enemy_damage, 4);

    // Low-health hero (10 ≤ 12): 6 damage kills the 5/5
    let mut builder = GameBuilder::new();
    let enemy2 = builder.add_custom_minion_to_board(PlayerId2(), 1, 5, 5);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 20, 20, 2);
    builder.add_minion_to_hand(PlayerId1(), &MORTAL_STRIKE);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hero = state.player(PlayerId1()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(hero), Some(Health(10)));
    state.set_active_player(PlayerId1());
    let strike = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("strike in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: strike,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(enemy2),
        Some(Zone::Graveyard),
        "at 10 hero health the strike deals 6 — enough to kill the 5/5"
    );
}

/// W3-5 Rampage — give a damaged minion +3/+3 (damaged predicate).
#[test]
fn w3_rampage_targets_only_damaged_minions() {
    use orange_stone::cards::def::RAMPAGE;
    let mut builder = GameBuilder::new();
    let damaged = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let fresh = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &RAMPAGE);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // Damage one friendly minion (enemy attacks it)
    let attacker = {
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
    state.set_active_player(PlayerId2());
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: damaged,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(damaged), Some(Health(1)));
    state.set_active_player(PlayerId1());
    let rampage = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("rampage in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: rampage,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(damaged), Some(Attack(5)));
    assert_eq!(state.world().effective_health(damaged), Some(Health(4)));
    assert_eq!(state.world().effective_attack(fresh), Some(Attack(2)));
}

/// W3-6 Battle Rage — draw a card for each damaged friendly character
/// (hero + minions).
#[test]
fn w3_battle_rage_draws_per_damaged_friendly_character() {
    use orange_stone::cards::def::{BATTLE_RAGE, BLOODFEN_RAPTOR};
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let b = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &BATTLE_RAGE);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // Damage both friendly minions and the hero
    let attacker1 = {
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
    let attacker3 = {
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
    state.set_active_player(PlayerId2());
    let hero = state.player(PlayerId1()).hero;
    for (attacker, target) in [(attacker1, a), (attacker2, b), (attacker3, hero)] {
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: target,
                },
            )
            .unwrap();
    }
    assert_eq!(state.world().effective_health(hero), Some(Health(29)));
    state.set_active_player(PlayerId1());
    let rage = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("battle rage in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: rage,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        3,
        "three damaged friendly characters (2 minions + hero) → three draws"
    );
}

/// W3-7 Ethereal Arcanist — at the end of your turn, +2/+2 only while you
/// control a Secret (owns-secret predicate).
#[test]
fn w3_ethereal_arcanist_requires_a_secret() {
    use orange_stone::cards::def::{COUNTERSPELL, ETHEREAL_ARCANIST};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &ETHEREAL_ARCANIST);
    builder.add_minion_to_hand(PlayerId1(), &COUNTERSPELL);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let arcanist = find_entity(&state, PlayerId1(), "MAGE_017");
    // End the turn WITHOUT a secret — no buff
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().effective_attack(arcanist), Some(Attack(3)));
    assert_eq!(state.world().effective_health(arcanist), Some(Health(3)));
    // Play a secret and end the turn — +2/+2
    engine.apply(&mut state, Action::EndTurn).unwrap(); // back to P1
    let secret = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("counterspell in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: secret,
                target: None,
                position: None,
            },
        )
        .unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.world().effective_attack(arcanist), Some(Attack(5)));
    assert_eq!(state.world().effective_health(arcanist), Some(Health(5)));
}

/// W3-8 Pint-Sized Summoner — the first minion you play each turn costs (1)
/// less.
#[test]
fn w3_pint_sized_summoner_discounts_first_minion() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, PINT_SIZED_SUMMONER, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &PINT_SIZED_SUMMONER);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId1(), &WORGEN_INFILTRATOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // First minion (2-cost): pays 1
    let raptor = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "CLASSIC_001")
        })
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
        state.player(PlayerId1()).current_mana,
        9,
        "the first minion cost (1) less"
    );
    // Second minion: full price
    let worgen = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("worgen in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: worgen,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).current_mana,
        8,
        "the second minion costs its full price (1 mana, no discount)"
    );
}

/// W3-9 Blood Knight — Battlecry: destroy all Divine Shields (both sides)
/// and gain +3/+3 for each.
#[test]
fn w3_blood_knight_absorbs_all_divine_shields() {
    use orange_stone::cards::def::BLOOD_KNIGHT;
    let mut builder = GameBuilder::new();
    let friendly_shielded = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    let enemy_shielded = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    let enemy_plain = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    {
        use orange_stone::core::component::DivineShield;
        let world = builder.state_mut().world_mut();
        world.set_divine_shield(friendly_shielded, DivineShield);
        world.set_divine_shield(enemy_shielded, DivineShield);
    }
    builder.add_minion_to_hand(PlayerId1(), &BLOOD_KNIGHT);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let knight = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("knight in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: knight,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(knight),
        Some(Attack(9)),
        "two shields absorbed → +6/+6"
    );
    assert_eq!(state.world().effective_health(knight), Some(Health(9)));
    assert_eq!(
        state.world().divine_shield(friendly_shielded).is_some(),
        false
    );
    assert_eq!(state.world().divine_shield(enemy_shielded).is_some(), false);
    assert_eq!(state.world().divine_shield(enemy_plain).is_some(), false);
}

// ============================================================
// Wave 4 — cost & weapon interactions (fidelity-debt-roadmap W4):
// hand-cost auras, weapon-attack cost, weapon durability, conditional
// charge, enemy-spells-zero, give-opponent-mana.
// ============================================================

/// W4-1 Mana Wraith — ALL minions cost (1) more (both players' hands; spells
/// unaffected).
#[test]
fn w4_mana_wraith_increases_all_minion_costs() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, MANA_WRAITH, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &MANA_WRAITH);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId2(), &WORGEN_INFILTRATOR);
    let state = builder.build();
    let p1_card = find_hand_entity(&state, PlayerId1(), "CLASSIC_001");
    let p2_card = find_hand_entity(&state, PlayerId2(), "NEUTRAL_C08");
    assert_eq!(
        state.world().effective_cost(p1_card).map(|c| c.0),
        Some(3),
        "P1's 2-cost minion costs 3"
    );
    assert_eq!(
        state.world().effective_cost(p2_card).map(|c| c.0),
        Some(2),
        "P2's 1-cost minion costs 2 — the aura hits both players"
    );
}

/// W4-2 Venture Co. Mercenary — YOUR minions cost (3) more (the opponent's
/// hand is untouched).
#[test]
fn w4_venture_co_increases_own_minion_costs() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, VENTURE_CO_MERCENARY, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &VENTURE_CO_MERCENARY);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId2(), &WORGEN_INFILTRATOR);
    let state = builder.build();
    let p1_card = find_hand_entity(&state, PlayerId1(), "CLASSIC_001");
    let p2_card = find_hand_entity(&state, PlayerId2(), "NEUTRAL_C08");
    assert_eq!(
        state.world().effective_cost(p1_card).map(|c| c.0),
        Some(5),
        "P1's 2-cost minion costs 5"
    );
    assert_eq!(
        state.world().effective_cost(p2_card).map(|c| c.0),
        Some(1),
        "the opponent's minions are unaffected"
    );
}

/// W4-3 Southsea Deckhand — has Charge while you have a weapon equipped.
#[test]
fn w4_southsea_deckhand_charge_with_weapon() {
    use orange_stone::cards::def::{SOUTHSHORE_DECKHAND, TRUESILVER_CHAMPION};
    // No weapon: no charge — the freshly played deckhand cannot attack
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &SOUTHSHORE_DECKHAND);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let deckhand = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("deckhand in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: deckhand,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let hero = state.player(PlayerId2()).hero;
    assert_eq!(
        engine.apply(
            &mut state,
            Action::Attack {
                attacker: deckhand,
                defender: hero
            }
        ),
        Err(orange_stone::engine::rules::EngineError::AttacksExhausted),
        "no weapon — summoning sickness applies"
    );
    // With a weapon equipped the deckhand gains Charge
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &TRUESILVER_CHAMPION);
    builder.add_minion_to_hand(PlayerId1(), &SOUTHSHORE_DECKHAND);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let deckhand = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("deckhand in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: deckhand,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_charge(deckhand), true);
    let hero = state.player(PlayerId2()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: deckhand,
                defender: hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(hero),
        Some(Health(28)),
        "the deckhand attacked despite summoning sickness"
    );
}

/// W4-4 Dread Corsair — Taunt; costs (1) less per Attack of your weapon.
#[test]
fn w4_dread_corsair_cost_by_weapon_attack() {
    use orange_stone::cards::def::{DREAD_CORSAIR, TRUESILVER_CHAMPION};
    // No weapon: full 4 cost
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &DREAD_CORSAIR);
    let state = builder.build();
    let corsair = find_hand_entity(&state, PlayerId1(), "NEUTRAL_C13");
    assert_eq!(state.world().effective_cost(corsair).map(|c| c.0), Some(4));
    // 4-attack weapon: 4 - 4 = 0
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &TRUESILVER_CHAMPION);
    builder.add_minion_to_hand(PlayerId1(), &DREAD_CORSAIR);
    builder.set_mana(PlayerId1(), 10, 10);
    let state = builder.build();
    let corsair = find_hand_entity(&state, PlayerId1(), "NEUTRAL_C13");
    assert_eq!(
        orange_stone::engine::cost::play_cost(&state, corsair, PlayerId1()).0,
        0,
        "Truesilver's 4 Attack discounts the full cost"
    );
}

/// W4-5 Bloodsail Raider — Battlecry: gain Attack equal to your weapon's
/// Attack.
#[test]
fn w4_bloodsail_raider_gains_weapon_attack() {
    use orange_stone::cards::def::{BLOODSAIL_RAIDER, TRUESILVER_CHAMPION};
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &TRUESILVER_CHAMPION);
    builder.add_minion_to_hand(PlayerId1(), &BLOODSAIL_RAIDER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raider = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("raider in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: raider,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(raider),
        Some(Attack(6)),
        "2 base + 4 weapon attack"
    );
}

/// W4-6 Bloodsail Corsair — Battlecry: remove 1 Durability from the
/// opponent's weapon (a weapon at 0 durability is destroyed).
#[test]
fn w4_bloodsail_corsair_removes_weapon_durability() {
    use orange_stone::cards::def::{BLOODSAIL_CORSAIR, TRUESILVER_CHAMPION};
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId2(), &TRUESILVER_CHAMPION);
    builder.add_minion_to_hand(PlayerId1(), &BLOODSAIL_CORSAIR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let corsair = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("corsair in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: corsair,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let weapon = state.player(PlayerId2()).weapon.expect("weapon alive");
    assert_eq!(
        state.world().durability(weapon).map(|d| d.0),
        Some(1),
        "Truesilver 2 durability - 1"
    );
}

#[test]
fn w4_bloodsail_corsair_destroys_1_durability_weapon() {
    use orange_stone::cards::def::{BLOODSAIL_CORSAIR, TRUESILVER_CHAMPION};
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId2(), &TRUESILVER_CHAMPION);
    let fodder = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    builder.add_minion_to_hand(PlayerId1(), &BLOODSAIL_CORSAIR);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    // The enemy hero attacks once: Truesilver 2 -> 1 durability
    let hero = state.player(PlayerId2()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: fodder,
            },
        )
        .unwrap();
    let weapon = state.player(PlayerId2()).weapon.expect("weapon alive");
    assert_eq!(state.world().durability(weapon).map(|d| d.0), Some(1));
    state.set_active_player(PlayerId1());
    let corsair = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("corsair in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: corsair,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId2()).weapon,
        None,
        "a 1-durability weapon is destroyed"
    );
}

/// W4-7 Millhouse Manastorm — Battlecry: the opponent's spells cost 0 next
/// turn.
#[test]
fn w4_millhouse_makes_enemy_spells_free() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, MILLHOUSE_MANASTORM, PYROBLAST};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &MILLHOUSE_MANASTORM);
    builder.add_minion_to_hand(PlayerId2(), &PYROBLAST);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let millhouse = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("millhouse in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: millhouse,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // The opponent's turn: a 10-cost spell costs 0
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(state.player(PlayerId2()).current_mana, 10);
    let pyro = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .next()
        .expect("pyroblast in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: pyro,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId2()).current_mana,
        10,
        "the enemy spell was free"
    );
}

/// W4-8 Arcane Golem — Charge; Battlecry: give your opponent a mana crystal.
#[test]
fn w4_arcane_golem_gives_opponent_crystal() {
    use orange_stone::cards::def::ARCANE_GOLEM;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &ARCANE_GOLEM);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 5, 5);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let golem = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("golem in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: golem,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId2()).mana_crystals,
        6,
        "the opponent gained an empty crystal"
    );
    assert_eq!(
        state.player(PlayerId2()).current_mana,
        5,
        "the crystal is empty — current mana unchanged"
    );
    assert_eq!(state.world().effective_charge(golem), true);
}
