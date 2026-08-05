//! Tier 2 integration tests — Classic set cards with new mechanics.
//!
//! Organized in batches by mechanic:
//! - Foundation: playable secret cards, weapons with battlecries
//! - Rogue: combo/shadowstep/betrayal/blade flurry/poison/stealth
//! - Secret redirection: misdirection/noble sacrifice/spellbender
//! - Cost reduction, immunity, mind control, transform, etc.

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
// Stage 1 Foundation
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

    // Secret card should go to SetAside, not the graveyard
    assert_eq!(state.world().zone(card), Some(Zone::SetAside));
    // Should have a Secret component attached
    let secret = state.world().secret(card);
    assert!(secret.is_some(), "secret card should have Secret component");
    let secret = secret.unwrap();
    assert_eq!(
        secret.trigger,
        orange_stone::core::component::SecretTrigger::WhenEnemyMinionAttacksHero
    );
    // Effect comes from the battlecry slot (2 damage, all enemies)
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

    // Player1 plays Explosive Trap
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
    assert_eq!(state.world().zone(card), Some(Zone::SetAside));

    // Player2's turn to attack
    state.set_active_player(PlayerId::Player2);

    // Enemy minion attacks Player1's hero → secret triggers
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
    // Attacking minion takes 2 damage
    assert_eq!(
        state.world().effective_health(attacker),
        Some(orange_stone::core::component::Health(1))
    );
    // Enemy hero also takes 2 damage (all enemies)
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
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

    // Player2's turn to attack
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

    // Attacking minion returns to hand, cost +2
    assert_eq!(state.world().zone(attacker), Some(Zone::Hand));
    assert_eq!(
        state.world().effective_cost(attacker),
        Some(orange_stone::core::component::Cost(5))
    );
}

/// Weapon with a battlecry: equips first, then resolves the battlecry on play.
#[test]
fn weapon_with_battlecry_resolves_on_play() {
    use orange_stone::cards::def::CardDef;
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    // Test weapon: 2/2 with a 1-damage battlecry (Perdition's Blade prototype)
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

    // Weapon is equipped
    assert_eq!(state.player(PlayerId::Player1).weapon, Some(card));
    // Battlecry deals 1 damage → enemy hero at 29 HP
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(orange_stone::core::component::Health(29))
    );
}

#[test]
fn card_id_registry_lookup() {
    // All registered cards can be looked up by id
    assert!(card_by_id("HUNTER_T01").is_some());
    assert!(card_by_id("HUNTER_T02").is_some());
}

// ============================================================
// Stage 2 Rogue batch
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

    // No combo: deals 2 damage, goes to the graveyard
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
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

    // Play a card first to activate combo
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: wisp,
                target: None,
                position: None,
            },
        )
        .unwrap();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: headcrack,
                target: None,
                position: None,
            },
        )
        .unwrap();

    // Combo: deals 2 damage, card returns to hand
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
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
        .apply(
            &mut state,
            Action::PlayCard {
                card: wisp,
                target: None,
                position: None,
            },
        )
        .unwrap();
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: kidnapper,
                target: None,
                position: None,
            },
        )
        .unwrap();

    // Combo: enemy minion returns to its owner's hand
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

    // Friendly minion returns to hand, cost 3 → 1
    assert_eq!(state.world().zone(minion), Some(Zone::Hand));
    assert_eq!(
        state.world().effective_cost(minion),
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
    // Flanking minions are stealthed (cannot be single-targeted); the middle 5/5 is the only target
    let left = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
    let middle = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 5, 5);
    let right = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 2, 2);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();
    // Test helper: manually apply stealth to the flanking minions
    state.world_mut().set_stealth(left, Stealth);
    state.world_mut().set_stealth(right, Stealth);

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

    // The middle 5/5 is chosen: each flank takes 5 damage and dies (damage
    // clears on the graveyard move, so the dead minion reads its base health)
    assert_eq!(state.world().zone(left), Some(Zone::Graveyard));
    assert_eq!(state.world().zone(right), Some(Zone::Graveyard));
    // The middle target takes no damage
    assert_eq!(
        state.world().effective_health(middle),
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

    // Weapon is destroyed
    assert!(state.player(PlayerId::Player1).weapon.is_none());
    // All enemies (hero + minions) take the weapon's attack (3) in damage
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(orange_stone::core::component::Health(27))
    );
    assert_eq!(
        state.world().effective_health(enemy),
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

    // Stealth component is attached via apply_card_keywords
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

    // Poison: the 3/3 minion is destroyed outright by 1 damage
    assert_eq!(state.world().zone(enemy), Some(Zone::Graveyard));
    // Assassin takes 3 retaliation damage and dies
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

    // Weapon is equipped (2/2)
    assert_eq!(state.player(PlayerId::Player1).weapon, Some(card));
    assert_eq!(
        state.world().effective_attack(card),
        Some(orange_stone::core::component::Attack(2))
    );
    assert_eq!(
        state.world().durability(card),
        Some(orange_stone::core::component::Durability(2))
    );
    // Battlecry deals 1 damage
    let enemy_hero = state.player(PlayerId::Player2).hero;
    assert_eq!(
        state.world().effective_health(enemy_hero),
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

    // Friendly minion gains stealth; Master of Disguise itself does not (cannot target itself)
    assert!(state.world().stealth(ally).is_some());
    assert!(state.world().stealth(card).is_none());
}

// ============================================================
// Stage 2 Regression: resurrect does not exceed the board cap
// ============================================================

#[test]
fn resurrect_skipped_when_board_full() {
    use orange_stone::cards::def::HIGH_INQUISITOR_WHITEMANE;

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    // 6 minions occupy the board
    for _ in 0..6 {
        builder.add_custom_minion_to_board(PlayerId::Player1, 1, 1, 1);
    }
    // A dead minion awaiting resurrection
    let dead = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 2, 2);
    builder.add_minion_to_hand(PlayerId::Player1, &HIGH_INQUISITOR_WHITEMANE);
    builder.set_mana(PlayerId::Player1, 10, 10);
    let mut state = builder.build();
    // Kill dead and record it in this turn's death list
    state
        .world_mut()
        .move_to_zone(dead, Zone::Graveyard)
        .unwrap();
    state.make_mut().players[PlayerId::Player1.index()]
        .died_this_turn
        .push(dead);

    // Play Whitemane (7th minion); the battlecry tries to resurrect
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

    // Board still has 7 minions (resurrect skipped); the corpse stays in the graveyard
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
// Stage 3 Secret batch
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

    // Player1 plays Misdirection
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

    // Player2 attacks Player1's hero
    state.set_active_player(PlayerId::Player2);
    let hero = state.player(PlayerId::Player1).hero;
    let enemy_hero = state.player(PlayerId::Player2).hero;
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
    // Own hero takes no damage
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(30))
    );
    // 2 damage lands on another character: enemy hero / other enemy minion / own minion
    // (counted from the event log — a killed minion's damage clears on the
    // graveyard move, so post-state health cannot measure it)
    let candidates = [enemy_hero, other_enemy, own_minion];
    let damage_spread: i32 = log
        .iter()
        .filter_map(|e| match e {
            Event::DamageDealt { target, amount, .. } if candidates.contains(target) => {
                Some(*amount)
            }
            _ => None,
        })
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

    // Player2 attacks Player1's hero → Noble Sacrifice summons a 2/1 defender
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
    // Own hero takes no damage
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(30))
    );
    // Defender is summoned and takes 3 damage (2/1 → dies)
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
    // Attacker takes 2 retaliation from the defender
    assert_eq!(
        state.world().effective_health(attacker),
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

    // Player1 plays Snipe
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

    // Player2 plays a 4/4 minion
    state.set_active_player(PlayerId::Player2);
    let played = builder_add_custom_hand(&mut state, PlayerId::Player2, 4, 4, 4);
    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: played,
                target: None,
                position: None,
            },
        )
        .unwrap();

    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. })),
        "snipe should be revealed"
    );
    // Minion is killed by 4 damage
    assert_eq!(state.world().zone(played), Some(Zone::Graveyard));
}

/// Helper: add a custom minion directly to a built state's hand (for cross-turn tests).
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

    // Player1 plays Snake Trap
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

    // Player2 attacks Player1's minion → it takes damage, summoning three snakes
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
    // Player1's board should have 3 1/1 snakes + the wounded friendly minion
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
            state.world().effective_attack(*s),
            Some(orange_stone::core::component::Attack(1))
        );
        assert_eq!(
            state.world().effective_health(*s),
            Some(orange_stone::core::component::Health(1))
        );
    }
    // Friendly minion takes 2 damage and dies
    assert_eq!(state.world().zone(friendly), Some(Zone::Graveyard));
}

#[test]
fn spellbender_redirects_spell_damage_to_itself() {
    use orange_stone::cards::def::CardDef;
    use orange_stone::cards::def::SPELLBENDER;
    use orange_stone::core::effect::{CardEffect, EffectTarget};

    // Test spell: deals 5 damage to an enemy minion (the target is necessarily the only friendly minion on board)
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

    // Player1 plays Spellbender
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

    // Player2 casts a spell (targeting Player1's minion)
    state.set_active_player(PlayerId::Player2);
    let spell = builder_add_custom_hand(&mut state, PlayerId::Player2, 0, 0, 1);
    {
        // The test spell is not in the registry; set its type and effect components manually
        let world = state.world_mut();
        world.set_card_type(spell, CardType::Spell);
        world.set_battlecry(
            spell,
            orange_stone::core::component::Battlecry(test_spell.battlecry.unwrap()),
        );
    }
    let log = engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: spell,
                target: None,
                position: None,
            },
        )
        .unwrap();

    assert!(
        log.iter()
            .any(|e| matches!(e, Event::SecretRevealed { .. })),
        "spellbender should be revealed"
    );
    // The 1/3 Spellbender is summoned and takes 5 damage
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
    // The original target minion takes no damage
    assert_eq!(
        state.world().effective_health(own_minion),
        Some(orange_stone::core::component::Health(3))
    );
}

// ============================================================
// Stage 4 Cost reduction batch
// ============================================================

#[test]
fn sorcerers_apprentice_reduces_spell_cost() {
    use orange_stone::cards::def::{FIREBALL, SORCERERS_APPRENTICE};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(PlayerId::Player1, &SORCERERS_APPRENTICE);
    builder.add_minion_to_hand(PlayerId::Player1, &FIREBALL);
    // Fireball costs 4; the apprentice reduces it by 1 → 3; only 3 mana is available
    builder.set_mana(PlayerId::Player1, 3, 3);
    let mut state = builder.build();

    let hand: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    let card = hand[0];
    // Effective cost is 3
    assert_eq!(
        state.world().effective_cost(card),
        Some(orange_stone::core::component::Cost(3))
    );

    // Playable with 3 mana
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
    assert!(
        log.iter().any(|e| matches!(e, Event::SpellCast { .. })),
        "fireball should be cast with discounted cost"
    );
    // Mana is deducted by 3 (not 4)
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

    // 6-cost Ogre → 4; 0-cost Wisp → at least 1
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

    // Play Kirin Tor Mage (3 cost)
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: ktm,
                target: None,
                position: None,
            },
        )
        .unwrap();
    assert_eq!(state.player(PlayerId::Player1).current_mana, 7);
    assert!(state.player(PlayerId::Player1).next_secret_free);

    // Next secret is free
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
    // Mana is not deducted
    assert_eq!(state.player(PlayerId::Player1).current_mana, 7);
    // One-time effect has been consumed
    assert!(!state.player(PlayerId::Player1).next_secret_free);
    // Secret is attached in SetAside
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

    // The drawn Ogre Magi (4 cost) has its cost reduced by 3 → 1
    let drawn: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, PlayerId::Player1)
        .collect();
    assert_eq!(drawn.len(), 1);
    assert_eq!(
        state.world().effective_cost(drawn[0]),
        Some(orange_stone::core::component::Cost(1))
    );
}

// ============================================================
// Stage 5 Druid batch
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

    // Random choose-one: either all minions get +2/+2, or two 2/2 treants are summoned
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
                state.world().effective_attack(*t),
                Some(orange_stone::core::component::Attack(2))
            );
            assert_eq!(
                state.world().effective_health(*t),
                Some(orange_stone::core::component::Health(2))
            );
        }
    } else {
        // +2/+2 branch: friendly minion goes 2/3 → 4/5
        assert_eq!(
            state.world().effective_attack(ally),
            Some(orange_stone::core::component::Attack(4))
        );
        assert_eq!(
            state.world().effective_health(ally),
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

    // Find the taunt minion (Goldshire Footman)
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

    // Random choose-one: 2 damage (on the taunt minion or the enemy hero) or silence the taunt minion
    let enemy_hero = state.player(PlayerId::Player2).hero;
    let footman_dead = state.world().zone(footman) == Some(Zone::Graveyard);
    let hero_hp = state.world().effective_health(enemy_hero).unwrap().0;
    if state.world().taunt(footman).is_none() {
        // Silence branch: taunt is removed
    } else {
        // Damage branch: the taunt minion (1/2, dies to 2 damage) or the hero takes 2 damage
        assert!(
            footman_dead || hero_hp == 28,
            "damage branch should hit the footman (1/2) or the enemy hero, got footman_dead={footman_dead} hero={hero_hp}"
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

    // Friendly minion gains a deathrattle: summon a 2/2 treant
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

    // Kill the friendly minion → deathrattle summons a treant
    let attacker = builder_add_custom_hand(&mut state, PlayerId::Player1, 0, 0, 0);
    // Kill it directly with a damage event: via an attack
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
    // Deathrattle summoned a 2/2 treant
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

    // Opponent has 2 bananas in hand
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
// Stage 6 Immunity batch
// ============================================================

#[test]
fn bestial_wrath_grants_attack_and_immune_until_turn_end() {
    use orange_stone::cards::def::{BESTIAL_WRATH, WISP};

    let engine = GameEngine::new();
    let mut builder = GameBuilder::new();
    builder.add_minion_to_hand(PlayerId::Player1, &BESTIAL_WRATH);
    // Only minion on the board (Wisp) → guaranteed target
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

    // Gains +2 attack and immunity
    let beast: Entity = state
        .world()
        .zones()
        .iter(Zone::Play, PlayerId::Player1)
        .find(|&e| {
            state.world().card_type(e) == Some(orange_stone::core::component::CardType::Minion)
        })
        .unwrap();
    assert_eq!(
        state.world().effective_attack(beast),
        Some(orange_stone::core::component::Attack(3))
    );
    assert!(state.world().immune(beast).is_some());

    // Enemy attacks it: immunity ignores damage (no retaliation taken; it does not die)
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
        state.world().effective_health(beast),
        Some(orange_stone::core::component::Health(1)),
        "immune minion should not take damage"
    );

    // End of turn → immunity is cleared
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

    // Hero takes 0 damage (immune); defender takes 5 damage and dies
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(30))
    );
    assert_eq!(state.world().zone(defender), Some(Zone::Graveyard));
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

    // The enemy's only minion is frozen (no damage)
    assert!(state.world().freeze(enemy).is_some());
    assert_eq!(
        state.world().effective_health(enemy),
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
    // Pre-freeze it
    state.world_mut().set_freeze(enemy, Freeze);

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

    // Already frozen → deals 2 damage
    assert_eq!(
        state.world().effective_health(enemy),
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

    // Enemy 6 HP minion is destroyed; Natalie gains 6 health (4/5 → 4/11)
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
        state.world().effective_health(card),
        Some(orange_stone::core::component::Health(11))
    );
}

// ============================================================
// Stage 7 Control/corruption/commanding shout/overload/transform batch
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

    // Enemy minion is mind-controlled (belongs to Player1)
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player1));
    assert!(
        state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .any(|e| e == enemy_minion)
    );

    // End of turn → returned
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

    // A 5-attack minion is unaffected by Shadow Madness
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
    assert_eq!(state.world().player(enemy_minion), Some(PlayerId::Player1));

    // Still belongs to Player1 after the turn ends
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
    assert_eq!(state.world().zone(enemy_minion), Some(Zone::Play));

    // P1 ends → P2's turn → P2 ends → the corrupted minion dies at the start of P1's turn
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
    assert_eq!(state.player(PlayerId::Player1).minion_min_health, 1);

    // Enemy 3-attack minion attacks the 1/2 minion → health clamped to 1 (no death)
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
        state.world().effective_health(ally),
        Some(orange_stone::core::component::Health(1))
    );
    assert_eq!(state.world().zone(ally), Some(Zone::Play));

    // Effect is cleared at the end of the turn
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

    // Play Lightning Bolt with overload
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

    // Unbound Elemental gains +1/+1 (2/4 → 3/5)
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
        state.world().effective_attack(elemental),
        Some(orange_stone::core::component::Attack(3))
    );
    assert_eq!(
        state.world().effective_health(elemental),
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

    // Enemy minion is transformed into a 5/5 Devilsaur or a 1/1 Squirrel
    let atk = state.world().effective_attack(enemy_minion).unwrap().0;
    let hp = state.world().effective_health(enemy_minion).unwrap().0;
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
    // Effect components are cleared (no battlecry/taunt, etc.)
    assert!(state.world().battlecry(enemy_minion).is_none());
}

// ============================================================
// Stage 8 Tier 3 random card pool batch
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

    // A legendary minion is added to hand
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

    // Generates a shadow spell at end of turn
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

    // Board: Stablehand + one random beast
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
        .apply(
            &mut state,
            Action::PlayCard {
                card: antonidas,
                target: None,
                position: None,
            },
        )
        .unwrap();

    // Cast Moonfire → a Fireball is added to hand
    engine
        .apply(
            &mut state,
            Action::PlayCard {
                card: moonfire,
                target: None,
                position: None,
            },
        )
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

    // Random target: the 1/1 minion (dies → summon a demon) or the enemy hero (only takes 2 damage)
    let enemy_hero = state.player(PlayerId::Player2).hero;
    let minion_dead = state.world().zone(enemy_minion) == Some(Zone::Graveyard);
    let hero_hp = state.world().effective_health(enemy_hero).unwrap().0;
    if minion_dead {
        // Minion died → summon a random demon (the demon may die to its own battlecry in the simplified implementation, e.g. Flame Imp)
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
        // Hit the hero: hero takes 2 damage, no summon
        assert_eq!(hero_hp, 28, "enemy hero should take 2 damage");
        assert_eq!(state.world().zone(enemy_minion), Some(Zone::Play));
    }
}

// ============================================================
// Milestone B — unified attack pipeline verification
// ============================================================

/// Noble Sacrifice: after the attack is redirected, the summoned defender retaliates automatically (the unified pipeline computes retaliation from the current state).
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
        .apply(
            &mut state,
            Action::PlayCard {
                card: hand[0],
                target: None,
                position: None,
            },
        )
        .unwrap();

    // Player2 attacks Player1's hero → Noble Sacrifice summons a 2/1 defender
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

    // Defender takes 3 damage and dies, while retaliating for 2 (resolved simultaneously)
    assert_eq!(
        state.world().effective_health(hero),
        Some(orange_stone::core::component::Health(30)),
        "hero should take no damage"
    );
    assert_eq!(
        state.world().effective_health(attacker),
        Some(orange_stone::core::component::Health(1)),
        "attacker should take 2 retaliation from the defender"
    );
}

/// Weapon breaks during the attack: the attack damage still includes the weapon bonus (damage is fixed when enqueued).
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

    // All 7 weapon damage applies, even though the weapon breaks from this attack
    assert_eq!(
        state.world().effective_health(enemy_hero),
        Some(orange_stone::core::component::Health(23))
    );
    // Weapon is destroyed
    assert!(state.player(PlayerId::Player1).weapon.is_none());
}

/// A defender that dies still retaliates (Hearthstone's simultaneous resolution semantics).
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

    // Defender is killed by 4 damage (health goes negative, enters the graveyard), but its 2 retaliation still applies
    assert_eq!(
        state.world().zone(defender),
        Some(Zone::Graveyard),
        "defender should be dead"
    );
    assert_eq!(
        state.world().effective_health(attacker),
        Some(orange_stone::core::component::Health(3)),
        "attacker should take 2 retaliation even though defender died"
    );
}
