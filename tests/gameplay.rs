//! Integration tests — full game flow.
//!
//! Covers all Phase 1 features:
//! - Playing cards (vanilla minions)
//! - Minion trading
//! - Attacking heroes
//! - Turn switching
//! - Game over
//! - Deterministic replay

use orange_stone::cards::def::{BLOODFEN_RAPTOR, MURLOC_RAIDER, OGRE_MAGI};
use orange_stone::core::action::Action;
use orange_stone::core::component::Health;
use orange_stone::core::entity::Entity;
use orange_stone::core::event::Event;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::{GameState, Step};
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::engine::rules::EngineError;
use orange_stone::sim::game::GameBuilder;

// ============================================================
// Play card tests
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

    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();

    // Check events
    assert_eq!(log.len(), 2);
    assert!(matches!(log[0], Event::CardPlayed { .. }));
    assert!(matches!(log[1], Event::MinionSummoned { .. }));

    // The card should be on the board
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
    // No mana set; defaults to 0
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    let result = engine.apply(
        &mut state,
        Action::PlayCard {
            card,
            target: None,
            position: None,
        },
    );
    assert_eq!(result, Err(EngineError::NotEnoughMana));
}

#[test]
fn play_card_when_board_has_7_minions_fails() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    // Fill the board with 7 minions
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

    let result = engine.apply(
        &mut state,
        Action::PlayCard {
            card,
            target: None,
            position: None,
        },
    );
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

    let result = engine.apply(
        &mut state,
        Action::PlayCard {
            card,
            target: None,
            position: None,
        },
    );
    assert_eq!(result, Err(EngineError::NotYourCard));
}

// ============================================================
// Attack tests
// ============================================================

#[test]
fn attack_trade_deals_damage_both_ways() {
    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 4, 5, 3);
    let defender = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    let state = builder.build();

    let mut state_copy = state.clone();

    // Execute the attack
    let _log = engine
        .apply(&mut state_copy, Action::Attack { attacker, defender })
        .unwrap();

    // defender should take 4 damage and die (damage clears on the graveyard move)
    assert_eq!(state_copy.world().zone(defender), Some(Zone::Graveyard));

    // attacker should take 2 damage: 5 - 2 = 3
    assert_eq!(
        state_copy.world().effective_health(attacker),
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

    // defender takes 4 damage: 10 → 6, survives
    assert_eq!(
        state.world().effective_health(defender),
        Some(orange_stone::core::component::Health(6))
    );
    // attacker takes 3 damage and dies
    assert_eq!(state.world().zone(attacker), Some(Zone::Graveyard));
    // Event log contains MinionDied
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

    // Hero takes 5 damage
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(25))
    );
    // Attacker takes no damage (hero does not retaliate)
    assert_eq!(
        state.world().effective_health(attacker),
        Some(orange_stone::core::component::Health(3))
    );
    // Only one DamageDealt event (attacker → hero only)
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
        state.step(),
        Step::GameOver {
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

    // First attack
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: defender1,
            },
        )
        .unwrap();

    // Second attack should fail
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

    // Attack
    engine
        .apply(&mut state, Action::Attack { attacker, defender })
        .unwrap();

    assert_eq!(
        state.world().attacks_used(attacker),
        Some(orange_stone::core::component::AttacksUsed(1))
    );

    // Player1 ends turn → Player2's turn
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Player2 ends turn → back to Player1
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // Player1's minion attacks should have been reset
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
// Turn switching tests
// ============================================================

#[test]
fn end_turn_switches_active_player() {
    let engine = GameEngine::new();
    let mut state = GameState::new();

    let log = engine.apply(&mut state, Action::EndTurn).unwrap();

    assert_eq!(state.active_player(), PlayerId::Player2);
    assert_eq!(state.turn(), 2);
    assert_eq!(state.step(), Step::Main);
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
    // End the game first
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hero,
            },
        )
        .unwrap();

    // All actions should be rejected
    assert_eq!(
        engine.apply(&mut state, Action::EndTurn),
        Err(EngineError::GameAlreadyOver)
    );
}

// ============================================================
// Determinism tests
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

    // Execute the same action sequence
    let log1 = engine
        .apply(
            &mut state1,
            Action::PlayCard {
                card: card1a,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let log2 = engine
        .apply(
            &mut state2,
            Action::PlayCard {
                card: card1b,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(log1, log2);

    let log1 = engine
        .apply(
            &mut state1,
            Action::PlayCard {
                card: card2a,
                target: None,
                position: None,
            },
        )
        .unwrap();
    let log2 = engine
        .apply(
            &mut state2,
            Action::PlayCard {
                card: card2b,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(log1, log2);

    let log1 = engine.apply(&mut state1, Action::EndTurn).unwrap();
    let log2 = engine.apply(&mut state2, Action::EndTurn).unwrap();
    assert_eq!(log1, log2);

    // Player2 attacks (Player2 is now the active player)
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

    // Final states should match
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

    // Turn 1: Player1 plays Yeti (4/5)
    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: yeti,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert!(log.iter().any(|e| matches!(e, Event::CardPlayed { .. })));
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::MinionSummoned { .. }))
    );

    // Player1 plays Crocolisk (2/3)
    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: croc,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(log.len(), 2);

    // Summoning sickness: freshly played minions cannot attack; wait two turns for it to wear off
    // Player1 ends turn → Player2's turn
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Player2 ends turn → back to Player1's turn
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // Try attacking Ogre (6/7) with Croc (2/3) — suicidal but legal
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: croc,
                defender: ogre,
            },
        )
        .unwrap();
    // Croc takes 6 damage: 3 → -3 dies; Ogre takes 2 damage: 7 → 5
    assert_eq!(state.world().zone(croc), Some(Zone::Graveyard));
    assert_eq!(
        state.world().effective_health(ogre),
        Some(orange_stone::core::component::Health(5))
    );

    // Attack the wounded Ogre (6/5) with Yeti (4/5)
    engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: yeti,
                defender: ogre,
            },
        )
        .unwrap();
    // Yeti takes 6 damage: 5 → -1 dies; Ogre takes 4 damage: 5 → 1 survives
    assert_eq!(state.world().zone(yeti), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(ogre), Some(Zone::Play));
    assert_eq!(
        state.world().effective_health(ogre),
        Some(orange_stone::core::component::Health(1))
    );

    // Player1 ends turn
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // Now Player2's turn; Ogre can attack; first get the hero entity
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

    // Hero takes 6 damage
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(24))
    );
}

// ============================================================
// Phase 3: Weapon tests
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
        state.world().effective_attack(weapon),
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

    // Weapon durability should drop from 2 to 1
    let weapon = state.player(PlayerId::Player1).weapon.unwrap();
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(1))
    );
    // Enemy hero should take 3 damage (weapon attack)
    assert_eq!(
        state.world().effective_health(defender_hero),
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
    // First attack: durability 2 → 1
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

    // Need two end-turns before attacking again (hero attacks once per turn)
    engine.apply(&mut state, Action::EndTurn).unwrap(); // Player2's turn
    engine.apply(&mut state, Action::EndTurn).unwrap(); // Back to Player1

    // Second attack: durability 1 → 0, weapon should be destroyed
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
// Phase 3: Armor tests
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

    // Hero takes 3 damage; armor should absorb it all: 5 → 2
    assert_eq!(state.player(PlayerId::Player2).armor, 2);
    // Health should not decrease (fully absorbed by armor)
    assert_eq!(
        state.world().effective_health(hero),
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

    // All 3 armor is consumed
    assert_eq!(state.player(PlayerId::Player2).armor, 0);
    // Remaining 7 damage spills over to health: 30 → 23
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(23))
    );
}

// ============================================================
// Phase 3: Hero power tests
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
    // First use: succeeds
    let log = engine
        .apply(&mut state, Action::HeroPower { hero, target: None })
        .unwrap();
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::HeroPowerActivated { .. }))
    );
    // Mana should decrease by 2
    assert_eq!(state.player(PlayerId::Player1).current_mana, 8);

    // Second use: should fail (already used this turn)
    let result = engine.apply(&mut state, Action::HeroPower { hero, target: None });
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
    // Use the hero power
    engine
        .apply(&mut state, Action::HeroPower { hero, target: None })
        .unwrap();
    assert_eq!(
        state.world().hero_power_used(hero),
        Some(orange_stone::core::component::HeroPowerUsed(true))
    );

    // Player1 ends turn → Player2
    engine.apply(&mut state, Action::EndTurn).unwrap();
    // Player2 ends turn → back to Player1 (TurnStarted resets it)
    engine.apply(&mut state, Action::EndTurn).unwrap();

    // Hero power should have been reset
    assert_eq!(
        state.world().hero_power_used(hero),
        Some(orange_stone::core::component::HeroPowerUsed(false))
    );
    // Can be used again
    let result = engine.apply(&mut state, Action::HeroPower { hero, target: None });
    assert!(result.is_ok());
}

// ============================================================
// Phase 3: Aura tests
// ============================================================

#[test]
fn aura_buffs_other_friendly_minions() {
    use orange_stone::cards::def::GRIMSCALE_ORACLE;

    let mut builder = GameBuilder::new();
    // Raid Leader: other friendly minions get +1 attack
    builder.add_minion_to_board(PlayerId::Player1, &GRIMSCALE_ORACLE);
    let croc = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 3, 2);
    let state = builder.build();

    // Croc's effective attack should go from 2 to 3 (aura +1)
    assert_eq!(
        state.world().effective_attack(croc),
        Some(orange_stone::core::component::Attack(3))
    );
    // Raid Leader itself should not be buffed (OtherFriendlyMinions)
    let leader: Vec<_> = state
        .world()
        .zones()
        .iter(orange_stone::core::zone::Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .collect();
    // Leader's effective attack should be its base attack plus other auras
    // GRIMSCALE_ORACLE has 1 base attack; no self-buff
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
    // Raid Leader is killed by the enemy minion's attack
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

    // Raid Leader should die (1 HP vs 5 ATK → -4)
    assert_eq!(
        state.world().zone(leader_entity),
        Some(orange_stone::core::zone::Zone::Graveyard)
    );
    // Croc's effective attack should return to 2 (no aura bonus)
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

    // Friendly minion should get +2/+1
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
// Phase 3: Secret tests
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

    // Equip a weapon for Player2
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

    // Create a secret in Player1's SetAside
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
                effect: Some(CardEffect::DealDamage {
                    amount: 2,
                    target: EffectTarget::AllEnemyMinions,
                }),
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

    // Secret should be triggered
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. }))
    );
}

// ============================================================
// Phase 3: Complex ordering tests
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

    // Give the minion a deathrattle: draw a card
    state
        .world_mut()
        .set_deathrattle(minion, Deathrattle(CardEffect::DrawCard { count: 1 }));

    // Put a card into Player2's deck
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

    // Minion should die and trigger the deathrattle draw
    assert_eq!(state.world().zone(minion), Some(Zone::Graveyard));
    // The deathrattle-drawn card should be in hand
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

    // Create two secrets in Player1's SetAside
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
                effect: Some(CardEffect::DealDamage {
                    amount: 1,
                    target: EffectTarget::AnyEnemy,
                }),
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
                effect: Some(CardEffect::GainArmor {
                    amount: 5,
                    target: EffectTarget::Self_,
                }),
            },
        );
        world
            .zones_mut()
            .insert(Zone::SetAside, PlayerId::Player1, s2);
    }

    // Equip a weapon for Player2
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

    // Both secrets should be triggered
    let secret_count = log
        .iter()
        .filter(|e| matches!(e, Event::SecretRevealed { .. }))
        .count();
    assert_eq!(secret_count, 2, "Both secrets should trigger");
}

// ============================================================
// Phase 4: Hero Attack tests (Heroic Strike, Claw, Bite)
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

    // Hero's base attack should be 0
    assert_eq!(
        state.world().effective_attack(hero),
        Some(orange_stone::core::component::Attack(0))
    );

    // Find Heroic Strike in hand
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    // Play Heroic Strike
    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();

    // Check events
    assert!(log.iter().any(|e| matches!(e, Event::CardPlayed { .. })));

    // Hero attack should be 4 (enchantment, roadmap G4)
    assert_eq!(
        state.world().effective_attack(hero),
        Some(orange_stone::core::component::Attack(4))
    );

    // The spell should go to the graveyard
    assert_eq!(state.world().zone(card), Some(Zone::Graveyard));

    // End turn → hero attack should return to 0 (enchantment expired at wrap-up)
    engine.apply(&mut state, Action::EndTurn).unwrap();

    assert_eq!(
        state.world().effective_attack(hero),
        Some(orange_stone::core::component::Attack(0))
    );
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

    // Play Heroic Strike
    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
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

    // Hero now has 4 attack and can attack (no weapon needed)
    let _log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker: hero,
                defender: enemy_hero,
            },
        )
        .unwrap();

    // Enemy hero should take 4 damage
    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(orange_stone::core::component::Health(26))
    );
}

// ============================================================
// Phase 4: Grant Windfury / Charge / Double stat tests
// ============================================================

#[test]
fn grant_windfury_gives_minion_windfury() {
    use orange_stone::cards::def::WINDFURY;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &WINDFURY);
    // Put a friendly minion on the board
    let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 5, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

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

    // Minion should gain windfury
    assert!(state.world().windfury(minion).is_some());
    // max_attacks should become 2
    assert_eq!(state.world().max_attacks(minion), 2);
}

#[test]
fn double_attack_doubles_minion_attack() {
    use orange_stone::cards::def::BLESSED_CHAMPION;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BLESSED_CHAMPION);
    let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 4, 5, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

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

    // Minion attack should double from 4 to 8
    assert_eq!(
        state.world().effective_attack(minion),
        Some(orange_stone::core::component::Attack(8))
    );
}

#[test]
fn double_health_doubles_minion_health() {
    use orange_stone::cards::def::DIVINE_SPIRIT;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &DIVINE_SPIRIT);
    let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 6, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

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

    // Minion health should double from 6 to 12 (cap 30)
    assert_eq!(
        state.world().effective_health(minion),
        Some(orange_stone::core::component::Health(12))
    );
}

#[test]
fn grant_charge_allows_immediate_attack() {
    use orange_stone::cards::def::CHARGE_SPELL;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &CHARGE_SPELL);
    let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 5, 3);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    // Put the minion into summoning sickness first
    state
        .world_mut()
        .set_attacks_used(minion, orange_stone::core::component::AttacksUsed(1));

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

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

    // Minion should gain charge
    assert!(state.world().charge(minion).is_some());
    // Attacks used should reset, allowing an immediate attack
    assert_eq!(
        state.world().attacks_used(minion),
        Some(orange_stone::core::component::AttacksUsed(0))
    );
    // Attack should go from 3 to 5 (+2 bonus)
    assert_eq!(
        state.world().effective_attack(minion),
        Some(orange_stone::core::component::Attack(5))
    );
}

// ============================================================
// Tier 1: Classic set completion cards (docs/classic-cards-roadmap.md)
// ============================================================

#[test]
fn tier1_vanilla_neutrals_registered() {
    use orange_stone::cards::def::card_by_id;

    // Vanilla minions + stealth minions (simplified: vanilla)
    let checks = [
        ("NEUTRAL_T01", "Wisp", 0, 1, 1),
        ("NEUTRAL_T05", "River Crocolisk", 2, 2, 3),
        ("NEUTRAL_T08", "Chillwind Yeti", 4, 4, 5),
        ("NEUTRAL_T09", "Boulderfist Ogre", 6, 6, 7),
        ("NEUTRAL_T11", "War Golem", 7, 7, 7),
        ("NEUTRAL_T14", "Stranglethorn Tiger", 5, 5, 5),
        ("NEUTRAL_T15", "Ravenholdt Assassin", 7, 7, 5),
    ];
    for (id, name, cost, atk, hp) in checks {
        let card = card_by_id(id).unwrap_or_else(|| panic!("{name} ({id}) not registered"));
        assert_eq!(card.name, name);
        assert_eq!(card.cost, cost);
        assert_eq!(card.attack, atk);
        assert_eq!(card.health, hp);
    }
}

#[test]
fn elven_archer_battlecry_deals_one_damage() {
    use orange_stone::cards::def::ELVEN_ARCHER;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &ELVEN_ARCHER);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

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

    // Enemy has only the hero, so 1 damage must hit the hero
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(orange_stone::core::component::Health(29))
    );
}

#[test]
fn goldshire_footman_and_siegebreaker_have_taunt() {
    use orange_stone::cards::def::{GOLDSHIRE_FOOTMAN, SIEGEBREAKER};

    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &GOLDSHIRE_FOOTMAN);
    builder.add_minion_to_board(PlayerId::Player1, &SIEGEBREAKER);
    let state = builder.build();

    let minions: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .filter(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .collect();
    let footman = minions[0];
    let siegebreaker = minions[1];

    assert!(state.world().taunt(footman).is_some());
    assert!(state.world().taunt(siegebreaker).is_some());
    assert_eq!(
        state.world().effective_attack(siegebreaker),
        Some(orange_stone::core::component::Attack(5))
    );
    assert_eq!(
        state.world().effective_health(siegebreaker),
        Some(orange_stone::core::component::Health(8))
    );
}

#[test]
fn novice_engineer_battlecry_draws_card() {
    use orange_stone::cards::def::NOVICE_ENGINEER;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &NOVICE_ENGINEER);
    builder.add_minion_to_deck(PlayerId::Player1, &orange_stone::cards::def::WISP);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        )
        .unwrap();

    assert!(log.iter().any(|e| matches!(e, Event::CardDrawn { .. })));
    // The Wisp in the deck should be drawn into hand
    let hand_count = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .count();
    assert_eq!(hand_count, 1);
}

#[test]
fn raid_leader_buffs_other_friendly_minions() {
    use orange_stone::cards::def::RAID_LEADER;

    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &RAID_LEADER);
    let ally = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 3, 2);
    let state = builder.build();

    assert_eq!(
        state.world().effective_attack(ally),
        Some(orange_stone::core::component::Attack(3))
    );
}

#[test]
fn shattered_sun_cleric_buffs_friendly_minion() {
    use orange_stone::cards::def::SHATTERED_SUN_CLERIC;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &SHATTERED_SUN_CLERIC);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];

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

    // The cleric is the only friendly minion; the battlecry must buff itself
    assert_eq!(
        state.world().effective_attack(card),
        Some(orange_stone::core::component::Attack(4))
    );
    assert_eq!(
        state.world().effective_health(card),
        Some(orange_stone::core::component::Health(3))
    );
}

#[test]
fn stormwind_champion_buffs_allies_plus_one_plus_one() {
    use orange_stone::cards::def::STORMWIND_CHAMPION;

    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
    let ally = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 4, 2);
    let state = builder.build();

    assert_eq!(
        state.world().effective_attack(ally),
        Some(orange_stone::core::component::Attack(4))
    );
    assert_eq!(
        state.world().effective_health(ally),
        Some(orange_stone::core::component::Health(5))
    );
}

#[test]
fn dire_wolf_alpha_buffs_adjacent_minions() {
    use orange_stone::cards::def::DIRE_WOLF_ALPHA;

    let mut builder = GameBuilder::new();
    let left = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 1, 1);
    builder.add_minion_to_board(PlayerId::Player1, &DIRE_WOLF_ALPHA);
    let right = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 1, 1);
    let state = builder.build();

    let minions: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .collect();
    let wolf = minions[1];

    assert_eq!(
        state.world().effective_attack(left),
        Some(orange_stone::core::component::Attack(2))
    );
    assert_eq!(
        state.world().effective_attack(right),
        Some(orange_stone::core::component::Attack(2))
    );
    // The wolf is not affected by its own aura
    assert_eq!(
        state.world().effective_attack(wolf),
        Some(orange_stone::core::component::Attack(2))
    );
}

#[test]
fn loot_hoarder_deathrattle_draws_card() {
    use orange_stone::cards::def::LOOT_HOARDER;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.active_player(PlayerId::Player2);
    builder.add_minion_to_board(PlayerId::Player1, &LOOT_HOARDER);
    builder.add_minion_to_deck(PlayerId::Player1, &orange_stone::cards::def::WISP);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 5, 5);
    let mut state = builder.build();

    let hoarder: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .find(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .unwrap();

    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: hoarder,
            },
        )
        .unwrap();

    assert_eq!(state.world().zone(hoarder), Some(Zone::Graveyard));
    assert!(log.iter().any(|e| matches!(e, Event::CardDrawn { .. })));
}

#[test]
fn fiery_war_axe_and_arcanite_reaper_equip() {
    use orange_stone::cards::def::{ARCANITE_REAPER, FIERY_WAR_AXE};

    let mut builder = GameBuilder::new();
    builder.equip_weapon(PlayerId::Player1, &FIERY_WAR_AXE);
    builder.equip_weapon(PlayerId::Player1, &ARCANITE_REAPER);
    let state = builder.build();

    // The later weapon replaces the earlier one
    let weapon = state.player(PlayerId::Player1).weapon.unwrap();
    assert_eq!(
        state.world().effective_attack(weapon),
        Some(orange_stone::core::component::Attack(5))
    );
    assert_eq!(
        state.world().durability(weapon),
        Some(orange_stone::core::component::Durability(2))
    );
}

#[test]
fn explosive_trap_deals_damage_to_all_enemies() {
    use orange_stone::core::component::{CardType, Secret, SecretTrigger};
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.active_player(PlayerId::Player2);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 2);
    let mut state = builder.build();

    // Place Explosive Trap in Player1's SetAside
    {
        let world = state.world_mut();
        let secret_entity = world.spawn();
        world.set_card_type(secret_entity, CardType::Spell);
        world.set_player(secret_entity, PlayerId::Player1);
        world.set_zone(secret_entity, Zone::SetAside);
        world.set_secret(
            secret_entity,
            Secret {
                trigger: SecretTrigger::WhenEnemyMinionAttacksHero,
                effect: Some(CardEffect::DealDamage {
                    amount: 2,
                    target: EffectTarget::AllEnemies,
                }),
            },
        );
        world
            .zones_mut()
            .insert(Zone::SetAside, PlayerId::Player1, secret_entity);
    }

    let defender_hero = state.player(PlayerId::Player1).hero;

    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: defender_hero,
            },
        )
        .unwrap();

    // Secret is revealed
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. }))
    );
    // Enemy hero takes 2 damage
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(orange_stone::core::component::Health(28))
    );
    // Enemy minion takes 2 damage
    assert_eq!(
        state.world().effective_health(attacker),
        Some(orange_stone::core::component::Health(1))
    );
    // Attack still resolves: own hero takes the attacker's 2 damage (matches Hearthstone rules; traps do not cancel the attack)
    assert_eq!(
        state.world().effective_health(defender_hero),
        Some(orange_stone::core::component::Health(28))
    );
}

#[test]
fn freezing_trap_returns_attacker_to_hand_with_cost_increase() {
    use orange_stone::core::component::{CardType, Cost, Secret, SecretTrigger};
    use orange_stone::core::effect::CardEffect;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.active_player(PlayerId::Player2);
    let attacker = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 3, 3);
    let mut state = builder.build();

    // Place Freezing Trap in Player1's SetAside
    {
        let world = state.world_mut();
        let secret_entity = world.spawn();
        world.set_card_type(secret_entity, CardType::Spell);
        world.set_player(secret_entity, PlayerId::Player1);
        world.set_zone(secret_entity, Zone::SetAside);
        world.set_secret(
            secret_entity,
            Secret {
                trigger: SecretTrigger::WhenEnemyMinionAttacksHero,
                effect: Some(CardEffect::ReturnToHandAndIncreaseCost { amount: 2 }),
            },
        );
        world
            .zones_mut()
            .insert(Zone::SetAside, PlayerId::Player1, secret_entity);
    }

    let defender_hero = state.player(PlayerId::Player1).hero;

    let log = engine
        .apply(
            &mut state,
            Action::Attack {
                attacker,
                defender: defender_hero,
            },
        )
        .unwrap();

    // Secret is revealed; the attacking minion is returned to its owner's hand
    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. }))
    );
    assert_eq!(state.world().zone(attacker), Some(Zone::Hand));
    // Cost increases (2): 3 -> 5
    assert_eq!(state.world().effective_cost(attacker), Some(Cost(5)));
}

// ============================================================
// C1 — explicit play targets
// ============================================================

/// Explicit target: spell damage precisely hits the chosen character (face vs minion decision surface).
#[test]
fn spell_with_explicit_target_hits_that_target() {
    use orange_stone::cards::def::FROSTBOLT;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &FROSTBOLT);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let minion_a = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
    let minion_b = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let enemy_hero = state.player(PlayerId::Player2).hero;
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: Some(enemy_hero),
                position: None,
            },
        )
        .unwrap();

    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(Health(27)),
        "explicit target hero should take exactly 3"
    );
    assert_eq!(
        state.world().effective_health(minion_a),
        Some(Health(5)),
        "untargeted minion untouched"
    );
    assert_eq!(
        state.world().effective_health(minion_b),
        Some(Health(5)),
        "untargeted minion untouched"
    );
}

/// No explicit target: falls back to the engine's random selection (total damage unchanged).
#[test]
fn spell_without_target_falls_back_to_random() {
    use orange_stone::cards::def::FROSTBOLT;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &FROSTBOLT);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let minion = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let enemy_hero = state.player(PlayerId::Player2).hero;
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

    let dmg_hero = 30 - state.world().effective_health(enemy_hero).unwrap().0;
    let dmg_minion = 5 - state.world().effective_health(minion).unwrap().0;
    assert_eq!(dmg_hero + dmg_minion, 3, "exactly 3 damage lands somewhere");
    assert!(dmg_hero == 3 || dmg_minion == 3);
}

/// Explicit target not in the candidate set (e.g. own minion) → falls back to random selection.
#[test]
fn invalid_explicit_target_falls_back_to_random() {
    use orange_stone::cards::def::FROSTBOLT;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &FROSTBOLT);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let own_minion = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 5, 3);
    let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    // Own minion is not an "enemy character" candidate — falls back to random; own minion must not be damaged
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: Some(own_minion),
                position: None,
            },
        )
        .unwrap();

    assert_eq!(
        state.world().effective_health(own_minion),
        Some(Health(5)),
        "own minion never damaged"
    );
    let enemy_hero = state.player(PlayerId::Player2).hero;
    let dmg = 30 - state.world().effective_health(enemy_hero).unwrap().0 + 5
        - state.world().effective_health(enemy_minion).unwrap().0;
    assert_eq!(dmg, 3, "3 damage lands on an enemy");
}

/// Battlecry damage: the explicit target is hit precisely.
#[test]
fn battlecry_with_explicit_target_hits_that_target() {
    use orange_stone::cards::def::IRONFORGE_RIFLEMAN;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &IRONFORGE_RIFLEMAN);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let minion_a = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: Some(minion_a),
                position: None,
            },
        )
        .unwrap();

    assert_eq!(
        state.world().effective_health(minion_a),
        Some(Health(4)),
        "battlecry should hit the explicit target"
    );
}
