//! M4-W3 cards (exp_cata_w3) — the Cataclysm sub-roadmap W3: the 6
//! **Shatter** cards and their 2 board tokens (2025–2026 expansions
//! master roadmap M4-W3, the C3 Shatter mechanic).
//!
//! Implementation decision — **the D2 simplification** (sanctioned by the
//! roadmap, pinned in the M4-W3 spec 2026-08-09; the fidelity rows are
//! fidelity-debt.md §25, en + zh):
//! - The official dump has NO half-card tokens — a full split/recombine
//!   pipeline would synthesize halves from card texts with no data anchor,
//!   and CATA_202 Stolen Power's text literally says "(It's already
//!   combined)" — the combined form is the playable norm.
//! - **Shatter cards are played as their combined full cards**: each card
//!   carries ONE effect (or one new combined CardEffect variant) that
//!   resolves the whole "Shatter. <combined effect>" text — no draw-split,
//!   no halves, no recombination. The `CATA_xxx t/t2` half-card tokens of
//!   the dump are NOT implemented.
//! - All effects map to existing primitives (Treant / Drake tokens from
//!   the full dump; the attach-deathrattle from M2-W4c's Sheep Mask
//!   precedent — `GrantDeathrattleAll`; the copy from M2-W4b — the
//!   `SummonCopyOfFriendlyMinion` convention; the D2 random pool from the
//!   Mask-pool precedent — `pool::SHATTER_POOL`).
//! - The new combined variants (`SummonMinionsAndGrantDeathrattleAll`,
//!   `GainStatsElusiveAndSummonCopy`,
//!   `SummonMinionsAndGrantFriendlyAttackDivineShield`,
//!   `DealDamageAndDamageAllEnemies`, `DrawMinionsAndBuffHandMinions`,
//!   `AddRandomShatterCardToHand`) ride the spell_effect slot (the
//!   battlecry component at spawn) and are mirrored by the bincode
//!   `CardEffectDe` deserializer + the bot scoring arm.

use crate::cards::def::CardDef;
use crate::core::component::{CardType, Race};
use crate::core::effect::CardEffect;

/// CATA_134 Wildwood Circle — 4-cost (SPELL, Druid).
/// "Shatter. Summon two 2/2 Treants. Give your minions 'Deathrattle:
/// Summon a 2/2 Treant.'" — the D2 combined form: the two CATA_134t3
/// Treants, then the Treant-summon deathrattle attaches to every friendly
/// minion (the freshly summoned Treants included — they are your minions
/// when the buff resolves, the Longneck-Egg convention).
pub const WILDWOOD_CIRCLE: CardDef = CardDef {
    id: "CATA_134",
    name: "Wildwood Circle",
    card_type: CardType::Spell,
    cost: 4,
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
    spell_effect: Some(CardEffect::SummonMinionsAndGrantDeathrattleAll {
        card_id: "CATA_134t3",
        count: 2,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_202 Stolen Power — 3-cost (SPELL, Rogue).
/// "Get a random Shatter card from another class. (It's already
/// combined)." — the D2 random pool: the fixed `SHATTER_POOL` (the other
/// 5 Shatter cards — Wildwood Circle, Schism, Flight Maneuvers, Arcane
/// Flow, Supply Run — all non-Rogue, so the "from another class" filter
/// leaves the full set); the gotten card is the combined playable form.
pub const STOLEN_POWER: CardDef = CardDef {
    id: "CATA_202",
    name: "Stolen Power",
    card_type: CardType::Spell,
    cost: 3,
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
    spell_effect: Some(CardEffect::AddRandomShatterCardToHand),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_306 Schism — 4-cost (SPELL, Priest).
/// "Shatter. Give a friendly minion +2/+3 and Elusive. Summon a copy of
/// it." — the D2 combined form: one pick feeds all three parts (buff,
/// Elusive, copy — the copy is the base card, the
/// SummonCopyOfFriendlyMinion convention). Targeted: "Select a friendly
/// minion."
pub const SCHISM: CardDef = CardDef {
    id: "CATA_306",
    name: "Schism",
    card_type: CardType::Spell,
    cost: 4,
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
    spell_effect: Some(CardEffect::GainStatsElusiveAndSummonCopy {
        attack: 2,
        health: 3,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_479 Flight Maneuvers — 4-cost (SPELL, Paladin).
/// "Shatter. Summon two 4/2 Drakes. Give your minions +1 Attack and
/// Divine Shield." — the D2 combined form: the two CATA_479t3 Sky Drakes,
/// then every friendly minion (the fresh Drakes included) gains +1 Attack
/// and Divine Shield.
pub const FLIGHT_MANEUVERS: CardDef = CardDef {
    id: "CATA_479",
    name: "Flight Maneuvers",
    card_type: CardType::Spell,
    cost: 4,
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
    spell_effect: Some(
        CardEffect::SummonMinionsAndGrantFriendlyAttackDivineShield {
            card_id: "CATA_479t3",
            count: 2,
            attack: 1,
        },
    ),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_489 Arcane Flow — 4-cost (SPELL, Mage).
/// "Shatter. Deal $4 damage. Deal $2 damage to all enemies." — the D2
/// combined form: the primary part targets any character (the official
/// "$4 damage" has no target filter, the Rite-of-Twilight §24 pin), then
/// the splash hits all enemies (the enemy hero included). Both amounts
/// ride the spell-damage pipeline (`apply_spell_power`, the "$" marker).
pub const ARCANE_FLOW: CardDef = CardDef {
    id: "CATA_489",
    name: "Arcane Flow",
    card_type: CardType::Spell,
    cost: 4,
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
    spell_effect: Some(CardEffect::DealDamageAndDamageAllEnemies { amount: 4, aoe: 2 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// CATA_820 Supply Run — 4-cost (SPELL, Hunter).
/// "Shatter. Draw 3 minions. Give minions in your hand +2/+2." — the D2
/// combined form: three random minions drawn from the deck (the
/// Disciple-of-the-Dove §20 convention — a random minion per draw, no
/// deck-order digging), then every minion in hand (the drawn ones
/// included) gains +2/+2.
pub const SUPPLY_RUN: CardDef = CardDef {
    id: "CATA_820",
    name: "Supply Run",
    card_type: CardType::Spell,
    cost: 4,
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
    spell_effect: Some(CardEffect::DrawMinionsAndBuffHandMinions {
        count: 3,
        attack: 2,
        health: 2,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

// ---------------------------------------------------------------------
// Board tokens (from the full dump — the only tokens the Shatter cards
// summon; the half-card `CATA_xxx t/t2` "Shattered" tokens are NOT
// implemented, §25).
// ---------------------------------------------------------------------

/// CATA_134t3 Treant — 1-cost 2/2 (MINION, Druid). The Wildwood Circle
/// token — summoned twice by the card and once by the attached
/// "Deathrattle: Summon a 2/2 Treant."
pub const TREANT: CardDef = CardDef {
    id: "CATA_134t3",
    name: "Treant",
    card_type: CardType::Minion,
    cost: 1,
    attack: 2,
    health: 2,
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

/// CATA_479t3 Sky Drake — 3-cost 4/2 (MINION, Paladin, Dragon). The
/// Flight Maneuvers token — summoned twice by the card, then buffed by
/// the "your minions" part like every other friendly minion.
pub const SKY_DRAKE: CardDef = CardDef {
    id: "CATA_479t3",
    name: "Sky Drake",
    card_type: CardType::Minion,
    cost: 3,
    attack: 4,
    health: 2,
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
