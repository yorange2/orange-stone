//! MEND W4 cards (exp_cata_w8) — the Cataclysm class-set W4 wave (2025–
//! 2026 expansions master roadmap M5 follow-up): the Paladin class set
//! (7 cards: MEND_800~805 + MEND_900). These are registered in `sets.rs`
//! (HANDWRITTEN_EXPANSION_CARDS) like the other effect waves; the MEND_
//! cards have no generated baselines, so the `expansion_differential_gate`
//! tripwire skips them.
//!
//! Implementation decisions (per the MEND W4 spec, verified against
//! `cards/data/CATACLYSM.json` and the official card texts 2026-08-12; the
//! fidelity rows are fidelity-debt.md §32, en + zh):
//! - **The Silver Hand Recruit game-long bonus** is the wave's headline
//!   mechanic: Brash Battlemaster (MEND_800, deathrattle +1 Attack),
//!   Resilient Savior (MEND_801, trigger +1 Health) and Emboldening Blade
//!   (MEND_803, battlecry +1/+1) give "your Silver Hand Recruits" a
//!   permanent stat bonus for the rest of the game. Two per-player flags
//!   (`silver_hand_attack_bonus` / `silver_hand_health_bonus`,
//!   `src/core/player.rs`) accumulate the bonuses; every CORE_GVG_061t
//!   token created afterwards — summoned (`resolve_summon_doubled`) or
//!   added to hand (`add_card_to_hand`, so hand copies and
//!   played-from-hand Recruits honor them too) — carries them.
//! - **Resilient Savior** (MEND_801) fires on `TriggerEvent::DivineShieldLost`
//!   (the Fordragon event); the resolution checks the event subject so
//!   only the minion's own shield break counts.
//! - **Arator the Redeemer** (MEND_804) doubles the stats of the friendly
//!   Silver Hand Recruits in play at the battlecry only (the official
//!   ruling reads "all friendly Silver Hand Recruits" as the current
//!   board; future summons are unaffected) and gives them Taunt — the
//!   doubling is a permanent enchantment equal to the current stats, so
//!   already-buffed Recruits double their full effective stats.
//! - **Convalescence** (MEND_802) summons two 1/1 Recruits with Divine
//!   Shield — the {0} base is 1/1, fixed (the Paladin class set carries
//!   no upgrade mechanic for it). The Divine Shield lands on the summon
//!   the effect got back; a Khadgar-doubled twin misses the shield
//!   (the D2 edge, §32).
//! - **Charity** (MEND_805) reads the `Player::died_this_turn` snapshot
//!   (maintained by the death step, cleared at the owner's turn end) —
//!   copies of the dead friendly minions land in the hand with +3/+3
//!   baked into their base stats (the Grimestreet hand-buff convention).
//! - **Teamwork** (MEND_900) "Summons and gets four 1/1 Silver Hand
//!   Recruits": the even 2+2 split — two summoned, two added to hand —
//!   is the D2 decision (§32).

use crate::cards::def::CardDef;
use crate::core::component::{CardType, Race};
use crate::core::effect::CardEffect;

/// MEND_800 Brash Battlemaster — 2-cost 2/1 (MINION, Paladin).
/// "Rush. Deathrattle: Give your Silver Hand Recruits +1 Attack this
/// game." — Rush rides the keyword ID list (`apply_card_keywords`); the
/// deathrattle sets the `silver_hand_attack_bonus` flag (§32).
pub const BRASH_BATTLEMASTER: CardDef = CardDef {
    id: "MEND_800",
    name: "Brash Battlemaster",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 1,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SetSilverHandRecruitStats {
        attack: 1,
        health: 0,
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

/// MEND_801 Resilient Savior — 3-cost 3/1 (MINION, Paladin, Draenei).
/// "Divine Shield. After this loses Divine Shield, give your Silver Hand
/// Recruits +1 Health this game." — the DivineShieldLost trigger rides
/// the minion (registered in `apply_card_keywords`, the Fordragon
/// pattern); the effect's subject check pins the bonus to this minion's
/// own shield break (§32).
pub const RESILIENT_SAVIOR: CardDef = CardDef {
    id: "MEND_801",
    name: "Resilient Savior",
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
    race: Some(Race::Draenei),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: true,
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

/// MEND_802 Convalescence — 2-cost (SPELL, Paladin, Holy). "Summon two
/// 1/1 Silver Hand Recruits with Divine Shield." — the {0} base is 1/1,
/// fixed (§32 — the Paladin class set carries no upgrade mechanic for
/// it).
pub const CONVALESCENCE: CardDef = CardDef {
    id: "MEND_802",
    name: "Convalescence",
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
    spell_effect: Some(CardEffect::SummonSilverHandRecruitsWithDivineShield { count: 2 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_803 Emboldening Blade — 5-cost 3/2 (WEAPON, Paladin). "Battlecry:
/// Give your Silver Hand Recruits +1/+1 this game." — sets both
/// `silver_hand_attack_bonus` and `silver_hand_health_bonus` (§32).
pub const EMBOLDENING_BLADE: CardDef = CardDef {
    id: "MEND_803",
    name: "Emboldening Blade",
    card_type: CardType::Weapon,
    cost: 5,
    attack: 3,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::SetSilverHandRecruitStats {
        attack: 1,
        health: 1,
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

/// MEND_804 Arator the Redeemer — 5-cost 5/6 (MINION, Paladin,
/// LEGENDARY). "Battlecry: Double the stats of all friendly Silver Hand
/// Recruits and give them Taunt." — the doubling hits the CURRENT board
/// only (the official ruling; future summons are unaffected, §32); each
/// Recruit gains a permanent enchantment equal to its current stats plus
/// Taunt.
pub const ARATOR_THE_REDEEMER: CardDef = CardDef {
    id: "MEND_804",
    name: "Arator the Redeemer",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::AratorDoubleSilverHandRecruits),
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

/// MEND_805 Charity — 3-cost (SPELL, Paladin). "Get copies of all
/// friendly minions that died this turn. Give them +3/+3." — the copies
/// land in the hand with +3/+3 baked into their base stats (the
/// Grimestreet hand-buff convention, §32); a full hand burns the add
/// (F-A11).
pub const CHARITY: CardDef = CardDef {
    id: "MEND_805",
    name: "Charity",
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
    spell_effect: Some(CardEffect::CharityCopiesDiedThisTurn {
        attack: 3,
        health: 3,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_900 Teamwork — 4-cost (SPELL, Paladin). "Summon and get four
/// 1/1 Silver Hand Recruits." — the even 2+2 split (two summoned, two
/// added to hand) is the D2 decision (§32).
pub const TEAMWORK: CardDef = CardDef {
    id: "MEND_900",
    name: "Teamwork",
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
    spell_effect: Some(CardEffect::TeamworkSummonAndGetRecruits),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};
