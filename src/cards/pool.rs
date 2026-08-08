//! Random pools — filtered sampling from the Classic pool.
//!
//! Pool closure is guaranteed: every sampling pool is a filtered subset of `ALL_CARDS`
//! (all Classic cards) or built-in token pools, so no cards outside the Classic set are introduced.
//!
//! Beast/Demon pools are field-driven (fidelity-debt W1): `CardDef.race` decides
//! membership, so the pools stay in sync with the card data automatically.
//! Legendary/class filtering is computed dynamically from the group lists in `sets`.

use crate::cards::def::{CardDef, card_by_id};
use crate::core::component::{CardType, Race};
use crate::core::effect::RandomPool;
use crate::core::player::PlayerId;
use crate::sim::rng::GameRng;

/// All Classic cards of the given race (field-driven — `CardDef.race`).
fn race_pool(race: Race) -> Vec<&'static CardDef> {
    crate::cards::sets::ALL_CARDS
        .iter()
        .filter(|c| c.race == Some(race))
        .collect()
}

/// Whether the card with the given ID has the given race (field-driven).
#[must_use]
pub fn card_has_race(id: &str, race: Race) -> bool {
    card_by_id(id).is_some_and(|c| c.race == Some(race))
}

/// Pilfer (and friends): "a random card from another class" — the card must
/// belong to one of the other eight classes' groups in `sets`; neutral cards
/// are not class cards. (2026-08 fidelity fix: the previous "any non-Rogue
/// card" filter also pulled neutral cards into the pool.)
/// Whether the card belongs to a class other than the given owner's class.
/// The engine has no class model — this is the pilfer-style check: the card
/// must belong to one of the OTHER eight classes' groups in `sets`. Used by
/// Jackpot! (Core Set W3b) to filter other-class spells.
pub(crate) fn is_other_class_card_for(card: &CardDef, _owner: PlayerId) -> bool {
    is_other_class_card(card)
}

fn is_other_class_card(card: &CardDef) -> bool {
    [
        crate::cards::sets::DRUID_CLASSIC,
        crate::cards::sets::HUNTER_CLASSIC,
        crate::cards::sets::MAGE_CLASSIC,
        crate::cards::sets::PALADIN_CLASSIC,
        crate::cards::sets::PRIEST_CLASSIC,
        crate::cards::sets::SHAMAN_CLASSIC,
        crate::cards::sets::WARLOCK_CLASSIC,
        crate::cards::sets::WARRIOR_CLASSIC,
        // Core Set W1 — Demon Hunter / Death Knight groups (new classes,
        // pilferable like the classic ones)
        crate::cards::sets::DEMON_HUNTER_W1,
        crate::cards::sets::DEATH_KNIGHT_W1,
    ]
    .iter()
    .any(|class| class.iter().any(|c| c.id == card.id))
}

/// Dream card pool — Classic built-in tokens (Ysera).
pub const DREAM_POOL: &[&str] = &[
    "NEUTRAL_T21a", // Emerald Drake
    "NEUTRAL_T21b", // Laughing Sister
    "NEUTRAL_T21c", // Dream
    "NEUTRAL_T21d", // Nightmare
    "NEUTRAL_T21e", // Ysera Awakens
];

/// Animal Companion pool — Classic built-in tokens.
pub const COMPANION_POOL: &[&str] = &["HUNTER_023a", "HUNTER_023b", "HUNTER_023c"];

/// Draws a random card definition from an ID pool.
pub(crate) fn random_from_pool(pool: &[&str], rng: &mut GameRng) -> Option<&'static CardDef> {
    if pool.is_empty() {
        return None;
    }
    let idx = rng.next_usize(pool.len());
    card_by_id(pool[idx])
}

/// The full card list of a pool, in a deterministic order (roadmap G6 — the
/// option list for Discover choices). Filtered pools compute the list at call
/// time.
pub(crate) fn pool_cards(pool: RandomPool) -> Vec<&'static CardDef> {
    match pool {
        RandomPool::Beast => race_pool(Race::Beast),
        RandomPool::Demon => race_pool(Race::Demon),
        RandomPool::Dream => DREAM_POOL.iter().filter_map(|id| card_by_id(id)).collect(),
        RandomPool::Dragon => race_pool(Race::Dragon),
        RandomPool::Mechanical => race_pool(Race::Mechanical),
        RandomPool::Companion => COMPANION_POOL
            .iter()
            .filter_map(|id| card_by_id(id))
            .collect(),
        RandomPool::Legendary => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && crate::cards::sets::LEGENDARY_CLASSIC
                        .iter()
                        .any(|l| l.id == c.id)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::MageSpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && crate::cards::sets::MAGE_CLASSIC
                        .iter()
                        .any(|m| m.id == c.id)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::ShadowSpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| c.card_type == CardType::Spell && c.name.contains("Shadow"))
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::Spell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| c.card_type == crate::core::component::CardType::Spell)
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::OtherClass => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| is_other_class_card(c))
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
    }
}

/// Resolves a card id reference to its static definition.
fn card_by_id_ref(card: &CardDef) -> Option<&'static CardDef> {
    card_by_id(card.id)
}

/// Draws a random card definition by pool type.
pub(crate) fn random_card(rng: &mut GameRng, pool: RandomPool) -> Option<&'static CardDef> {
    match pool {
        RandomPool::Beast => random_filtered(rng, |c| c.race == Some(Race::Beast)),
        RandomPool::Demon => random_filtered(rng, |c| c.race == Some(Race::Demon)),
        RandomPool::Dream => random_from_pool(DREAM_POOL, rng),
        RandomPool::Dragon => random_filtered(rng, |c| c.race == Some(Race::Dragon)),
        RandomPool::Mechanical => random_filtered(rng, |c| c.race == Some(Race::Mechanical)),
        RandomPool::Spell => random_filtered(rng, |c| {
            c.card_type == crate::core::component::CardType::Spell
        }),
        RandomPool::Companion => random_from_pool(COMPANION_POOL, rng),
        RandomPool::Legendary => random_filtered(rng, |c| {
            c.card_type == CardType::Minion
                && crate::cards::sets::LEGENDARY_CLASSIC
                    .iter()
                    .any(|l| l.id == c.id)
        }),
        RandomPool::MageSpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell
                && crate::cards::sets::MAGE_CLASSIC
                    .iter()
                    .any(|m| m.id == c.id)
        }),
        RandomPool::ShadowSpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell && c.name.contains("Shadow")
        }),
        RandomPool::OtherClass => random_filtered(rng, is_other_class_card),
    }
}

/// Draws randomly from the full pool after filtering by a predicate.
fn random_filtered(
    rng: &mut GameRng,
    predicate: impl Fn(&CardDef) -> bool,
) -> Option<&'static CardDef> {
    let pool: Vec<&CardDef> = crate::cards::sets::ALL_CARDS
        .iter()
        .filter(|c| predicate(c))
        .collect();
    if pool.is_empty() {
        return None;
    }
    let idx = rng.next_usize(pool.len());
    Some(pool[idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::sets;

    /// OtherClass (Pilfer) — "another class" is exactly the other eight
    /// classes' class cards: every class-list member is reachable, and no
    /// neutral (NEUTRAL_CLASSIC / LEGENDARY_CLASSIC) or Rogue card is.
    #[test]
    fn other_class_pool_is_class_cards_of_other_classes() {
        let pool = pool_cards(RandomPool::OtherClass);
        assert!(!pool.is_empty(), "the OtherClass pool must not be empty");
        let ids: Vec<&str> = pool.iter().map(|c| c.id).collect();
        let other_classes = [
            sets::DRUID_CLASSIC,
            sets::HUNTER_CLASSIC,
            sets::MAGE_CLASSIC,
            sets::PALADIN_CLASSIC,
            sets::PRIEST_CLASSIC,
            sets::SHAMAN_CLASSIC,
            sets::WARLOCK_CLASSIC,
            sets::WARRIOR_CLASSIC,
        ];
        for class in other_classes {
            for card in class {
                assert!(
                    ids.contains(&card.id),
                    "{} must be in the OtherClass pool",
                    card.id
                );
            }
        }
        for card in pool {
            assert!(
                !sets::NEUTRAL_CLASSIC.iter().any(|c| c.id == card.id),
                "{} is a neutral card — not 'another class'",
                card.id
            );
            assert!(
                !sets::LEGENDARY_CLASSIC.iter().any(|c| c.id == card.id),
                "{} is a neutral legendary — not 'another class'",
                card.id
            );
            assert!(
                !sets::ROGUE_CLASSIC.iter().any(|c| c.id == card.id),
                "{} is a Rogue card — not 'another class'",
                card.id
            );
        }
    }
}
