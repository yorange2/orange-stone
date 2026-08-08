//! Random pools — filtered sampling from the active card window.
//!
//! Pool closure is guaranteed: every sampling pool is a filtered subset of the
//! active window (`ALL_CARDS` ∩ `in_active_window`, i.e. Classic-era + Core as
//! of 2025–2026 expansions M0.3) or built-in token pools, so no cards outside
//! the active window are introduced.
//!
//! Beast/Demon pools are field-driven (fidelity-debt W1): `CardDef.race` decides
//! membership, so the pools stay in sync with the card data automatically.
//! Legendary/class filtering is computed dynamically from the group lists in `sets`.

use crate::cards::def::{CardDef, CardSet, card_by_id};
use crate::core::component::{CardType, Race};
use crate::core::effect::RandomPool;
use crate::core::player::PlayerId;
use crate::sim::rng::GameRng;

/// Whether the card is inside the active sampling window (decision D3,
/// 2025–2026 expansions M0.3): the current training pool (Classic-era + Core).
///
/// The 2025–2026 expansion cards are engine-available via `ALL_CARDS` but are
/// **not** sampled until the single cut-over — at which point this predicate
/// flips to the Standard window (`CardSet::Core | the five expansions`, see
/// `is_standard`).
pub(crate) fn in_active_window(card: &CardDef) -> bool {
    matches!(
        crate::cards::generated::card_set(card.id),
        CardSet::Classic | CardSet::Core
    )
}

/// Whether the card belongs to the Standard window (decision D3): Core + the
/// five 2025–2026 expansions. This is the future training pool; today it only
/// drives explicit filters, never the sampling pools.
#[must_use]
pub fn is_standard(card: &CardDef) -> bool {
    matches!(
        crate::cards::generated::card_set(card.id),
        CardSet::Core
            | CardSet::EmeraldDream
            | CardSet::TheLostCity
            | CardSet::TimeTravel
            | CardSet::Cataclysm
            | CardSet::EscapeFromVioletHold
    )
}

/// Whether the card belongs to one of the five 2025–2026 expansions — the
/// cards excluded from the sampling pools until the D3 cut-over.
#[must_use]
pub fn is_expansion(card: &CardDef) -> bool {
    matches!(
        crate::cards::generated::card_set(card.id),
        CardSet::EmeraldDream
            | CardSet::TheLostCity
            | CardSet::TimeTravel
            | CardSet::Cataclysm
            | CardSet::EscapeFromVioletHold
    )
}

/// All cards of the given race inside the active window (field-driven —
/// `CardDef.race`).
fn race_pool(race: Race) -> Vec<&'static CardDef> {
    crate::cards::sets::ALL_CARDS
        .iter()
        .filter(|c| c.race == Some(race) && in_active_window(c))
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

/// Wild God pool — the eight Wild Gods of the Emerald Dream set (Malorne the
/// Waywatcher's "Discover a Legendary Wild God" pool, 2025–2026 expansions
/// M1-W1). The real pool is the "Legendary Wild God" tag filter; the fixed
/// table is the simplified Discover pool (fidelity-debt §14).
pub const WILD_GOD_POOL: &[&str] = &[
    "EDR_031", // Ohn'ahra
    "EDR_238", // Merithra
    "EDR_259", // Ursol
    "EDR_430", // Aessina
    "EDR_465", // Ysondre
    "EDR_480", // Goldrinn
    "EDR_489", // Agamaggan
    "EDR_819", // Ursoc
];

/// Other-class Choose One pool — Symbiosis's "Discover a Choose One card
/// from another class" pool (2025–2026 expansions M1-W3). The design brief's
/// formula ("in-window choose-one cards of the other class groups") yields an
/// empty set because every in-window choose-one card is Druid, so the pool is
/// the fixed table of the non-Druid EDR choose-one cards of this wave — the
/// WILD_GOD_POOL precedent (fidelity-debt §14.2).
pub const OTHER_CLASS_CHOOSE_ONE_POOL: &[&str] = &[
    "EDR_233", // Spirits of the Forest (Shaman)
    "EDR_257", // Lightmender (Paladin)
    "EDR_263", // Grace of the Greatwolf (Hunter)
    "EDR_463", // Twilight Influence (Priest)
    "EDR_490", // Sleep Paralysis (Warlock)
    "EDR_525", // Barbed Thorn (Rogue)
    "EDR_570", // Ominous Nightmares (Warrior)
    "EDR_813", // Morbid Swarm (Death Knight)
    "EDR_820", // Wyvern's Slumber (Demon Hunter)
    "EDR_872", // Spark of Life (Mage)
];

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
                    && in_active_window(c)
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
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::ShadowSpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell && c.name.contains("Shadow") && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::Spell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == crate::core::component::CardType::Spell && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::OtherClass => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| is_other_class_card(c) && in_active_window(c))
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::PriestCard => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                (c.card_type == CardType::Minion || c.card_type == CardType::Spell)
                    && crate::cards::sets::PRIEST_CLASSIC
                        .iter()
                        .any(|p| p.id == c.id)
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::DeathrattleMinion => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && (c.deathrattle.is_some() || c.death_trigger.is_some())
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::UndeadMinion => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Undead)
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::DemonCost5Plus => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Demon)
                    && c.cost >= 5
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::OtherClassChooseOne => OTHER_CLASS_CHOOSE_ONE_POOL
            .iter()
            .filter_map(|id| card_by_id(id))
            .collect(),
        RandomPool::Murloc => race_pool(Race::Murloc),
        RandomPool::Elemental => race_pool(Race::Elemental),
        RandomPool::WarriorMinion => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && crate::cards::sets::WARRIOR_CLASSIC
                        .iter()
                        .any(|w| w.id == c.id)
                    && in_active_window(c)
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
        RandomPool::PriestCard => random_filtered(rng, |c| {
            (c.card_type == CardType::Minion || c.card_type == CardType::Spell)
                && crate::cards::sets::PRIEST_CLASSIC
                    .iter()
                    .any(|p| p.id == c.id)
        }),
        RandomPool::DeathrattleMinion => random_filtered(rng, |c| {
            c.card_type == CardType::Minion
                && (c.deathrattle.is_some() || c.death_trigger.is_some())
        }),
        RandomPool::UndeadMinion => random_filtered(rng, |c| {
            c.card_type == CardType::Minion && c.race == Some(Race::Undead)
        }),
        RandomPool::DemonCost5Plus => random_filtered(rng, |c| {
            c.card_type == CardType::Minion && c.race == Some(Race::Demon) && c.cost >= 5
        }),
        RandomPool::OtherClassChooseOne => random_from_pool(OTHER_CLASS_CHOOSE_ONE_POOL, rng),
        RandomPool::Murloc => random_filtered(rng, |c| c.race == Some(Race::Murloc)),
        RandomPool::Elemental => random_filtered(rng, |c| c.race == Some(Race::Elemental)),
        RandomPool::WarriorMinion => random_filtered(rng, |c| {
            c.card_type == CardType::Minion
                && crate::cards::sets::WARRIOR_CLASSIC
                    .iter()
                    .any(|w| w.id == c.id)
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
        .filter(|c| predicate(c) && in_active_window(c))
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

    /// PriestCard (Blessing of the Moon — M1-W1) — every Priest class
    /// minion or spell is reachable and nothing else (no neutrals, no
    /// non-minion/spell types).
    #[test]
    fn priest_card_pool_is_priest_minions_and_spells() {
        let pool = pool_cards(RandomPool::PriestCard);
        assert!(!pool.is_empty(), "the PriestCard pool must not be empty");
        let ids: Vec<&str> = pool.iter().map(|c| c.id).collect();
        for card in sets::PRIEST_CLASSIC {
            if card.card_type == CardType::Minion || card.card_type == CardType::Spell {
                assert!(
                    ids.contains(&card.id),
                    "{} must be in the PriestCard pool",
                    card.id
                );
            }
        }
        for card in pool {
            assert!(
                sets::PRIEST_CLASSIC.iter().any(|c| c.id == card.id),
                "{} is not a Priest class card",
                card.id
            );
            assert!(
                card.card_type == CardType::Minion || card.card_type == CardType::Spell,
                "{} is not a Priest minion or spell",
                card.id
            );
        }
    }

    /// DeathrattleMinion (Avant-Gardening — M1-W2 dark gifts) — exactly the
    /// in-window minions carrying a Deathrattle effect or a death trigger.
    #[test]
    fn deathrattle_minion_pool_is_minions_with_deathrattle() {
        let pool = pool_cards(RandomPool::DeathrattleMinion);
        assert!(
            !pool.is_empty(),
            "the DeathrattleMinion pool must not be empty"
        );
        let ids: Vec<&str> = pool.iter().map(|c| c.id).collect();
        for card in sets::ALL_CARDS {
            let in_window = crate::cards::generated::card_set(card.id)
                == crate::cards::def::CardSet::Classic
                || crate::cards::generated::card_set(card.id) == crate::cards::def::CardSet::Core;
            if card.card_type == CardType::Minion
                && (card.deathrattle.is_some() || card.death_trigger.is_some())
                && in_window
            {
                assert!(
                    ids.contains(&card.id),
                    "{} must be in the DeathrattleMinion pool",
                    card.id
                );
            }
        }
        for card in pool {
            assert_eq!(
                card.card_type,
                CardType::Minion,
                "{} is not a minion",
                card.id
            );
            assert!(
                card.deathrattle.is_some() || card.death_trigger.is_some(),
                "{} has no Deathrattle",
                card.id
            );
        }
    }

    /// UndeadMinion (Rite of Atrocity — M1-W2 dark gifts) — exactly the
    /// in-window Undead minions.
    #[test]
    fn undead_minion_pool_is_undead() {
        let pool = pool_cards(RandomPool::UndeadMinion);
        assert!(!pool.is_empty(), "the UndeadMinion pool must not be empty");
        let ids: Vec<&str> = pool.iter().map(|c| c.id).collect();
        for card in sets::ALL_CARDS {
            let in_window = crate::cards::generated::card_set(card.id)
                == crate::cards::def::CardSet::Classic
                || crate::cards::generated::card_set(card.id) == crate::cards::def::CardSet::Core;
            if card.card_type == CardType::Minion && card.race == Some(Race::Undead) && in_window {
                assert!(
                    ids.contains(&card.id),
                    "{} must be in the UndeadMinion pool",
                    card.id
                );
            }
        }
        for card in pool {
            assert_eq!(
                card.card_type,
                CardType::Minion,
                "{} is not a minion",
                card.id
            );
            assert_eq!(card.race, Some(Race::Undead), "{} is not Undead", card.id);
        }
    }

    /// DemonCost5Plus (Jumpscare! — M1-W2 dark gifts) — exactly the
    /// in-window Demons costing (5) or more.
    #[test]
    fn demon_cost_5_plus_pool_is_costly_demons() {
        let pool = pool_cards(RandomPool::DemonCost5Plus);
        assert!(
            !pool.is_empty(),
            "the DemonCost5Plus pool must not be empty"
        );
        let ids: Vec<&str> = pool.iter().map(|c| c.id).collect();
        for card in sets::ALL_CARDS {
            let in_window = crate::cards::generated::card_set(card.id)
                == crate::cards::def::CardSet::Classic
                || crate::cards::generated::card_set(card.id) == crate::cards::def::CardSet::Core;
            if card.card_type == CardType::Minion
                && card.race == Some(Race::Demon)
                && card.cost >= 5
                && in_window
            {
                assert!(
                    ids.contains(&card.id),
                    "{} must be in the DemonCost5Plus pool",
                    card.id
                );
            }
        }
        for card in pool {
            assert_eq!(
                card.card_type,
                CardType::Minion,
                "{} is not a minion",
                card.id
            );
            assert_eq!(card.race, Some(Race::Demon), "{} is not a Demon", card.id);
            assert!(card.cost >= 5, "{} costs less than 5", card.id);
        }
    }

    /// OtherClassChooseOne (Symbiosis — M1-W3) — exactly the fixed
    /// OTHER_CLASS_CHOOSE_ONE_POOL table of the ten non-Druid EDR choose-one
    /// cards; every member is resolvable through card_by_id.
    #[test]
    fn other_class_choose_one_pool_is_fixed_table() {
        let pool = pool_cards(RandomPool::OtherClassChooseOne);
        assert!(
            !pool.is_empty(),
            "the OtherClassChooseOne pool must not be empty"
        );
        let ids: Vec<&str> = pool.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), OTHER_CLASS_CHOOSE_ONE_POOL.len());
        for id in OTHER_CLASS_CHOOSE_ONE_POOL {
            assert!(ids.contains(id), "{id} must be in the pool");
            assert!(
                card_by_id(id).is_some(),
                "{id} must resolve through card_by_id"
            );
        }
        for card in pool {
            assert!(
                card.choose_one_effect.is_some() || card.id == "EDR_525",
                "{} is not a choose-one card (Barbed Thorn uses the weapon slots)",
                card.id
            );
            // Every member belongs to a non-Druid class: none is a Druid
            // class-list member.
            assert!(
                !sets::DRUID_CLASSIC.iter().any(|c| c.id == card.id),
                "{} is a Druid card — not 'another class'",
                card.id
            );
        }
    }

    /// Murloc (Gnawing Greenfin — M1-W4a) — every Murloc race member in the
    /// active window is reachable and nothing else. Expansion cards like
    /// Gnawing Greenfin (EDR_999) stay out of the sampling pools until the
    /// D3 cut-over (in_active_window), so a Classic-era member anchors the
    /// "non-empty" assertion.
    #[test]
    fn murloc_pool_is_murloc_minions() {
        let pool = pool_cards(RandomPool::Murloc);
        assert!(!pool.is_empty(), "the Murloc pool must not be empty");
        for card in &pool {
            assert!(
                card.race == Some(Race::Murloc),
                "{} is not a Murloc",
                card.id
            );
        }
        assert!(
            pool.iter().any(|c| c.id == "CLASSIC_006"),
            "Murloc Tidehunter must be in the Murloc pool"
        );
    }
}
