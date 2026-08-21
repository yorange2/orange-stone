//! Destroy is not damage (ledger F-A13).
//!
//! The engine used to enact "destroy a minion" as "the victim deals its own
//! Health to itself", which routed every destroy through the damage pipeline.
//! Three things were wrong with that, and each has a test here:
//!
//!   1. a Divine Shield absorbed the destroy and the minion lived,
//!   2. damage triggers fired (Acolyte of Pain drew a card off being destroyed),
//!   3. a heal landing before the death step could rescue the victim.
//!
//! What must NOT change: deathrattles, death ordering and the Colossal cascade
//! still run — only the "how it died" part differs.

use orange_stone::core::action::Action;
use orange_stone::core::component::{CardType, Health};
use orange_stone::core::entity::Entity;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::GameState;
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::game::GameBuilder;

const P1: PlayerId = PlayerId::Player1;
const P2: PlayerId = PlayerId::Player2;

fn minions(state: &GameState, player: PlayerId) -> Vec<Entity> {
    state
        .world()
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect()
}

fn assassinate(state: &mut GameState, victim: Entity) {
    let card = state
        .world()
        .zones()
        .iter(Zone::Hand, P1)
        .next()
        .expect("Assassinate in hand");
    GameEngine::new()
        .apply(
            state,
            Action::PlayCard {
                card,
                target: Some(victim),
                position: None,
            },
        )
        .expect("play Assassinate");
}

/// The reproduction from the ledger entry: Assassinate on an Argent Squire.
#[test]
fn destroy_ignores_divine_shield() {
    use orange_stone::cards::def::{ARGENT_SQUIRE, ASSASSINATE};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P2, &ARGENT_SQUIRE);
    builder.add_minion_to_hand(P1, &ASSASSINATE);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let squire = minions(&state, P2)[0];
    assert!(
        state.world().divine_shield(squire).is_some(),
        "the Squire starts with its shield"
    );

    assassinate(&mut state, squire);

    assert_eq!(
        state.world().zone(squire),
        Some(Zone::Graveyard),
        "destroy is not damage — the shield does not absorb it"
    );
}

/// Acolyte of Pain draws "whenever this minion takes damage". Being destroyed
/// is not taking damage, so it must not draw.
#[test]
fn destroy_does_not_fire_damage_triggers() {
    use orange_stone::cards::def::{ACOLYTE_OF_PAIN, ASSASSINATE, BLOODFEN_RAPTOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P2, &ACOLYTE_OF_PAIN);
    for _ in 0..5 {
        builder.add_minion_to_deck(P2, &BLOODFEN_RAPTOR);
    }
    builder.add_minion_to_hand(P1, &ASSASSINATE);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let acolyte = minions(&state, P2)[0];
    let hand_before = state.world().zones().iter(Zone::Hand, P2).count();

    assassinate(&mut state, acolyte);

    assert_eq!(state.world().zone(acolyte), Some(Zone::Graveyard));
    assert_eq!(
        state.world().zones().iter(Zone::Hand, P2).count(),
        hand_before,
        "no card drawn — destroy deals no damage, so the trigger never fires"
    );
}

/// Deathrattles are unchanged: Loot Hoarder still draws when destroyed.
#[test]
fn destroy_still_fires_deathrattles() {
    use orange_stone::cards::def::{ASSASSINATE, BLOODFEN_RAPTOR, LOOT_HOARDER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P2, &LOOT_HOARDER);
    for _ in 0..5 {
        builder.add_minion_to_deck(P2, &BLOODFEN_RAPTOR);
    }
    builder.add_minion_to_hand(P1, &ASSASSINATE);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let hoarder = minions(&state, P2)[0];
    let hand_before = state.world().zones().iter(Zone::Hand, P2).count();

    assassinate(&mut state, hoarder);

    assert_eq!(state.world().zone(hoarder), Some(Zone::Graveyard));
    assert_eq!(
        state.world().zones().iter(Zone::Hand, P2).count(),
        hand_before + 1,
        "the Deathrattle still draws"
    );
}

/// A destroyed minion is beyond rescue: the death step re-checks health for
/// minions damaged to 0, but a destroy marker dies regardless of how healthy
/// the minion looks when the batch is processed.
#[test]
fn a_destroyed_minion_cannot_be_healed_back() {
    use orange_stone::cards::def::{ASSASSINATE, CHILLWIND_YETI};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P2, &CHILLWIND_YETI); // 4/5, untouched
    builder.add_minion_to_hand(P1, &ASSASSINATE);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let yeti = minions(&state, P2)[0];
    assert_eq!(
        state.world().effective_health(yeti),
        Some(Health(5)),
        "at full health when the destroy lands"
    );

    assassinate(&mut state, yeti);

    assert_eq!(
        state.world().zone(yeti),
        Some(Zone::Graveyard),
        "full health does not save a destroyed minion"
    );
}
