//! Random pools — filtered sampling from the Classic pool.
//!
//! Pool closure is guaranteed: every sampling pool is a filtered subset of `ALL_CARDS`
//! (all Classic cards) or built-in token pools, so no cards outside the Classic set are introduced.
//!
//! Races (Beast/Demon) are not modeled, so these pools are hardcoded by card ID;
//! Legendary/class filtering is computed dynamically from the group lists in `sets`.

use crate::cards::def::{CardDef, card_by_id};
use crate::core::component::CardType;
use crate::core::effect::RandomPool;
use crate::sim::rng::GameRng;

/// Beast pool — Classic Beast card IDs.
pub const BEAST_POOL: &[&str] = &[
    "NEUTRAL_B08", // Bloodfen Raptor / Ironfur Grizzly
    "NEUTRAL_B03", // River Crocolisk
    "NEUTRAL_B11", // Silverback Patriarch
    "NEUTRAL_C04", // Stonetusk Boar / Young Dragonhawk
    "NEUTRAL_T14", // Stranglethorn Tiger
    "NEUTRAL_C10", // Jungle Panther
    "NEUTRAL_B13", // Oasis Snapjaw
    "NEUTRAL_R20", // Stampeding Kodo
    "NEUTRAL_T05", // River Crocolisk (Tier 1)
    "HUNTER_010",  // Timber Wolf
    "HUNTER_006",  // Savannah Highmane
    "HUNTER_011",  // King Krush
    "HUNTER_014",  // Starving Buzzard
    "HUNTER_016",  // Tundra Rhino
];

/// Demon pool — Classic Demon card IDs.
pub const DEMON_POOL: &[&str] = &[
    "WARLOCK_004", // Voidwalker
    "WARLOCK_002", // Flame Imp
    "WARLOCK_007", // Doomguard
    "WARLOCK_011", // Dread Infernal
    "WARLOCK_012", // Pit Lord
    "WARLOCK_016", // Felstalker
    "WARLOCK_019", // Felguard
    "WARLOCK_022", // Void Terror
];

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
        RandomPool::Beast => BEAST_POOL.iter().filter_map(|id| card_by_id(id)).collect(),
        RandomPool::Demon => DEMON_POOL.iter().filter_map(|id| card_by_id(id)).collect(),
        RandomPool::Dream => DREAM_POOL.iter().filter_map(|id| card_by_id(id)).collect(),
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
        RandomPool::OtherClass => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                // Pilfer: a random card from another class — simplified to any non-Rogue card
                !crate::cards::sets::ROGUE_CLASSIC
                    .iter()
                    .any(|r| r.id == c.id)
            })
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
        RandomPool::Beast => random_from_pool(BEAST_POOL, rng),
        RandomPool::Demon => random_from_pool(DEMON_POOL, rng),
        RandomPool::Dream => random_from_pool(DREAM_POOL, rng),
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
        RandomPool::OtherClass => random_filtered(rng, |c| {
            // Pilfer: a random card from another class — simplified to any non-Rogue card
            !crate::cards::sets::ROGUE_CLASSIC
                .iter()
                .any(|r| r.id == c.id)
        }),
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
