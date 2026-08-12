//! MEND W3 cards (exp_cata_w7) — the Cataclysm class-set W3 wave (2025–
//! 2026 expansions master roadmap M5 follow-up): the Mage class set
//! (7 cards: MEND_500~506). These are registered in `sets.rs`
//! (HANDWRITTEN_EXPANSION_CARDS) like the other effect waves; the MEND_
//! cards have no generated baselines, so the `expansion_differential_gate`
//! tripwire skips them.
//!
//! Implementation decisions (per the MEND W3 spec, verified against
//! `cards/data/CATACLYSM.json` and the official rules 2026-08-12; the
//! fidelity rows are fidelity-debt.md §31, en + zh):
//! - **Leylines** (the card-group keyword on MEND_500/502/504) are
//!   upgraded by the wave's other cards via three persistent per-player
//!   flags (`leyline_discount` / `leyline_extra_trigger` /
//!   `leyline_effect_bonus`, `src/core/player.rs`); the registry
//!   `cards::leyline` keys the cost discount, the random Leyline draws
//!   and Arcanomicon's "Get all 3 Leylines".
//! - **Surge Needle** (MEND_503): "trigger an additional time" = one
//!   extra REPETITION of the Leyline's effect — an extra hit, an extra
//!   summoned minion, an extra drawn card (the official {1} "times"
//!   scalar).
//! - **Mystic Runesaber** (MEND_506): "increase the effects by 1" = one
//!   extra unit of the Leyline's {0} scalar — +1 damage, a +1-Cost
//!   summoned minion, one more cost reduction.
//! - **The Arcanomicon** (MEND_505) is a real three-option Choose One:
//!   every branch gets all 3 Leylines, then applies one of the three
//!   upgrades (option 0 = battlecry slot, option 1 = choose-one slot,
//!   option 2 = the `choose_one_three_branch` table, see `cards/mod.rs`).
//! - **Bursting Leyline** (MEND_500) is ImmuneToSpellpower — the
//!   `apply_spell_power` exemption list (trigger.rs) skips it (defense in
//!   depth; the excess-damage variant never picks up the bonus anyway,
//!   §31).
//! - The excess damage follows the Briarspawn Drake convention
//!   (`AttackRandomEnemyMinionExcess`): a target that dies to the hit
//!   passes the remainder to the enemy hero (Divine Shield absorbs the
//!   first point and leaks nothing).

use crate::cards::def::CardDef;
use crate::core::component::{CardType, LeylineUpgrade, Race};
use crate::core::effect::CardEffect;

/// MEND_500 Bursting Leyline — 4-cost (SPELL, Mage, Leyline). "Deal 3
/// damage to a random enemy minion. Excess damage hits the enemy hero."
/// — one hit, with the excess piped to the hero (§31). ImmuneToSpellpower:
/// the spell-power exemption list (trigger.rs) keeps it unboosted.
pub const BURSTING_LEYLINE: CardDef = CardDef {
    id: "MEND_500",
    name: "Bursting Leyline",
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
    spell_effect: Some(CardEffect::DealDamageToRandomEnemyMinionExcessToHero {
        amount: 3,
        times: 1,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_501 Ley Walker — 3-cost 4/2 (MINION, Mage). "Battlecry: Your
/// Leylines cost (1) less this game. Deathrattle: Get a random Leyline."
/// — sets the `leyline_discount` flag; the deathrattle draws one of the
/// three Leylines at random (the `cards::leyline` registry).
pub const LEY_WALKER: CardDef = CardDef {
    id: "MEND_501",
    name: "Ley Walker",
    card_type: CardType::Minion,
    cost: 3,
    attack: 4,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::SetLeylineDiscount { amount: 1 }),
    deathrattle: Some(CardEffect::AddRandomLeylineToHand),
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

/// MEND_502 Crystallized Leyline — 5-cost (SPELL, Mage, Leyline).
/// "Summon a random 5-Cost minion." — one summon from the full
/// `random_minion_of_cost` catalog (tokens excluded, the W1 §29
/// convention); "effect +1" raises the summoned minion's Cost by 1.
pub const CRYSTALLIZED_LEYLINE: CardDef = CardDef {
    id: "MEND_502",
    name: "Crystallized Leyline",
    card_type: CardType::Spell,
    cost: 5,
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
    spell_effect: Some(CardEffect::SummonRandomCostMinionTimes { cost: 5, times: 1 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_503 Surge Needle — 4-cost 3/5 (MINION, Mage). "Battlecry: Your
/// Leylines trigger an additional time this game." — sets the
/// `leyline_extra_trigger` flag; each Leyline card resolves one extra
/// repetition (§31).
pub const SURGE_NEEDLE: CardDef = CardDef {
    id: "MEND_503",
    name: "Surge Needle",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::SetLeylineExtraTrigger { amount: 1 }),
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

/// MEND_504 Leyline Nexus — 2-cost (SPELL, Mage, Leyline). "Draw a card.
/// It costs (1) less." — draws one card and applies a (1) cost
/// enchantment to the drawn card; "effect +1" deepens the reduction and
/// "trigger an additional time" draws another card (§31).
pub const LEYLINE_NEXUS: CardDef = CardDef {
    id: "MEND_504",
    name: "Leyline Nexus",
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
    spell_effect: Some(CardEffect::DrawCardsCostsLess {
        reduction: 1,
        count: 1,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_505 The Arcanomicon — 6-cost (SPELL, Mage, LEGENDARY). "Get all
/// 3 Leylines. Choose an upgrade for your Leylines." — a real three-option
/// Choose One: every branch adds all 3 Leylines to hand, then applies one
/// of the three upgrades (option 0 = Discount via the battlecry slot,
/// option 1 = ExtraTrigger via the choose-one slot, option 2 =
/// EffectBonus via the `choose_one_three_branch` table). The choice
/// surfaces because `choose_one_effect` is set; the shared
/// get-all-Leylines half lives inside every branch (the Talanji
/// convention) so it resolves exactly once regardless of the chosen
/// option (§31).
pub const THE_ARCANOMICON: CardDef = CardDef {
    id: "MEND_505",
    name: "The Arcanomicon",
    card_type: CardType::Spell,
    cost: 6,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::GetAllLeylinesAndUpgrade {
        upgrade: LeylineUpgrade::Discount,
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
    choose_one_effect: Some(CardEffect::GetAllLeylinesAndUpgrade {
        upgrade: LeylineUpgrade::ExtraTrigger,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_506 Mystic Runesaber — 2-cost 2/3 (MINION, Mage, Beast).
/// "Elusive. Battlecry: Increase the effects of your Leylines by 1 this
/// game." — Elusive and the `leyline_effect_bonus` flag. Registered as a
/// Beast (the Cataclysm set is in the standard window, so it joins the
/// in-window Beast pools — the barrens_stablehand test was re-pinned,
/// §31).
pub const MYSTIC_RUNESABER: CardDef = CardDef {
    id: "MEND_506",
    name: "Mystic Runesaber",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::SetLeylineEffectBonus { amount: 1 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: true,
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
