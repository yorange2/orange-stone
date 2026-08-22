//! Guard for the third silent catch-all (see docs/play-target-gap-audit.md).
//!
//! `CardEffect::play_target` says what the player may point at, but each
//! effect's resolution carries its own `match target { … _ => return }` that
//! knows only the domains it was written for. When the two disagree the card
//! offers a target and then does **nothing** — the failure mode that made the
//! trial widening of Dark Iron Dwarf worse than the narrow original it
//! replaced.
//!
//! This test plays every card that declares a target at one of its own offered
//! targets and asserts the board actually moved. It is what makes widening a
//! domain safe to attempt: get it wrong and this fails loudly instead of
//! quietly turning the card into a no-op.

use std::collections::BTreeMap;

use orange_stone::cards::sets::ALL_CARDS;
use orange_stone::core::action::Action;
use orange_stone::core::component::{Attack, Damage};
use orange_stone::core::entity::Entity;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::GameState;
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::game::GameBuilder;

const P1: PlayerId = PlayerId::Player1;
const P2: PlayerId = PlayerId::Player2;

/// Everything a played effect could plausibly move, **keyed by entity** — a
/// positional list would make this vacuous, because a minion joining the board
/// shifts every later row and so always looks like a change.
///
/// Hand and deck sizes get their own rows: the play itself removes a card from
/// hand, which the caller normalises away.
fn fingerprint(state: &GameState) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for p in [P1, P2] {
        let hero = state.player(p).hero;
        out.insert(
            format!("{p:?}-hero"),
            format!(
                "{:?}/{:?}/{:?} armor={}",
                state.world().effective_health(hero),
                state.world().effective_attack(hero),
                state.world().damage(hero),
                state.player(p).armor,
            ),
        );
        out.insert(
            format!("{p:?}-hand"),
            state
                .world()
                .zones()
                .iter(Zone::Hand, p)
                .count()
                .to_string(),
        );
        out.insert(
            format!("{p:?}-deck"),
            state
                .world()
                .zones()
                .iter(Zone::Deck, p)
                .count()
                .to_string(),
        );
        // Player-side bookkeeping some effects write instead of touching the
        // board (Corruption's kill list, Timeway Warden's imprison pairs).
        out.insert(
            format!("{p:?}-marks"),
            format!(
                "corrupted={:?} imprisoned={:?}",
                state.player(p).corrupted,
                state.player(p).timeway_imprisoned
            ),
        );
        for e in state.world().zones().iter(Zone::Play, p) {
            out.insert(
                format!("{e:?}"),
                format!(
                    "{p:?} {:?}/{:?} dmg={:?} ench={} \
                     taunt={} shield={} stealth={} frozen={} dormant={} \
                     wf={} mwf={} charge={} rush={:?} lifesteal={} poison={} \
                     immune={} elusive={} cant_attack={} reborn={} \
                     dr={} bc={} spelldmg={:?}",
                    state.world().effective_attack(e),
                    state.world().effective_health(e),
                    state.world().damage(e),
                    state.world().enchantments(e).iter().count(),
                    state.world().taunt(e).is_some(),
                    state.world().divine_shield(e).is_some(),
                    state.world().stealth(e).is_some(),
                    state.world().freeze(e).is_some(),
                    state.world().dormant(e).is_some(),
                    state.world().windfury(e).is_some(),
                    state.world().mega_windfury(e).is_some(),
                    state.world().charge(e).is_some(),
                    state.world().rush(e).is_some(),
                    state.world().lifesteal(e).is_some(),
                    state.world().poison(e).is_some(),
                    state.world().immune(e).is_some(),
                    state.world().elusive(e).is_some(),
                    state.world().cant_attack(e).is_some(),
                    state.world().reborn(e).is_some(),
                    state.world().deathrattle(e).is_some(),
                    state.world().battlecry(e).is_some(),
                    state.world().spell_damage(e),
                ),
            );
        }
    }
    out
}

/// A board broad enough that most domains find a candidate **and** show what
/// the effect did: minions on both sides, a friendly Beast, a Deathrattle
/// minion per side, decks to draw, damaged heroes and damaged minions (so a
/// heal is visible), and a hero with Attack (so "damage equal to your hero's
/// Attack" is not silently zero).
fn rich_board(card: &'static orange_stone::cards::def::CardDef) -> (GameState, Entity) {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, CHILLWIND_YETI, LOOT_HOARDER};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P1, &BLOODFEN_RAPTOR);
    builder.add_minion_to_board(P1, &LOOT_HOARDER);
    builder.add_minion_to_board(P1, &CHILLWIND_YETI);
    builder.add_minion_to_board(P2, &CHILLWIND_YETI);
    builder.add_minion_to_board(P2, &LOOT_HOARDER);
    builder.add_minion_to_board(P2, &BLOODFEN_RAPTOR);
    for _ in 0..8 {
        builder.add_minion_to_deck(P1, &CHILLWIND_YETI);
        builder.add_minion_to_deck(P2, &CHILLWIND_YETI);
    }
    builder.add_minion_to_hand(P1, card);
    builder.set_mana(P1, 10, 10);
    builder.hero_health(P1, 20);
    builder.hero_health(P2, 20);
    let mut state = builder.build();
    {
        // Damage both heroes and one minion per side, and arm P1's hero.
        let heroes = [state.player(P1).hero, state.player(P2).hero];
        for h in heroes {
            state.world_mut().set_damage(h, Damage(6));
        }
        state.world_mut().set_attack(heroes[0], Attack(3));
        let hurt: Vec<Entity> = [P1, P2]
            .iter()
            .filter_map(|&p| state.world().zones().iter(Zone::Play, p).next())
            .collect();
        for m in hurt {
            state.world_mut().set_damage(m, Damage(1));
        }
    }
    let state = state;
    let hand = state
        .world()
        .zones()
        .iter(Zone::Hand, P1)
        .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == card.id))
        .expect("card in hand");
    (state, hand)
}

/// Cards whose play-time effect is legitimately conditional — on this board the
/// condition is false, so doing nothing is correct. Reason per entry.
const CONDITIONAL: &[(&str, &str)] = &[(
    "CLASSIC_FM",
    "Faceless Manipulator copies the target's stats onto ITSELF, and the played      entity is excluded from the comparison below (otherwise every minion card      would look like it did something just by arriving). Covered instead by its      own scenario in tests/differential.rs.",
)];

#[test]
fn every_declared_target_reaches_its_resolution() {
    let conditional: BTreeMap<&str, &str> = CONDITIONAL.iter().copied().collect();
    let (mut inert, mut unoffered, mut checked) = (Vec::new(), 0usize, 0usize);

    for card in ALL_CARDS {
        let Some(effect) = card.battlecry.or(card.spell_effect) else {
            continue;
        };
        if effect.play_target().is_none() || conditional.contains_key(card.id) {
            continue;
        }
        let (state, hand_card) = rich_board(card);
        let targets: Vec<Entity> = orange_stone::rl::env::legal_actions(&state)
            .into_iter()
            .filter_map(|a| match a {
                Action::PlayCard {
                    card: c, target, ..
                } if c == hand_card => target,
                _ => None,
            })
            .collect();
        if targets.is_empty() {
            unoffered += 1;
            continue;
        }
        checked += 1;

        // Try EVERY offered target, not just the first: a Silence on a vanilla
        // minion leaves no trace, so "the first offer did nothing" is not
        // evidence the card is broken — "no offer does anything" is.
        let mut moved = false;
        for target in targets {
            let mut trial = state.clone();
            let mut before = fingerprint(&trial);
            let hand_after_play: usize =
                before[&format!("{P1:?}-hand")].parse::<usize>().unwrap() - 1;
            before.insert(format!("{P1:?}-hand"), hand_after_play.to_string());
            if GameEngine::new()
                .apply(
                    &mut trial,
                    Action::PlayCard {
                        card: hand_card,
                        target: Some(target),
                        position: None,
                    },
                )
                .is_err()
            {
                continue;
            }
            let mut after = fingerprint(&trial);
            after.remove(&format!("{hand_card:?}")); // the played minion joins on its own account
            if before != after {
                moved = true;
                break;
            }
        }
        if !moved {
            inert.push(card.id);
        }
    }

    println!("checked {checked} cards, {unoffered} had no candidate on this board");
    assert!(
        inert.is_empty(),
        "these cards offer a target and then do nothing — their resolution does \
         not handle the domain they declare: {inert:?}"
    );
}
