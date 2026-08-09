//! M4-W2 cards (exp_cata_w2) — the Cataclysm sub-roadmap W2: the 13
//! **Herald** cards and their 6 Colossal **Soldier** tokens + the
//! Ritual-of-Power Breezling token (2025–2026 expansions master roadmap
//! M4-W2, the C2 Herald primitive).
//!
//! The Herald mechanic itself (the per-player counter, the patron/soldier
//! registry, the upgrade tiers and the `resolve_herald` hook) lives in
//! `cards/herald.rs`; the CardDef here only carries the plain card data
//! and the card's own non-Herald effects.
//!
//! Implementation decisions (per the M4-W2 spec, verified against
//! `/tmp/hs_full.json` 2026-08-09; the fidelity rows are fidelity-debt.md
//! §24, en + zh):
//! - **Herald is a hook, not a CardEffect variant** (Design B): the
//!   Herald cards' CardDefs carry NO battlecry/deathrattle for the
//!   keyword — the play paths call `cards::herald::resolve_herald` at the
//!   battlecry / spell / weapon / deathrattle / location-activation
//!   resolution points (see §24 for the pinned shapes).
//! - **Upgrade tiers** (pinned 2026-08-09): the Soldier numbers are the
//!   family base times ×1 (counter 1) / ×2 (counters 2–3) / ×4 (counters
//!   4+). The bases follow the W1 §23 fixes of the same families: Azshara
//!   +2, Al'Akir +1, Ragnaros 2, Cho'gall +2/+2, Onyxia 1-Cost, Sinestra
//!   reduction dropped.
//! - **The Soldier {0} numbers are baked** at summon (base values sit in
//!   the CardDefs below) and re-baked on every later Herald — the aura
//!   system cannot read the per-player counter at query time.
//! - **CATA_160 Scorching Ravager's "Give the Soldier Rush"** is granted
//!   inside `resolve_herald`, keyed by the source id.
//! - **CATA_722 Envoy of the End** is the neutral patron — the dump has
//!   no neutral Soldier token, so its Herald only increments the counter.
//! - **CATA_492 Shrine of Twilight**: "Herald {0}. Draw a card." — the
//!   Herald sits in the location's ACTIVATE text (resolved on activation,
//!   before the draw); the battlecry slot carries the draw.
//! - **CATA_785 Rite of Twilight**: "Herald {0}. Combo: Deal 3 damage" —
//!   the combo effect (`AnyCharacter` — the official "deal $3 damage" has
//!   no target filter, pinned from the dump) rides the combo slot; the
//!   Herald fires regardless of the combo status.
//! - **CATA_530 Fel Infusion**: "Your hero has Lifesteal this turn" — the
//!   new `GrantHeroLifestealThisTurn` variant (the GrantPoisonousThisTurn
//!   convention: hero component + per-player flag cleared at turn end).
//! - **CATA_561 Ritual of Power**: "Get two 1/1 Elementals with Rush" —
//!   the two CATA_561t Breezlings, Rush via apply_card_keywords (the
//!   Breezling carries NO {0} — no Herald scaling).
//! - **CATA_725 Shadowsworn Disciple**: "Battlecry: Herald {0}.
//!   Deathrattle: Restore 3 Health to your hero" — the deathrattle rides
//!   the CardDef; CATA_158 Maniacal Follower's "Deathrattle: Herald {0}"
//!   is the ONLY deathrattle Herald (its CardDef deathrattle is None —
//!   the deathrattle path calls the hook, keyed by the id).

use crate::cards::def::CardDef;
use crate::core::component::{AuraEffect, AuraTarget, CardType, Race};
use crate::core::effect::{CardEffect, EffectTarget};

/// CATA_156 Experimental Animation — 6-cost (SPELL, Death Knight).
/// "Herald {0}. Deal 4 damage to all enemy minions."
pub const EXPERIMENTAL_ANIMATION: CardDef = CardDef {
    id: "CATA_156",
    name: "Experimental Animation",
    card_type: CardType::Spell,
    cost: 6,
    attack: 0,
    health: 0,
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
    spell_effect: Some(CardEffect::DealDamageToAllEnemyMinions { damage: 4 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_158 Maniacal Follower — 3-cost 4/1 (MINION, Rogue).
/// "Stealth. Deathrattle: Herald {0}." — the deathrattle is the Herald
/// keyword (the CardDef carries no deathrattle; the deathrattle path
/// calls `resolve_herald`, keyed by the id).
pub const MANIACAL_FOLLOWER: CardDef = CardDef {
    id: "CATA_158",
    name: "Maniacal Follower",
    card_type: CardType::Minion,
    cost: 3,
    attack: 4,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: None,
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
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_160 Scorching Ravager — 4-cost 4/3 (MINION, Warrior).
/// "Battlecry: Herald {0}. Give the Soldier Rush." — the Herald is the
/// battlecry (hook); the Rush is granted inside `resolve_herald`, keyed
/// by the id.
pub const SCORCHING_RAVAGER: CardDef = CardDef {
    id: "CATA_160",
    name: "Scorching Ravager",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
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

/// CATA_492 Shrine of Twilight — 4-cost location 4/2 (LOCATION, Warlock).
/// "Herald {0}. Draw a card." — the Herald sits in the location's
/// ACTIVATE text (the LocationActivated path calls the hook, keyed by
/// the id, BEFORE the draw below); the draw is the activate effect
/// (battlecry slot).
pub const SHRINE_OF_TWILIGHT: CardDef = CardDef {
    id: "CATA_492",
    name: "Shrine of Twilight",
    card_type: CardType::Location,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::DrawCard { count: 1 }),
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

/// CATA_525 Armored Bloodletter — 3-cost 3/1 (MINION, Demon Hunter).
/// "Rush. Battlecry: Herald {0}." — the battlecry is the Herald keyword
/// (hook); Rush rides apply_card_keywords.
pub const ARMORED_BLOODLETTER: CardDef = CardDef {
    id: "CATA_525",
    name: "Armored Bloodletter",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 1,
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

/// CATA_530 Fel Infusion — 2-cost (SPELL, Demon Hunter).
/// "Herald {0}. Your hero has Lifesteal this turn." — the Herald is the
/// hook; the Lifesteal is the GrantHeroLifestealThisTurn variant (hero
/// component + per-player flag, cleared at turn end).
pub const FEL_INFUSION: CardDef = CardDef {
    id: "CATA_530",
    name: "Fel Infusion",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
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
    spell_effect: Some(CardEffect::GrantHeroLifestealThisTurn),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_561 Ritual of Power — 2-cost (SPELL, Shaman).
/// "Herald {0}. Get two 1/1 Elementals with Rush." — the Herald is the
/// hook; the two CATA_561t Breezlings come from SummonMultipleMinions
/// (the Breezling carries no {0} — no Herald scaling).
pub const RITUAL_OF_POWER: CardDef = CardDef {
    id: "CATA_561",
    name: "Ritual of Power",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
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
    spell_effect: Some(CardEffect::SummonMultipleMinions {
        card_id: "CATA_561t",
        count: 2,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_565 Skywall Sentinel — 2-cost 0/2 (MINION, Shaman).
/// "Taunt. Battlecry: Herald {0}." — the battlecry is the Herald keyword
/// (hook).
pub const SKYWALL_SENTINEL: CardDef = CardDef {
    id: "CATA_565",
    name: "Skywall Sentinel",
    card_type: CardType::Minion,
    cost: 2,
    attack: 0,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
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

/// CATA_580 Cataclysmic War Axe — 3-cost weapon 3/2 (WEAPON, Warrior).
/// "Battlecry: Herald {0}." — the battlecry is the Herald keyword (the
/// weapon path calls the hook after equipping).
pub const CATACLYSMIC_WAR_AXE: CardDef = CardDef {
    id: "CATA_580",
    name: "Cataclysmic War Axe",
    card_type: CardType::Weapon,
    cost: 3,
    attack: 3,
    health: 0,
    durability: 2,
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

/// CATA_722 Envoy of the End — 5-cost 5/4 (MINION, Neutral).
/// "Taunt. Battlecry: Herald {0}." — the NEUTRAL patron: the full dump
/// has no neutral Soldier token, so the Herald only increments the
/// counter (nothing summons, §24).
pub const ENVOY_OF_THE_END: CardDef = CardDef {
    id: "CATA_722",
    name: "Envoy of the End",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
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

/// CATA_725 Shadowsworn Disciple — 2-cost 2/1 (MINION, Warlock).
/// "Battlecry: Herald {0}. Deathrattle: Restore 3 Health to your hero." —
/// the battlecry is the Herald keyword (hook); the deathrattle rides the
/// CardDef.
pub const SHADOWSWORN_DISCIPLE: CardDef = CardDef {
    id: "CATA_725",
    name: "Shadowsworn Disciple",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::RestoreHealth {
        amount: 3,
        target: EffectTarget::FriendlyHero,
    }),
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

/// CATA_780 Obsessive Technician — 4-cost 3/5 (MINION, Death Knight).
/// "Lifesteal. Battlecry: Herald {0}." — the battlecry is the Herald
/// keyword (hook); Lifesteal rides apply_card_keywords.
pub const OBSESSIVE_TECHNICIAN: CardDef = CardDef {
    id: "CATA_780",
    name: "Obsessive Technician",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
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

/// CATA_785 Rite of Twilight — 2-cost (SPELL, Rogue).
/// "Herald {0}. Combo: Deal 3 damage." — the Herald is the hook (fires
/// regardless of the combo status); the combo effect targets any
/// character (the official "$3 damage" has no target filter, pinned from
/// the dump). Without the combo the spell resolves nothing.
pub const RITE_OF_TWILIGHT: CardDef = CardDef {
    id: "CATA_785",
    name: "Rite of Twilight",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
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
    combo_effect: Some(CardEffect::DealDamage {
        amount: 3,
        target: EffectTarget::AnyCharacter,
    }),
    attack_equals_health: false,
};

// ---------------------------------------------------------------------
// Soldier tokens (the 6 Colossal Soldiers — the {0} numbers are baked at
// summon and re-baked on every Herald by cards/herald.rs; the CardDef
// values below are the family BASE = the counter-1 tier, §24) + the
// Ritual-of-Power Breezling.
// ---------------------------------------------------------------------

/// CATA_525t Soldier of Azshara — 1-cost 2/1 (MINION, Demon Hunter).
/// "When summoned, give your hero +{0} Attack this turn." — the on-summon
/// effect is resolved by the Herald hook with the counter read at
/// resolution time (base +2, §24); the CardDef carries no effect.
pub const SOLDIER_OF_AZSHARA: CardDef = CardDef {
    id: "CATA_525t",
    name: "Soldier of Azshara",
    card_type: CardType::Minion,
    cost: 1,
    attack: 2,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Naga),
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

/// CATA_565t Soldier of Al'Akir — 1-cost 1/2 (MINION, Shaman).
/// "Adjacent minions have +{0} Attack." — the aura (base +1, §24) is
/// re-baked on every Herald; the CardDef carries the base aura.
pub const SOLDIER_OF_ALAKIR: CardDef = CardDef {
    id: "CATA_565t",
    name: "Soldier of Al'Akir",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Elemental),
    hero_power: None,
    aura: Some((AuraEffect::GainAttack(1), AuraTarget::AdjacentMinions)),
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

/// CATA_580t Soldier of Ragnaros — 1-cost 2/1 (MINION, Warrior).
/// "Deathrattle: Deal {0} damage to a random enemy." — the deathrattle
/// (base 2, §24) is re-baked on every Herald; the CardDef carries the
/// base deathrattle.
pub const SOLDIER_OF_RAGNAROS: CardDef = CardDef {
    id: "CATA_580t",
    name: "Soldier of Ragnaros",
    card_type: CardType::Minion,
    cost: 1,
    attack: 2,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DealDamageRandomly {
        amount: 2,
        count: 1,
        target: EffectTarget::AnyEnemy,
    }),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Elemental),
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

/// CATA_725t Soldier of Cho'gall — 1-cost 1/1 (MINION, Warlock).
/// "At the end of your turn, destroy the minion to the right to gain
/// +{0}/+{0}." — the end-of-turn trigger (base +2/+2, §24) is re-baked
/// on every Herald; the Cho'gall deck-destroy redirect is baked into
/// ColossalArmDestroyRight itself. The CardDef carries the base trigger.
pub const SOLDIER_OF_CHOGALL: CardDef = CardDef {
    id: "CATA_725t",
    name: "Soldier of Cho'gall",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
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
    end_turn_effect: Some(CardEffect::ColossalArmDestroyRight {
        attack: 2,
        health: 2,
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

/// CATA_780t Soldier of Onyxia — 1-cost 1/1 (MINION, Death Knight).
/// "When summoned, get a random {0}-Cost minion. It costs Health this
/// turn." — the on-summon effect is resolved by the Herald hook with the
/// counter read at resolution time (base 1-Cost, §24); the CardDef
/// carries no effect.
pub const SOLDIER_OF_ONYXIA: CardDef = CardDef {
    id: "CATA_780t",
    name: "Soldier of Onyxia",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
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

/// CATA_158t Soldier of Sinestra — 1-cost 1/1 (MINION, Rogue).
/// "When summoned, get a random spell from another class. It costs ({0})
/// less." — the on-summon effect is resolved by the Herald hook (the
/// {0}-less reduction is DROPPED — the Sinestra's-Wing §23 convention,
/// §24); the CardDef carries no effect.
pub const SOLDIER_OF_SINESTRA: CardDef = CardDef {
    id: "CATA_158t",
    name: "Soldier of Sinestra",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
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

/// CATA_561t Breezling — 1-cost 1/1 (MINION, Shaman).
/// "Rush." — the Ritual-of-Power token, no {0} (no Herald scaling);
/// Rush rides apply_card_keywords.
pub const BREEZLING: CardDef = CardDef {
    id: "CATA_561t",
    name: "Breezling",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Elemental),
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
