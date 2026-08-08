//! 2025–2026 expansions M2-W2 cards (exp_tlc_w2) — the Un'Goro quest wave:
//! the 11 quest cards + their reward tokens.
//!
//! The quest mechanic itself is M2-W1 (the `Zone::Quest` slot, the `Quest`
//! component, the `cards::quest` registry, `engine::quest` dispatch). W2
//! adds the cards those registries reference: the quest cards' handwritten
//! CardDefs (they shadow the generated baselines through the `card_by_id`
//! chain — same stat lines, no spell effect, since the play path diverts
//! quests to the quest zone) and the reward tokens.
//!
//! Reward effects per the full/simplified matrix (fidelity-debt §15):
//! - TLC_229t14 Ashalon — full body, battlecry simplified (Adapt has no
//!   engine primitive): a one-time +1/+1 to the quest owner's board.
//! - TLC_239t The Everbloom — FULL: the weapon-attack trigger rides the
//!   weapon entity (Truesilver/Gorehowl pattern, `apply_card_keywords`).
//! - TLC_433t Tyrax — deathrattle simplified (the Terror's Grave location
//!   chain collapses to "resummon a copy of Tyrax").
//! - TLC_446t1 Underfel Rift — body only (the "throw a card in" activation
//!   is simplified away).
//! - TLC_460t The Origin Stone — body only (the discover-replay trigger
//!   needs discover-option storage the engine lacks).
//! - TLC_513t Master Dusk — data-completeness hero def (hero replacement
//!   is simplified away; the reward summons two Ninjas directly).
//! - TLC_513t2 Tortollan Ninja — real token (Stealth via
//!   `apply_card_keywords`).
//! - TLC_602t Latorvius — body only (the Quest-Rewards pool lands in W4).
//! - TLC_631t Gorishi Colossus — FULL: the battlecry sets the owner's
//!   permanent "exactly-2-damage bonus" flag.
//! - TLC_817t3/t4 Sol'etos halves — FULL (copy-summon battlecry / Reborn +
//!   random-enemy damage deathrattle); the combine is simplified away.
//! - TLC_830t Shokk — body + Rush (the attack-filtered Discover battlecry
//!   is simplified away).
//!
//! These consts are handwritten effect-wave implementations of the generated
//! expansion baselines: they never enter `ALL_CARDS` (the sampling pools
//! stay closed, decision D3) but are reachable through the `card_by_id`
//! chain (ALL_CARDS → HANDWRITTEN_EXPANSION_CARDS → EXPANSION_CARDS) and are
//! compared field-by-field against the generated baseline by the
//! `expansion_differential_gate` test (M0.4). Simplifications registered in
//! `docs/finished/fidelity-debt.md` §15 (2025–2026 expansions M2-W2).
//!
//! Card data: `cards/data/THE_LOST_CITY.json` + the full dump
//! (`/tmp/hs_full.json` for the tokens; verified 2026-08-09).

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::EffectTarget;

// ============================================================
// The 11 quest cards — 1-cost legendary SPELLs (mechanics QUEST).
// The play path diverts them to the quest zone, so they cast nothing
// (`spell_effect: None`) and match the generated baselines field-by-field
// (the `expansion_differential_gate` compares card_type/cost/attack/
// health/durability/taunt/divine_shield/windfury/charge/spell_damage).
// ============================================================

/// TLC_229 Spirit of the Mountain — 1-cost Shaman quest. Play 6 minions of
/// unique types. Reward: Ashalon, Ridge Guardian (TLC_229t14).
pub const SPIRIT_OF_THE_MOUNTAIN: CardDef = CardDef {
    id: "TLC_229",
    name: "Spirit of the Mountain",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_239 Restore the Wild — 1-cost Druid quest. Fill your board on 3 of
/// your turns. Reward: The Everbloom weapon (TLC_239t).
pub const RESTORE_THE_WILD: CardDef = CardDef {
    id: "TLC_239",
    name: "Restore the Wild",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_426 Dive the Golakka Depths — 1-cost Paladin Repeatable Quest.
/// Summon 6 Murlocs. Reward: permanently, Murlocs you summon gain +1/+1
/// (a player flag set by `CardEffect::SetMurlocSummonBuff`).
pub const DIVE_THE_GOLAKKA_DEPTHS: CardDef = CardDef {
    id: "TLC_426",
    name: "Dive the Golakka Depths",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_433 Reanimate the Terror — 1-cost Death Knight quest. Spend 15
/// Corpses. Reward: Tyrax, Bone Terror (TLC_433t).
pub const REANIMATE_THE_TERROR: CardDef = CardDef {
    id: "TLC_433",
    name: "Reanimate the Terror",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_446 Escape the Underfel — 1-cost Warlock quest. Play 6 Temporary
/// cards. Reward: Underfel Rift (TLC_446t1).
pub const ESCAPE_THE_UNDERFEL: CardDef = CardDef {
    id: "TLC_446",
    name: "Escape the Underfel",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_460 The Forbidden Sequence — 1-cost Mage quest. Discover 7 cards.
/// Reward: The Origin Stone weapon (TLC_460t).
pub const THE_FORBIDDEN_SEQUENCE: CardDef = CardDef {
    id: "TLC_460",
    name: "The Forbidden Sequence",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_513 Lie in Wait — 1-cost Rogue quest. Shuffle 5 cards into your
/// deck. Reward (simplified): summon two TLC_513t2 Tortollan Ninjas (the
/// real reward replaces the hero with TLC_513t Master Dusk — hero
/// replacement is simplified away, §15).
pub const LIE_IN_WAIT: CardDef = CardDef {
    id: "TLC_513",
    name: "Lie in Wait",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_602 Enter the Lost City — 1-cost Warrior quest. Survive 10 turns.
/// Reward: Latorvius, Gaze of the City (TLC_602t).
pub const ENTER_THE_LOST_CITY: CardDef = CardDef {
    id: "TLC_602",
    name: "Enter the Lost City",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_631 Unleash the Colossus — 1-cost Demon Hunter quest. Deal exactly
/// 2 damage to an enemy on your turn, 12 times. Reward: Gorishi Colossus
/// (TLC_631t).
pub const UNLEASH_THE_COLOSSUS: CardDef = CardDef {
    id: "TLC_631",
    name: "Unleash the Colossus",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_817 Reach Equilibrium — 1-cost Priest quest with TWO progress bars
/// (the only dual-bar quest, `QuestDef::second`): cast 4 Holy spells
/// (reward TLC_817t3) AND 4 Shadow spells (reward TLC_817t4); the card
/// leaves the quest slot only when both bars complete.
pub const REACH_EQUILIBRIUM: CardDef = CardDef {
    id: "TLC_817",
    name: "Reach Equilibrium",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TLC_830 The Food Chain — 1-cost Hunter quest. Play a 1, 3, 5, and
/// 7-Attack Beast. Reward: Shokk, Jungle Tyrant (TLC_830t).
pub const THE_FOOD_CHAIN: CardDef = CardDef {
    id: "TLC_830",
    name: "The Food Chain",
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
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

// ============================================================
// The reward tokens (handwritten only — no generated baselines).
// ============================================================

/// TLC_229t14 Ashalon, Ridge Guardian — 5/8/8 Elemental. Rush (applied by
/// `apply_card_keywords`). Battlecry (simplified: Adapt has no engine
/// primitive — §15): the quest owner's minions gain +1/+1 once.
pub const ASHALON_RIDGE_GUARDIAN: CardDef = CardDef {
    id: "TLC_229t14",
    name: "Ashalon, Ridge Guardian",
    card_type: CardType::Minion,
    cost: 5,
    attack: 8,
    health: 8,
    durability: 0,
    battlecry: Some(CardEffect::GainStats {
        attack: 1,
        health: 1,
        target: EffectTarget::AllFriendlyMinions,
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

/// TLC_239t The Everbloom — 3-cost weapon, 2 attack, 5 durability. FULL:
/// "After your hero attacks, give your minions +2/+2" — the trigger rides
/// the weapon entity (the Truesilver/Gorehowl pattern in
/// `apply_card_keywords`; `trigger_applies` pins `Attacked` to the
/// attacking hero's equipped weapon).
pub const THE_EVERBLOOM: CardDef = CardDef {
    id: "TLC_239t",
    name: "The Everbloom",
    card_type: CardType::Weapon,
    cost: 3,
    attack: 2,
    health: 0,
    durability: 5,
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

/// TLC_433t Tyrax, Bone Terror — 5/8/8 Undead. Deathrattle (simplified:
/// the Terror's Grave location chain collapses to a direct resummon —
/// §15): summon a copy of Tyrax.
pub const TYRAX_BONE_TERROR: CardDef = CardDef {
    id: "TLC_433t",
    name: "Tyrax, Bone Terror",
    card_type: CardType::Minion,
    cost: 5,
    attack: 8,
    health: 8,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonCopyOfSelf),
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

/// TLC_446t1 Underfel Rift — 5-cost minion, 0 attack, 1 health.
/// (simplified: the "throw a card in" activation is not modeled — §15.)
pub const UNDERFEL_RIFT: CardDef = CardDef {
    id: "TLC_446t1",
    name: "Underfel Rift",
    card_type: CardType::Minion,
    cost: 5,
    attack: 0,
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

/// TLC_460t The Origin Stone — 3-cost weapon, 0 attack, 8 durability.
/// (simplified: the "after you Discover, play the other options" trigger
/// needs discover-option storage the engine lacks — §15.)
pub const THE_ORIGIN_STONE: CardDef = CardDef {
    id: "TLC_460t",
    name: "The Origin Stone",
    card_type: CardType::Weapon,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 8,
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

/// TLC_513t Master Dusk — hero card, cost 3, 30 health. Written for data
/// completeness only: the reward's hero replacement is simplified away
/// (§15), so this hero is never equipped — the reward summons the two
/// TLC_513t2 Ninjas directly instead.
pub const MASTER_DUSK: CardDef = CardDef {
    id: "TLC_513t",
    name: "Master Dusk",
    card_type: CardType::Hero,
    cost: 3,
    attack: 0,
    health: 30,
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

/// TLC_513t2 Tortollan Ninja — 3/3 Stealth. The real token summoned by
/// TLC_513's simplified reward (two of them).
pub const TORTOLLAN_NINJA: CardDef = CardDef {
    id: "TLC_513t2",
    name: "Tortollan Ninja",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
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

/// TLC_602t Latorvius, Gaze of the City — 5/8/8. (simplified: the
/// "2 random Quest Rewards" battlecry waits for the Quest-Rewards pool,
/// which lands in W4 — §15.)
pub const LATORVIUS_GAZE_OF_THE_CITY: CardDef = CardDef {
    id: "TLC_602t",
    name: "Latorvius, Gaze of the City",
    card_type: CardType::Minion,
    cost: 5,
    attack: 8,
    health: 8,
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

/// TLC_631t Gorishi Colossus — 5/8/8 Beast. Battlecry FULL: sets the
/// owner's permanent "whenever you deal exactly 2 damage to an enemy, deal
/// 2 more" flag (`CardEffect::SetDealExact2Bonus` — the damage hook
/// reuses the DealExactDamage quest call site).
pub const GORISHI_COLOSSUS: CardDef = CardDef {
    id: "TLC_631t",
    name: "Gorishi Colossus",
    card_type: CardType::Minion,
    cost: 5,
    attack: 8,
    health: 8,
    durability: 0,
    battlecry: Some(CardEffect::SetDealExact2Bonus),
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

/// TLC_817t3 Sol'etos, Life's Breath — 5/4/4 Elemental. Taunt. Battlecry
/// FULL: summon a copy of this (`CardEffect::SummonCopyOfSelf` — the copy's
/// own battlecry is stripped, so the summon does not recurse).
pub const SOLETOS_LIFES_BREATH: CardDef = CardDef {
    id: "TLC_817t3",
    name: "Sol'etos, Life's Breath",
    card_type: CardType::Minion,
    cost: 5,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::SummonCopyOfSelf),
    deathrattle: None,
    taunt: true,
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

/// TLC_817t4 Sol'etos, Death's Touch — 5/4/4 Elemental. Reborn (applied by
/// `apply_card_keywords`). Deathrattle FULL: deal 5 damage to a random
/// enemy.
pub const SOLETOS_DEATHS_TOUCH: CardDef = CardDef {
    id: "TLC_817t4",
    name: "Sol'etos, Death's Touch",
    card_type: CardType::Minion,
    cost: 5,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DealDamage {
        amount: 5,
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

/// TLC_830t Shokk, Jungle Tyrant — 5/9/9 Beast. Rush (applied by
/// `apply_card_keywords`). (simplified: the "Discover any 8, 6, and
/// 4-Attack Beast" battlecry is not modeled — §15.)
pub const SHOKK_JUNGLE_TYRANT: CardDef = CardDef {
    id: "TLC_830t",
    name: "Shokk, Jungle Tyrant",
    card_type: CardType::Minion,
    cost: 5,
    attack: 9,
    health: 9,
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
