//! MEND W2 cards (exp_cata_w6) — the Cataclysm class-set W2 wave (2025–
//! 2026 expansions master roadmap M5 follow-up): the Hunter class set
//! (7 cards: MEND_300~305 + MEND_307; MEND_306 has no card data). These
//! are registered in `sets.rs` (HANDWRITTEN_EXPANSION_CARDS) like the
//! other effect waves; the MEND_ cards have no generated baselines, so the
//! `expansion_differential_gate` tripwire skips them.
//!
//! Implementation decisions (per the MEND W2 spec, verified against
//! `cards/cards.json` 2026-08-12 and the official rules; the fidelity rows
//! are fidelity-debt.md §30, en + zh):
//! - **Future Animal Companion replacement** (MEND_300 Tame Pet,
//!   MEND_303 Migrating Elekk, MEND_307 Roam Free) is a Player flag
//!   (`companion_replacement`, `src/core/player.rs`): every
//!   `RandomPool::Companion` summon — the HUNTER_023 Animal Companion
//!   battlecry, Call of the Wild's `SummonAllCompanions`, and Broll
//!   Bearmantle's `SummonRandomAnimalCompanion` — resolves through
//!   `trigger::resolve_companion_summons` instead, sampling a random Beast
//!   of cost `3 + bump` from the active window per summon (§30: the
//!   official replacement picks among three fixed Beasts and upgrades the
//!   trio on repeated casts; we re-sample per summon and accumulate the
//!   bump).
//! - **Talya Earthstrider** (MEND_304) rides a second Player flag
//!   (`companion_bonus`): each Animal Companion summon summons one extra
//!   companion, each independently subject to the replacement flag.
//! - **Spiritspeaker** (MEND_301) is a real Choose One with the three
//!   companions as branches (Huffer/Leokk/Misha); each branch summons its
//!   companion directly, bypassing the replacement/bonus pipeline (§30).
//! - **Roam Free** (MEND_307) is a three-branch Choose One: every branch
//!   sets the shared replacement flag (bump 2) and summons a random Beast
//!   of its own cost tier (5/6/7 — §30: a random Beast of the exact tier
//!   cost instead of the official fixed trio).
//! - **Wasteland Vanguard** (MEND_302) splits 3 damage among all enemies
//!   one point at a time (the Erupting Volcano ping convention) and, if
//!   any enemy dies, deals 3 more — exactly one extra wave (official
//!   ruling: the chain does not loop even if the second wave kills; §30).
//!   The death check is the Shadow Rounds prediction convention with the
//!   Divine Shield refinement (the first point is absorbed).
//! - **Nurturing Nature** (MEND_305) buffs a friendly Beast (the explicit
//!   play target when valid, else a random one) and a random Beast in
//!   hand; the hand buff writes the base stats directly (the FordragonBuff
//!   convention, §30).

use crate::cards::def::CardDef;
use crate::core::component::{CardType, Race};
use crate::core::effect::CardEffect;

/// MEND_300 Tame Pet — 1-cost (SPELL, Hunter). "Replace your future
/// Animal Companions with random Beasts that cost (1) more. Draw a card."
/// — sets the `companion_replacement` flag (bump 1) and draws (§30).
pub const TAME_PET: CardDef = CardDef {
    id: "MEND_300",
    name: "Tame Pet",
    card_type: CardType::Spell,
    cost: 1,
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
    spell_effect: Some(CardEffect::SetCompanionReplacementAndDraw { bump: 1, draw: 1 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_301 Spiritspeaker — 4-cost 2/2 (MINION, Hunter). "Battlecry:
/// Choose an Animal Companion to summon." — a real Choose One: Huffer /
/// Leokk / Misha as the three branches, each summoning its companion
/// directly (bypassing the replacement/bonus pipeline, §30).
pub const SPIRITSPEAKER: CardDef = CardDef {
    id: "MEND_301",
    name: "Spiritspeaker",
    card_type: CardType::Minion,
    cost: 4,
    attack: 2,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::SummonMinion {
        card_id: "HUNTER_023a",
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
    choose_one_effect: Some(CardEffect::SummonMinion {
        card_id: "HUNTER_023b",
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_302 Wasteland Vanguard — 4-cost 3/3 (MINION, Hunter). "Battlecry:
/// Deal 3 damage split among all enemies. If any die, deal 3 more." — the
/// split pings re-pick among the enemies per point; the chain fires at
/// most once (official ruling; §30).
pub const WASTELAND_VANGUARD: CardDef = CardDef {
    id: "MEND_302",
    name: "Wasteland Vanguard",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::SplitDamageAmongAllEnemiesChainOnDeath { amount: 3 }),
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

/// MEND_303 Migrating Elekk — 2-cost 2/3 (MINION, Hunter, Beast). "Taunt.
/// Battlecry: Replace your future Animal Companions with random Beasts
/// that cost (1) more." — sets the `companion_replacement` flag (bump 1);
/// repeated casts accumulate the bump (§30).
pub const MIGRATING_ELEKK: CardDef = CardDef {
    id: "MEND_303",
    name: "Migrating Elekk",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::SetCompanionReplacement { bump: 1 }),
    deathrattle: None,
    taunt: true,
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

/// MEND_304 Talya Earthstrider — 5-cost 4/6 (MINION, Hunter, Legendary).
/// "Battlecry: Your cards that summon Animal Companions summon 1 more
/// this game." — sets the `companion_bonus` flag; every Companion summon
/// gains one extra companion (§30).
pub const TALYA_EARTHSTRIDER: CardDef = CardDef {
    id: "MEND_304",
    name: "Talya Earthstrider",
    card_type: CardType::Minion,
    cost: 5,
    attack: 4,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::SetCompanionBonus { amount: 1 }),
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

/// MEND_305 Nurturing Nature — 2-cost (SPELL, Hunter, Nature). "Give a
/// friendly Beast +2/+2. Give a random Beast in your hand +2/+2." — the
/// board target honours the explicit play target when it is a friendly
/// Beast, else a random friendly Beast; the hand buff writes the base
/// stats directly (§30).
pub const NURTURING_NATURE: CardDef = CardDef {
    id: "MEND_305",
    name: "Nurturing Nature",
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
    spell_effect: Some(CardEffect::BuffFriendlyBeastAndRandomHandBeast {
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

/// MEND_307 Roam Free — 7-cost (SPELL, Hunter). "Replace your future
/// Animal Companions with random Beasts that cost (2) more. Choose one to
/// summon." — a three-branch Choose One; every branch sets the shared
/// replacement flag (bump 2, §30: random Beast of the exact tier cost
/// instead of the official fixed trio) and summons a Beast of its own
/// cost tier (5/6/7).
pub const ROAM_FREE: CardDef = CardDef {
    id: "MEND_307",
    name: "Roam Free",
    card_type: CardType::Spell,
    cost: 7,
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
    spell_effect: Some(CardEffect::ReplaceCompanionsAndSummonRandomBeast { bump: 2, cost: 5 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: Some(CardEffect::ReplaceCompanionsAndSummonRandomBeast { bump: 2, cost: 6 }),
    combo_effect: None,
    attack_equals_health: false,
};
