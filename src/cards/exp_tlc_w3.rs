//! 2025–2026 expansions M2-W3 cards (exp_tlc_w3) — the Un'Goro Kindred
//! wave: the 23 Kindred cards + their tokens.
//!
//! The Kindred mechanic itself is M2-W3 (the `cards::kindred` registry —
//! type + effect shape keyed by card id, the `Player::kindred_played`
//! played-type list, the resolution points in the play path / cost
//! pipeline). This wave adds the cards the registry references: the
//! handwritten CardDefs (they shadow the generated baselines through the
//! `card_by_id` chain — same stat lines) and the tokens.
//!
//! Kindred shapes per the matrix (fidelity-debt §16):
//! - OnPlay effects: TLC_107 GainRush, TLC_226 SummonCopyOfSelf, TLC_243
//!   GainImmuneThisTurn, TLC_428 GiveNextMurlocDivineShield, TLC_429
//!   SummonMultipleMinions (two TLC_429t), TLC_440 DrawCard, TLC_447
//!   DamageAllMinions, TLC_519 SummonMinion (one TLC_519t), TLC_815
//!   SummonRandomMinionCostTaunt, TLC_825 DealSelfAttackDamage, TLC_903
//!   GainHeroAttack — resolved by `kindred::resolve_on_play` after the
//!   base resolution, activation counted at >= 2 (the card's own push +
//!   an earlier same-type card).
//! - Cost discounts: TLC_366 (2), TLC_600 (3), TLC_816 (2) — applied by
//!   the cost pipeline, activation counted at >= 1 (before the push).
//! - Battlecry modifiers: TLC_454/463/829 replace the battlecry when
//!   active, TLC_482 adds `TriggerFriendlyCinderDeathrattles` after it —
//!   `kindred::apply_battlecry_modifier` at the battlecry resolution.
//! - Folded drawn-card modifiers: TLC_223 (Fire spell + Spell Damage +2),
//!   TLC_236 (one minion of each cost 1-4, then (1) less each), TLC_432
//!   (Deathrattle minion <= 3, then costs (0)) — the kindred modification
//!   is checked inside the dedicated battlecry variants at >= 2.
//! - TLC_102 Torga: the special `DrawKindredAndActivator` battlecry.
//! - TLC_251 Primalfin Challenger: the `SetNextKindredTwice` battlecry.
//!
//! Tokens: TLC_429t Juvenile Steamfin (1/1/1 Murloc Rush), TLC_519t
//! Venomous Spitter (2/1/1 Beast Stealth+Poisonous), and TLC_249 Sizzling
//! Cinder (1/2/1 Elemental — a collectible card written here as well,
//! since Slagclaw's battlecry summons it and the W3 scenarios pin the
//! Cinder deathrattle; the handwritten def shadows the generated baseline,
//! same stat lines).
//!
//! Card data: `cards/data/THE_LOST_CITY.json` + the full dump
//! (`/tmp/hs_full.json` for the tokens; verified 2026-08-09).

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::EffectTarget;

// ============================================================
// The 23 Kindred cards. Minions carry their primary race (the Kindred
// text keys on it; the second tribe of the six multi-race cards lands via
// `apply_card_keywords`); spells cast through the battlecry slot (the
// engine convention — the spell branch resolves `battlecry`).
// ============================================================

/// TLC_102 Torga — 4-cost 2/7 Beast. Battlecry: draw the first Kindred
/// card from the deck, then the first remaining card of the same Kindred
/// type (the special `DrawKindredAndActivator` scan — `cards::kindred`
/// enumerates the registry for the first half, `played_type_of` for the
/// activator half).
pub const TORGA: CardDef = CardDef {
    id: "TLC_102",
    name: "Torga",
    card_type: CardType::Minion,
    cost: 4,
    attack: 2,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::DrawKindredAndActivator),
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

/// TLC_107 Stormbrewer — 5-cost 3/6 Elemental. "Whenever this attacks,
/// deal 3 damage to the target first" — the attack hook lives in the
/// AttackDeclared handler (the Lake Thresher precedent: the trigger
/// effects only see the attacker, not the defender), the strike enqueued
/// before the attack damage. Kindred: Gain Rush.
pub const STORMBREWER: CardDef = CardDef {
    id: "TLC_107",
    name: "Stormbrewer",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 6,
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

/// TLC_223 Volcanic Thrasher — 3-cost 2/3 Elemental (+Beast). Battlecry:
/// draw a Fire spell (the W1 spell-school registry filters the deck);
/// Kindred: give it Spell Damage +2 (checked inside
/// `DrawSpellGiveSpellDamage`).
pub const VOLCANIC_THRASHER: CardDef = CardDef {
    id: "TLC_223",
    name: "Volcanic Thrasher",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::DrawSpellGiveSpellDamage { amount: 2 }),
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

/// TLC_226 Conjured Bookkeeper — 3-cost 2/2 Elemental. Deathrattle: draw a
/// spell. Kindred: summon a copy of this (the copy never re-fires — only
/// played cards push their type into `kindred_played`).
pub const CONJURED_BOOKKEEPER: CardDef = CardDef {
    id: "TLC_226",
    name: "Conjured Bookkeeper",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DrawCardByType {
        count: 1,
        card_type: CardType::Spell,
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

/// TLC_236 Hybridization — 5-cost Druid spell (Nature). Draw a minion of
/// each cost 1-4; Kindred: they cost (1) less (both inside
/// `DrawMinionsOfEachCost`).
pub const HYBRIDIZATION: CardDef = CardDef {
    id: "TLC_236",
    name: "Hybridization",
    card_type: CardType::Spell,
    cost: 5,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DrawMinionsOfEachCost { up_to: 4 }),
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

/// TLC_243 Whirling Stormdrake — 9-cost 8/8 Elemental (+Dragon). Rush (via
/// `apply_card_keywords`), Windfury. Kindred: gain Immune this turn.
pub const WHIRLING_STORMDRAGON: CardDef = CardDef {
    id: "TLC_243",
    name: "Whirling Stormdrake",
    card_type: CardType::Minion,
    cost: 9,
    attack: 8,
    health: 8,
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
    windfury: true,
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

/// TLC_249 Sizzling Cinder — 1-cost 2/1 Elemental. Deathrattle: deal 2
/// damage randomly split among all enemies (the existing
/// `DealDamageSplitAmongAllEnemies` — N independent 1-damage pings).
/// Written here as Slagclaw's summon token (shadowing the generated
/// baseline, same stat lines).
pub const SIZZLING_CINDER: CardDef = CardDef {
    id: "TLC_249",
    name: "Sizzling Cinder",
    card_type: CardType::Minion,
    cost: 1,
    attack: 2,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DealDamageSplitAmongAllEnemies { amount: 2 }),
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

/// TLC_251 Primalfin Challenger — 3-cost 3/2 Murloc. Battlecry: your next
/// Kindred triggers twice (the `next_kindred_twice` flag, consumed by the
/// next OnPlay Kindred resolution).
pub const PRIMALFIN_CHALLENGER: CardDef = CardDef {
    id: "TLC_251",
    name: "Primalfin Challenger",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::SetNextKindredTwice),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Murloc),
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

/// TLC_366 Pterrorwing Ravager — 6-cost 7/5 Beast. Rush (via
/// `apply_card_keywords`). Kindred: costs (2) less.
pub const PTERRORWING_RAVAGER: CardDef = CardDef {
    id: "TLC_366",
    name: "Pterrorwing Ravager",
    card_type: CardType::Minion,
    cost: 6,
    attack: 7,
    health: 5,
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

/// TLC_428 Hot Spring Glider — 3-cost 2/4 Murloc. Battlecry: your next
/// Murloc costs (1) less (the `next_murloc_discount` flag). Kindred: your
/// next Murloc gains Divine Shield (the `next_murloc_divine_shield` flag —
/// both consumed by the next Murloc play, applied in the play path).
pub const HOT_SPRING_GLIDER: CardDef = CardDef {
    id: "TLC_428",
    name: "Hot Spring Glider",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::NextMurlocCostsLess { amount: 1 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Murloc),
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

/// TLC_429 Steamfin Thief — 4-cost 4/3 Murloc. Kindred: summon two 1/1
/// Juvenile Steamfins with Rush (TLC_429t).
pub const STEAMFIN_THIEF: CardDef = CardDef {
    id: "TLC_429",
    name: "Steamfin Thief",
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
    race: Some(Race::Murloc),
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

/// TLC_429t Juvenile Steamfin — 1-cost 1/1 Murloc token with Rush
/// (Steamfin Thief's Kindred summons it; Rush via `apply_card_keywords`).
pub const JUVENILE_STEAMFIN: CardDef = CardDef {
    id: "TLC_429t",
    name: "Juvenile Steamfin",
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
    race: Some(Race::Murloc),
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

/// TLC_432 Dread Raptor — 4-cost 3/4 Undead (+Beast). Battlecry: draw a
/// Deathrattle minion costing (3) or less; Kindred: it costs (0) (both
/// inside `DrawDeathrattleMinionCostLE`).
pub const DREAD_RAPTOR: CardDef = CardDef {
    id: "TLC_432",
    name: "Dread Raptor",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DrawDeathrattleMinionCostLE { max_cost: 3 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Undead),
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

/// TLC_440 Cryosleep — 4-cost Death Knight spell (Frost). Deal 4 damage to
/// an enemy and draw a card. Kindred: draw another card.
pub const CRYOSLEEP: CardDef = CardDef {
    id: "TLC_440",
    name: "Cryosleep",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DealDamageAndDraw {
        damage: 4,
        target: EffectTarget::AnyEnemy,
        draw: 1,
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

/// TLC_447 Caustic Fumes — 4-cost Warlock spell (Fel). Destroy an enemy
/// minion. Kindred: deal 2 damage to all minions.
pub const CAUSTIC_FUMES: CardDef = CardDef {
    id: "TLC_447",
    name: "Caustic Fumes",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DestroyMinion {
        target: EffectTarget::AnyEnemyMinion,
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

/// TLC_454 Scalehide Kodo — 6-cost 3/6 Beast. Battlecry: destroy the
/// lowest-Attack enemy minion; Kindred: the highest instead (the
/// `BattlecryModifier` replace).
pub const SCALEHIDE_KODO: CardDef = CardDef {
    id: "TLC_454",
    name: "Scalehide Kodo",
    card_type: CardType::Minion,
    cost: 6,
    attack: 3,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::DestroyLowestAttackEnemy),
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

/// TLC_463 Razidir — 4-cost 7/7 Demon (+Beast). Battlecry: discard a
/// random card from your hand; Kindred: from your opponent's hand instead
/// (the `BattlecryModifier` replace).
pub const RAZIDIR: CardDef = CardDef {
    id: "TLC_463",
    name: "Razidir",
    card_type: CardType::Minion,
    cost: 4,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::DiscardRandomCard),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Demon),
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

/// TLC_482 Slagclaw — 5-cost 3/4 Elemental (+Dragon). Battlecry: summon
/// two 2/1 Sizzling Cinders (TLC_249); Kindred: trigger their
/// Deathrattles (the `BattlecryModifier` add-on).
pub const SLAGCLAW: CardDef = CardDef {
    id: "TLC_482",
    name: "Slagclaw",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::SummonMultipleMinions {
        card_id: "TLC_249",
        count: 2,
    }),
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

/// TLC_519 Ambush Predators — 3-cost Rogue spell (Shadow). Summon a 1/1
/// Venomous Spitter with Stealth and Poisonous (TLC_519t); Kindred: do it
/// again.
pub const AMBUSH_PREDATORS: CardDef = CardDef {
    id: "TLC_519",
    name: "Ambush Predators",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::SummonMinion {
        card_id: "TLC_519t",
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

/// TLC_519t Venomous Spitter — 2-cost 1/1 Beast token with Stealth and
/// Poisonous (Ambush Predators summons it; the keywords land via
/// `apply_card_keywords`, the Patient Assassin pattern).
pub const VENOMOUS_SPITTER: CardDef = CardDef {
    id: "TLC_519t",
    name: "Venomous Spitter",
    card_type: CardType::Minion,
    cost: 2,
    attack: 1,
    health: 1,
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

/// TLC_600 Windpeak Wyrm — 8-cost 6/6 Dragon. Battlecry: deal 5 damage and
/// gain 5 Armor. Kindred: costs (3) less.
pub const WINDPEAK_WYRM: CardDef = CardDef {
    id: "TLC_600",
    name: "Windpeak Wyrm",
    card_type: CardType::Minion,
    cost: 8,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::DamageAndGainArmor {
        damage: 5,
        armor: 5,
        target: EffectTarget::AnyEnemy,
    }),
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

/// TLC_815 Gravedawn Voidbulb — 4-cost Priest spell (Shadow). Summon a
/// random 4-Cost minion and give it Taunt; Kindred: do it again (the
/// random pool follows the D2 simplification — ALL_CARDS minions of cost
/// 4, token-excluded, §16).
pub const GRAVEDAWN_VOIDBULB: CardDef = CardDef {
    id: "TLC_815",
    name: "Gravedawn Voidbulb",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::SummonRandomMinionCostTaunt { cost: 4 }),
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

/// TLC_816 Gravedawn Sunbloom — 4-cost Priest spell (Holy). Draw 2 cards.
/// Kindred: this costs (2) less.
pub const GRAVEDAWN_SUNBLOOM: CardDef = CardDef {
    id: "TLC_816",
    name: "Gravedawn Sunbloom",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DrawCard { count: 2 }),
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

/// TLC_825 Ravasaur Matriarch — 4-cost 5/4 Beast. Kindred: deal damage to
/// an enemy minion equal to this minion's Attack. The targeting is
/// unsurfaced by the RL view (no battlecry slot) — the engine picks a
/// random enemy minion when no explicit target is passed (§16).
pub const RAVASAUR_MATRIARCH: CardDef = CardDef {
    id: "TLC_825",
    name: "Ravasaur Matriarch",
    card_type: CardType::Minion,
    cost: 4,
    attack: 5,
    health: 4,
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

/// TLC_829 Ravenous Devilsaur — 7-cost 3/3 Beast. Battlecry: destroy a
/// minion; Kindred: gain its stats (the `BattlecryModifier` replace —
/// stats are read before the destroy).
pub const RAVENOUS_DEVILSAUR: CardDef = CardDef {
    id: "TLC_829",
    name: "Ravenous Devilsaur",
    card_type: CardType::Minion,
    cost: 7,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::DestroyMinion {
        target: EffectTarget::AnyMinion,
    }),
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

/// TLC_903 Silithid Queen — 5-cost 5/2 Beast. Rush (via
/// `apply_card_keywords`). Kindred: give your hero +5 Attack this turn.
pub const SILITHID_QUEEN: CardDef = CardDef {
    id: "TLC_903",
    name: "Silithid Queen",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 2,
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
