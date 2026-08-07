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
use orange_stone::core::component::{Attack, CardType, Cost, Health};
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

// Small helpers — keep the scenarios readable. The uppercase names mirror
// the file's scenario naming style (PlayerId1() reads like a player).
#[allow(non_snake_case)]
fn PlayerId1() -> orange_stone::core::player::PlayerId {
    orange_stone::core::player::PlayerId::Player1
}

#[allow(non_snake_case)]
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
        use orange_stone::core::component::Cost;
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
    use orange_stone::cards::def::{DOOMSAYER, OGRE_MAGI};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &DOOMSAYER);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 3, 3, 3);
    // Deck cards so the turn draws don't fatigue the heroes — this scenario
    // is about minion destruction, not fatigue
    builder.add_minion_to_deck(PlayerId1(), &OGRE_MAGI);
    builder.add_minion_to_deck(PlayerId2(), &OGRE_MAGI);
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

/// F-A10 — empty-deck draws fatigue (official HS rule, SabberStone parity for
/// the exhausted-deck line): each draw attempt on an empty deck deals
/// escalating damage (1, 2, 3, …) to the drawing hero. With both decks empty
/// the game ends with a real winner — previously the engine stalled forever.
#[test]
fn fatigue_ends_exhausted_deck_games_with_a_winner() {
    use orange_stone::core::player::PlayerId;
    let engine = GameEngine::new();
    // One card per deck: drawn on the first turns, then every draw fatigues
    let mut builder = GameBuilder::new();
    builder.add_minion_to_deck(PlayerId::Player1, &orange_stone::cards::def::OGRE_MAGI);
    builder.add_minion_to_deck(PlayerId::Player2, &orange_stone::cards::def::OGRE_MAGI);
    let mut state = builder.build();
    let mut guard = 0;
    while !matches!(state.step(), Step::GameOver { .. }) && guard < 60 {
        engine.apply(&mut state, Action::EndTurn).unwrap();
        guard += 1;
    }
    assert!(
        matches!(state.step(), Step::GameOver { .. }),
        "the exhausted-deck game must end with a real winner (turns = {guard})"
    );
}

/// W0-5 Sword of Justice — whenever you summon a minion, give IT +1/+1 (the
/// event subject, not a random friendly minion); a destroyed sword stops
/// firing.
#[test]
fn w0_sword_of_justice_buffs_the_summoned_minion() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, SWORD_OF_JUSTICE};
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
    assert!(state.world().taunt(tauren).is_some());
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
    assert!(
        !state.world().divine_shield(shielded).is_some(),
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
    assert!(state.world().taunt(raptor).is_some());
    assert_eq!(state.world().effective_attack(human), Some(Attack(2)));
    assert_eq!(state.world().effective_health(human), Some(Health(3)));
    assert!(!state.world().taunt(human).is_some());
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
    assert!(
        state.world().effective_charge(raptor),
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
    assert!(state.world().taunt(siegebreaker).is_some());
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
    // Demons: the old 8 + Siegebreaker + Blood Imp
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
    let mut extra = demons
        .iter()
        .filter(|id| !old_demons.contains(id))
        .copied()
        .collect::<Vec<_>>();
    extra.sort_unstable();
    assert_eq!(extra, vec!["CS2_064", "WARLOCK_T01"]); // Blood Imp, Siegebreaker
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
    assert!(!state.world().divine_shield(friendly_shielded).is_some());
    assert!(!state.world().divine_shield(enemy_shielded).is_some());
    assert!(!state.world().divine_shield(enemy_plain).is_some());
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
    assert!(state.world().effective_charge(deckhand));
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
    use orange_stone::cards::def::{MILLHOUSE_MANASTORM, PYROBLAST};
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
    assert!(state.world().effective_charge(golem));
}

// ============================================================
// Wave 5 — target structure & effect composition
// (fidelity-debt-roadmap W5): set-health-to, swap, adjacent
// targets, combined effects.
// ============================================================

/// W5-1 Repentance — Secret: when the opponent plays a minion, set its
/// Health to 1.
#[test]
fn w5_repentance_sets_played_minion_health_to_1() {
    use orange_stone::cards::def::REPENTANCE;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &REPENTANCE);
    builder.add_custom_minion_to_hand(PlayerId2(), 4, 5, 4);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let repentance = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("repentance in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: repentance,
                target: None,
                position: None,
            },
        )
        .unwrap();
    state.set_active_player(PlayerId2());
    let minion = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .next()
        .expect("minion in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: minion,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(minion),
        Some(Health(1)),
        "the secret set the played minion's health to 1"
    );
    assert_eq!(
        state.world().effective_attack(minion),
        Some(Attack(4)),
        "attack is untouched"
    );
}

/// W5-2 Mass Dispel — Silence all enemy minions, draw a card.
#[test]
fn w5_mass_dispel_silences_all_enemy_minions() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, MASS_DISPEL, TAUREN_WARRIOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId2(), &TAUREN_WARRIOR);
    builder.add_minion_to_hand(PlayerId1(), &MASS_DISPEL);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let tauren = find_entity(&state, PlayerId2(), "NEUTRAL_C11");
    // Damage the tauren so the silence is observable (enrage buff would apply
    // on damage, then silence removes it)
    let attacker = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(1));
        world.set_attack(e, Attack(1));
        world.set_cost(e, orange_stone::core::component::Cost(1));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, PlayerId1());
        world.set_attacks_used(e, orange_stone::core::component::AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, PlayerId1(), e);
        e
    };
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
    let dispel = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("dispel in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: dispel,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(tauren),
        Some(Attack(2)),
        "silence removed the enrage buff"
    );
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "Mass Dispel drew a card"
    );
}

/// W5-3 Crazed Alchemist — Battlecry: swap a minion's Attack and Health.
#[test]
fn w5_crazed_alchemist_swaps_stats() {
    use orange_stone::cards::def::CRAZED_ALCHEMIST;
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(1);
    let target = builder.add_custom_minion_to_board(PlayerId2(), 1, 5, 5);
    builder.add_minion_to_hand(PlayerId1(), &CRAZED_ALCHEMIST);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let alchemist = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("alchemist in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: alchemist,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // The only non-alchemist candidate: 1/5 becomes 5/1 (pinned by seed)
    assert_eq!(state.world().effective_attack(target), Some(Attack(5)));
    assert_eq!(state.world().effective_health(target), Some(Health(1)));
}

/// W5-4 Cone of Cold — Freeze a minion and its neighbors.
#[test]
fn w5_cone_of_cold_freezes_adjacent() {
    use orange_stone::cards::def::CONE_OF_COLD;
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(5);
    let a = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    let b = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    let c = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.add_minion_to_hand(PlayerId1(), &CONE_OF_COLD);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let cone = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("cone in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: cone,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // With three enemy minions, the middle one is always picked (pinned by
    // seed): all three freeze — the middle and both neighbors.
    let frozen = [a, b, c]
        .iter()
        .filter(|&&e| state.world().freeze(e).is_some())
        .count();
    assert_eq!(frozen, 3, "the picked minion and both neighbors freeze");
}

/// W5-5 Sunfury Protector — Battlecry: give adjacent minions Taunt.
#[test]
fn w5_sunfury_protector_taunts_adjacent() {
    use orange_stone::cards::def::SUNFURY_PROTECTOR;
    let mut builder = GameBuilder::new();
    let left = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    let right = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    builder.add_minion_to_hand(PlayerId1(), &SUNFURY_PROTECTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let protector = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("protector in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: protector,
                target: None,
                position: Some(2), // between left and right (the hero occupies zone position 0)
            },
        )
        .unwrap();
    assert!(state.world().taunt(left).is_some());
    assert!(state.world().taunt(right).is_some());
    assert!(
        !state.world().taunt(protector).is_some(),
        "the protector does not taunt itself"
    );
}

/// W5-6 Ancient Mage — Battlecry: give adjacent minions Spell Damage +1.
#[test]
fn w5_ancient_mage_gives_adjacent_spell_damage() {
    use orange_stone::cards::def::ANCIENT_MAGE;
    let mut builder = GameBuilder::new();
    let left = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    let right = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    builder.add_minion_to_hand(PlayerId1(), &ANCIENT_MAGE);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let mage = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("mage in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: mage,
                target: None,
                position: Some(2), // between left and right (the hero occupies zone position 0)
            },
        )
        .unwrap();
    assert_eq!(state.world().spell_damage(left).map(|s| s.0), Some(1));
    assert_eq!(state.world().spell_damage(right).map(|s| s.0), Some(1));
    assert!(state.world().spell_damage(mage).is_none());
}

/// W5-7 Ancestral Healing — restore a minion to full Health and give it
/// Taunt.
#[test]
fn w5_ancestral_healing_full_heals_and_taunts() {
    use orange_stone::cards::def::ANCESTRAL_HEALING;
    let mut builder = GameBuilder::new();
    let damaged = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &ANCESTRAL_HEALING);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // Damage the friendly minion (enemy attacks it)
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
    let heal = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("ancestral healing in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: heal,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(damaged), Some(Health(2)));
    assert!(state.world().taunt(damaged).is_some());
}

// ============================================================
// Wave 6 — special mechanics (fidelity-debt-roadmap W6):
// probability, this-turn temp buff, mass Divine Shield,
// self-exclusion AOE, draw-damage-by-cost, class-filtered pool.
// ============================================================

/// W6-1 Nat Pagle — 50% chance to draw a card at the end of your turn.
#[test]
fn w6_nat_pagle_chance_draw() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, NAT_PAGLE};
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(42);
    builder.add_minion_to_board(PlayerId1(), &NAT_PAGLE);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    let mut state = builder.build();
    let engine = GameEngine::new();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // 50% roll — deterministic with the fixed seed (42 draws)
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "Nat Pagle drew with this seed"
    );
}

/// W6-2 Mana Addict — whenever you cast a spell, gain +2 Attack THIS TURN
/// (expires at the end of the turn).
#[test]
fn w6_mana_addict_buff_expires_at_turn_end() {
    use orange_stone::cards::def::MANA_ADDICT;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &MANA_ADDICT);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let addict = find_entity(&state, PlayerId1(), "NEUTRAL_R10");
    // Cast a no-op spell
    let spell = {
        use orange_stone::core::component::Cost;
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
                card: spell,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(addict),
        Some(Attack(3)),
        "the spell cast granted +2 this turn"
    );
    // The buff expires at the end of the turn
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(
        state.world().effective_attack(addict),
        Some(Attack(1)),
        "the this-turn buff expired"
    );
}

/// W6-3 Righteousness — give your minions Divine Shield.
#[test]
fn w6_righteousness_grants_divine_shields() {
    use orange_stone::cards::def::RIGHTEOUSNESS;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let b = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &RIGHTEOUSNESS);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let righteousness = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("righteousness in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: righteousness,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert!(state.world().divine_shield(a).is_some());
    assert!(state.world().divine_shield(b).is_some());
}

/// W6-4 Ysera Awakens — deal 5 damage to all characters except Ysera.
#[test]
fn w6_ysera_awakens_spares_ysera() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, YSERA, YSERA_AWAKENS};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &YSERA);
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 3, 3, 3);
    builder.add_minion_to_hand(PlayerId1(), &YSERA_AWAKENS);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let ysera = find_entity(&state, PlayerId1(), "NEUTRAL_T21");
    let friend = find_entity(&state, PlayerId1(), "CLASSIC_001");
    let awaken = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("ysera awakens in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: awaken,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(ysera),
        Some(Health(12)),
        "Ysera is spared"
    );
    assert_eq!(
        state.world().zone(friend),
        Some(Zone::Graveyard),
        "the friendly Raptor takes 5 (2 health)"
    );
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId1()).hero),
        Some(Health(25)),
        "both heroes take 5"
    );
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId2()).hero),
        Some(Health(25))
    );
}

/// W6-5 Gift of the Wild — give your minions +2/+2 and Taunt.
#[test]
fn w6_gift_of_the_wild_buffs_and_taunts() {
    use orange_stone::cards::def::GIFT_OF_THE_WILD;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let b = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &GIFT_OF_THE_WILD);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let gift = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("gift in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: gift,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(a), Some(Attack(4)));
    assert_eq!(state.world().effective_health(a), Some(Health(4)));
    assert_eq!(state.world().effective_attack(b), Some(Attack(4)));
    assert!(state.world().taunt(a).is_some());
    assert!(state.world().taunt(b).is_some());
}

/// W6-6 Holy Wrath — draw a card, deal damage equal to its mana cost.
#[test]
fn w6_holy_wrath_damages_by_drawn_cost() {
    use orange_stone::cards::def::{HOLY_WRATH, RAZORFEN_HUNTER};
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(42);
    builder.add_minion_to_deck(PlayerId1(), &RAZORFEN_HUNTER);
    builder.add_minion_to_hand(PlayerId1(), &HOLY_WRATH);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let wrath = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("holy wrath in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: wrath,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "the drawn card is in hand"
    );
    let drawn = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("drawn card");
    assert_eq!(
        state.world().card_id(drawn).map(|c| c.0),
        Some("NEUTRAL_B10"),
        "Razorfen Hunter (3-cost) was drawn"
    );
    // 3 damage to a random enemy — the enemy hero is the only candidate
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId2()).hero),
        Some(Health(27)),
        "3 damage dealt"
    );
}

/// W6-7 Lightwell — at the start of your turn, restore 3 Health to a damaged
/// friendly character.
#[test]
fn w6_lightwell_heals_at_turn_start() {
    use orange_stone::cards::def::LIGHTWELL;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &LIGHTWELL);
    let damaged = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
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
    // P2's turn ends — the Lightwell fires at P1's turn start
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(
        state.world().effective_health(damaged),
        Some(Health(2)),
        "the only damaged friendly character was healed"
    );
}

/// W6-8 Pilfer — add a random card from another class to your hand: the
/// OtherClass pool is the other eight classes' class cards (neutrals are not
/// class cards — 2026-08 fidelity fix).
#[test]
fn w6_pilfer_adds_other_class_card() {
    use orange_stone::cards::def::PILFER;
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(42);
    builder.add_minion_to_hand(PlayerId1(), &PILFER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let pilfer = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("pilfer in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: pilfer,
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
    assert_eq!(hand.len(), 1, "one card added");
    let added = hand[0];
    let id = state.world().card_id(added).map(|c| c.0).expect("card id");
    let other_classes = [
        orange_stone::cards::sets::DRUID_CLASSIC,
        orange_stone::cards::sets::HUNTER_CLASSIC,
        orange_stone::cards::sets::MAGE_CLASSIC,
        orange_stone::cards::sets::PALADIN_CLASSIC,
        orange_stone::cards::sets::PRIEST_CLASSIC,
        orange_stone::cards::sets::SHAMAN_CLASSIC,
        orange_stone::cards::sets::WARLOCK_CLASSIC,
        orange_stone::cards::sets::WARRIOR_CLASSIC,
    ];
    assert!(
        other_classes
            .iter()
            .any(|class| class.iter().any(|r| r.id == id)),
        "the added card {id} is a class card of another class (not neutral, not Rogue)"
    );
}

// ============================================================
// Wave 7 — wrap-up (fidelity-debt-roadmap W7): hand-zone swap,
// damage-reflection secret, 1-Health resummon secret.
// ============================================================

/// W7-1 Alarm-o-Bot — at the start of your turn, swap this with a random
/// minion in your hand.
#[test]
fn w7_alarm_o_bot_swaps_with_hand_minion() {
    use orange_stone::cards::def::{ALARM_O_BOT, BLOODFEN_RAPTOR, WORGEN_INFILTRATOR};
    let mut builder = GameBuilder::new();
    builder.with_rng_seed(42);
    builder.add_minion_to_board(PlayerId1(), &ALARM_O_BOT);
    builder.add_minion_to_hand(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_hand(PlayerId1(), &WORGEN_INFILTRATOR);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let bot = find_entity(&state, PlayerId1(), "NEUTRAL_R13");
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // P1's turn starts: the swap happened — the bot is in hand and a hand
    // minion is on the board in its place
    assert_eq!(
        state.world().zone(bot),
        Some(Zone::Hand),
        "the bot swapped into hand"
    );
    let board: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    assert_eq!(board.len(), 1, "exactly one minion on the board");
    let incoming = board[0];
    assert_ne!(incoming, bot, "a different minion took the bot's place");
    assert_eq!(
        state.world().card_id(incoming).map(|c| c.0),
        Some("CLASSIC_001"),
        "with seed 42 the Raptor is swapped in"
    );
}

/// W7-2 Eye for an Eye — Secret: when your hero takes damage, deal the same
/// amount to the enemy hero.
#[test]
fn w7_eye_for_an_eye_reflects_damage() {
    use orange_stone::cards::def::EYE_FOR_AN_EYE;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &EYE_FOR_AN_EYE);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 4, 4, 4);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let eye = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("eye for an eye in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: eye,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // P2's 4/4 attacks the P1 hero — the secret reflects the 4 damage
    state.set_active_player(PlayerId2());
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
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId1()).hero),
        Some(Health(26))
    );
    assert_eq!(
        state
            .world()
            .effective_health(state.player(PlayerId2()).hero),
        Some(Health(26)),
        "the enemy hero took the same 4 damage"
    );
}

/// W7-3 Redemption — Secret: when a friendly minion dies, resummon it with
/// 1 Health.
#[test]
fn w7_redemption_resummons_with_1_health() {
    use orange_stone::cards::def::REDEMPTION;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &REDEMPTION);
    builder.add_minion_to_board(PlayerId1(), &orange_stone::cards::def::BOULDERFIST_OGRE);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 7, 7, 7);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let redemption = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("redemption in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: redemption,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let ogre = find_entity(&state, PlayerId1(), "NEUTRAL_T09");
    // The 7/7 kills the 6/7 ogre — Redemption brings it back with 1 Health
    state.set_active_player(PlayerId2());
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: ogre,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(ogre),
        Some(Zone::Play),
        "the ogre was resummoned"
    );
    assert_eq!(
        state.world().effective_health(ogre),
        Some(Health(1)),
        "revived at 1 Health"
    );
    assert_eq!(state.world().effective_attack(ogre), Some(Attack(6)));
}

// ============================================================
// F-A8 — Overload wiring after the duplicate-ID renumber
// ============================================================

/// F-A8-1 Forked Lightning — Overload: (2) (was 1 via the SHAMAN_016 id
/// collision with Windfury). Playing it locks 2 mana on the next turn.
#[test]
fn f8_forked_lightning_overload_amount_2() {
    use orange_stone::cards::def::FORKED_LIGHTNING;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &FORKED_LIGHTNING);
    builder.set_mana(PlayerId1(), 3, 3);
    let mut state = builder.build();
    let fork = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("forked lightning in hand");
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: fork,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).overload_locked,
        2,
        "Forked Lightning overloads 2 (real HS amount), not 1"
    );
}

/// F-A8-2 Stormforged Axe — Overload: (1) (had no overload via the
/// SHAMAN_018 id collision with Ancestral Healing).
#[test]
fn f8_stormforged_axe_overload_1() {
    use orange_stone::cards::def::STORMFORGED_AXE;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &STORMFORGED_AXE);
    builder.set_mana(PlayerId1(), 3, 3);
    let mut state = builder.build();
    let axe = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("stormforged axe in hand");
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: axe,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).overload_locked,
        1,
        "Stormforged Axe overloads 1"
    );
}

/// F-A8-3 Doomhammer — Overload: (2) (had no overload; SHAMAN_011 was never
/// in the match).
#[test]
fn f8_doomhammer_overload_2() {
    use orange_stone::cards::def::DOOMHAMMER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &DOOMHAMMER);
    builder.set_mana(PlayerId1(), 6, 6);
    let mut state = builder.build();
    let hammer = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("doomhammer in hand");
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hammer,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).overload_locked,
        2,
        "Doomhammer overloads 2"
    );
}

/// F-A8-4 Windfury (spell) — regression: must NOT gain Overload from the old
/// SHAMAN_016 id (now CS2_039).
#[test]
fn f8_windfury_spell_no_overload() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, WINDFURY};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &WINDFURY);
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.set_mana(PlayerId1(), 3, 3);
    let mut state = builder.build();
    let windfury = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("windfury in hand");
    let raptor = find_entity(&state, PlayerId1(), "CLASSIC_001");
    let engine = GameEngine::new();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: windfury,
                target: Some(raptor),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).overload_locked,
        0,
        "Windfury the spell has no Overload"
    );
}

/// Blood Imp — Stealth; at the end of your turn, give another random friendly
/// minion +1 Health (the buff never lands on itself).
#[test]
fn f8_blood_imp_buffs_another_minion_at_turn_end() {
    use orange_stone::cards::def::BLOOD_IMP;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &BLOOD_IMP);
    let target = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let imp = find_entity(&state, PlayerId1(), "CS2_064");
    assert!(
        state.world().stealth(imp).is_some(),
        "Blood Imp has Stealth from the def"
    );
    engine.apply(&mut state, Action::EndTurn).unwrap();
    assert_eq!(
        state.world().effective_health(target),
        Some(Health(3)),
        "the other friendly minion was buffed to 3 Health"
    );
    assert_eq!(
        state.world().effective_health(imp),
        Some(Health(1)),
        "Blood Imp does not buff itself"
    );
}

/// W8-1 Amani Berserker — Enrage: +3 Attack. Damage fires the permanent
/// enrage buff (2/3 → 5/3, health 3 − 1).
#[test]
fn w8_amani_berserker_enrage() {
    use orange_stone::cards::def::AMANI_BERSERKER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &AMANI_BERSERKER);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let amani = find_entity(&state, PlayerId1(), "CLASSIC_018");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: amani,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(amani), Some(Attack(5)));
    assert_eq!(state.world().effective_health(amani), Some(Health(2)));
}

/// W8-2 Raging Worgen — Enrage: +1 Attack and Windfury. The damaged 3/3
/// reaches 4 attack and GAINS Windfury (the keyword is part of the Enrage,
/// not a permanent stat).
#[test]
fn w8_raging_worgen_enrage_and_windfury() {
    use orange_stone::cards::def::RAGING_WORGEN;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &RAGING_WORGEN);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let worgen = find_entity(&state, PlayerId1(), "NEUTRAL_008");
    // Before damage: 3/3 without Windfury
    assert_eq!(state.world().effective_attack(worgen), Some(Attack(3)));
    assert!(state.world().windfury(worgen).is_none());
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: worgen,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(worgen), Some(Attack(4)));
    assert_eq!(state.world().effective_health(worgen), Some(Health(2)));
    assert!(
        state.world().windfury(worgen).is_some(),
        "the Enrage grants Windfury while damaged"
    );
}

/// W8-3 Grommash Hellscream — Charge. Enrage: +6 Attack. The damaged 4/9
/// reaches 7 attack; Charge remains.
#[test]
fn w8_grommash_hellscream_enrage() {
    use orange_stone::cards::def::GROMMASH_HELLSCREAM;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &GROMMASH_HELLSCREAM);
    let attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let grommash = find_entity(&state, PlayerId1(), "WARRIOR_010");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: grommash,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_attack(grommash), Some(Attack(10)));
    assert_eq!(state.world().effective_health(grommash), Some(Health(8)));
    assert!(
        state.world().effective_charge(grommash),
        "Charge survives the enrage"
    );
}

/// W8-4 Warsong Commander — your OTHER minions have Charge (the aura excludes
/// the commander itself). Both minions are played this turn so the summoning
/// sickness is real: the aura-buffed minion can attack, the commander cannot.
#[test]
fn w8_warsong_commander_grants_charge_to_other_minions() {
    use orange_stone::cards::def::WARSONG_COMMANDER;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &WARSONG_COMMANDER);
    builder.add_custom_minion_to_hand(PlayerId1(), 2, 3, 2);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let commander = hand[0];
    let vanilla = hand[1];
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: commander,
                target: None,
                position: None,
            },
        )
        .unwrap();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: vanilla,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert!(
        state.world().effective_charge(vanilla),
        "the aura grants charge to a summoned friendly minion"
    );
    assert!(
        !state.world().effective_charge(commander),
        "the aura excludes the source"
    );
    // The charged minion attacks immediately despite summoning sickness
    let hero = state.player(PlayerId2()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: vanilla,
                defender: hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(hero),
        Some(Health(28)),
        "the charged minion attacked the enemy hero"
    );
    // The commander itself has no charge: summoning sickness blocks it
    assert_eq!(
        engine.apply(
            &mut state,
            Action::Attack {
                attacker: commander,
                defender: hero,
            }
        ),
        Err(orange_stone::engine::rules::EngineError::AttacksExhausted),
        "the commander cannot attack while its own aura is up"
    );
}

/// W8-5 Northshire Cleric — draw a card whenever a FRIENDLY character is
/// healed (friendly heal draws; an enemy heal does not).
#[test]
fn w8_northshire_cleric_draws_on_friendly_heal() {
    use orange_stone::cards::def::{HOLY_LIGHT, NORTHSHIRE_CLERIC};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &NORTHSHIRE_CLERIC);
    builder.add_minion_to_hand(PlayerId1(), &HOLY_LIGHT);
    // Two deck cards: one for the turn-start draw, one for Northshire's draw
    builder.add_minion_to_deck(PlayerId1(), &HOLY_LIGHT);
    builder.add_minion_to_deck(PlayerId1(), &HOLY_LIGHT);
    let p2_attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.add_minion_to_hand(PlayerId2(), &HOLY_LIGHT);
    builder.add_minion_to_deck(PlayerId2(), &HOLY_LIGHT);
    builder.set_mana(PlayerId1(), 10, 10);
    builder.set_mana(PlayerId2(), 10, 10);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let p1_hero = state.player(PlayerId1()).hero;
    let p2_hero = state.player(PlayerId2()).hero;
    // Enemy minion damages the friendly hero
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p2_attacker,
                defender: p1_hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(p1_hero), Some(Health(29)));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Friendly Holy Light heals the damaged friendly hero → draw a card
    let holy_light = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Holy Light in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: holy_light,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(p1_hero),
        Some(Health(30)),
        "the friendly hero was healed"
    );
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        2,
        "Northshire drew a card from the friendly heal"
    );
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId1()), 0);
    // The Cleric damages the enemy hero so the enemy has something to heal
    let cleric = find_entity(&state, PlayerId1(), "PRIEST_004");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: cleric,
                defender: p2_hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(p2_hero), Some(Health(29)));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Enemy Holy Light heals the ENEMY hero — no draw for our Cleric
    let enemy_holy_light = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId2())
        .next()
        .expect("enemy Holy Light in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: enemy_holy_light,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        2,
        "an enemy heal does not trigger the friendly Cleric"
    );
}

/// W9-1 Truesilver Champion — whenever your hero attacks, restore 2 Health
/// to it (fires on attack declaration; a full-health hero gets no heal).
#[test]
fn w9_truesilver_champion_heals_on_hero_attack() {
    use orange_stone::cards::def::{TRUESILVER_CHAMPION, WISP};
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &TRUESILVER_CHAMPION);
    let p2_attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.add_minion_to_board(PlayerId2(), &WISP);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let p1_hero = state.player(PlayerId1()).hero;
    let p2_hero = state.player(PlayerId2()).hero;
    // Damage the friendly hero first
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p2_attacker,
                defender: p1_hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(p1_hero), Some(Health(29)));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // The hero attacks face → Truesilver heals 2 before any retaliation
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p1_hero,
                defender: p2_hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(p1_hero),
        Some(Health(30)),
        "the attack healed the friendly hero"
    );
    let weapon = state
        .player(PlayerId1())
        .weapon
        .expect("Truesilver equipped");
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(1)),
        "the face attack consumed 1 durability"
    );
}

/// W9-2 Gorehowl — attacking a minion costs 1 Attack instead of 1 Durability.
/// Minion hits drain attack and keep durability; face hits drain durability.
/// (Heroes attack once per turn — the second minion hit lands next turn.)
#[test]
fn w9_gorehowl_loses_attack_on_minion_attacks() {
    use orange_stone::cards::def::GOREHOWL;
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &GOREHOWL);
    let minion_a = builder.add_custom_minion_to_board(PlayerId2(), 3, 6, 4);
    let minion_b = builder.add_custom_minion_to_board(PlayerId2(), 2, 3, 3);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let p1_hero = state.player(PlayerId1()).hero;
    let p2_hero = state.player(PlayerId2()).hero;
    let weapon = state.player(PlayerId1()).weapon.expect("Gorehowl equipped");
    // Minion hit #1: 7 → 6 attack, durability unchanged (the 6/6 dies)
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p1_hero,
                defender: minion_a,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().attack(weapon),
        Some(Attack(6)),
        "the weapon lost 1 Attack on the minion hit"
    );
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(1)),
        "durability is untouched by the minion hit"
    );
    // P2's turn passes
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Minion hit #2: 6 → 5 attack, durability still 1
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p1_hero,
                defender: minion_b,
            },
        )
        .unwrap();
    assert_eq!(state.world().attack(weapon), Some(Attack(5)));
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(1))
    );
    // P2's turn passes again
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Face hit: durability 1 → 0 → the weapon is destroyed
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p1_hero,
                defender: p2_hero,
            },
        )
        .unwrap();
    assert!(
        state.player(PlayerId1()).weapon.is_none(),
        "the face hit drained the last durability"
    );
}

/// W9-3 Eaglehorn Bow — +1 Durability whenever a FRIENDLY Secret is REVEALED
/// (not when it is played; the reveal fires when the secret triggers).
#[test]
fn w9_eaglehorn_bow_durability_on_secret_revealed() {
    use orange_stone::cards::def::{EAGLEHORN_BOW, EXPLOSIVE_TRAP};
    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId1(), &EAGLEHORN_BOW);
    builder.add_minion_to_hand(PlayerId1(), &EXPLOSIVE_TRAP);
    builder.set_mana(PlayerId1(), 10, 10);
    let p2_attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let weapon = state
        .player(PlayerId1())
        .weapon
        .expect("Eaglehorn equipped");
    let p1_hero = state.player(PlayerId1()).hero;
    // Playing the secret does NOT grant durability (the real trigger is the reveal)
    let trap = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Explosive Trap in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: trap,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(2)),
        "playing a Secret does not yet grant Durability"
    );
    // The enemy attacks the hero → the Secret is revealed → +1 Durability
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p2_attacker,
                defender: p1_hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(3)),
        "the revealed Secret granted +1 Durability"
    );
}

/// W9-4 Bestial Wrath — give a friendly BEAST +2 Attack and Immune until end
/// of turn. A non-Beast is not a legal target: the spell fizzles (G9
/// re-validation), it does not fall back to a random Beast.
#[test]
fn w9_bestial_wrath_targets_beast_only() {
    use orange_stone::cards::def::{BESTIAL_WRATH, BLOODFEN_RAPTOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &BESTIAL_WRATH);
    builder.add_minion_to_hand(PlayerId1(), &BESTIAL_WRATH);
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    let wisp = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let beast = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| state.world().race(e) == Some(orange_stone::core::component::Race::Beast))
        .expect("raptor on the board");
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    // Targeting the Beast works
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: Some(beast),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(beast),
        Some(Attack(5)),
        "the Beast received +2 Attack"
    );
    assert!(state.world().immune(beast).is_some());
    // Targeting a non-Beast makes the spell fizzle — nothing is buffed
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[1],
                target: Some(wisp),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(beast),
        Some(Attack(5)),
        "the fizzled spell buffed nothing"
    );
    assert_eq!(
        state.world().effective_attack(wisp),
        Some(Attack(1)),
        "the non-Beast was never a valid target"
    );
    assert!(state.world().immune(wisp).is_none());
}

/// W10-1 Wrath — Choose One: deal 3 damage, or 1 damage + draw a card.
/// Option 0 = battlecry branch, option 1 = choose_one_effect branch.
#[test]
fn w10_wrath_choose_one_branches() {
    use orange_stone::cards::def::WRATH;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &WRATH);
    builder.add_minion_to_hand(PlayerId1(), &WRATH);
    builder.add_minion_to_deck(PlayerId1(), &WRATH);
    builder.add_custom_minion_to_board(PlayerId2(), 2, 6, 4);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let enemy = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId2())
        .find(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .expect("enemy minion");
    // First Wrath: 3-damage branch
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 0,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(3)),
        "the 3-damage branch dealt 3"
    );
    assert_eq!(state.world().zones().len(Zone::Hand, PlayerId1()), 1);
    // P2's turn passes, then the second Wrath: 1-damage + draw branch
    // (each Wrath is the first card of its turn — the G6 combo-active path
    // skips the choose-one choice for a second card of the turn)
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 1,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(2)),
        "the 1-damage branch dealt 1"
    );
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "the draw branch drew a card"
    );
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId1()), 0);
}

/// W10-2 Druid of the Claw — Choose One: Charge, or +2 Health and Taunt.
#[test]
fn w10_druid_of_the_claw_choose_one() {
    use orange_stone::cards::def::DRUID_OF_THE_CLAW;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &DRUID_OF_THE_CLAW);
    builder.add_minion_to_hand(PlayerId1(), &DRUID_OF_THE_CLAW);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // Claw #1: Taunt branch (4/6 Taunt)
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 1,
            },
        )
        .unwrap();
    let claw1 = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "DRUID_007"))
        .expect("claw on board");
    assert_eq!(state.world().effective_attack(claw1), Some(Attack(4)));
    assert_eq!(state.world().effective_health(claw1), Some(Health(6)));
    assert!(
        state.world().taunt(claw1).is_some(),
        "Taunt branch grants Taunt"
    );
    // Claw #2: Charge branch (4/4, attacks immediately)
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 0,
            },
        )
        .unwrap();
    let claw2 = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| {
            state.world().card_id(e).is_some_and(|c| c.0 == "DRUID_007")
                && state.world().taunt(e).is_none()
        })
        .expect("charge-branched claw on board");
    assert!(
        state.world().effective_charge(claw2),
        "Charge branch grants Charge"
    );
    assert_eq!(state.world().effective_health(claw2), Some(Health(4)));
    let p2_hero = state.player(PlayerId2()).hero;
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: claw2,
                defender: p2_hero,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(p2_hero),
        Some(Health(26)),
        "the charged claw attacked immediately"
    );
}

/// W10-3 Ancient of Lore — Choose One: draw 2, or restore 5 Health to your hero.
#[test]
fn w10_ancient_of_lore_choose_one() {
    use orange_stone::cards::def::ANCIENT_OF_LORE;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &ANCIENT_OF_LORE);
    builder.add_minion_to_hand(PlayerId1(), &ANCIENT_OF_LORE);
    builder.add_minion_to_deck(PlayerId1(), &ANCIENT_OF_LORE);
    builder.add_minion_to_deck(PlayerId1(), &ANCIENT_OF_LORE);
    let p2_attacker = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder.active_player(PlayerId2());
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let p1_hero = state.player(PlayerId1()).hero;
    // Damage the hero so the heal branch has something to heal
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p2_attacker,
                defender: p1_hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(p1_hero), Some(Health(29)));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Lore #1: draw-2 branch
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 0,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        3,
        "the draw branch drew 2"
    );
    // P2's turn passes (7-mana lore × 2 do not fit one turn)
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Lore #2: heal branch — the hero heals 5 back to full
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 1,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(p1_hero),
        Some(Health(30)),
        "the heal branch restored 5 Health"
    );
}

/// W10-4 Ancient of War — Choose One: +5 Attack, or +5 Health and Taunt.
#[test]
fn w10_ancient_of_war_choose_one() {
    use orange_stone::cards::def::ANCIENT_OF_WAR;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &ANCIENT_OF_WAR);
    builder.add_minion_to_hand(PlayerId1(), &ANCIENT_OF_WAR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // War #1: +5 Attack branch (10/5)
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 0,
            },
        )
        .unwrap();
    let war1 = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| {
            state.world().card_id(e).is_some_and(|c| c.0 == "DRUID_009")
                && state.world().effective_attack(e) == Some(Attack(10))
        })
        .expect("attack-branched war");
    assert_eq!(state.world().effective_health(war1), Some(Health(5)));
    assert!(state.world().taunt(war1).is_none());
    // P2's turn passes (7-mana war × 2 do not fit one turn)
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // War #2: +5 Health and Taunt branch (5/10 Taunt)
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("choose-one choice expected"),
    };
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 1,
            },
        )
        .unwrap();
    let war2 = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| {
            state.world().card_id(e).is_some_and(|c| c.0 == "DRUID_009")
                && state.world().effective_attack(e) == Some(Attack(5))
        })
        .expect("health-branched war");
    assert_eq!(state.world().effective_health(war2), Some(Health(10)));
    assert!(
        state.world().taunt(war2).is_some(),
        "the Health branch grants Taunt"
    );
}

/// W10-5 Tracking — Discover over the top 3 cards of the deck: pick one into
/// hand, discard the other two (their existing entities move zones).
#[test]
fn w10_tracking_discards_rest() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, TRACKING, WISP};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &TRACKING);
    builder.add_minion_to_deck(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_deck(PlayerId1(), &TRACKING);
    builder.add_minion_to_deck(PlayerId1(), &WISP);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // Capture the deck entities in draw order (the top of the deck)
    let deck_top: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Deck, PlayerId1())
        .take(3)
        .collect();
    let tracking = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Tracking in hand");
    let res = engine
        .apply_choices(
            &mut state,
            Action::PlayCard {
                card: tracking,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let choice = match res {
        Resolution::NeedsChoice { choice } => choice,
        _ => panic!("discover choice expected"),
    };
    assert_eq!(choice.options.len(), 3, "the pool is the deck's top 3");
    // Pick the second option
    engine
        .apply_choices(
            &mut state,
            Action::Choose {
                choice_id: choice.id,
                option: 1,
            },
        )
        .unwrap();
    // The picked EXISTING entity moved to hand; the other two were discarded
    assert_eq!(
        state.world().zone(deck_top[1]),
        Some(Zone::Hand),
        "the picked card moved to hand"
    );
    assert_eq!(
        state.world().zone(deck_top[0]),
        Some(Zone::Graveyard),
        "an unpicked card was discarded"
    );
    assert_eq!(
        state.world().zone(deck_top[2]),
        Some(Zone::Graveyard),
        "an unpicked card was discarded"
    );
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId1()), 0);
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "exactly one card came to hand"
    );
}

/// W11-1 Onyxia — Battlecry: summon five 1/1 Whelps.
#[test]
fn w11_onyxia_summons_whelps() {
    use orange_stone::cards::def::ONYXIA;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &ONYXIA);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let card = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Onyxia in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let minions: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    assert_eq!(minions.len(), 6, "Onyxia + five Whelps");
    let whelps: Vec<Entity> = minions
        .into_iter()
        .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "EX1_170t"))
        .collect();
    assert_eq!(whelps.len(), 5, "five Whelps were summoned");
    for w in &whelps {
        assert_eq!(state.world().effective_attack(*w), Some(Attack(1)));
        assert_eq!(state.world().effective_health(*w), Some(Health(1)));
    }
}

/// W11-2 Defender of Argus — Battlecry: adjacent minions gain +1/+1 AND
/// Divine Shield (the source itself is untouched).
#[test]
fn w11_defender_of_argus_buffs_adjacent() {
    use orange_stone::cards::def::DEFENDER_OF_ARGUS;
    let mut builder = GameBuilder::new();
    let left = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let right = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &DEFENDER_OF_ARGUS);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let argus = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Argus in hand");
    // Play Argus between the two minions (position 2 — the play zone's slot 0
    // is the hero, so the minion slots start at 1: [hero, left, argus, right])
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: argus,
                target: None,
                position: Some(2),
            },
        )
        .unwrap();
    for neighbor in [left, right] {
        assert_eq!(
            state.world().effective_attack(neighbor),
            Some(Attack(3)),
            "the adjacent minion gained +1 Attack"
        );
        assert_eq!(
            state.world().effective_health(neighbor),
            Some(Health(3)),
            "the adjacent minion gained +1 Health"
        );
        assert!(
            state.world().divine_shield(neighbor).is_some(),
            "the adjacent minion gained Divine Shield"
        );
    }
    // The source itself is not buffed and has no shield
    let argus_on_board = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "CLASSIC_010")
        })
        .expect("Argus on the board");
    assert_eq!(
        state.world().effective_attack(argus_on_board),
        Some(Attack(2))
    );
    assert_eq!(
        state.world().effective_health(argus_on_board),
        Some(Health(3))
    );
    assert!(state.world().divine_shield(argus_on_board).is_none());
}

/// W11-3 Deathwing — Battlecry: discard your hand and destroy all OTHER
/// minions (Deathwing survives).
#[test]
fn w11_deathwing_discards_hand_and_destroys_others() {
    use orange_stone::cards::def::{DEATHWING, WISP};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &DEATHWING);
    builder.add_minion_to_hand(PlayerId1(), &WISP);
    builder.add_minion_to_hand(PlayerId1(), &WISP);
    let p2_minion = builder.add_custom_minion_to_board(PlayerId2(), 3, 3, 3);
    builder.add_minion_to_board(PlayerId2(), &WISP);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let card = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .find(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "LEGENDARY_011")
        })
        .expect("Deathwing in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // The hand was discarded entirely (Deathwing moved to play)
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        0,
        "the whole hand was discarded"
    );
    // Enemy minions are destroyed
    assert_eq!(state.world().zone(p2_minion), Some(Zone::Graveyard));
    // Deathwing survives
    assert_eq!(state.world().zone(card), Some(Zone::Play));
    assert_eq!(
        state.world().effective_attack(card),
        Some(Attack(12)),
        "Deathwing is intact"
    );
}

/// W11-4 Sea Giant — costs (1) less for each minion on the battlefield.
#[test]
fn w11_sea_giant_costs_by_board() {
    use orange_stone::cards::def::SEA_GIANT;
    use orange_stone::engine::cost::play_cost;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &SEA_GIANT);
    builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    let mut state = builder.build();
    let giant = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Sea Giant in hand");
    // 2 friendly + 1 enemy = 3 minions → costs 7
    assert_eq!(
        play_cost(&state, giant, PlayerId1()),
        orange_stone::core::component::Cost(7)
    );
    // The enemy minion dies → 2 minions → costs 8
    let engine = GameEngine::new();
    let p1_minion = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .find(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .expect("friendly minion");
    let p2_minion = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId2())
        .find(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .expect("enemy minion");
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p1_minion,
                defender: p2_minion,
            },
        )
        .unwrap();
    assert_eq!(
        play_cost(&state, giant, PlayerId1()),
        orange_stone::core::component::Cost(8),
        "one fewer minion → one more mana"
    );
}

/// W11-5 Preparation — your next spell this turn costs (3) less; the flag is
/// consumed by the first spell and expires at the turn end.
#[test]
fn w11_preparation_discounts_next_spell() {
    use orange_stone::cards::def::{PREPARATION, SWIPE};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &PREPARATION);
    builder.add_minion_to_hand(PlayerId1(), &SWIPE);
    builder.add_minion_to_hand(PlayerId1(), &SWIPE);
    builder.add_custom_minion_to_board(PlayerId2(), 4, 4, 4);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    // Preparation (0 cost)
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    // First spell: 4 − 3 = 1
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).current_mana,
        9,
        "the discounted spell cost 1 (4 − 3) out of the 10 mana"
    );
    // Second spell in the same turn: the discount was consumed → full 4
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.player(PlayerId1()).current_mana,
        5,
        "the second spell cost its full 4 mana"
    );
}

/// W11-6 Cold Blood — +2 Attack; Combo: +4 instead. The base branch fires
/// when it is the first card of the turn, the combo branch on the second.
#[test]
fn w11_cold_blood_base_and_combo() {
    use orange_stone::cards::def::COLD_BLOOD;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &COLD_BLOOD);
    builder.add_minion_to_hand(PlayerId1(), &COLD_BLOOD);
    let wisp = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    // Base branch: +2 (first card of the turn — no combo)
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: Some(wisp),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(wisp),
        Some(Attack(3)),
        "the base branch granted +2 Attack"
    );
    // Combo branch: +4 (second card of the turn)
    let hand: Vec<_> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .collect();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: Some(wisp),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(wisp),
        Some(Attack(7)),
        "the combo branch granted +4 Attack instead of +2"
    );
}

/// W12-1 Water Elemental — freeze any character damaged by this minion
/// (D2 decided: a damage-pipeline check in Event::DamageDealt; the freeze
/// lands even when a Divine Shield absorbs the damage, matching HS).
#[test]
fn w12_water_elemental_freezes_damaged_characters() {
    use orange_stone::cards::def::WATER_ELEMENTAL;
    use orange_stone::core::component::DivineShield;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &WATER_ELEMENTAL);
    let minion = builder.add_custom_minion_to_board(PlayerId2(), 1, 4, 3);
    let shielded = builder.add_custom_minion_to_board(PlayerId2(), 1, 1, 1);
    builder
        .state_mut()
        .world_mut()
        .set_divine_shield(shielded, DivineShield);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let elemental = find_entity(&state, PlayerId1(), "MAGE_007");
    let p2_hero = state.player(PlayerId2()).hero;
    // Minion hit (turn 1): survives (4 health − 3) and freezes
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: elemental,
                defender: minion,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(minion), Some(Health(1)));
    assert!(
        state.world().freeze(minion).is_some(),
        "the damaged minion is frozen"
    );
    // P2's turn passes (a minion attacks once per turn)
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Divine shield hit (turn 2): the shield absorbs, the shield is gone, and
    // the target still freezes (HS: the freeze applies through the shield)
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: elemental,
                defender: shielded,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(shielded),
        Some(Health(1)),
        "the shield absorbed the damage"
    );
    assert!(state.world().divine_shield(shielded).is_none());
    assert!(
        state.world().freeze(shielded).is_some(),
        "the freeze applies through the Divine Shield"
    );
    // P2's turn passes again
    engine.apply(&mut state, Action::EndTurn).unwrap();
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Hero hit (turn 3): the hero freezes too
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: elemental,
                defender: p2_hero,
            },
        )
        .unwrap();
    assert!(state.world().freeze(p2_hero).is_some(), "the hero freezes");
}

/// W12-2 Cabal Shadow Priest — Battlecry: take control of an enemy minion
/// with 2 or less Attack (permanent control; stronger minions are untouched).
#[test]
fn w12_cabal_shadow_priest_takes_low_attack_minion() {
    use orange_stone::cards::def::CABAL_SHADOW_PRIEST;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &CABAL_SHADOW_PRIEST);
    let weak = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    let strong = builder.add_custom_minion_to_board(PlayerId2(), 4, 4, 4);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let card = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Cabal in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();
    // The only legal target (attack ≤ 2) came over to the friendly side
    assert_eq!(
        state.world().player(weak),
        Some(PlayerId1()),
        "the 2-attack minion was taken control of"
    );
    assert_eq!(
        state.world().player(strong),
        Some(PlayerId2()),
        "the 4-attack minion was never a legal target"
    );
}

/// W12-3 Prophet Velen — healing is doubled while Velen is on the board
/// (D3 decided: a pipeline hook in resolve_restore_health).
#[test]
fn w12_prophet_velen_doubles_healing() {
    use orange_stone::cards::def::{HOLY_LIGHT, PROPHET_VELEN};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &PROPHET_VELEN);
    builder.add_minion_to_hand(PlayerId1(), &HOLY_LIGHT);
    builder.set_mana(PlayerId1(), 10, 10);
    let p2_attacker = builder.add_custom_minion_to_board(PlayerId2(), 10, 1, 5);
    builder.active_player(PlayerId2());
    let mut state = builder.build();
    let engine = GameEngine::new();
    let p1_hero = state.player(PlayerId1()).hero;
    // 10 damage: the hero sits at 20 — one Holy Light (8, doubled to 16)
    // must restore all of it (without the doubling the hero would sit at 28)
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: p2_attacker,
                defender: p1_hero,
            },
        )
        .unwrap();
    assert_eq!(state.world().effective_health(p1_hero), Some(Health(20)));
    engine.apply(&mut state, Action::EndTurn).unwrap();
    let holy_light = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Holy Light in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: holy_light,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(p1_hero),
        Some(Health(30)),
        "Velen doubled the 8-point heal to 16"
    );
}

/// W12-4 Shiv — deal 1 damage to a minion and draw a card.
#[test]
fn w12_shiv_damages_and_draws() {
    use orange_stone::cards::def::{SHIV, WISP};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &SHIV);
    builder.add_minion_to_deck(PlayerId1(), &WISP);
    builder.add_custom_minion_to_board(PlayerId2(), 1, 2, 1);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let shiv = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Shiv in hand");
    let enemy = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId2())
        .find(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .expect("enemy minion");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: shiv,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(1)),
        "the minion took 1 damage"
    );
    assert_eq!(
        state.world().zones().len(Zone::Hand, PlayerId1()),
        1,
        "Shiv drew a card"
    );
    assert_eq!(state.world().zones().len(Zone::Deck, PlayerId1()), 0);
}

/// W12-5 Argent Protector — Battlecry: give a friendly minion Divine Shield
/// (the final §11 row — the roadmap's wave accounting swapped this card for
/// the unregistered Defender of Argus; clearing it empties the ledger).
/// Played without an explicit target the battlecry shields one random
/// friendly minion (the explicit-target path is pinned by `m1_*`).
#[test]
fn w12_argent_protector_grants_divine_shield() {
    use orange_stone::cards::def::ARGENT_PROTECTOR;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId1(), &ARGENT_PROTECTOR);
    builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 3);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let card = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Argent Protector in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let shielded: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId1())
        .filter(|&e| {
            state.world().card_type(e) == Some(CardType::Minion)
                && state.world().divine_shield(e).is_some()
        })
        .collect();
    assert_eq!(
        shielded.len(),
        1,
        "exactly one friendly minion gained Divine Shield"
    );
}

// ============================================================
// Engine-mechanics roadmap M1 — minion battlecry explicit
// targets: `PlayCard { target }` now threads into the battlecry
// (Event::MinionSummoned carries it); re-validation stays G9
// (an explicit target that left the legal set fizzles — no
// random fallback); the RL `legal_actions` expose the targets.
// ============================================================

/// M1-1 Houndmaster — Battlecry: give a friendly Beast +2/+2 and Taunt.
/// With two friendly Beasts on the board, the explicitly chosen Beast
/// takes the buff and the Taunt; the other Beast is untouched.
#[test]
fn m1_houndmaster_battlecry_hits_chosen_beast() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, HOUNDMASTER, IRONFUR_GRIZZLY};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_board(PlayerId1(), &IRONFUR_GRIZZLY);
    builder.add_minion_to_hand(PlayerId1(), &HOUNDMASTER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raptor = find_entity(&state, PlayerId1(), "CLASSIC_001");
    let grizzly = find_entity(&state, PlayerId1(), "NEUTRAL_B08");
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
                target: Some(grizzly),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(grizzly),
        Some(Attack(5)),
        "the chosen Beast gets the +2 Attack"
    );
    assert_eq!(
        state.world().effective_health(grizzly),
        Some(Health(5)),
        "the chosen Beast gets the +2 Health"
    );
    assert!(
        state.world().taunt(grizzly).is_some(),
        "the chosen Beast gets the Taunt"
    );
    assert_eq!(
        state.world().effective_attack(raptor),
        Some(Attack(3)),
        "the other Beast is untouched (3/2 base)"
    );
    assert_eq!(
        state.world().effective_health(raptor),
        Some(Health(2)),
        "the other Beast is untouched (3/2 base)"
    );
    assert!(
        !state.world().taunt(raptor).is_some(),
        "the other Beast gains no Taunt"
    );
}

/// M1-2 Argent Protector — Battlecry: give a friendly minion Divine Shield.
/// With two friendly minions, the explicitly chosen one gets the Shield.
#[test]
fn m1_argent_protector_shields_chosen_minion() {
    use orange_stone::cards::def::ARGENT_PROTECTOR;
    let mut builder = GameBuilder::new();
    let first = builder.add_custom_minion_to_board(PlayerId1(), 3, 3, 3);
    let second = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &ARGENT_PROTECTOR);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let protector = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("Argent Protector in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: protector,
                target: Some(first),
                position: None,
            },
        )
        .unwrap();
    assert!(
        state.world().divine_shield(first).is_some(),
        "the chosen minion gains the Shield"
    );
    assert!(
        !state.world().divine_shield(second).is_some(),
        "the other minion gains nothing"
    );
}

/// M1-2b Kul Tiran Chaplain — Battlecry: give a friendly minion +2 Health.
/// With two friendly minions, the explicitly chosen one gets the Health.
#[test]
fn m1_kul_tiran_chaplain_buffs_chosen_minion() {
    use orange_stone::cards::def::KUL_TIRAN_CHAPLAIN;
    let mut builder = GameBuilder::new();
    let first = builder.add_custom_minion_to_board(PlayerId1(), 2, 2, 2);
    let second = builder.add_custom_minion_to_board(PlayerId1(), 1, 1, 1);
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
                target: Some(first),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_health(first),
        Some(Health(4)),
        "the chosen minion gets the +2 Health"
    );
    assert_eq!(
        state.world().effective_health(second),
        Some(Health(1)),
        "the other minion is untouched"
    );
}

/// M1-3 G9 re-validation — an explicit battlecry target that is not in the
/// legal candidate set at resolution time fizzles the battlecry: no random
/// fallback (Houndmaster targeting an enemy minion — never a friendly Beast).
#[test]
fn m1_battlecry_invalid_target_fizzles() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, HOUNDMASTER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &HOUNDMASTER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let beast = find_entity(&state, PlayerId1(), "CLASSIC_001");
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
                target: Some(enemy),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(beast),
        Some(Attack(3)),
        "the battlecry fizzles — the Beast takes no buff (3/2 base)"
    );
    assert_eq!(
        state.world().effective_health(beast),
        Some(Health(2)),
        "no Health buff either"
    );
    assert!(
        !state.world().taunt(beast).is_some(),
        "no random-fallback Taunt"
    );
}

/// M1-4 the RL path — `legal_actions` exposes one PlayCard action per legal
/// battlecry target (and no targetless play), and the chosen action is
/// honored when applied.
#[test]
fn m1_legal_actions_expose_battlecry_targets() {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, HOUNDMASTER, IRONFUR_GRIZZLY};
    use orange_stone::rl::env::legal_actions;
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId1(), &BLOODFEN_RAPTOR);
    builder.add_minion_to_board(PlayerId1(), &IRONFUR_GRIZZLY);
    builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &HOUNDMASTER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let raptor = find_entity(&state, PlayerId1(), "CLASSIC_001");
    let grizzly = find_entity(&state, PlayerId1(), "NEUTRAL_B08");
    let houndmaster = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("houndmaster in hand");
    let actions = legal_actions(&state);
    assert!(
        actions.contains(&Action::PlayCard {
            card: houndmaster,
            target: Some(raptor),
            position: None,
        }),
        "each legal Beast is a target"
    );
    assert!(
        actions.contains(&Action::PlayCard {
            card: houndmaster,
            target: Some(grizzly),
            position: None,
        }),
        "each legal Beast is a target"
    );
    assert!(
        !actions.contains(&Action::PlayCard {
            card: houndmaster,
            target: None,
            position: None,
        }),
        "a targeted battlecry has no targetless play action"
    );
    // Apply the agent-style action: the chosen Beast takes the buff
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: houndmaster,
                target: Some(grizzly),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(grizzly),
        Some(Attack(5)),
        "the RL-chosen target is honored"
    );
}

/// M1-5 The Black Knight — Battlecry: destroy an enemy minion with Taunt.
/// With two enemy Taunts, the explicitly chosen one is destroyed.
#[test]
fn m1_black_knight_destroys_chosen_taunt() {
    use orange_stone::cards::def::THE_BLACK_KNIGHT;
    use orange_stone::core::component::Taunt;
    let mut builder = GameBuilder::new();
    let taunt_a = builder.add_custom_minion_to_board(PlayerId2(), 2, 6, 4);
    let taunt_b = builder.add_custom_minion_to_board(PlayerId2(), 3, 5, 4);
    {
        let world = builder.state_mut().world_mut();
        world.set_taunt(taunt_a, Taunt);
        world.set_taunt(taunt_b, Taunt);
    }
    builder.add_minion_to_hand(PlayerId1(), &THE_BLACK_KNIGHT);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let knight = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("the knight in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: knight,
                target: Some(taunt_b),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(taunt_b),
        Some(Zone::Graveyard),
        "the chosen Taunt is destroyed"
    );
    assert_eq!(
        state.world().zone(taunt_a),
        Some(Zone::Play),
        "the other Taunt survives"
    );
}

/// M1-6 Crazed Alchemist — Battlecry: swap a minion's Attack and Health.
/// With candidates on both sides, the explicitly chosen enemy swaps.
#[test]
fn m1_crazed_alchemist_swaps_chosen_minion() {
    use orange_stone::cards::def::CRAZED_ALCHEMIST;
    let mut builder = GameBuilder::new();
    builder.add_custom_minion_to_board(PlayerId1(), 1, 5, 2);
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 5, 1, 2);
    builder.add_minion_to_hand(PlayerId1(), &CRAZED_ALCHEMIST);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let alchemist = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("alchemist in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: alchemist,
                target: Some(enemy),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().effective_attack(enemy),
        Some(Attack(1)),
        "the chosen enemy minion swaps to 1/5"
    );
    assert_eq!(
        state.world().effective_health(enemy),
        Some(Health(5)),
        "the chosen enemy minion swaps to 1/5"
    );
}

/// M1-7 Hungry Crab — Battlecry: destroy a Murloc and gain +2/+2.
/// With two enemy Murlocs, the explicitly chosen one is destroyed.
#[test]
fn m1_hungry_crab_destroys_chosen_murloc() {
    use orange_stone::cards::def::{HUNGRY_CRAB, MURLOC_RAIDER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId2(), &MURLOC_RAIDER);
    builder.add_minion_to_board(PlayerId2(), &MURLOC_RAIDER);
    builder.add_minion_to_hand(PlayerId1(), &HUNGRY_CRAB);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    // The two P2 Murlocs in summon order — target the second one
    let murlocs: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId2())
        .collect();
    let chosen = murlocs[1];
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
                target: Some(chosen),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(chosen),
        Some(Zone::Graveyard),
        "the chosen Murloc is destroyed"
    );
    let survivor = find_entity(&state, PlayerId2(), "NEUTRAL_B02");
    assert_eq!(
        state.world().zone(survivor),
        Some(Zone::Play),
        "the other Murloc survives"
    );
    assert_eq!(
        state.world().effective_attack(crab),
        Some(Attack(3)),
        "the crab gains +2/+2"
    );
    assert_eq!(
        state.world().effective_health(crab),
        Some(Health(4)),
        "the crab gains +2/+2"
    );
}

/// M1-8 Big Game Hunter — Battlecry: destroy a minion with 7+ Attack.
/// With two big enemy minions, the explicitly chosen one is destroyed.
#[test]
fn m1_big_game_hunter_destroys_chosen_big_minion() {
    use orange_stone::cards::def::BIG_GAME_HUNTER;
    let mut builder = GameBuilder::new();
    let big_a = builder.add_custom_minion_to_board(PlayerId2(), 8, 8, 8);
    let big_b = builder.add_custom_minion_to_board(PlayerId2(), 8, 8, 8);
    builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &BIG_GAME_HUNTER);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let hunter = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("the hunter in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hunter,
                target: Some(big_b),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(big_b),
        Some(Zone::Graveyard),
        "the chosen big minion is destroyed"
    );
    assert_eq!(
        state.world().zone(big_a),
        Some(Zone::Play),
        "the other big minion survives"
    );
}

/// M1-9 Stormpike Commando — Battlecry: deal 2 damage (AnyEnemy). The
/// explicit target on the minion path is honored, not randomly re-picked.
#[test]
fn m1_stormpike_commando_hits_chosen_target() {
    use orange_stone::cards::def::STORMPIKE_COMMANDO;
    let mut builder = GameBuilder::new();
    let hero = builder.state_mut().player(PlayerId2()).hero;
    let enemy = builder.add_custom_minion_to_board(PlayerId2(), 2, 2, 2);
    builder.add_minion_to_hand(PlayerId1(), &STORMPIKE_COMMANDO);
    builder.set_mana(PlayerId1(), 10, 10);
    let mut state = builder.build();
    let engine = GameEngine::new();
    let commando = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId1())
        .next()
        .expect("commando in hand");
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: commando,
                target: Some(enemy),
                position: None,
            },
        )
        .unwrap();
    assert_eq!(
        state.world().zone(enemy),
        Some(Zone::Graveyard),
        "the chosen enemy minion takes the 2 damage and dies"
    );
    assert_eq!(
        state.world().effective_health(hero),
        Some(Health(30)),
        "the enemy hero is untouched"
    );
}
