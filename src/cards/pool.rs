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
use crate::core::effect::{DiscoverPool, RandomPool};
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::sim::rng::GameRng;
use serde::{Deserialize, Serialize};

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

/// Whether the card belongs to ANY class group (2025–2026 expansions
/// M3-W2a — TIME_002 Aeon Wizard's "spells from your class": the engine
/// has no per-player class, so "your class" is approximated by the union
/// of all class groups — including Rogue, which Pilfer's
/// `is_other_class_card` deliberately excludes, §20).
fn is_class_card(card: &CardDef) -> bool {
    is_other_class_card(card)
        || crate::cards::sets::ROGUE_CLASSIC
            .iter()
            .any(|c| c.id == card.id)
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

/// The fixed Frost Rune pool — Crypt Map's "Discover a Frost Rune card"
/// pool (2025–2026 expansions M2-W4a): the four in-window Frost Rune
/// Death Knight cards (TLC_435 Crypt Map itself is excluded — a card
/// cannot discover itself). The real pool is the Frost Rune tag filter;
/// the fixed table is the registered simplification (fidelity-debt §17).
pub const FROST_RUNE_POOL: &[&str] = &[
    "TLC_401", // Bonechill Stegodon
    "TLC_432", // (W2 — Frost Rune Death Knight card)
    "TLC_439", // Wave of Tar
    "TLC_440", // (W1 — Frost Rune Death Knight card)
];

/// The Mask pool — Costume Merchant's "get a random Mask from another
/// class" (2025–2026 expansions M2-W4c): all five Masks are non-Rogue, so
/// the "from another class" filter leaves the full set. The fixed table is
/// the D2 random simplification (fidelity-debt §19).
pub const MASK_POOL: &[&str] = &[
    "DINO_402", // Bat Mask (Warlock)
    "DINO_403", // Devilsaur Mask (Hunter)
    "DINO_428", // Behemoth Mask (Priest)
    "DINO_429", // Sheep Mask (Mage)
    "DINO_432", // Panther Mask (Druid)
];

/// The multi-tribe minion pool — Tortotem's "get a random minion with
/// multiple minion types" (2025–2026 expansions M2-W4c). The engine models
/// a second tribe per-card in apply_card_keywords; filtering the active
/// window for cards that carry two tribes yields exactly one minion —
/// Mythical Terror (Core Set). The fixed table is the D2 random
/// simplification (fidelity-debt §19) — the pool-closure invariant (a
/// fixed pool is a subset of the active window) holds.
pub const MULTI_TRIBE_MINION_POOL: &[&str] = &["CORE_TTN_866"]; // Mythical Terror

/// Card rarity (2025–2026 expansions M2-W4a — Relic Miner's "Discover a
/// card of the same Rarity" pool). The engine has no rarity model; the
/// table below is the static extraction of the official dump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    /// Common
    Common,
    /// Rare
    Rare,
    /// Epic
    Epic,
    /// Legendary
    Legendary,
}

/// Rarity of a card id, extracted from the official card dump (2026-08-09).
///
/// Covers the active-window (Classic-era/Core) collectible cards plus the
/// Un'Goro collectible cards — the decks W4a games can hold. `None` for
/// tokens and every other card (Relic Miner falls back to the full
/// in-window pool then, see fidelity-debt §17).
#[must_use]
pub fn rarity_of(card_id: &str) -> Option<Rarity> {
    match card_id {
        "BG31_BOB" => Some(Rarity::Legendary),
        "BT_173" => Some(Rarity::Common),
        "BT_175" => Some(Rarity::Common),
        "BT_271" => Some(Rarity::Epic),
        "BT_351" => Some(Rarity::Common),
        "BT_354" => Some(Rarity::Rare),
        "BT_355" => Some(Rarity::Epic),
        "BT_407" => Some(Rarity::Common),
        "BT_416" => Some(Rarity::Rare),
        "BT_427" => Some(Rarity::Rare),
        "BT_481" => Some(Rarity::Legendary),
        "BT_487" => Some(Rarity::Rare),
        "BT_488" => Some(Rarity::Rare),
        "BT_490" => Some(Rarity::Common),
        "BT_510" => Some(Rarity::Epic),
        "BT_752" => Some(Rarity::Common),
        "BT_753" => Some(Rarity::Common),
        "BT_801" => Some(Rarity::Epic),
        "BT_814" => Some(Rarity::Rare),
        "BT_922" => Some(Rarity::Common),
        "BT_937" => Some(Rarity::Legendary),
        "CORE_AT_037" => Some(Rarity::Common),
        "CORE_AT_052" => Some(Rarity::Common),
        "CORE_AT_055" => Some(Rarity::Common),
        "CORE_AT_062" => Some(Rarity::Rare),
        "CORE_AT_064" => Some(Rarity::Common),
        "CORE_AT_123" => Some(Rarity::Legendary),
        "CORE_AV_107" => Some(Rarity::Rare),
        "CORE_AV_337" => Some(Rarity::Common),
        "CORE_BAR_310" => Some(Rarity::Rare),
        "CORE_BAR_311" => Some(Rarity::Common),
        "CORE_BAR_313" => Some(Rarity::Epic),
        "CORE_BAR_541" => Some(Rarity::Common),
        "CORE_BAR_801" => Some(Rarity::Common),
        "CORE_BAR_812" => Some(Rarity::Common),
        "CORE_BAR_878" => Some(Rarity::Epic),
        "CORE_BOT_222" => Some(Rarity::Common),
        "CORE_BOT_256" => Some(Rarity::Epic),
        "CORE_BOT_451" => Some(Rarity::Rare),
        "CORE_BOT_576" => Some(Rarity::Common),
        "CORE_BRM_013" => Some(Rarity::Common),
        "CORE_BT_035" => Some(Rarity::Common),
        "CORE_BT_072" => Some(Rarity::Rare),
        "CORE_BT_120" => Some(Rarity::Epic),
        "CORE_BT_156" => Some(Rarity::Common),
        "CORE_BT_187" => Some(Rarity::Legendary),
        "CORE_BT_201" => Some(Rarity::Epic),
        "CORE_BT_292" => Some(Rarity::Common),
        "CORE_BT_321" => Some(Rarity::Common),
        "CORE_BT_351" => Some(Rarity::Common),
        "CORE_BT_416" => Some(Rarity::Rare),
        "CORE_BT_480" => Some(Rarity::Common),
        "CORE_BT_491" => Some(Rarity::Common),
        "CORE_BT_493" => Some(Rarity::Rare),
        "CORE_BT_510" => Some(Rarity::Epic),
        "CORE_BT_701" => Some(Rarity::Common),
        "CORE_BT_781" => Some(Rarity::Legendary),
        "CORE_BT_801" => Some(Rarity::Epic),
        "CORE_BT_921" => Some(Rarity::Common),
        "CORE_CATA_001" => Some(Rarity::Legendary),
        "CORE_CATA_002" => Some(Rarity::Legendary),
        "CORE_CATA_004" => Some(Rarity::Legendary),
        "CORE_CATA_006" => Some(Rarity::Legendary),
        "CORE_CATA_007" => Some(Rarity::Rare),
        "CORE_CATA_009" => Some(Rarity::Common),
        "CORE_CFM_344" => Some(Rarity::Legendary),
        "CORE_CFM_604" => Some(Rarity::Rare),
        "CORE_CFM_670" => Some(Rarity::Legendary),
        "CORE_CFM_753" => Some(Rarity::Common),
        "CORE_CFM_781" => Some(Rarity::Legendary),
        "CORE_CFM_790" => Some(Rarity::Epic),
        "CORE_CS1_112" => Some(Rarity::Common),
        "CORE_CS1_130" => Some(Rarity::Common),
        "CORE_CS2_004" => Some(Rarity::Common),
        "CORE_CS2_009" => Some(Rarity::Rare),
        "CORE_CS2_023" => Some(Rarity::Common),
        "CORE_CS2_024" => Some(Rarity::Common),
        "CORE_CS2_028" => Some(Rarity::Rare),
        "CORE_CS2_029" => Some(Rarity::Common),
        "CORE_CS2_032" => Some(Rarity::Epic),
        "CORE_CS2_042" => Some(Rarity::Common),
        "CORE_CS2_053" => Some(Rarity::Epic),
        "CORE_CS2_062" => Some(Rarity::Common),
        "CORE_CS2_072" => Some(Rarity::Common),
        "CORE_CS2_074" => Some(Rarity::Common),
        "CORE_CS2_076" => Some(Rarity::Common),
        "CORE_CS2_093" => Some(Rarity::Common),
        "CORE_CS2_094" => Some(Rarity::Common),
        "CORE_CS2_108" => Some(Rarity::Common),
        "CORE_CS2_122" => Some(Rarity::Common),
        "CORE_CS2_179" => Some(Rarity::Common),
        "CORE_CS2_188" => Some(Rarity::Common),
        "CORE_CS2_189" => Some(Rarity::Common),
        "CORE_CS2_222" => Some(Rarity::Common),
        "CORE_DAL_575" => Some(Rarity::Legendary),
        "CORE_DAL_720" => Some(Rarity::Epic),
        "CORE_DMF_067" => Some(Rarity::Common),
        "CORE_DMF_511" => Some(Rarity::Common),
        "CORE_DRG_024" => Some(Rarity::Common),
        "CORE_DRG_079" => Some(Rarity::Common),
        "CORE_DRG_107" => Some(Rarity::Common),
        "CORE_DRG_256" => Some(Rarity::Legendary),
        "CORE_DRG_403" => Some(Rarity::Epic),
        "CORE_DS1_184" => Some(Rarity::Rare),
        "CORE_DS1_185" => Some(Rarity::Rare),
        "CORE_EDR_001" => Some(Rarity::Rare),
        "CORE_EDR_002" => Some(Rarity::Rare),
        "CORE_EDR_003" => Some(Rarity::Legendary),
        "CORE_EDR_004" => Some(Rarity::Epic),
        "CORE_EDR_004_2026" => Some(Rarity::Epic),
        "CORE_ETC_111" => Some(Rarity::Common),
        "CORE_ETC_523" => Some(Rarity::Rare),
        "CORE_EX1_002" => Some(Rarity::Legendary),
        "CORE_EX1_005" => Some(Rarity::Epic),
        "CORE_EX1_007" => Some(Rarity::Common),
        "CORE_EX1_010" => Some(Rarity::Common),
        "CORE_EX1_011" => Some(Rarity::Common),
        "CORE_EX1_012" => Some(Rarity::Legendary),
        "CORE_EX1_014" => Some(Rarity::Legendary),
        "CORE_EX1_028" => Some(Rarity::Common),
        "CORE_EX1_043" => Some(Rarity::Rare),
        "CORE_EX1_058" => Some(Rarity::Rare),
        "CORE_EX1_059" => Some(Rarity::Rare),
        "CORE_EX1_082" => Some(Rarity::Common),
        "CORE_EX1_096" => Some(Rarity::Common),
        "CORE_EX1_100" => Some(Rarity::Legendary),
        "CORE_EX1_103" => Some(Rarity::Rare),
        "CORE_EX1_110" => Some(Rarity::Legendary),
        "CORE_EX1_129" => Some(Rarity::Common),
        "CORE_EX1_131" => Some(Rarity::Common),
        "CORE_EX1_134" => Some(Rarity::Rare),
        "CORE_EX1_145" => Some(Rarity::Epic),
        "CORE_EX1_154" => Some(Rarity::Common),
        "CORE_EX1_160" => Some(Rarity::Common),
        "CORE_EX1_162" => Some(Rarity::Common),
        "CORE_EX1_169" => Some(Rarity::Rare),
        "CORE_EX1_189" => Some(Rarity::Legendary),
        "CORE_EX1_193" => Some(Rarity::Common),
        "CORE_EX1_197" => Some(Rarity::Epic),
        "CORE_EX1_198" => Some(Rarity::Legendary),
        "CORE_EX1_238" => Some(Rarity::Common),
        "CORE_EX1_246" => Some(Rarity::Common),
        "CORE_EX1_250" => Some(Rarity::Epic),
        "CORE_EX1_259" => Some(Rarity::Rare),
        "CORE_EX1_278" => Some(Rarity::Common),
        "CORE_EX1_287" => Some(Rarity::Rare),
        "CORE_EX1_289" => Some(Rarity::Common),
        "CORE_EX1_302" => Some(Rarity::Common),
        "CORE_EX1_309" => Some(Rarity::Rare),
        "CORE_EX1_310" => Some(Rarity::Rare),
        "CORE_EX1_312" => Some(Rarity::Epic),
        "CORE_EX1_319" => Some(Rarity::Common),
        "CORE_EX1_323" => Some(Rarity::Legendary),
        "CORE_EX1_362" => Some(Rarity::Common),
        "CORE_EX1_383" => Some(Rarity::Legendary),
        "CORE_EX1_391" => Some(Rarity::Common),
        "CORE_EX1_414" => Some(Rarity::Legendary),
        "CORE_EX1_506" => Some(Rarity::Common),
        "CORE_EX1_507" => Some(Rarity::Epic),
        "CORE_EX1_509" => Some(Rarity::Rare),
        "CORE_EX1_559" => Some(Rarity::Legendary),
        "CORE_EX1_604" => Some(Rarity::Rare),
        "CORE_EX1_606" => Some(Rarity::Common),
        "CORE_EX1_610" => Some(Rarity::Common),
        "CORE_EX1_611" => Some(Rarity::Common),
        "CORE_EX1_619" => Some(Rarity::Rare),
        "CORE_GIL_531" => Some(Rarity::Common),
        "CORE_GIL_534" => Some(Rarity::Common),
        "CORE_GIL_558" => Some(Rarity::Common),
        "CORE_GIL_577" => Some(Rarity::Epic),
        "CORE_GIL_622" => Some(Rarity::Rare),
        "CORE_GIL_623" => Some(Rarity::Rare),
        "CORE_GIL_836" => Some(Rarity::Rare),
        "CORE_GVG_059" => Some(Rarity::Epic),
        "CORE_GVG_061" => Some(Rarity::Rare),
        "CORE_GVG_085" => Some(Rarity::Common),
        "CORE_GVG_103" => Some(Rarity::Common),
        "CORE_GVG_114" => Some(Rarity::Legendary),
        "CORE_ICC_038" => Some(Rarity::Common),
        "CORE_ICC_055" => Some(Rarity::Common),
        "CORE_ICC_210" => Some(Rarity::Common),
        "CORE_ICC_214" => Some(Rarity::Epic),
        "CORE_ICC_407" => Some(Rarity::Epic),
        "CORE_KAR_057" => Some(Rarity::Rare),
        "CORE_KAR_061" => Some(Rarity::Legendary),
        "CORE_KAR_062" => Some(Rarity::Common),
        "CORE_KAR_069" => Some(Rarity::Common),
        "CORE_KAR_077" => Some(Rarity::Common),
        "CORE_LOE_039" => Some(Rarity::Common),
        "CORE_LOOT_013" => Some(Rarity::Common),
        "CORE_LOOT_044" => Some(Rarity::Epic),
        "CORE_LOOT_101" => Some(Rarity::Rare),
        "CORE_LOOT_137" => Some(Rarity::Common),
        "CORE_LOOT_309" => Some(Rarity::Common),
        "CORE_LOOT_368" => Some(Rarity::Epic),
        "CORE_LOOT_373" => Some(Rarity::Common),
        "CORE_LOOT_413" => Some(Rarity::Common),
        "CORE_NEW1_018" => Some(Rarity::Common),
        "CORE_NEW1_020" => Some(Rarity::Rare),
        "CORE_NEW1_021" => Some(Rarity::Epic),
        "CORE_NEW1_022" => Some(Rarity::Common),
        "CORE_NEW1_023" => Some(Rarity::Common),
        "CORE_NEW1_027" => Some(Rarity::Epic),
        "CORE_NEW1_031" => Some(Rarity::Common),
        "CORE_NX2_028" => Some(Rarity::Common),
        "CORE_OG_031" => Some(Rarity::Epic),
        "CORE_OG_044" => Some(Rarity::Legendary),
        "CORE_OG_047" => Some(Rarity::Common),
        "CORE_OG_149" => Some(Rarity::Common),
        "CORE_OG_211" => Some(Rarity::Epic),
        "CORE_OG_218" => Some(Rarity::Common),
        "CORE_ONY_018" => Some(Rarity::Rare),
        "CORE_ONY_022" => Some(Rarity::Rare),
        "CORE_REV_023" => Some(Rarity::Epic),
        "CORE_REV_308" => Some(Rarity::Common),
        "CORE_REV_946" => Some(Rarity::Rare),
        "CORE_REV_990" => Some(Rarity::Rare),
        "CORE_RLK_062" => Some(Rarity::Common),
        "CORE_RLK_063" => Some(Rarity::Epic),
        "CORE_RLK_066" => Some(Rarity::Rare),
        "CORE_RLK_083" => Some(Rarity::Rare),
        "CORE_RLK_086" => Some(Rarity::Legendary),
        "CORE_RLK_087" => Some(Rarity::Common),
        "CORE_RLK_116" => Some(Rarity::Common),
        "CORE_RLK_118" => Some(Rarity::Rare),
        "CORE_RLK_121" => Some(Rarity::Common),
        "CORE_RLK_505" => Some(Rarity::Rare),
        "CORE_RLK_506" => Some(Rarity::Rare),
        "CORE_RLK_567" => Some(Rarity::Legendary),
        "CORE_RLK_657" => Some(Rarity::Epic),
        "CORE_RLK_706" => Some(Rarity::Legendary),
        "CORE_RLK_712" => Some(Rarity::Rare),
        "CORE_RLK_745" => Some(Rarity::Common),
        "CORE_RLK_814" => Some(Rarity::Common),
        "CORE_SCH_181" => Some(Rarity::Legendary),
        "CORE_SCH_512" => Some(Rarity::Rare),
        "CORE_SCH_605" => Some(Rarity::Common),
        "CORE_SCH_713" => Some(Rarity::Rare),
        "CORE_SCH_717" => Some(Rarity::Legendary),
        "CORE_SW_047" => Some(Rarity::Legendary),
        "CORE_SW_066" => Some(Rarity::Common),
        "CORE_SW_068" => Some(Rarity::Common),
        "CORE_SW_072" => Some(Rarity::Common),
        "CORE_SW_088" => Some(Rarity::Common),
        "CORE_SW_108" => Some(Rarity::Rare),
        "CORE_SW_429" => Some(Rarity::Common),
        "CORE_SW_439" => Some(Rarity::Rare),
        "CORE_SW_442" => Some(Rarity::Common),
        "CORE_TID_931" => Some(Rarity::Common),
        "CORE_TRL_111" => Some(Rarity::Common),
        "CORE_TRL_240" => Some(Rarity::Common),
        "CORE_TRL_307" => Some(Rarity::Common),
        "CORE_TRL_345" => Some(Rarity::Legendary),
        "CORE_TRL_900" => Some(Rarity::Legendary),
        "CORE_TSC_076" => Some(Rarity::Common),
        "CORE_TSC_650" => Some(Rarity::Common),
        "CORE_TTN_843" => Some(Rarity::Common),
        "CORE_TTN_866" => Some(Rarity::Epic),
        "CORE_ULD_133" => Some(Rarity::Epic),
        "CORE_ULD_152" => Some(Rarity::Common),
        "CORE_ULD_165" => Some(Rarity::Epic),
        "CORE_ULD_178" => Some(Rarity::Legendary),
        "CORE_ULD_191" => Some(Rarity::Common),
        "CORE_ULD_271" => Some(Rarity::Common),
        "CORE_ULD_280" => Some(Rarity::Rare),
        "CORE_ULD_723" => Some(Rarity::Common),
        "CORE_UNG_084" => Some(Rarity::Common),
        "CORE_UNG_205" => Some(Rarity::Common),
        "CORE_UNG_809" => Some(Rarity::Common),
        "CORE_UNG_848" => Some(Rarity::Epic),
        "CORE_UNG_912" => Some(Rarity::Common),
        "CORE_UNG_928" => Some(Rarity::Common),
        "CORE_UNG_952" => Some(Rarity::Rare),
        "CORE_WC_042" => Some(Rarity::Common),
        "CORE_WC_701" => Some(Rarity::Common),
        "CORE_WON_096" => Some(Rarity::Common),
        "CORE_WON_141" => Some(Rarity::Common),
        "CORE_WON_145" => Some(Rarity::Legendary),
        "CORE_WON_337" => Some(Rarity::Common),
        "CORE_WON_350" => Some(Rarity::Common),
        "CORE_WON_351" => Some(Rarity::Rare),
        "CORE_WW_329" => Some(Rarity::Common),
        "CORE_WW_374" => Some(Rarity::Epic),
        "CORE_YOD_026" => Some(Rarity::Common),
        "CORE_YOP_001" => Some(Rarity::Common),
        "CORE_YOP_034" => Some(Rarity::Rare),
        "CS1_069" => Some(Rarity::Common),
        "CS1_129" => Some(Rarity::Common),
        "CS2_028" => Some(Rarity::Rare),
        "CS2_031" => Some(Rarity::Common),
        "CS2_038" => Some(Rarity::Rare),
        "CS2_053" => Some(Rarity::Epic),
        "CS2_059" => Some(Rarity::Common),
        "CS2_073" => Some(Rarity::Common),
        "CS2_104" => Some(Rarity::Common),
        "CS2_117" => Some(Rarity::Common),
        "CS2_146" => Some(Rarity::Common),
        "CS2_151" => Some(Rarity::Common),
        "CS2_161" => Some(Rarity::Rare),
        "CS2_169" => Some(Rarity::Common),
        "CS2_181" => Some(Rarity::Rare),
        "CS2_188" => Some(Rarity::Common),
        "CS2_203" => Some(Rarity::Common),
        "CS2_221" => Some(Rarity::Common),
        "CS2_227" => Some(Rarity::Common),
        "CS2_231" => Some(Rarity::Common),
        "CS2_233" => Some(Rarity::Rare),
        "CS2_235" => Some(Rarity::Common),
        "CS2_236" => Some(Rarity::Common),
        "CS3_002" => Some(Rarity::Rare),
        "CS3_003" => Some(Rarity::Epic),
        "CS3_005" => Some(Rarity::Legendary),
        "CS3_007" => Some(Rarity::Common),
        "CS3_008" => Some(Rarity::Common),
        "CS3_009" => Some(Rarity::Rare),
        "CS3_012" => Some(Rarity::Rare),
        "CS3_014" => Some(Rarity::Rare),
        "CS3_016" => Some(Rarity::Epic),
        "CS3_019" => Some(Rarity::Legendary),
        "CS3_020" => Some(Rarity::Rare),
        "CS3_021" => Some(Rarity::Common),
        "CS3_022" => Some(Rarity::Rare),
        "CS3_024" => Some(Rarity::Legendary),
        "CS3_025" => Some(Rarity::Legendary),
        "CS3_027" => Some(Rarity::Rare),
        "CS3_029" => Some(Rarity::Epic),
        "CS3_030" => Some(Rarity::Common),
        "CS3_035" => Some(Rarity::Legendary),
        "CS3_038" => Some(Rarity::Common),
        "Core_CS2_200" => Some(Rarity::Common),
        "Core_LOE_115" => Some(Rarity::Common),
        "Core_UNG_072" => Some(Rarity::Rare),
        "DINO_130" => Some(Rarity::Common),
        "DINO_131" => Some(Rarity::Rare),
        "DINO_132" => Some(Rarity::Rare),
        "DINO_136" => Some(Rarity::Common),
        "DINO_137" => Some(Rarity::Common),
        "DINO_138" => Some(Rarity::Rare),
        "DINO_400" => Some(Rarity::Rare),
        "DINO_401" => Some(Rarity::Legendary),
        "DINO_402" => Some(Rarity::Common),
        "DINO_403" => Some(Rarity::Common),
        "DINO_404" => Some(Rarity::Common),
        "DINO_405" => Some(Rarity::Common),
        "DINO_406" => Some(Rarity::Common),
        "DINO_407" => Some(Rarity::Legendary),
        "DINO_408" => Some(Rarity::Rare),
        "DINO_409" => Some(Rarity::Rare),
        "DINO_410" => Some(Rarity::Legendary),
        "DINO_410t2" => Some(Rarity::Legendary),
        "DINO_410t3" => Some(Rarity::Legendary),
        "DINO_410t4" => Some(Rarity::Legendary),
        "DINO_410t5" => Some(Rarity::Legendary),
        "DINO_411" => Some(Rarity::Rare),
        "DINO_412" => Some(Rarity::Rare),
        "DINO_413" => Some(Rarity::Rare),
        "DINO_414" => Some(Rarity::Common),
        "DINO_414e" => Some(Rarity::Common),
        "DINO_415" => Some(Rarity::Rare),
        "DINO_416" => Some(Rarity::Rare),
        "DINO_417" => Some(Rarity::Common),
        "DINO_419" => Some(Rarity::Common),
        "DINO_421" => Some(Rarity::Rare),
        "DINO_422" => Some(Rarity::Rare),
        "DINO_424" => Some(Rarity::Rare),
        "DINO_426" => Some(Rarity::Common),
        "DINO_427" => Some(Rarity::Common),
        "DINO_428" => Some(Rarity::Rare),
        "DINO_429" => Some(Rarity::Rare),
        "DINO_430" => Some(Rarity::Legendary),
        "DINO_431" => Some(Rarity::Common),
        "DINO_432" => Some(Rarity::Rare),
        "DINO_433" => Some(Rarity::Common),
        "DINO_434" => Some(Rarity::Common),
        "DINO_435" => Some(Rarity::Epic),
        "DS1_188" => Some(Rarity::Epic),
        "DS1_233" => Some(Rarity::Common),
        "EX1_001" => Some(Rarity::Rare),
        "EX1_002" => Some(Rarity::Legendary),
        "EX1_004" => Some(Rarity::Rare),
        "EX1_005" => Some(Rarity::Epic),
        "EX1_006" => Some(Rarity::Rare),
        "EX1_007" => Some(Rarity::Common),
        "EX1_008" => Some(Rarity::Common),
        "EX1_009" => Some(Rarity::Rare),
        "EX1_010" => Some(Rarity::Common),
        "EX1_012" => Some(Rarity::Legendary),
        "EX1_014" => Some(Rarity::Legendary),
        "EX1_016" => Some(Rarity::Legendary),
        "EX1_017" => Some(Rarity::Common),
        "EX1_020" => Some(Rarity::Common),
        "EX1_021" => Some(Rarity::Common),
        "EX1_023" => Some(Rarity::Common),
        "EX1_028" => Some(Rarity::Common),
        "EX1_029" => Some(Rarity::Common),
        "EX1_032" => Some(Rarity::Rare),
        "EX1_033" => Some(Rarity::Common),
        "EX1_043" => Some(Rarity::Rare),
        "EX1_044" => Some(Rarity::Rare),
        "EX1_045" => Some(Rarity::Rare),
        "EX1_046" => Some(Rarity::Common),
        "EX1_048" => Some(Rarity::Common),
        "EX1_049" => Some(Rarity::Common),
        "EX1_050" => Some(Rarity::Rare),
        "EX1_055" => Some(Rarity::Rare),
        "EX1_057" => Some(Rarity::Common),
        "EX1_058" => Some(Rarity::Rare),
        "EX1_059" => Some(Rarity::Rare),
        "EX1_062" => Some(Rarity::Legendary),
        "EX1_067" => Some(Rarity::Rare),
        "EX1_076" => Some(Rarity::Rare),
        "EX1_080" => Some(Rarity::Rare),
        "EX1_082" => Some(Rarity::Common),
        "EX1_083" => Some(Rarity::Legendary),
        "EX1_085" => Some(Rarity::Rare),
        "EX1_089" => Some(Rarity::Rare),
        "EX1_091" => Some(Rarity::Epic),
        "EX1_093" => Some(Rarity::Rare),
        "EX1_095" => Some(Rarity::Rare),
        "EX1_096" => Some(Rarity::Common),
        "EX1_097" => Some(Rarity::Rare),
        "EX1_100" => Some(Rarity::Legendary),
        "EX1_102" => Some(Rarity::Rare),
        "EX1_103" => Some(Rarity::Rare),
        "EX1_105" => Some(Rarity::Epic),
        "EX1_110" => Some(Rarity::Legendary),
        "EX1_112" => Some(Rarity::Legendary),
        "EX1_116" => Some(Rarity::Legendary),
        "EX1_124" => Some(Rarity::Common),
        "EX1_126" => Some(Rarity::Common),
        "EX1_128" => Some(Rarity::Common),
        "EX1_130" => Some(Rarity::Common),
        "EX1_131" => Some(Rarity::Common),
        "EX1_132" => Some(Rarity::Common),
        "EX1_133" => Some(Rarity::Rare),
        "EX1_134" => Some(Rarity::Rare),
        "EX1_136" => Some(Rarity::Common),
        "EX1_137" => Some(Rarity::Rare),
        "EX1_144" => Some(Rarity::Common),
        "EX1_145" => Some(Rarity::Epic),
        "EX1_154" => Some(Rarity::Common),
        "EX1_155" => Some(Rarity::Common),
        "EX1_158" => Some(Rarity::Common),
        "EX1_160" => Some(Rarity::Common),
        "EX1_161" => Some(Rarity::Common),
        "EX1_162" => Some(Rarity::Common),
        "EX1_164" => Some(Rarity::Rare),
        "EX1_165" => Some(Rarity::Common),
        "EX1_166" => Some(Rarity::Rare),
        "EX1_170" => Some(Rarity::Rare),
        "EX1_178" => Some(Rarity::Epic),
        "EX1_179" => Some(Rarity::Epic),
        "EX1_180" => Some(Rarity::Common),
        "EX1_181" => Some(Rarity::Common),
        "EX1_182" => Some(Rarity::Common),
        "EX1_183" => Some(Rarity::Common),
        "EX1_184" => Some(Rarity::Rare),
        "EX1_185" => Some(Rarity::Rare),
        "EX1_186" => Some(Rarity::Rare),
        "EX1_187" => Some(Rarity::Rare),
        "EX1_188" => Some(Rarity::Epic),
        "EX1_189" => Some(Rarity::Legendary),
        "EX1_190" => Some(Rarity::Legendary),
        "EX1_195" => Some(Rarity::Rare),
        "EX1_196" => Some(Rarity::Rare),
        "EX1_197" => Some(Rarity::Epic),
        "EX1_198" => Some(Rarity::Legendary),
        "EX1_238" => Some(Rarity::Common),
        "EX1_241" => Some(Rarity::Rare),
        "EX1_243" => Some(Rarity::Common),
        "EX1_245" => Some(Rarity::Common),
        "EX1_247" => Some(Rarity::Common),
        "EX1_248" => Some(Rarity::Rare),
        "EX1_249" => Some(Rarity::Legendary),
        "EX1_250" => Some(Rarity::Epic),
        "EX1_251" => Some(Rarity::Common),
        "EX1_258" => Some(Rarity::Common),
        "EX1_259" => Some(Rarity::Rare),
        "EX1_274" => Some(Rarity::Rare),
        "EX1_275" => Some(Rarity::Common),
        "EX1_279" => Some(Rarity::Epic),
        "EX1_283" => Some(Rarity::Common),
        "EX1_284" => Some(Rarity::Rare),
        "EX1_287" => Some(Rarity::Rare),
        "EX1_289" => Some(Rarity::Common),
        "EX1_294" => Some(Rarity::Common),
        "EX1_295" => Some(Rarity::Epic),
        "EX1_298" => Some(Rarity::Legendary),
        "EX1_301" => Some(Rarity::Rare),
        "EX1_303" => Some(Rarity::Rare),
        "EX1_304" => Some(Rarity::Rare),
        "EX1_309" => Some(Rarity::Rare),
        "EX1_310" => Some(Rarity::Rare),
        "EX1_312" => Some(Rarity::Epic),
        "EX1_313" => Some(Rarity::Epic),
        "EX1_315" => Some(Rarity::Common),
        "EX1_316" => Some(Rarity::Common),
        "EX1_317" => Some(Rarity::Common),
        "EX1_319" => Some(Rarity::Common),
        "EX1_320" => Some(Rarity::Epic),
        "EX1_323" => Some(Rarity::Legendary),
        "EX1_332" => Some(Rarity::Common),
        "EX1_334" => Some(Rarity::Rare),
        "EX1_335" => Some(Rarity::Common),
        "EX1_339" => Some(Rarity::Common),
        "EX1_341" => Some(Rarity::Rare),
        "EX1_345" => Some(Rarity::Epic),
        "EX1_349" => Some(Rarity::Rare),
        "EX1_350" => Some(Rarity::Legendary),
        "EX1_354" => Some(Rarity::Epic),
        "EX1_355" => Some(Rarity::Rare),
        "EX1_362" => Some(Rarity::Common),
        "EX1_363" => Some(Rarity::Common),
        "EX1_365" => Some(Rarity::Rare),
        "EX1_366" => Some(Rarity::Epic),
        "EX1_379" => Some(Rarity::Common),
        "EX1_382" => Some(Rarity::Rare),
        "EX1_383" => Some(Rarity::Legendary),
        "EX1_384" => Some(Rarity::Epic),
        "EX1_390" => Some(Rarity::Common),
        "EX1_391" => Some(Rarity::Common),
        "EX1_392" => Some(Rarity::Common),
        "EX1_393" => Some(Rarity::Common),
        "EX1_396" => Some(Rarity::Common),
        "EX1_398" => Some(Rarity::Common),
        "EX1_402" => Some(Rarity::Rare),
        "EX1_405" => Some(Rarity::Common),
        "EX1_407" => Some(Rarity::Epic),
        "EX1_408" => Some(Rarity::Rare),
        "EX1_409" => Some(Rarity::Rare),
        "EX1_410" => Some(Rarity::Epic),
        "EX1_411" => Some(Rarity::Epic),
        "EX1_412" => Some(Rarity::Common),
        "EX1_414" => Some(Rarity::Legendary),
        "EX1_507" => Some(Rarity::Epic),
        "EX1_509" => Some(Rarity::Rare),
        "EX1_522" => Some(Rarity::Epic),
        "EX1_531" => Some(Rarity::Common),
        "EX1_533" => Some(Rarity::Rare),
        "EX1_534" => Some(Rarity::Rare),
        "EX1_536" => Some(Rarity::Rare),
        "EX1_537" => Some(Rarity::Rare),
        "EX1_538" => Some(Rarity::Common),
        "EX1_543" => Some(Rarity::Legendary),
        "EX1_544" => Some(Rarity::Rare),
        "EX1_549" => Some(Rarity::Epic),
        "EX1_554" => Some(Rarity::Epic),
        "EX1_556" => Some(Rarity::Common),
        "EX1_557" => Some(Rarity::Legendary),
        "EX1_558" => Some(Rarity::Legendary),
        "EX1_559" => Some(Rarity::Legendary),
        "EX1_560" => Some(Rarity::Legendary),
        "EX1_561" => Some(Rarity::Legendary),
        "EX1_562" => Some(Rarity::Legendary),
        "EX1_563" => Some(Rarity::Legendary),
        "EX1_564" => Some(Rarity::Epic),
        "EX1_567" => Some(Rarity::Epic),
        "EX1_570" => Some(Rarity::Rare),
        "EX1_571" => Some(Rarity::Epic),
        "EX1_572" => Some(Rarity::Legendary),
        "EX1_573" => Some(Rarity::Legendary),
        "EX1_575" => Some(Rarity::Rare),
        "EX1_577" => Some(Rarity::Legendary),
        "EX1_578" => Some(Rarity::Rare),
        "EX1_583" => Some(Rarity::Common),
        "EX1_584" => Some(Rarity::Rare),
        "EX1_586" => Some(Rarity::Epic),
        "EX1_590" => Some(Rarity::Epic),
        "EX1_591" => Some(Rarity::Rare),
        "EX1_594" => Some(Rarity::Rare),
        "EX1_595" => Some(Rarity::Common),
        "EX1_596" => Some(Rarity::Common),
        "EX1_597" => Some(Rarity::Rare),
        "EX1_603" => Some(Rarity::Common),
        "EX1_604" => Some(Rarity::Rare),
        "EX1_607" => Some(Rarity::Common),
        "EX1_608" => Some(Rarity::Common),
        "EX1_609" => Some(Rarity::Common),
        "EX1_610" => Some(Rarity::Common),
        "EX1_611" => Some(Rarity::Common),
        "EX1_612" => Some(Rarity::Rare),
        "EX1_613" => Some(Rarity::Legendary),
        "EX1_614" => Some(Rarity::Legendary),
        "EX1_616" => Some(Rarity::Rare),
        "EX1_617" => Some(Rarity::Common),
        "EX1_619" => Some(Rarity::Rare),
        "EX1_620" => Some(Rarity::Epic),
        "EX1_621" => Some(Rarity::Common),
        "EX1_623" => Some(Rarity::Common),
        "EX1_624" => Some(Rarity::Rare),
        "EX1_625" => Some(Rarity::Epic),
        "EX1_626" => Some(Rarity::Rare),
        "GIFT_01" => Some(Rarity::Legendary),
        "GIFT_02" => Some(Rarity::Rare),
        "GIFT_03" => Some(Rarity::Rare),
        "GIFT_04" => Some(Rarity::Rare),
        "GIFT_05" => Some(Rarity::Rare),
        "GIFT_06" => Some(Rarity::Rare),
        "GIFT_07" => Some(Rarity::Rare),
        "GIFT_08" => Some(Rarity::Rare),
        "GIFT_09" => Some(Rarity::Rare),
        "GIFT_10" => Some(Rarity::Rare),
        "GIFT_11" => Some(Rarity::Rare),
        "GIFT_12" => Some(Rarity::Rare),
        "LEG_CS3_001" => Some(Rarity::Legendary),
        "LEG_CS3_013" => Some(Rarity::Common),
        "LEG_CS3_015" => Some(Rarity::Rare),
        "LEG_CS3_017" => Some(Rarity::Common),
        "LEG_CS3_028" => Some(Rarity::Rare),
        "LEG_CS3_031" => Some(Rarity::Legendary),
        "LEG_CS3_032" => Some(Rarity::Legendary),
        "LEG_CS3_033" => Some(Rarity::Legendary),
        "LEG_CS3_034" => Some(Rarity::Legendary),
        "LEG_CS3_036" => Some(Rarity::Legendary),
        "LEG_CS3_037" => Some(Rarity::Common),
        "LEG_RLK_034" => Some(Rarity::Common),
        "LEG_RLK_039" => Some(Rarity::Rare),
        "LEG_RLK_071" => Some(Rarity::Legendary),
        "LEG_RLK_077" => Some(Rarity::Common),
        "LEG_RLK_079" => Some(Rarity::Rare),
        "LEG_RLK_082" => Some(Rarity::Legendary),
        "LEG_RLK_085" => Some(Rarity::Legendary),
        "LEG_RLK_101" => Some(Rarity::Rare),
        "LEG_RLK_115" => Some(Rarity::Epic),
        "LEG_RLK_125" => Some(Rarity::Epic),
        "LEG_RLK_224" => Some(Rarity::Legendary),
        "LEG_RLK_226" => Some(Rarity::Rare),
        "LEG_RLK_705" => Some(Rarity::Rare),
        "LEG_RLK_710" => Some(Rarity::Rare),
        "LEG_RLK_715" => Some(Rarity::Rare),
        "LEG_RLK_744" => Some(Rarity::Epic),
        "LEG_RLK_752" => Some(Rarity::Rare),
        "LEG_RLK_753" => Some(Rarity::Rare),
        "LEG_TTN_908" => Some(Rarity::Epic),
        "NEW1_004" => Some(Rarity::Common),
        "NEW1_005" => Some(Rarity::Epic),
        "NEW1_007" => Some(Rarity::Rare),
        "NEW1_008" => Some(Rarity::Epic),
        "NEW1_010" => Some(Rarity::Legendary),
        "NEW1_012" => Some(Rarity::Common),
        "NEW1_014" => Some(Rarity::Rare),
        "NEW1_016" => Some(Rarity::Epic),
        "NEW1_017" => Some(Rarity::Epic),
        "NEW1_018" => Some(Rarity::Common),
        "NEW1_019" => Some(Rarity::Rare),
        "NEW1_020" => Some(Rarity::Rare),
        "NEW1_021" => Some(Rarity::Epic),
        "NEW1_022" => Some(Rarity::Common),
        "NEW1_023" => Some(Rarity::Common),
        "NEW1_024" => Some(Rarity::Legendary),
        "NEW1_025" => Some(Rarity::Rare),
        "NEW1_026" => Some(Rarity::Rare),
        "NEW1_027" => Some(Rarity::Epic),
        "NEW1_029" => Some(Rarity::Legendary),
        "NEW1_030" => Some(Rarity::Legendary),
        "NEW1_036" => Some(Rarity::Rare),
        "NEW1_037" => Some(Rarity::Rare),
        "NEW1_038" => Some(Rarity::Legendary),
        "NEW1_040" => Some(Rarity::Legendary),
        "NEW1_041" => Some(Rarity::Rare),
        "PRO_001" => Some(Rarity::Legendary),
        "RLK_024" => Some(Rarity::Common),
        "RLK_025" => Some(Rarity::Common),
        "RLK_048" => Some(Rarity::Rare),
        "RLK_060" => Some(Rarity::Common),
        "RLK_061" => Some(Rarity::Common),
        "RLK_067" => Some(Rarity::Common),
        "RLK_223" => Some(Rarity::Legendary),
        "RLK_503" => Some(Rarity::Common),
        "RLK_511" => Some(Rarity::Common),
        "RLK_707" => Some(Rarity::Epic),
        "RLK_708" => Some(Rarity::Common),
        "RLK_709" => Some(Rarity::Common),
        "RLK_720" => Some(Rarity::Common),
        "RLK_958" => Some(Rarity::Common),
        "TLC_100" => Some(Rarity::Legendary),
        "TLC_101" => Some(Rarity::Common),
        "TLC_102" => Some(Rarity::Legendary),
        "TLC_106" => Some(Rarity::Legendary),
        "TLC_107" => Some(Rarity::Epic),
        "TLC_109" => Some(Rarity::Common),
        "TLC_110" => Some(Rarity::Legendary),
        "TLC_220" => Some(Rarity::Common),
        "TLC_221" => Some(Rarity::Rare),
        "TLC_222" => Some(Rarity::Common),
        "TLC_223" => Some(Rarity::Rare),
        "TLC_224" => Some(Rarity::Common),
        "TLC_225" => Some(Rarity::Common),
        "TLC_226" => Some(Rarity::Epic),
        "TLC_227" => Some(Rarity::Epic),
        "TLC_228" => Some(Rarity::Legendary),
        "TLC_229" => Some(Rarity::Legendary),
        "TLC_230" => Some(Rarity::Common),
        "TLC_231" => Some(Rarity::Common),
        "TLC_232" => Some(Rarity::Rare),
        "TLC_233" => Some(Rarity::Rare),
        "TLC_234" => Some(Rarity::Epic),
        "TLC_235" => Some(Rarity::Epic),
        "TLC_236" => Some(Rarity::Rare),
        "TLC_237" => Some(Rarity::Common),
        "TLC_239" => Some(Rarity::Legendary),
        "TLC_240" => Some(Rarity::Rare),
        "TLC_241" => Some(Rarity::Legendary),
        "TLC_242" => Some(Rarity::Common),
        "TLC_243" => Some(Rarity::Common),
        "TLC_244" => Some(Rarity::Common),
        "TLC_245" => Some(Rarity::Epic),
        "TLC_246" => Some(Rarity::Rare),
        "TLC_247" => Some(Rarity::Common),
        "TLC_248" => Some(Rarity::Common),
        "TLC_249" => Some(Rarity::Common),
        "TLC_250" => Some(Rarity::Common),
        "TLC_251" => Some(Rarity::Rare),
        "TLC_252" => Some(Rarity::Rare),
        "TLC_253" => Some(Rarity::Common),
        "TLC_254" => Some(Rarity::Epic),
        "TLC_255" => Some(Rarity::Epic),
        "TLC_256" => Some(Rarity::Common),
        "TLC_257" => Some(Rarity::Legendary),
        "TLC_334" => Some(Rarity::Rare),
        "TLC_364" => Some(Rarity::Rare),
        "TLC_365" => Some(Rarity::Rare),
        "TLC_366" => Some(Rarity::Rare),
        "TLC_401" => Some(Rarity::Common),
        "TLC_426" => Some(Rarity::Legendary),
        "TLC_427" => Some(Rarity::Common),
        "TLC_428" => Some(Rarity::Common),
        "TLC_429" => Some(Rarity::Common),
        "TLC_430" => Some(Rarity::Epic),
        "TLC_432" => Some(Rarity::Rare),
        "TLC_433" => Some(Rarity::Legendary),
        "TLC_434" => Some(Rarity::Rare),
        "TLC_435" => Some(Rarity::Common),
        "TLC_436" => Some(Rarity::Epic),
        "TLC_438" => Some(Rarity::Rare),
        "TLC_439" => Some(Rarity::Epic),
        "TLC_440" => Some(Rarity::Rare),
        "TLC_441" => Some(Rarity::Common),
        "TLC_442" => Some(Rarity::Common),
        "TLC_443" => Some(Rarity::Common),
        "TLC_444" => Some(Rarity::Rare),
        "TLC_446" => Some(Rarity::Legendary),
        "TLC_447" => Some(Rarity::Common),
        "TLC_449" => Some(Rarity::Rare),
        "TLC_450" => Some(Rarity::Common),
        "TLC_451" => Some(Rarity::Epic),
        "TLC_452" => Some(Rarity::Legendary),
        "TLC_452t1" => Some(Rarity::Legendary),
        "TLC_452t13" => Some(Rarity::Legendary),
        "TLC_452t14" => Some(Rarity::Legendary),
        "TLC_452t15" => Some(Rarity::Legendary),
        "TLC_452t16" => Some(Rarity::Legendary),
        "TLC_452t17" => Some(Rarity::Legendary),
        "TLC_452t18" => Some(Rarity::Legendary),
        "TLC_452t19" => Some(Rarity::Legendary),
        "TLC_452t2" => Some(Rarity::Legendary),
        "TLC_452t20" => Some(Rarity::Legendary),
        "TLC_452t21" => Some(Rarity::Legendary),
        "TLC_452t22" => Some(Rarity::Legendary),
        "TLC_452t23" => Some(Rarity::Legendary),
        "TLC_452t24" => Some(Rarity::Legendary),
        "TLC_452t26" => Some(Rarity::Legendary),
        "TLC_452t27" => Some(Rarity::Legendary),
        "TLC_452t28" => Some(Rarity::Legendary),
        "TLC_452t29" => Some(Rarity::Legendary),
        "TLC_452t3" => Some(Rarity::Legendary),
        "TLC_452t30" => Some(Rarity::Legendary),
        "TLC_452t31" => Some(Rarity::Legendary),
        "TLC_452t32" => Some(Rarity::Legendary),
        "TLC_452t33" => Some(Rarity::Legendary),
        "TLC_452t34" => Some(Rarity::Legendary),
        "TLC_452t35" => Some(Rarity::Legendary),
        "TLC_452t4" => Some(Rarity::Legendary),
        "TLC_452t5" => Some(Rarity::Legendary),
        "TLC_452t6" => Some(Rarity::Legendary),
        "TLC_452t7" => Some(Rarity::Legendary),
        "TLC_452t8" => Some(Rarity::Legendary),
        "TLC_452t9" => Some(Rarity::Legendary),
        "TLC_454" => Some(Rarity::Common),
        "TLC_460" => Some(Rarity::Legendary),
        "TLC_461" => Some(Rarity::Common),
        "TLC_462" => Some(Rarity::Epic),
        "TLC_463" => Some(Rarity::Legendary),
        "TLC_464" => Some(Rarity::Rare),
        "TLC_465" => Some(Rarity::Rare),
        "TLC_466" => Some(Rarity::Rare),
        "TLC_467" => Some(Rarity::Epic),
        "TLC_468" => Some(Rarity::Common),
        "TLC_469" => Some(Rarity::Common),
        "TLC_477" => Some(Rarity::Epic),
        "TLC_478" => Some(Rarity::Common),
        "TLC_479" => Some(Rarity::Rare),
        "TLC_480" => Some(Rarity::Legendary),
        "TLC_482" => Some(Rarity::Epic),
        "TLC_483" => Some(Rarity::Common),
        "TLC_513" => Some(Rarity::Legendary),
        "TLC_514" => Some(Rarity::Common),
        "TLC_515" => Some(Rarity::Rare),
        "TLC_516" => Some(Rarity::Common),
        "TLC_517" => Some(Rarity::Epic),
        "TLC_518" => Some(Rarity::Common),
        "TLC_519" => Some(Rarity::Rare),
        "TLC_520" => Some(Rarity::Epic),
        "TLC_521" => Some(Rarity::Rare),
        "TLC_522" => Some(Rarity::Legendary),
        "TLC_600" => Some(Rarity::Common),
        "TLC_601" => Some(Rarity::Rare),
        "TLC_602" => Some(Rarity::Legendary),
        "TLC_603" => Some(Rarity::Common),
        "TLC_605" => Some(Rarity::Common),
        "TLC_606" => Some(Rarity::Rare),
        "TLC_620" => Some(Rarity::Common),
        "TLC_621" => Some(Rarity::Common),
        "TLC_622" => Some(Rarity::Epic),
        "TLC_623" => Some(Rarity::Epic),
        "TLC_624" => Some(Rarity::Legendary),
        "TLC_630" => Some(Rarity::Epic),
        "TLC_631" => Some(Rarity::Legendary),
        "TLC_632" => Some(Rarity::Rare),
        "TLC_633" => Some(Rarity::Rare),
        "TLC_810" => Some(Rarity::Legendary),
        "TLC_811" => Some(Rarity::Legendary),
        "TLC_814" => Some(Rarity::Common),
        "TLC_815" => Some(Rarity::Rare),
        "TLC_816" => Some(Rarity::Common),
        "TLC_817" => Some(Rarity::Legendary),
        "TLC_817t" => Some(Rarity::Legendary),
        "TLC_817t2" => Some(Rarity::Legendary),
        "TLC_818" => Some(Rarity::Epic),
        "TLC_819" => Some(Rarity::Epic),
        "TLC_820" => Some(Rarity::Rare),
        "TLC_821" => Some(Rarity::Rare),
        "TLC_822" => Some(Rarity::Common),
        "TLC_823" => Some(Rarity::Common),
        "TLC_824" => Some(Rarity::Common),
        "TLC_825" => Some(Rarity::Epic),
        "TLC_826" => Some(Rarity::Rare),
        "TLC_827" => Some(Rarity::Rare),
        "TLC_828" => Some(Rarity::Epic),
        "TLC_829" => Some(Rarity::Epic),
        "TLC_830" => Some(Rarity::Legendary),
        "TLC_831" => Some(Rarity::Common),
        "TLC_833" => Some(Rarity::Rare),
        "TLC_835" => Some(Rarity::Common),
        "TLC_836" => Some(Rarity::Legendary),
        "TLC_840" => Some(Rarity::Common),
        "TLC_841" => Some(Rarity::Legendary),
        "TLC_888" => Some(Rarity::Rare),
        "TLC_900" => Some(Rarity::Common),
        "TLC_901" => Some(Rarity::Epic),
        "TLC_902" => Some(Rarity::Rare),
        "TLC_903" => Some(Rarity::Common),
        "TLC_987" => Some(Rarity::Common),
        "TOY_100" => Some(Rarity::Legendary),
        "TOY_101" => Some(Rarity::Common),
        "TOY_102" => Some(Rarity::Rare),
        "TOY_103" => Some(Rarity::Common),
        "TTN_851" => Some(Rarity::Common),
        "VAN_CS1_069" => Some(Rarity::Common),
        "VAN_CS1_129" => Some(Rarity::Common),
        "VAN_CS2_028" => Some(Rarity::Rare),
        "VAN_CS2_031" => Some(Rarity::Common),
        "VAN_CS2_038" => Some(Rarity::Rare),
        "VAN_CS2_053" => Some(Rarity::Epic),
        "VAN_CS2_059" => Some(Rarity::Common),
        "VAN_CS2_073" => Some(Rarity::Common),
        "VAN_CS2_104" => Some(Rarity::Common),
        "VAN_CS2_117" => Some(Rarity::Common),
        "VAN_CS2_146" => Some(Rarity::Common),
        "VAN_CS2_151" => Some(Rarity::Common),
        "VAN_CS2_161" => Some(Rarity::Rare),
        "VAN_CS2_169" => Some(Rarity::Common),
        "VAN_CS2_181" => Some(Rarity::Rare),
        "VAN_CS2_188" => Some(Rarity::Common),
        "VAN_CS2_203" => Some(Rarity::Common),
        "VAN_CS2_221" => Some(Rarity::Common),
        "VAN_CS2_227" => Some(Rarity::Common),
        "VAN_CS2_231" => Some(Rarity::Common),
        "VAN_CS2_233" => Some(Rarity::Rare),
        "VAN_CS2_235" => Some(Rarity::Common),
        "VAN_CS2_236" => Some(Rarity::Common),
        "VAN_DS1_188" => Some(Rarity::Epic),
        "VAN_DS1_233" => Some(Rarity::Common),
        "VAN_EX1_001" => Some(Rarity::Rare),
        "VAN_EX1_002" => Some(Rarity::Legendary),
        "VAN_EX1_004" => Some(Rarity::Rare),
        "VAN_EX1_005" => Some(Rarity::Epic),
        "VAN_EX1_006" => Some(Rarity::Rare),
        "VAN_EX1_007" => Some(Rarity::Common),
        "VAN_EX1_008" => Some(Rarity::Common),
        "VAN_EX1_009" => Some(Rarity::Rare),
        "VAN_EX1_010" => Some(Rarity::Common),
        "VAN_EX1_012" => Some(Rarity::Legendary),
        "VAN_EX1_014" => Some(Rarity::Legendary),
        "VAN_EX1_016" => Some(Rarity::Legendary),
        "VAN_EX1_017" => Some(Rarity::Common),
        "VAN_EX1_020" => Some(Rarity::Common),
        "VAN_EX1_021" => Some(Rarity::Common),
        "VAN_EX1_023" => Some(Rarity::Common),
        "VAN_EX1_028" => Some(Rarity::Common),
        "VAN_EX1_029" => Some(Rarity::Common),
        "VAN_EX1_032" => Some(Rarity::Rare),
        "VAN_EX1_033" => Some(Rarity::Common),
        "VAN_EX1_043" => Some(Rarity::Rare),
        "VAN_EX1_044" => Some(Rarity::Rare),
        "VAN_EX1_045" => Some(Rarity::Rare),
        "VAN_EX1_046" => Some(Rarity::Common),
        "VAN_EX1_048" => Some(Rarity::Common),
        "VAN_EX1_049" => Some(Rarity::Common),
        "VAN_EX1_050" => Some(Rarity::Rare),
        "VAN_EX1_055" => Some(Rarity::Rare),
        "VAN_EX1_057" => Some(Rarity::Common),
        "VAN_EX1_058" => Some(Rarity::Rare),
        "VAN_EX1_059" => Some(Rarity::Rare),
        "VAN_EX1_062" => Some(Rarity::Legendary),
        "VAN_EX1_067" => Some(Rarity::Rare),
        "VAN_EX1_076" => Some(Rarity::Rare),
        "VAN_EX1_080" => Some(Rarity::Rare),
        "VAN_EX1_082" => Some(Rarity::Common),
        "VAN_EX1_083" => Some(Rarity::Legendary),
        "VAN_EX1_085" => Some(Rarity::Rare),
        "VAN_EX1_089" => Some(Rarity::Rare),
        "VAN_EX1_091" => Some(Rarity::Epic),
        "VAN_EX1_093" => Some(Rarity::Rare),
        "VAN_EX1_095" => Some(Rarity::Rare),
        "VAN_EX1_096" => Some(Rarity::Common),
        "VAN_EX1_097" => Some(Rarity::Rare),
        "VAN_EX1_100" => Some(Rarity::Legendary),
        "VAN_EX1_102" => Some(Rarity::Rare),
        "VAN_EX1_103" => Some(Rarity::Rare),
        "VAN_EX1_105" => Some(Rarity::Epic),
        "VAN_EX1_110" => Some(Rarity::Legendary),
        "VAN_EX1_112" => Some(Rarity::Legendary),
        "VAN_EX1_116" => Some(Rarity::Legendary),
        "VAN_EX1_124" => Some(Rarity::Common),
        "VAN_EX1_126" => Some(Rarity::Common),
        "VAN_EX1_128" => Some(Rarity::Common),
        "VAN_EX1_130" => Some(Rarity::Common),
        "VAN_EX1_131" => Some(Rarity::Common),
        "VAN_EX1_132" => Some(Rarity::Common),
        "VAN_EX1_133" => Some(Rarity::Rare),
        "VAN_EX1_134" => Some(Rarity::Rare),
        "VAN_EX1_136" => Some(Rarity::Common),
        "VAN_EX1_137" => Some(Rarity::Rare),
        "VAN_EX1_144" => Some(Rarity::Common),
        "VAN_EX1_145" => Some(Rarity::Epic),
        "VAN_EX1_154" => Some(Rarity::Common),
        "VAN_EX1_155" => Some(Rarity::Common),
        "VAN_EX1_158" => Some(Rarity::Common),
        "VAN_EX1_160" => Some(Rarity::Common),
        "VAN_EX1_161" => Some(Rarity::Common),
        "VAN_EX1_162" => Some(Rarity::Common),
        "VAN_EX1_164" => Some(Rarity::Rare),
        "VAN_EX1_165" => Some(Rarity::Common),
        "VAN_EX1_166" => Some(Rarity::Rare),
        "VAN_EX1_170" => Some(Rarity::Rare),
        "VAN_EX1_178" => Some(Rarity::Epic),
        "VAN_EX1_238" => Some(Rarity::Common),
        "VAN_EX1_241" => Some(Rarity::Rare),
        "VAN_EX1_243" => Some(Rarity::Common),
        "VAN_EX1_245" => Some(Rarity::Common),
        "VAN_EX1_247" => Some(Rarity::Common),
        "VAN_EX1_248" => Some(Rarity::Rare),
        "VAN_EX1_249" => Some(Rarity::Legendary),
        "VAN_EX1_250" => Some(Rarity::Epic),
        "VAN_EX1_251" => Some(Rarity::Common),
        "VAN_EX1_258" => Some(Rarity::Common),
        "VAN_EX1_259" => Some(Rarity::Rare),
        "VAN_EX1_274" => Some(Rarity::Rare),
        "VAN_EX1_275" => Some(Rarity::Common),
        "VAN_EX1_279" => Some(Rarity::Epic),
        "VAN_EX1_283" => Some(Rarity::Common),
        "VAN_EX1_284" => Some(Rarity::Rare),
        "VAN_EX1_287" => Some(Rarity::Rare),
        "VAN_EX1_289" => Some(Rarity::Common),
        "VAN_EX1_294" => Some(Rarity::Common),
        "VAN_EX1_295" => Some(Rarity::Epic),
        "VAN_EX1_298" => Some(Rarity::Legendary),
        "VAN_EX1_301" => Some(Rarity::Rare),
        "VAN_EX1_303" => Some(Rarity::Rare),
        "VAN_EX1_304" => Some(Rarity::Rare),
        "VAN_EX1_309" => Some(Rarity::Rare),
        "VAN_EX1_310" => Some(Rarity::Rare),
        "VAN_EX1_312" => Some(Rarity::Epic),
        "VAN_EX1_313" => Some(Rarity::Epic),
        "VAN_EX1_315" => Some(Rarity::Common),
        "VAN_EX1_316" => Some(Rarity::Common),
        "VAN_EX1_317" => Some(Rarity::Common),
        "VAN_EX1_319" => Some(Rarity::Common),
        "VAN_EX1_320" => Some(Rarity::Epic),
        "VAN_EX1_323" => Some(Rarity::Legendary),
        "VAN_EX1_332" => Some(Rarity::Common),
        "VAN_EX1_334" => Some(Rarity::Rare),
        "VAN_EX1_335" => Some(Rarity::Common),
        "VAN_EX1_339" => Some(Rarity::Common),
        "VAN_EX1_341" => Some(Rarity::Rare),
        "VAN_EX1_345" => Some(Rarity::Epic),
        "VAN_EX1_349" => Some(Rarity::Rare),
        "VAN_EX1_350" => Some(Rarity::Legendary),
        "VAN_EX1_354" => Some(Rarity::Epic),
        "VAN_EX1_355" => Some(Rarity::Rare),
        "VAN_EX1_362" => Some(Rarity::Common),
        "VAN_EX1_363" => Some(Rarity::Common),
        "VAN_EX1_365" => Some(Rarity::Rare),
        "VAN_EX1_366" => Some(Rarity::Epic),
        "VAN_EX1_379" => Some(Rarity::Common),
        "VAN_EX1_382" => Some(Rarity::Rare),
        "VAN_EX1_383" => Some(Rarity::Legendary),
        "VAN_EX1_384" => Some(Rarity::Epic),
        "VAN_EX1_390" => Some(Rarity::Common),
        "VAN_EX1_391" => Some(Rarity::Common),
        "VAN_EX1_392" => Some(Rarity::Common),
        "VAN_EX1_393" => Some(Rarity::Common),
        "VAN_EX1_396" => Some(Rarity::Common),
        "VAN_EX1_398" => Some(Rarity::Common),
        "VAN_EX1_402" => Some(Rarity::Rare),
        "VAN_EX1_405" => Some(Rarity::Common),
        "VAN_EX1_407" => Some(Rarity::Epic),
        "VAN_EX1_408" => Some(Rarity::Rare),
        "VAN_EX1_409" => Some(Rarity::Rare),
        "VAN_EX1_410" => Some(Rarity::Epic),
        "VAN_EX1_411" => Some(Rarity::Epic),
        "VAN_EX1_412" => Some(Rarity::Common),
        "VAN_EX1_414" => Some(Rarity::Legendary),
        "VAN_EX1_507" => Some(Rarity::Epic),
        "VAN_EX1_509" => Some(Rarity::Rare),
        "VAN_EX1_522" => Some(Rarity::Epic),
        "VAN_EX1_531" => Some(Rarity::Common),
        "VAN_EX1_533" => Some(Rarity::Rare),
        "VAN_EX1_534" => Some(Rarity::Rare),
        "VAN_EX1_536" => Some(Rarity::Rare),
        "VAN_EX1_537" => Some(Rarity::Rare),
        "VAN_EX1_538" => Some(Rarity::Common),
        "VAN_EX1_543" => Some(Rarity::Legendary),
        "VAN_EX1_544" => Some(Rarity::Rare),
        "VAN_EX1_549" => Some(Rarity::Epic),
        "VAN_EX1_554" => Some(Rarity::Epic),
        "VAN_EX1_556" => Some(Rarity::Common),
        "VAN_EX1_557" => Some(Rarity::Legendary),
        "VAN_EX1_558" => Some(Rarity::Legendary),
        "VAN_EX1_559" => Some(Rarity::Legendary),
        "VAN_EX1_560" => Some(Rarity::Legendary),
        "VAN_EX1_561" => Some(Rarity::Legendary),
        "VAN_EX1_562" => Some(Rarity::Legendary),
        "VAN_EX1_563" => Some(Rarity::Legendary),
        "VAN_EX1_564" => Some(Rarity::Epic),
        "VAN_EX1_567" => Some(Rarity::Epic),
        "VAN_EX1_570" => Some(Rarity::Rare),
        "VAN_EX1_571" => Some(Rarity::Epic),
        "VAN_EX1_572" => Some(Rarity::Legendary),
        "VAN_EX1_573" => Some(Rarity::Legendary),
        "VAN_EX1_575" => Some(Rarity::Rare),
        "VAN_EX1_577" => Some(Rarity::Legendary),
        "VAN_EX1_578" => Some(Rarity::Rare),
        "VAN_EX1_583" => Some(Rarity::Common),
        "VAN_EX1_584" => Some(Rarity::Rare),
        "VAN_EX1_586" => Some(Rarity::Epic),
        "VAN_EX1_590" => Some(Rarity::Epic),
        "VAN_EX1_591" => Some(Rarity::Rare),
        "VAN_EX1_594" => Some(Rarity::Rare),
        "VAN_EX1_595" => Some(Rarity::Common),
        "VAN_EX1_596" => Some(Rarity::Common),
        "VAN_EX1_597" => Some(Rarity::Rare),
        "VAN_EX1_603" => Some(Rarity::Common),
        "VAN_EX1_604" => Some(Rarity::Rare),
        "VAN_EX1_607" => Some(Rarity::Common),
        "VAN_EX1_608" => Some(Rarity::Common),
        "VAN_EX1_609" => Some(Rarity::Common),
        "VAN_EX1_610" => Some(Rarity::Common),
        "VAN_EX1_611" => Some(Rarity::Common),
        "VAN_EX1_612" => Some(Rarity::Rare),
        "VAN_EX1_613" => Some(Rarity::Legendary),
        "VAN_EX1_614" => Some(Rarity::Legendary),
        "VAN_EX1_616" => Some(Rarity::Rare),
        "VAN_EX1_617" => Some(Rarity::Common),
        "VAN_EX1_619" => Some(Rarity::Rare),
        "VAN_EX1_620" => Some(Rarity::Epic),
        "VAN_EX1_621" => Some(Rarity::Common),
        "VAN_EX1_623" => Some(Rarity::Common),
        "VAN_EX1_624" => Some(Rarity::Rare),
        "VAN_EX1_625" => Some(Rarity::Epic),
        "VAN_EX1_626" => Some(Rarity::Rare),
        "VAN_NEW1_004" => Some(Rarity::Common),
        "VAN_NEW1_005" => Some(Rarity::Epic),
        "VAN_NEW1_007" => Some(Rarity::Rare),
        "VAN_NEW1_008" => Some(Rarity::Epic),
        "VAN_NEW1_010" => Some(Rarity::Legendary),
        "VAN_NEW1_012" => Some(Rarity::Common),
        "VAN_NEW1_014" => Some(Rarity::Rare),
        "VAN_NEW1_016" => Some(Rarity::Epic),
        "VAN_NEW1_017" => Some(Rarity::Epic),
        "VAN_NEW1_018" => Some(Rarity::Common),
        "VAN_NEW1_019" => Some(Rarity::Rare),
        "VAN_NEW1_020" => Some(Rarity::Rare),
        "VAN_NEW1_021" => Some(Rarity::Epic),
        "VAN_NEW1_022" => Some(Rarity::Common),
        "VAN_NEW1_023" => Some(Rarity::Common),
        "VAN_NEW1_024" => Some(Rarity::Legendary),
        "VAN_NEW1_025" => Some(Rarity::Rare),
        "VAN_NEW1_026" => Some(Rarity::Rare),
        "VAN_NEW1_027" => Some(Rarity::Epic),
        "VAN_NEW1_029" => Some(Rarity::Legendary),
        "VAN_NEW1_030" => Some(Rarity::Legendary),
        "VAN_NEW1_036" => Some(Rarity::Rare),
        "VAN_NEW1_037" => Some(Rarity::Rare),
        "VAN_NEW1_038" => Some(Rarity::Legendary),
        "VAN_NEW1_040" => Some(Rarity::Legendary),
        "VAN_NEW1_041" => Some(Rarity::Rare),
        "VAN_PRO_001" => Some(Rarity::Legendary),
        "VAN_tt_004" => Some(Rarity::Common),
        "VAN_tt_010" => Some(Rarity::Epic),
        "WON_113" => Some(Rarity::Epic),
        "WON_145" => Some(Rarity::Legendary),
        "tt_004" => Some(Rarity::Common),
        "tt_010" => Some(Rarity::Epic),
        _ => None,
    }
}

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
                    && (crate::cards::sets::LEGENDARY_CLASSIC
                        .iter()
                        .any(|l| l.id == c.id)
                        // M3-W2a: Nozdormu the Timeless (TIME_063) — the
                        // only 2025–2026 legendary in the pool (§20 — the
                        // Clocksworth legendary summon spans the full
                        // active window)
                        || c.id == "TIME_063")
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
        // M2-W4a pools (school/class filters read the official dump tables;
        // all in-window unless the card text says otherwise).
        RandomPool::FelSpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && crate::cards::quest::spell_school(c.id)
                        == Some(crate::cards::quest::SpellSchool::Fel)
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::HolySpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && crate::cards::quest::spell_school(c.id)
                        == Some(crate::cards::quest::SpellSchool::Holy)
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::HolySpellCost1 => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && c.cost == 1
                    && crate::cards::quest::spell_school(c.id)
                        == Some(crate::cards::quest::SpellSchool::Holy)
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::FelBeast => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Beast)
                    && (crate::cards::sets::WARLOCK_CLASSIC
                        .iter()
                        .any(|w| w.id == c.id)
                        || crate::cards::sets::DEMON_HUNTER_W1
                            .iter()
                            .any(|d| d.id == c.id))
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        // "A random weapon from another class" (Neferset Weaponsmith) —
        // cross-class pool: ALL_CARDS weapons of the other class groups,
        // no window restriction (the card itself is W4a; the pool pulls
        // the full weapon catalog like the official filter).
        RandomPool::WeaponAnotherClass => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Weapon
                    && [
                        crate::cards::sets::DRUID_CLASSIC,
                        crate::cards::sets::HUNTER_CLASSIC,
                        crate::cards::sets::MAGE_CLASSIC,
                        crate::cards::sets::PALADIN_CLASSIC,
                        crate::cards::sets::PRIEST_CLASSIC,
                        crate::cards::sets::SHAMAN_CLASSIC,
                        crate::cards::sets::WARLOCK_CLASSIC,
                        crate::cards::sets::WARRIOR_CLASSIC,
                        crate::cards::sets::DEMON_HUNTER_W1,
                        crate::cards::sets::DEATH_KNIGHT_W1,
                    ]
                    .iter()
                    .any(|class| class.iter().any(|g| g.id == c.id))
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        // M3-W2a pools (Across the Timeways).
        RandomPool::AnyMinion => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| c.card_type == CardType::Minion && in_active_window(c))
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::Cost5Minion => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| c.card_type == CardType::Minion && c.cost == 5 && in_active_window(c))
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::RewindCard => crate::cards::rewind::REWIND_CARD_IDS
            .iter()
            .filter_map(|id| card_by_id(id))
            .collect(),
        RandomPool::ClassSpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| c.card_type == CardType::Spell && is_class_card(c) && in_active_window(c))
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::NatureSpell => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && crate::cards::quest::spell_school(c.id)
                        == Some(crate::cards::quest::SpellSchool::Nature)
                    && in_active_window(c)
            })
            .copied()
            .collect::<Vec<CardDef>>()
            .iter()
            .filter_map(card_by_id_ref)
            .collect(),
        RandomPool::RandomWeapon => crate::cards::sets::ALL_CARDS
            .iter()
            .filter(|c| c.card_type == CardType::Weapon && in_active_window(c))
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
                && (crate::cards::sets::LEGENDARY_CLASSIC
                    .iter()
                    .any(|l| l.id == c.id)
                    || c.id == "TIME_063")
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
        RandomPool::FelSpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell
                && crate::cards::quest::spell_school(c.id)
                    == Some(crate::cards::quest::SpellSchool::Fel)
        }),
        RandomPool::HolySpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell
                && crate::cards::quest::spell_school(c.id)
                    == Some(crate::cards::quest::SpellSchool::Holy)
        }),
        RandomPool::HolySpellCost1 => random_filtered(rng, |c| {
            c.card_type == CardType::Spell
                && c.cost == 1
                && crate::cards::quest::spell_school(c.id)
                    == Some(crate::cards::quest::SpellSchool::Holy)
        }),
        RandomPool::FelBeast => random_filtered(rng, |c| {
            c.card_type == CardType::Minion
                && c.race == Some(Race::Beast)
                && (crate::cards::sets::WARLOCK_CLASSIC
                    .iter()
                    .any(|w| w.id == c.id)
                    || crate::cards::sets::DEMON_HUNTER_W1
                        .iter()
                        .any(|d| d.id == c.id))
        }),
        RandomPool::WeaponAnotherClass => random_filtered_any_window(rng, |c| {
            c.card_type == CardType::Weapon
                && [
                    crate::cards::sets::DRUID_CLASSIC,
                    crate::cards::sets::HUNTER_CLASSIC,
                    crate::cards::sets::MAGE_CLASSIC,
                    crate::cards::sets::PALADIN_CLASSIC,
                    crate::cards::sets::PRIEST_CLASSIC,
                    crate::cards::sets::SHAMAN_CLASSIC,
                    crate::cards::sets::WARLOCK_CLASSIC,
                    crate::cards::sets::WARRIOR_CLASSIC,
                    crate::cards::sets::DEMON_HUNTER_W1,
                    crate::cards::sets::DEATH_KNIGHT_W1,
                ]
                .iter()
                .any(|class| class.iter().any(|g| g.id == c.id))
        }),
        RandomPool::AnyMinion => random_filtered(rng, |c| c.card_type == CardType::Minion),
        RandomPool::Cost5Minion => {
            random_filtered(rng, |c| c.card_type == CardType::Minion && c.cost == 5)
        }
        RandomPool::RewindCard => random_from_pool(crate::cards::rewind::REWIND_CARD_IDS, rng),
        RandomPool::ClassSpell => {
            random_filtered(rng, |c| c.card_type == CardType::Spell && is_class_card(c))
        }
        RandomPool::NatureSpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell
                && crate::cards::quest::spell_school(c.id)
                    == Some(crate::cards::quest::SpellSchool::Nature)
        }),
        RandomPool::RandomWeapon => random_filtered(rng, |c| c.card_type == CardType::Weapon),
    }
}

/// Draws randomly from the full card list after filtering by a predicate —
/// without the active-window restriction (the WeaponAnotherClass pool pulls
/// the full weapon catalog, see fidelity-debt §17).
fn random_filtered_any_window(
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

/// The full card list of a Discover pool (2025–2026 expansions M2-W4a) —
/// the option list for a `DiscoverPool` discover, in a deterministic order.
///
/// Some pools depend on game state: `CostEqualRemainingMana` reads the
/// player's remaining mana (Scrappy Scavenger), `MinionOfUnplayedType` the
/// races already played this game (Mountain Map). All other pools are
/// in-window filtered subsets of `ALL_CARDS` (or fixed tables — see
/// fidelity-debt §17).
pub(crate) fn discover_pool_cards(
    pool: DiscoverPool,
    state: &GameState,
    player: PlayerId,
) -> Vec<&'static CardDef> {
    let all = crate::cards::sets::ALL_CARDS;
    match pool {
        DiscoverPool::LegendaryMinion => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && (crate::cards::sets::LEGENDARY_CLASSIC
                        .iter()
                        .any(|l| l.id == c.id)
                        || c.id == "TIME_063")
                    && in_active_window(c)
            })
            .collect(),
        DiscoverPool::UndeadMinion => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Undead)
                    && in_active_window(c)
            })
            .collect(),
        DiscoverPool::FrostRune => FROST_RUNE_POOL
            .iter()
            .filter_map(|id| card_by_id(id))
            .collect(),
        DiscoverPool::Murloc => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Murloc)
                    && in_active_window(c)
            })
            .collect(),
        DiscoverPool::MinionOfUnplayedType => {
            // The set of races already played this game (from the minion ids
            // recorded at play — Mountain Map's "a type you haven't played";
            // cards with no race are not a type).
            let mut played: Vec<Race> = state
                .player(player)
                .played_minion_ids
                .iter()
                .filter_map(|id| card_by_id(id))
                .filter_map(|c| c.race)
                .collect();
            played.dedup();
            all.iter()
                .filter(|c| {
                    c.card_type == CardType::Minion
                        && c.race.is_some_and(|r| !played.contains(&r))
                        && in_active_window(c)
                })
                .collect()
        }
        DiscoverPool::BeastOddAttack => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Beast)
                    && c.attack % 2 == 1
                    && in_active_window(c)
            })
            .collect(),
        DiscoverPool::FelSpell => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && crate::cards::quest::spell_school(c.id)
                        == Some(crate::cards::quest::SpellSchool::Fel)
                    && in_active_window(c)
            })
            .collect(),
        // Relic of Kings — "a spell from any class that costs (8) or more":
        // the full catalog, no window restriction (fidelity-debt §17).
        DiscoverPool::SpellCostGE8 => all
            .iter()
            .filter(|c| c.card_type == CardType::Spell && c.cost >= 8)
            .collect(),
        // M3-W2a pools.
        // TIME_704 Highborne Mentor — "a spell that costs (7) or more from
        // the past": the SpellCostGE8 precedent, threshold 7.
        DiscoverPool::SpellCostGE7 => all
            .iter()
            .filter(|c| c.card_type == CardType::Spell && c.cost >= 7 && in_active_window(c))
            .collect(),
        // TIME_857 Alter Time — "two Arcane spells from the past": the
        // school filter is the FelSpell precedent.
        DiscoverPool::ArcaneSpell => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Spell
                    && crate::cards::quest::spell_school(c.id)
                        == Some(crate::cards::quest::SpellSchool::Arcane)
                    && in_active_window(c)
            })
            .collect(),
        // TIME_016 Neon Innovation — "a Paladin Mech from the past": the
        // class filter is the MageSpell precedent, restricted to Mechs
        // (§20 — the exact past-set filter is the active window).
        DiscoverPool::PaladinMech => all
            .iter()
            .filter(|c| {
                c.card_type == CardType::Minion
                    && c.race == Some(Race::Mechanical)
                    && crate::cards::sets::PALADIN_CLASSIC
                        .iter()
                        .any(|p| p.id == c.id)
                    && in_active_window(c)
            })
            .collect(),
        // M3-W2a — TIME_612 Blood Draw — "a spell from the past": the whole
        // active window, no class or school filter (§20).
        DiscoverPool::Spell => all
            .iter()
            .filter(|c| c.card_type == CardType::Spell && in_active_window(c))
            .collect(),
        // Scrappy Scavenger — "Cost equal to your remaining Mana Crystals"
        // (the static cost, computed after this card's own cost was paid;
        // fidelity-debt §17).
        DiscoverPool::CostEqualRemainingMana => {
            let remaining = state.player(player).current_mana;
            all.iter().filter(|c| c.cost == remaining).collect()
        }
        DiscoverPool::TemporaryOneCostMinion => all
            .iter()
            .filter(|c| c.card_type == CardType::Minion && c.cost == 1 && in_active_window(c))
            .collect(),
    }
}

/// Every in-window card of the given rarity (Relic Miner's "Discover a card
/// of the same Rarity" pool, 2025–2026 expansions M2-W4a). The destroyed
/// card's rarity comes from `rarity_of`; when that is `None` (a token or
/// out-of-table card) the caller fizzles the discover instead (see the
/// DestroyTopCardDiscoverSameRarity arm in trigger.rs; fidelity-debt §17).
pub(crate) fn cards_of_rarity(rarity: Rarity) -> Vec<&'static CardDef> {
    crate::cards::sets::ALL_CARDS
        .iter()
        .filter(|c| rarity_of(c.id) == Some(rarity) && in_active_window(c))
        .collect()
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
