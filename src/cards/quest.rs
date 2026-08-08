//! Quest definitions (2025–2026 expansions M2-W1 — the Un'Goro quest
//! mechanic): the static half of quest cards, keyed by card id.
//!
//! Quest cards are 1-cost legendary spells whose card text starts with
//! `Quest:` / `Repeatable Quest:`. They are played into the per-player quest
//! slot (`Zone::Quest`); game events accumulate progress (`engine::quest`
//! dispatches); at the target the reward resolves (usually summoning a token
//! minion; two rewards are weapons, one is a hero, one is a passive).
//!
//! This registry is the id-keyed analogue of `CardDef` fields — the quest
//! data (condition, target, reward) deliberately does NOT live in the ~800
//! `CardDef` literals; the play path looks it up via `quest_def(card_id)`
//! instead (mirror of `apply_card_keywords`, `POOL_OPEN_CARDS`, imbue).
//! The 11 quest cards + their reward tokens are the W2 wave; W2 fills the
//! reward refinements (token CardDefs, the TLC_426 passive, hero
//! replacement).

use serde::{Deserialize, Serialize};

use crate::core::component::Race;
use crate::core::effect::CardEffect;

/// Spell schools (2025–2026 expansions M2-W1) — the eight Hearthstone spell
/// schools, used by quest conditions like "Cast 4 Holy spells".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellSchool {
    /// Arcane
    Arcane,
    /// Fel
    Fel,
    /// Fire
    Fire,
    /// Frost
    Frost,
    /// Holy
    Holy,
    /// Nature
    Nature,
    /// Shadow
    Shadow,
}

/// Spell school of a card, from the official card dump.
///
/// Static table of the 22 Un'Goro collectible spells that carry a
/// `spellSchool` field in `cards/data/THE_LOST_CITY.json` (extracted
/// 2026-08-09); `None` for every other card. Needed by
/// `QuestCondition::CastSpellsOfSchool` (TLC_817 in W2).
#[must_use]
pub fn spell_school(card_id: &str) -> Option<SpellSchool> {
    match card_id {
        "DINO_406" => Some(SpellSchool::Fire),
        "DINO_417" => Some(SpellSchool::Shadow),
        "DINO_426" => Some(SpellSchool::Holy),
        "TLC_221" => Some(SpellSchool::Fire),
        "TLC_222" => Some(SpellSchool::Fire),
        "TLC_227" => Some(SpellSchool::Fire),
        "TLC_230" => Some(SpellSchool::Nature),
        "TLC_235" => Some(SpellSchool::Nature),
        "TLC_236" => Some(SpellSchool::Nature),
        "TLC_365" => Some(SpellSchool::Arcane),
        "TLC_434" => Some(SpellSchool::Shadow),
        "TLC_440" => Some(SpellSchool::Frost),
        "TLC_447" => Some(SpellSchool::Fel),
        "TLC_451" => Some(SpellSchool::Fel),
        "TLC_477" => Some(SpellSchool::Holy),
        "TLC_515" => Some(SpellSchool::Shadow),
        "TLC_519" => Some(SpellSchool::Shadow),
        "TLC_632" => Some(SpellSchool::Fire),
        "TLC_815" => Some(SpellSchool::Shadow),
        "TLC_816" => Some(SpellSchool::Holy),
        "TLC_818" => Some(SpellSchool::Shadow),
        "TLC_901" => Some(SpellSchool::Fel),
        _ => None,
    }
}

/// Quest conditions (2025–2026 expansions M2-W1) — the progress triggers the
/// 11 Un'Goro quest cards are built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestCondition {
    /// Play 6 minions of unique types (TLC_229)
    PlayMinionsOfUniqueTypes,
    /// Fill your board on 3 of your turns (TLC_239)
    FillBoardOnTurns,
    /// Summon 6 minions of the given race (TLC_426)
    SummonMinionsOfRace {
        /// Race of the summoned minions
        race: Race,
    },
    /// Spend 15 Corpses (TLC_433)
    SpendCorpses,
    /// Play 6 Temporary cards (TLC_446) — no call site in W1: the
    /// Temporary keyword does not exist in the engine yet (W2/W4
    /// introduce it). The variant exists so the full condition set is
    /// defined; W2 wires the call site.
    PlayTemporaryCards,
    /// Discover 7 cards (TLC_460)
    DiscoverCards,
    /// Shuffle cards into your deck, 5 times (TLC_513)
    ShuffleCards,
    /// Survive 10 turns (TLC_602)
    SurviveTurns,
    /// Deal exactly N damage to an enemy on your turn, 12 times (TLC_631)
    DealExactDamage {
        /// Exact damage amount per qualifying hit
        amount: u32,
    },
    /// Cast 4 spells of the given school (TLC_817)
    CastSpellsOfSchool {
        /// School of the cast spells
        school: SpellSchool,
    },
    /// Play a 1, 3, 5, and 7-Attack Beast (TLC_830)
    PlayBeastsOfAttack,
}

/// Static definition of a quest card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDef {
    /// The progress condition
    pub condition: QuestCondition,
    /// Progress needed to complete the quest
    pub target: u32,
    /// Reward resolved on completion (battlecry-style resolution; token
    /// CardDefs land in W2, so unregistered reward IDs no-op gracefully)
    pub reward: CardEffect,
    /// Repeatable quests reset to 0 on completion and stay in the slot
    pub repeatable: bool,
}

/// Look up the quest definition of a card id.
///
/// `Some` only for the 11 Un'Goro quest cards; the play path in
/// `engine/rules.rs` consults this to route a played card into the quest
/// zone instead of the spell path.
#[must_use]
pub fn quest_def(card_id: &str) -> Option<&'static QuestDef> {
    match card_id {
        "TLC_229" => Some(&QuestDef {
            condition: QuestCondition::PlayMinionsOfUniqueTypes,
            target: 6,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_229t14", // Ashalon, Ridge Guardian (W2)
            },
            repeatable: false,
        }),
        "TLC_239" => Some(&QuestDef {
            condition: QuestCondition::FillBoardOnTurns,
            target: 3,
            reward: CardEffect::EquipWeapon {
                card_id: "TLC_239t", // The Everbloom (W2)
            },
            repeatable: false,
        }),
        "TLC_426" => Some(&QuestDef {
            condition: QuestCondition::SummonMinionsOfRace { race: Race::Murloc },
            target: 6,
            // W2: pin the permanent "Murlocs you summon gain +1/+1" passive
            // — no existing effect expresses it, so the reward is a no-op
            // placeholder (unregistered id) until the passive lands.
            reward: CardEffect::SummonMinion {
                card_id: "TLC_426_W2_PASSIVE",
            },
            repeatable: true,
        }),
        "TLC_433" => Some(&QuestDef {
            condition: QuestCondition::SpendCorpses,
            target: 15,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_433t", // Tyrax, Bone Terror (W2)
            },
            repeatable: false,
        }),
        "TLC_446" => Some(&QuestDef {
            condition: QuestCondition::PlayTemporaryCards,
            target: 6,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_446t1", // Underfel Rift (W2)
            },
            repeatable: false,
        }),
        "TLC_460" => Some(&QuestDef {
            condition: QuestCondition::DiscoverCards,
            target: 7,
            reward: CardEffect::EquipWeapon {
                card_id: "TLC_460t", // The Origin Stone (W2)
            },
            repeatable: false,
        }),
        "TLC_513" => Some(&QuestDef {
            condition: QuestCondition::ShuffleCards,
            target: 5,
            // W2: hero replacement — no hero-replace effect exists yet; the
            // reward is a no-op placeholder (unregistered id) until W2 adds
            // the effect. A SummonMinion-shaped placeholder is safe: it
            // resolves via `card_by_id` and no-ops for unregistered ids.
            reward: CardEffect::SummonMinion {
                card_id: "TLC_513_W2_HERO",
            },
            repeatable: false,
        }),
        "TLC_602" => Some(&QuestDef {
            condition: QuestCondition::SurviveTurns,
            target: 10,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_602t", // Latorvius, Gaze of the City (W2)
            },
            repeatable: false,
        }),
        "TLC_631" => Some(&QuestDef {
            condition: QuestCondition::DealExactDamage { amount: 2 },
            target: 12,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_631t", // Gorishi Colossus (W2)
            },
            repeatable: false,
        }),
        // The W1 entry covers the first quest (Holy, reward TLC_817t3);
        // W2 wires the second (Shadow, reward TLC_817t4) bar — the official
        // card tracks both quests on one card.
        "TLC_817" => Some(&QuestDef {
            condition: QuestCondition::CastSpellsOfSchool {
                school: SpellSchool::Holy,
            },
            target: 4,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_817t3", // Sol'etos, Life's Breath (W2)
            },
            repeatable: false,
        }),
        "TLC_830" => Some(&QuestDef {
            condition: QuestCondition::PlayBeastsOfAttack,
            target: 4,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_830t", // Shokk, Jungle Tyrant (W2)
            },
            repeatable: false,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spell-school table covers exactly the 22 dump entries and nothing
    /// else.
    #[test]
    fn spell_school_table_is_complete() {
        let mut schools = Vec::new();
        for (id, school) in [
            ("DINO_406", SpellSchool::Fire),
            ("DINO_417", SpellSchool::Shadow),
            ("DINO_426", SpellSchool::Holy),
            ("TLC_221", SpellSchool::Fire),
            ("TLC_222", SpellSchool::Fire),
            ("TLC_227", SpellSchool::Fire),
            ("TLC_230", SpellSchool::Nature),
            ("TLC_235", SpellSchool::Nature),
            ("TLC_236", SpellSchool::Nature),
            ("TLC_365", SpellSchool::Arcane),
            ("TLC_434", SpellSchool::Shadow),
            ("TLC_440", SpellSchool::Frost),
            ("TLC_447", SpellSchool::Fel),
            ("TLC_451", SpellSchool::Fel),
            ("TLC_477", SpellSchool::Holy),
            ("TLC_515", SpellSchool::Shadow),
            ("TLC_519", SpellSchool::Shadow),
            ("TLC_632", SpellSchool::Fire),
            ("TLC_815", SpellSchool::Shadow),
            ("TLC_816", SpellSchool::Holy),
            ("TLC_818", SpellSchool::Shadow),
            ("TLC_901", SpellSchool::Fel),
        ] {
            assert_eq!(spell_school(id), Some(school));
            schools.push(id);
        }
        assert_eq!(schools.len(), 22);
        assert_eq!(spell_school("TLC_229"), None);
        assert_eq!(spell_school("UNKNOWN"), None);
    }

    /// All 11 quest entries resolve, with the expected condition/target.
    #[test]
    fn quest_def_table_is_complete() {
        let expected: &[(&str, QuestCondition, u32, bool)] = &[
            (
                "TLC_229",
                QuestCondition::PlayMinionsOfUniqueTypes,
                6,
                false,
            ),
            ("TLC_239", QuestCondition::FillBoardOnTurns, 3, false),
            (
                "TLC_426",
                QuestCondition::SummonMinionsOfRace { race: Race::Murloc },
                6,
                true,
            ),
            ("TLC_433", QuestCondition::SpendCorpses, 15, false),
            ("TLC_446", QuestCondition::PlayTemporaryCards, 6, false),
            ("TLC_460", QuestCondition::DiscoverCards, 7, false),
            ("TLC_513", QuestCondition::ShuffleCards, 5, false),
            ("TLC_602", QuestCondition::SurviveTurns, 10, false),
            (
                "TLC_631",
                QuestCondition::DealExactDamage { amount: 2 },
                12,
                false,
            ),
            (
                "TLC_817",
                QuestCondition::CastSpellsOfSchool {
                    school: SpellSchool::Holy,
                },
                4,
                false,
            ),
            ("TLC_830", QuestCondition::PlayBeastsOfAttack, 4, false),
        ];
        for (id, condition, target, repeatable) in expected {
            let def = quest_def(id).unwrap_or_else(|| panic!("{id}"));
            assert_eq!(def.condition, *condition);
            assert_eq!(def.target, *target);
            assert_eq!(def.repeatable, *repeatable);
        }
        assert_eq!(quest_def("TLC_228"), None);
        assert_eq!(quest_def("UNKNOWN"), None);
    }
}
