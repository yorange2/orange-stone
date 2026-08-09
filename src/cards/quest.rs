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
/// Static table of every spell that carries a `spellSchool` field in the
/// dumps: the 22 Un'Goro collectible spells (extracted 2026-08-09 from
/// `cards/data/THE_LOST_CITY.json`) plus the 200 Classic-era/Core spells
/// (the active sampling window, extracted from `/tmp/hs_full.json`
/// 2026-08-09 — the school predicates behind the M2-W4a Fel/Holy pools
/// and Gladesong Siren's cost condition read the in-window entries, and
/// the TLC_817 quest's CastSpellsOfSchool condition now also counts
/// in-window Holy/Shadow casts, matching the official "cast a Holy spell"
/// wording); `None` for every other card. Needed by
/// `QuestCondition::CastSpellsOfSchool` (TLC_817 in W2) and the W4a
/// school filters.
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
        // The 200 Classic-era/Core (active-window) spells with a school
        // field (extracted from the official card dump 2026-08-09). These
        // drive the M2-W4a Fel/Holy pool predicates and Gladesong Siren's
        // cost condition, and let the TLC_817 quest count in-window
        // Holy/Shadow casts (matching the official "cast a Holy spell"
        // wording).
        "BCON_004" => Some(SpellSchool::Arcane),
        "CORE_BAR_541" => Some(SpellSchool::Arcane),
        "CORE_CS2_023" => Some(SpellSchool::Arcane),
        "CORE_DS1_185" => Some(SpellSchool::Arcane),
        "CORE_EX1_287" => Some(SpellSchool::Arcane),
        "CS2_008" => Some(SpellSchool::Arcane),
        "CS2_022" => Some(SpellSchool::Arcane),
        "CS2_023" => Some(SpellSchool::Arcane),
        "CS2_025" => Some(SpellSchool::Arcane),
        "DS1_185" => Some(SpellSchool::Arcane),
        "EX1_173" => Some(SpellSchool::Arcane),
        "EX1_180" => Some(SpellSchool::Arcane),
        "EX1_277" => Some(SpellSchool::Arcane),
        "EX1_287" => Some(SpellSchool::Arcane),
        "EX1_294" => Some(SpellSchool::Arcane),
        "NEW1_007" => Some(SpellSchool::Arcane),
        "NEW1_007a" => Some(SpellSchool::Arcane),
        "NEW1_007b" => Some(SpellSchool::Arcane),
        "BT_035" => Some(SpellSchool::Fel),
        "BT_235" => Some(SpellSchool::Fel),
        "BT_753" => Some(SpellSchool::Fel),
        "BT_801" => Some(SpellSchool::Fel),
        "CORE_BT_035" => Some(SpellSchool::Fel),
        "CORE_BT_801" => Some(SpellSchool::Fel),
        "CORE_SW_088" => Some(SpellSchool::Fel),
        "EX1_596" => Some(SpellSchool::Fel),
        "TOY_400t8" => Some(SpellSchool::Fel),
        "CORE_CS2_029" => Some(SpellSchool::Fire),
        "CORE_CS2_032" => Some(SpellSchool::Fire),
        "CORE_CS2_062" => Some(SpellSchool::Fire),
        "CORE_EX1_610" => Some(SpellSchool::Fire),
        "CORE_GIL_836" => Some(SpellSchool::Fire),
        "CORE_LOOT_101" => Some(SpellSchool::Fire),
        "CORE_SW_108" => Some(SpellSchool::Fire),
        "CORE_WON_337" => Some(SpellSchool::Fire),
        "CS2_029" => Some(SpellSchool::Fire),
        "CS2_032" => Some(SpellSchool::Fire),
        "CS2_062" => Some(SpellSchool::Fire),
        "EX1_241" => Some(SpellSchool::Fire),
        "EX1_279" => Some(SpellSchool::Fire),
        "EX1_308" => Some(SpellSchool::Fire),
        "EX1_537" => Some(SpellSchool::Fire),
        "EX1_594" => Some(SpellSchool::Fire),
        "EX1_610" => Some(SpellSchool::Fire),
        "CORE_AV_107" => Some(SpellSchool::Frost),
        "CORE_BAR_812" => Some(SpellSchool::Frost),
        "CORE_BT_072" => Some(SpellSchool::Frost),
        "CORE_CATA_009" => Some(SpellSchool::Frost),
        "CORE_CS2_024" => Some(SpellSchool::Frost),
        "CORE_CS2_028" => Some(SpellSchool::Frost),
        "CORE_EX1_289" => Some(SpellSchool::Frost),
        "CORE_EX1_611" => Some(SpellSchool::Frost),
        "CORE_RLK_063" => Some(SpellSchool::Frost),
        "CS2_024" => Some(SpellSchool::Frost),
        "CS2_026" => Some(SpellSchool::Frost),
        "CS2_028" => Some(SpellSchool::Frost),
        "CS2_031" => Some(SpellSchool::Frost),
        "CS2_037" => Some(SpellSchool::Frost),
        "EX1_179" => Some(SpellSchool::Frost),
        "EX1_275" => Some(SpellSchool::Frost),
        "EX1_289" => Some(SpellSchool::Frost),
        "EX1_295" => Some(SpellSchool::Frost),
        "EX1_611" => Some(SpellSchool::Frost),
        "LEG_RLK_101" => Some(SpellSchool::Frost),
        "RLK_025" => Some(SpellSchool::Frost),
        "RLK_709" => Some(SpellSchool::Frost),
        "BCON_012" => Some(SpellSchool::Holy),
        "CORE_AT_055" => Some(SpellSchool::Holy),
        "CORE_BT_292" => Some(SpellSchool::Holy),
        "CORE_CFM_604" => Some(SpellSchool::Holy),
        "CORE_CS1_112" => Some(SpellSchool::Holy),
        "CORE_CS1_130" => Some(SpellSchool::Holy),
        "CORE_CS2_004" => Some(SpellSchool::Holy),
        "CORE_CS2_093" => Some(SpellSchool::Holy),
        "CORE_CS2_094" => Some(SpellSchool::Holy),
        "CORE_EX1_619" => Some(SpellSchool::Holy),
        "CORE_KAR_077" => Some(SpellSchool::Holy),
        "CORE_TRL_307" => Some(SpellSchool::Holy),
        "CORE_TSC_076" => Some(SpellSchool::Holy),
        "CS1_112" => Some(SpellSchool::Holy),
        "CS1_130" => Some(SpellSchool::Holy),
        "CS2_004" => Some(SpellSchool::Holy),
        "CS2_087" => Some(SpellSchool::Holy),
        "CS2_089" => Some(SpellSchool::Holy),
        "CS2_092" => Some(SpellSchool::Holy),
        "CS2_093" => Some(SpellSchool::Holy),
        "CS2_094" => Some(SpellSchool::Holy),
        "CS2_236" => Some(SpellSchool::Holy),
        "CS3_016" => Some(SpellSchool::Holy),
        "CS3_029" => Some(SpellSchool::Holy),
        "EX1_132" => Some(SpellSchool::Holy),
        "EX1_136" => Some(SpellSchool::Holy),
        "EX1_184" => Some(SpellSchool::Holy),
        "EX1_192" => Some(SpellSchool::Holy),
        "EX1_194" => Some(SpellSchool::Holy),
        "EX1_349" => Some(SpellSchool::Holy),
        "EX1_354" => Some(SpellSchool::Holy),
        "EX1_355" => Some(SpellSchool::Holy),
        "EX1_363" => Some(SpellSchool::Holy),
        "EX1_365" => Some(SpellSchool::Holy),
        "EX1_371" => Some(SpellSchool::Holy),
        "EX1_379" => Some(SpellSchool::Holy),
        "EX1_384" => Some(SpellSchool::Holy),
        "EX1_619" => Some(SpellSchool::Holy),
        "EX1_621" => Some(SpellSchool::Holy),
        "EX1_624" => Some(SpellSchool::Holy),
        "EX1_626" => Some(SpellSchool::Holy),
        "LEG_TTN_908" => Some(SpellSchool::Holy),
        "TTN_851" => Some(SpellSchool::Holy),
        "BCON_008" => Some(SpellSchool::Nature),
        "BCON_021" => Some(SpellSchool::Nature),
        "CORE_AT_037" => Some(SpellSchool::Nature),
        "CORE_BOT_451" => Some(SpellSchool::Nature),
        "CORE_CS2_009" => Some(SpellSchool::Nature),
        "CORE_CS2_074" => Some(SpellSchool::Nature),
        "CORE_EX1_154" => Some(SpellSchool::Nature),
        "CORE_EX1_169" => Some(SpellSchool::Nature),
        "CORE_EX1_238" => Some(SpellSchool::Nature),
        "CORE_EX1_246" => Some(SpellSchool::Nature),
        "CORE_EX1_259" => Some(SpellSchool::Nature),
        "CORE_LOOT_309" => Some(SpellSchool::Nature),
        "CORE_LOOT_373" => Some(SpellSchool::Nature),
        "CORE_TSC_650" => Some(SpellSchool::Nature),
        "CS2_007" => Some(SpellSchool::Nature),
        "CS2_009" => Some(SpellSchool::Nature),
        "CS2_013" => Some(SpellSchool::Nature),
        "CS2_039" => Some(SpellSchool::Nature),
        "CS2_041" => Some(SpellSchool::Nature),
        "CS2_045" => Some(SpellSchool::Nature),
        "CS2_074" => Some(SpellSchool::Nature),
        "DREAM_02" => Some(SpellSchool::Nature),
        "DREAM_04" => Some(SpellSchool::Nature),
        "EX1_154" => Some(SpellSchool::Nature),
        "EX1_154a" => Some(SpellSchool::Nature),
        "EX1_154b" => Some(SpellSchool::Nature),
        "EX1_155" => Some(SpellSchool::Nature),
        "EX1_158" => Some(SpellSchool::Nature),
        "EX1_161" => Some(SpellSchool::Nature),
        "EX1_164" => Some(SpellSchool::Nature),
        "EX1_164a" => Some(SpellSchool::Nature),
        "EX1_164b" => Some(SpellSchool::Nature),
        "EX1_169" => Some(SpellSchool::Nature),
        "EX1_183" => Some(SpellSchool::Nature),
        "EX1_238" => Some(SpellSchool::Nature),
        "EX1_245" => Some(SpellSchool::Nature),
        "EX1_246" => Some(SpellSchool::Nature),
        "EX1_248" => Some(SpellSchool::Nature),
        "EX1_251" => Some(SpellSchool::Nature),
        "EX1_259" => Some(SpellSchool::Nature),
        "EX1_571" => Some(SpellSchool::Nature),
        "BCON_024" => Some(SpellSchool::Shadow),
        "BT_427" => Some(SpellSchool::Shadow),
        "BT_488" => Some(SpellSchool::Shadow),
        "BT_490" => Some(SpellSchool::Shadow),
        "BT_740" => Some(SpellSchool::Shadow),
        "CORE_BAR_311" => Some(SpellSchool::Shadow),
        "CORE_BOT_222" => Some(SpellSchool::Shadow),
        "CORE_EX1_197" => Some(SpellSchool::Shadow),
        "CORE_EX1_302" => Some(SpellSchool::Shadow),
        "CORE_EX1_309" => Some(SpellSchool::Shadow),
        "CORE_EX1_312" => Some(SpellSchool::Shadow),
        "CORE_ICC_055" => Some(SpellSchool::Shadow),
        "CORE_RLK_087" => Some(SpellSchool::Shadow),
        "CORE_RLK_118" => Some(SpellSchool::Shadow),
        "CORE_RLK_567" => Some(SpellSchool::Shadow),
        "CORE_RLK_712" => Some(SpellSchool::Shadow),
        "CORE_SCH_512" => Some(SpellSchool::Shadow),
        "CORE_SW_442" => Some(SpellSchool::Shadow),
        "CS1_113" => Some(SpellSchool::Shadow),
        "CS2_003" => Some(SpellSchool::Shadow),
        "CS2_057" => Some(SpellSchool::Shadow),
        "CS2_061" => Some(SpellSchool::Shadow),
        "CS2_063" => Some(SpellSchool::Shadow),
        "CS2_234" => Some(SpellSchool::Shadow),
        "CS3_002" => Some(SpellSchool::Shadow),
        "DREAM_05" => Some(SpellSchool::Shadow),
        "DS1_233" => Some(SpellSchool::Shadow),
        "EX1_128" => Some(SpellSchool::Shadow),
        "EX1_144" => Some(SpellSchool::Shadow),
        "EX1_181" => Some(SpellSchool::Shadow),
        "EX1_197" => Some(SpellSchool::Shadow),
        "EX1_302" => Some(SpellSchool::Shadow),
        "EX1_303" => Some(SpellSchool::Shadow),
        "EX1_309" => Some(SpellSchool::Shadow),
        "EX1_312" => Some(SpellSchool::Shadow),
        "EX1_316" => Some(SpellSchool::Shadow),
        "EX1_317" => Some(SpellSchool::Shadow),
        "EX1_320" => Some(SpellSchool::Shadow),
        "EX1_332" => Some(SpellSchool::Shadow),
        "EX1_334" => Some(SpellSchool::Shadow),
        "EX1_339" => Some(SpellSchool::Shadow),
        "EX1_345" => Some(SpellSchool::Shadow),
        "EX1_622" => Some(SpellSchool::Shadow),
        "EX1_625" => Some(SpellSchool::Shadow),
        "LEG_CS3_028" => Some(SpellSchool::Shadow),
        "LEG_RLK_715" => Some(SpellSchool::Shadow),
        "NEW1_003" => Some(SpellSchool::Shadow),
        "RLK_048" => Some(SpellSchool::Shadow),
        "RLK_060" => Some(SpellSchool::Shadow),
        "RLK_707" => Some(SpellSchool::Shadow),
        // M3-W2a — the Across the Timeways spell schools (extracted from
        // the official card dump 2026-08-09): the Nature pool (TIME_033
        // Druid of Regrowth) and the Arcane pool (TIME_857 Alter Time)
        // read these.
        "TIME_000" => Some(SpellSchool::Arcane),
        "TIME_016" => Some(SpellSchool::Holy),
        "TIME_018" => Some(SpellSchool::Holy),
        "TIME_030" => Some(SpellSchool::Shadow),
        "TIME_212" => Some(SpellSchool::Nature),
        "TIME_215" => Some(SpellSchool::Nature),
        "TIME_216" => Some(SpellSchool::Nature),
        "TIME_218" => Some(SpellSchool::Nature),
        "TIME_441" => Some(SpellSchool::Fel),
        "TIME_447" => Some(SpellSchool::Holy),
        "TIME_611" => Some(SpellSchool::Frost),
        "TIME_612" => Some(SpellSchool::Shadow),
        "TIME_610" => Some(SpellSchool::Shadow),
        "TIME_616" => Some(SpellSchool::Shadow),
        "TIME_700" => Some(SpellSchool::Holy),
        "TIME_701" => Some(SpellSchool::Nature),
        "TIME_702" => Some(SpellSchool::Nature),
        "TIME_712" => Some(SpellSchool::Shadow),
        "TIME_855" => Some(SpellSchool::Arcane),
        "TIME_857" => Some(SpellSchool::Arcane),
        "TIME_859" => Some(SpellSchool::Arcane),
        // M3-W3 — The End of Time miniset (the school predicates read by
        // the quest progress and the Shadow-spell pool)
        "END_000" => Some(SpellSchool::Shadow),
        "END_005" => Some(SpellSchool::Shadow),
        "END_009" => Some(SpellSchool::Nature),
        "END_011" => Some(SpellSchool::Holy),
        "END_020" => Some(SpellSchool::Shadow),
        "END_023" => Some(SpellSchool::Frost),
        "END_024" => Some(SpellSchool::Fire),
        "END_025" => Some(SpellSchool::Fire),
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
    /// Play 6 Temporary cards (TLC_446) — wired in W2: the play path fires
    /// it when a played card carries the Temporary marker (the W2
    /// primitive; W4 cards create the markers).
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
    /// Play 3 Beasts or Undead (TLC_EVENT_400 Storm the Gates — the
    /// sidequest; a played minion of either race counts once per play)
    PlayBeastsOrUndead,
    /// Fill your hand, then empty it (END_017 Battle at the End Time — the
    /// M3-W3 miniset quest; a SEQUENCE: the hand must reach 10 once
    /// ("filled"), then be emptied to 0 by plays. The engine tracks the
    /// two markers per quest in `engine::quest` (filled / emptied) and
    /// completes on the second — the exact "fill, then empty" ordering is
    /// preserved, §22)
    FillThenEmptyHand,
}

/// Static definition of a quest card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDef {
    /// The progress condition
    pub condition: QuestCondition,
    /// Progress needed to complete the quest
    pub target: u32,
    /// Reward resolved on completion (battlecry-style resolution)
    pub reward: CardEffect,
    /// Repeatable quests reset to 0 on completion and stay in the slot
    pub repeatable: bool,
    /// Optional second progress bar (TLC_817, M2-W2): `None` for every
    /// single-bar quest. When present, each bar progresses independently
    /// (each condition matches its own bar) and its reward resolves at its
    /// own target; the card leaves the quest slot only when BOTH bars are
    /// done.
    pub second: Option<SecondQuestDef>,
}

/// The optional second progress bar of a dual-bar quest (TLC_817 Reach
/// Equilibrium, 2025–2026 expansions M2-W2 — cast 4 Holy spells AND 4
/// Shadow spells). Not repeatable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondQuestDef {
    /// The second bar's progress condition
    pub condition: QuestCondition,
    /// Progress needed to complete the second bar
    pub target: u32,
    /// Reward resolved when the second bar completes
    pub reward: CardEffect,
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
                card_id: "TLC_229t14", // Ashalon, Ridge Guardian
            },
            repeatable: false,
            second: None,
        }),
        "TLC_239" => Some(&QuestDef {
            condition: QuestCondition::FillBoardOnTurns,
            target: 3,
            reward: CardEffect::EquipWeapon {
                card_id: "TLC_239t", // The Everbloom
            },
            repeatable: false,
            second: None,
        }),
        "TLC_426" => Some(&QuestDef {
            condition: QuestCondition::SummonMinionsOfRace { race: Race::Murloc },
            target: 6,
            // The repeatable reward: permanently, Murlocs the owner summons
            // gain +1/+1 (the player flag consumed by the summon hook).
            reward: CardEffect::SetMurlocSummonBuff,
            repeatable: true,
            second: None,
        }),
        "TLC_433" => Some(&QuestDef {
            condition: QuestCondition::SpendCorpses,
            target: 15,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_433t", // Tyrax, Bone Terror
            },
            repeatable: false,
            second: None,
        }),
        "TLC_446" => Some(&QuestDef {
            condition: QuestCondition::PlayTemporaryCards,
            target: 6,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_446t1", // Underfel Rift
            },
            repeatable: false,
            second: None,
        }),
        "TLC_460" => Some(&QuestDef {
            condition: QuestCondition::DiscoverCards,
            target: 7,
            reward: CardEffect::EquipWeapon {
                card_id: "TLC_460t", // The Origin Stone
            },
            repeatable: false,
            second: None,
        }),
        "TLC_513" => Some(&QuestDef {
            condition: QuestCondition::ShuffleCards,
            target: 5,
            // The official reward replaces the hero with TLC_513t Master
            // Dusk — hero replacement is simplified away (§15): the reward
            // summons the two Tortollan Ninjas (TLC_513t2) directly.
            reward: CardEffect::SummonMultipleMinions {
                card_id: "TLC_513t2",
                count: 2,
            },
            repeatable: false,
            second: None,
        }),
        "TLC_602" => Some(&QuestDef {
            condition: QuestCondition::SurviveTurns,
            target: 10,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_602t", // Latorvius, Gaze of the City
            },
            repeatable: false,
            second: None,
        }),
        "TLC_631" => Some(&QuestDef {
            condition: QuestCondition::DealExactDamage { amount: 2 },
            target: 12,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_631t", // Gorishi Colossus
            },
            repeatable: false,
            second: None,
        }),
        // The dual-bar quest (M2-W2): the official card tracks both quests
        // on one card — cast 4 Holy spells (reward TLC_817t3 Sol'etos,
        // Life's Breath) AND 4 Shadow spells (reward TLC_817t4 Sol'etos,
        // Death's Touch); the card leaves the quest slot only when both
        // bars complete.
        "TLC_817" => Some(&QuestDef {
            condition: QuestCondition::CastSpellsOfSchool {
                school: SpellSchool::Holy,
            },
            target: 4,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_817t3", // Sol'etos, Life's Breath
            },
            repeatable: false,
            second: Some(SecondQuestDef {
                condition: QuestCondition::CastSpellsOfSchool {
                    school: SpellSchool::Shadow,
                },
                target: 4,
                reward: CardEffect::SummonMinion {
                    card_id: "TLC_817t4", // Sol'etos, Death's Touch
                },
            }),
        }),
        "TLC_830" => Some(&QuestDef {
            condition: QuestCondition::PlayBeastsOfAttack,
            target: 4,
            reward: CardEffect::SummonMinion {
                card_id: "TLC_830t", // Shokk, Jungle Tyrant
            },
            repeatable: false,
            second: None,
        }),
        // The sidequest (M2-W4a): TLC_EVENT_400 Storm the Gates — "Play 3
        // Beasts or Undead. Reward: Craft a custom Zombeast. It costs (3)
        // less." The "custom Zombeast" is simplified to a random Beast with
        // the cost reduction applied (§17 — the real card assembles a
        // Beast from two random halves; the random-pool shape is the D2
        // discover simplification).
        "TLC_EVENT_400" => Some(&QuestDef {
            condition: QuestCondition::PlayBeastsOrUndead,
            target: 3,
            reward: CardEffect::AddRandomBeastCostLess { amount: 3 },
            repeatable: false,
            second: None,
        }),
        // M3-W3 — The End of Time miniset: END_017 Battle at the End Time
        // — "Fill your hand, then empty it. Reward: Tick and Tock" (the
        // condition is the FillThenEmptyHand sequence; the reward summons
        // the 8/8 Dragon token END_017t).
        "END_017" => Some(&QuestDef {
            condition: QuestCondition::FillThenEmptyHand,
            target: 1,
            reward: CardEffect::SummonMinion {
                card_id: "END_017t", // Tick and Tock
            },
            repeatable: false,
            second: None,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spell-school table covers the 22 Un'Goro dump entries plus the
    /// 200 in-window Classic/Core spells, and nothing else. Spot-checks the
    /// school predicates the M2-W4a pools and Gladesong Siren depend on.
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
            // In-window representatives of each school (M2-W4a pools)
            ("CORE_CS2_004", SpellSchool::Holy),
            ("CORE_CS1_130", SpellSchool::Holy),
            ("CORE_SW_088", SpellSchool::Fel),
            ("CORE_BT_035", SpellSchool::Fel),
            ("CORE_EX1_302", SpellSchool::Shadow),
            ("CS2_234", SpellSchool::Shadow),
            ("EX1_610", SpellSchool::Fire),
            ("EX1_289", SpellSchool::Frost),
            ("EX1_246", SpellSchool::Nature),
            ("EX1_287", SpellSchool::Arcane),
        ] {
            assert_eq!(spell_school(id), Some(school));
            schools.push(id);
        }
        assert_eq!(schools.len(), 32);
        // The in-window table must cover the Fel/Holy pools' expected size —
        // every in-window Fel and Holy spell carries a school, so the
        // M2-W4a pools are never empty. The exact counts come from the
        // 2026-08-09 dump extraction.
        let holy: Vec<&str> = [
            "BCON_012",
            "CORE_AT_055",
            "CORE_BT_292",
            "CORE_CFM_604",
            "CORE_CS1_112",
            "CORE_CS1_130",
            "CORE_CS2_004",
            "CORE_CS2_093",
            "CORE_CS2_094",
            "CORE_EX1_619",
            "CORE_KAR_077",
            "CORE_TRL_307",
            "CORE_TSC_076",
            "CS1_112",
            "CS1_130",
            "CS2_004",
            "CS2_087",
            "CS2_089",
            "CS2_092",
            "CS2_093",
            "CS2_094",
            "CS2_236",
            "CS3_016",
            "CS3_029",
            "EX1_132",
            "EX1_136",
            "EX1_184",
            "EX1_192",
            "EX1_194",
            "EX1_349",
            "EX1_354",
            "EX1_355",
            "EX1_363",
            "EX1_365",
            "EX1_371",
            "EX1_379",
            "EX1_384",
            "EX1_619",
            "EX1_621",
            "EX1_624",
            "EX1_626",
            "LEG_TTN_908",
            "TTN_851",
        ]
        .iter()
        .filter(|id| spell_school(id) == Some(SpellSchool::Holy))
        .copied()
        .collect();
        assert_eq!(holy.len(), 43, "every in-window Holy spell is classified");
        assert_eq!(spell_school("TLC_229"), None);
        assert_eq!(spell_school("UNKNOWN"), None);
    }

    /// All 12 quest entries resolve, with the expected condition/target.
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
            (
                "TLC_EVENT_400",
                QuestCondition::PlayBeastsOrUndead,
                3,
                false,
            ),
            ("END_017", QuestCondition::FillThenEmptyHand, 1, false),
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
