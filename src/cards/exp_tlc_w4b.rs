//! 2025–2026 expansions M2-W4b cards (exp_tlc_w4b) — the Un'Goro legendary
//! wave: the 14 remaining TLC_* legendary cards (Torga TLC_102 landed in
//! W3 with the Kindred wave).
//!
//! Implementation decisions (per the M2-W4b spec, verified against
//! `cards/data/THE_LOST_CITY.json` 2026-08-09):
//! - **Full** implementations:
//!   - TLC_106 Endbringer Umbra — battlecry triggers the Deathrattles of
//!     up to 5 friendly minions that died this game (the friendly
//!     graveyard IS the died-this-game log; the W3
//!     `TriggerFriendlyCinderDeathrattles` scan generalized to
//!     `TriggerFriendlyDeadDeathrattles`).
//!   - TLC_110 City Chief Esho — the deck check ("every minion in your
//!     deck shares a minion type", current deck, empty deck passes
//!     vacuously — pinned in the resolve arm) + the "+2/+2 wherever they
//!     are" buff (`EshoDeckCheckBuffEverywhere`).
//!   - TLC_228 Bralma Searstone — "Your Elementals deal 1 extra damage"
//!     as a damage-pipeline hook at the Goldrinn entry point (the aura
//!     approximation — fidelity-debt §18); the CardDef is vanilla, the
//!     hook keys on TLC_228 being alive on the owner's board.
//!   - TLC_241 Ido of the Threshfleet — start-of-turn token grant
//!     (`AddCardToHand` of the handwritten TLC_241t); the TurnStart
//!     trigger fires only while Ido is alive on the board, so the "while
//!     this is alive" condition is exact.
//!   - TLC_257 Loh, the Living Legend — battlecry sets the
//!     `Player::minions_cost_5` flag; the play-cost pipeline SETs minion
//!     costs to 5.
//!   - TLC_480 Krog, Crater King — end-of-turn `SetStatsAllEnemyMinions`
//!     (Attack and Health of all enemy minions set to 1; permanent
//!     enchantments are stripped and damage cleared — the set semantics).
//!   - TLC_522 Opu the Unseen — Battlecry, Combo, and Deathrattle each
//!     cast 'Fan of Knives' (the classic `DealDamage { 1,
//!     AllEnemyMinions }` shape, draw omitted — the Fan of Knives
//!     precedent); the combo slot replaces the battlecry under the
//!     standard combo rule.
//!   - TLC_624 Nablya, the Watcher — battlecry summons a fresh copy of
//!     each damaged friendly minion, the copies gain Rush
//!     (`SummonDamagedCopiesRush`; the copies are base-stat entities per
//!     the engine copy convention).
//!   - TLC_811 Archaios — a per-ID `FriendlyMinionAttacked` trigger (new
//!     event class: friendly-scoped, attacker as subject — the pinned
//!     `Attacked` class cannot ride "another friendly minion") that sets
//!     the attacker's Health to Archaios's effective Health.
//!   - TLC_836 Niri of the Crater — two triggers: a per-ID `CardPlayed`
//!     trigger doubles the stats of a played 1-Cost minion (Cost read at
//!     trigger time, effective cost), and the CardDef `spell_trigger`
//!     re-casts a cast 1-Cost spell once (no explicit target, no second
//!     SpellCast event — the Tyrande timing simplification §14.4).
//! - **Simplified** (fidelity-debt §18): TLC_100 Elise the Navigator
//!   (the starting-deck check runs against the `Player::starting_deck`
//!   snapshot and sets the crafted-location marker — no custom-location
//!   machinery), TLC_452 Titanographer Osk (no Titan mechanism — the
//!   battlecry does nothing), TLC_810 High Cultist Herenn (the two
//!   Deathrattle minions are summoned as copies — the deck is untouched —
//!   and "they fight!" is each dealing its Attack damage to the other
//!   once), TLC_841 Entomologist Toru (no Jar transform/release mechanic
//!   — the battlecry does nothing).
//!
//! Card data: `cards/data/THE_LOST_CITY.json` (verified 2026-08-09). The
//! only token of the wave — TLC_241t "Call the Threshfleet!" — is absent
//! from the dump (verified against `/tmp/hs_full.json`) and handwritten
//! per Ido's card text.

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::EffectTarget;

// ============================================================
// Minions.
// ============================================================

/// TLC_100 Elise the Navigator — 4-cost 3/5. Battlecry: if your deck
/// started with 10 cards of different Costs, craft a custom location
/// (registered simplification §18: the starting-deck check runs against
/// the `Player::starting_deck` snapshot and sets the
/// `elise_location_crafted` marker; no custom-location machinery).
pub const ELISE_THE_NAVIGATOR: CardDef = CardDef {
    id: "TLC_100",
    name: "Elise the Navigator",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::EliseCraftLocation),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_106 Endbringer Umbra — 7-cost 6/6. Battlecry: trigger the
/// Deathrattles of up to 5 friendly minions that died this game (the
/// friendly graveyard is the died-this-game log — the W3
/// `TriggerFriendlyCinderDeathrattles` scan generalized).
pub const ENDBRINGER_UMBRA: CardDef = CardDef {
    id: "TLC_106",
    name: "Endbringer Umbra",
    card_type: CardType::Minion,
    cost: 7,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::TriggerFriendlyDeadDeathrattles { count: 5 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_110 City Chief Esho — 6-cost 5/7. Battlecry: if every minion in
/// your deck shares a minion type, give your other minions +2/+2 wherever
/// they are (the deck check and the wherever-buff resolve in
/// `EshoDeckCheckBuffEverywhere`; the check semantics are pinned in the
/// resolve arm).
pub const CITY_CHIEF_ESHO: CardDef = CardDef {
    id: "TLC_110",
    name: "City Chief Esho",
    card_type: CardType::Minion,
    cost: 6,
    attack: 5,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::EshoDeckCheckBuffEverywhere {
        attack: 2,
        health: 2,
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_228 Bralma Searstone — 3-cost 1/5. "Your Elementals deal 1 extra
/// damage." — a damage-pipeline hook at the Goldrinn entry point keys on
/// TLC_228 being alive on the owner's board (the aura approximation,
/// fidelity-debt §18); the CardDef itself is vanilla.
pub const BRALMA_SEARSTONE: CardDef = CardDef {
    id: "TLC_228",
    name: "Bralma Searstone",
    card_type: CardType::Minion,
    cost: 3,
    attack: 1,
    health: 5,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_241 Ido of the Threshfleet — 4-cost 2/7. "While this is alive, you
/// get a 2-Cost Holy spell that gives a minion +2/+2 and Divine Shield."
/// The start-of-turn grant adds the handwritten TLC_241t token to hand;
/// the TurnStart trigger fires only while Ido is alive on the board, so
/// the "while alive" condition is exact.
pub const IDO_OF_THE_THRESHFLEET: CardDef = CardDef {
    id: "TLC_241",
    name: "Ido of the Threshfleet",
    card_type: CardType::Minion,
    cost: 4,
    attack: 2,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: Some(CardEffect::AddCardToHand {
        card_id: "TLC_241t",
    }),
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_257 Loh, the Living Legend — 9-cost 5/5. Battlecry: your minions
/// cost (5) this game — the battlecry sets the `Player::minions_cost_5`
/// flag, a SET read by the play-cost pipeline.
pub const LOH_THE_LIVING_LEGEND: CardDef = CardDef {
    id: "TLC_257",
    name: "Loh, the Living Legend",
    card_type: CardType::Minion,
    cost: 9,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::LohMinionsCost5),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_452 Titanographer Osk — 6-cost 6/6. "Gains a random Titan ability
/// in your hand that changes each turn." — registered simplification §18:
/// the Titan keyword mechanism does not exist, the battlecry does nothing.
pub const TITANOGRAPHER_OSK: CardDef = CardDef {
    id: "TLC_452",
    name: "Titanographer Osk",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_480 Krog, Crater King — 9-cost 8/7 Beast. At the end of your turn,
/// set the Attack and Health of all enemy minions to 1 (the
/// end-of-turn effect fires only while Krog is alive, matching the
/// official "at the end of your turn" condition).
pub const KROG_CRATER_KING: CardDef = CardDef {
    id: "TLC_480",
    name: "Krog, Crater King",
    card_type: CardType::Minion,
    cost: 9,
    attack: 8,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: Some(CardEffect::SetStatsAllEnemyMinions {
        attack: 1,
        health: 1,
    }),
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_522 Opu the Unseen — 6-cost 6/4 with Stealth. Battlecry, Combo,
/// and Deathrattle each cast 'Fan of Knives' (deal 1 damage to all enemy
/// minions — the classic `DealDamage` shape, draw omitted); the combo
/// slot replaces the battlecry under the standard combo rule.
pub const OPU_THE_UNSEEN: CardDef = CardDef {
    id: "TLC_522",
    name: "Opu the Unseen",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DealDamage {
        amount: 1,
        target: EffectTarget::AllEnemyMinions,
    }),
    deathrattle: Some(CardEffect::DealDamage {
        amount: 1,
        target: EffectTarget::AllEnemyMinions,
    }),
    taunt: false,
    stealth: true,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: Some(CardEffect::DealDamage {
        amount: 1,
        target: EffectTarget::AllEnemyMinions,
    }),
    attack_equals_health: false,
};

/// TLC_624 Nablya, the Watcher — 6-cost 5/7. Battlecry: summon a copy of
/// each damaged friendly minion; the copies gain Rush (the copies are
/// fresh base-stat entities per the engine copy convention).
pub const NABLYA_THE_WATCHER: CardDef = CardDef {
    id: "TLC_624",
    name: "Nablya, the Watcher",
    card_type: CardType::Minion,
    cost: 6,
    attack: 5,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::SummonDamagedCopiesRush),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_810 High Cultist Herenn — 8-cost 8/6. Battlecry: summon two
/// Deathrattle minions from your deck. They fight! — registered
/// simplification §18: the minions are summoned as copies (the deck is
/// untouched) and "they fight!" is each dealing its Attack damage to the
/// other once.
pub const HIGH_CULTIST_HERENN: CardDef = CardDef {
    id: "TLC_810",
    name: "High Cultist Herenn",
    card_type: CardType::Minion,
    cost: 8,
    attack: 8,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::SummonTwoDeathrattleMinionsAndFight),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_811 Archaios — 3-cost 1/6 Beast. "Whenever another friendly minion
/// attacks, set its Health equal to this minion's Health." — the
/// FriendlyMinionAttacked trigger is registered per-ID in
/// `apply_card_keywords`; the effect sets the attacker's Health to
/// Archaios's effective Health.
pub const ARCHAIOS: CardDef = CardDef {
    id: "TLC_811",
    name: "Archaios",
    card_type: CardType::Minion,
    cost: 3,
    attack: 1,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_836 Niri of the Crater — 3-cost 2/5. Two effects, one trigger:
/// "whenever you play a 1-Cost minion, double its stats" and "whenever
/// you cast a 1-Cost spell, cast it twice" — the per-ID CardPlayed
/// trigger registered in `apply_card_keywords` resolves the combined
/// `NiriOfTheCrater` effect, which branches on the subject's card type
/// (CardPlayed fires for spell casts too; the Trigger component is a
/// single slot, so no CardDef `spell_trigger` is registered).
/// "1-Cost" reads the effective cost at trigger time (pinned in the
/// resolve arm).
pub const NIRI_OF_THE_CRATER: CardDef = CardDef {
    id: "TLC_836",
    name: "Niri of the Crater",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 5,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_841 Entomologist Toru — 8-cost 7/7. Battlecry: put each minion in
/// your hand into 0/1 Jars that cost (1); break them to release the
/// minions! — registered simplification §18: the Jar transform/release
/// mechanic is not implemented, the battlecry does nothing.
pub const ENTOMOLOGIST_TORU: CardDef = CardDef {
    id: "TLC_841",
    name: "Entomologist Toru",
    card_type: CardType::Minion,
    cost: 8,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

// ============================================================
// Tokens (handwritten only — no generated baselines).
// ============================================================

/// TLC_241t Call the Threshfleet! — 2-cost Holy spell (Ido of the
/// Threshfleet's start-of-turn token; absent from the dump — handwritten
/// per Ido's card text "a 2-Cost Holy spell that gives a minion +2/+2 and
/// Divine Shield"). The Holy school is not registered in the quest
/// spell-school table, so the token counts as school-less for
/// school-filtered effects.
pub const CALL_THE_THRESHFLEET: CardDef = CardDef {
    id: "TLC_241t",
    name: "Call the Threshfleet!",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsAndGrantDivineShield {
        attack: 2,
        health: 2,
        target: EffectTarget::AnyMinion,
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};
