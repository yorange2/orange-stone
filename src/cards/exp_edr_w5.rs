//! 2025–2026 expansions M1-W5 cards (exp_edr_w5) — the Embers of the World
//! Tree miniset (FIR_*, 38 cards, released 2025-05-13 as the Into the
//! Emerald Dream mini-set). The final wave of the Emerald Dream roadmap.
//!
//! Mechanics introduced by this wave:
//! - no new engine primitives — everything reuses the W1–W4 facilities:
//!   Imbue counts (Petal Picker's "Imbued twice"), Dark Gifts (Cremate,
//!   Smoke Bomb, Shadowflame Stalker, Shadowflame Suffusion — the
//!   Discover→random simplification — and the three holding-a-dark-gift
//!   conditions: Frostburn Matriarch, Cindersword, Dragon Turtle), corpses
//!   (Volcoross's spend-10/20/30), hero-power-used-this-turn checks (Spirit
//!   of the Kaldorei, Charred Chameleon), spell triggers (Felfire Blaze,
//!   Inferno Herald), and the attack trigger (Magma Hound);
//! - the Amirdrassil Location (the EDR_454 convention: CardType::Location,
//!   health 0, durability from the official data, effect in the battlecry
//!   slot);
//! - the "next turn only" Mana Crystal (Emberscarred Whelp) — a per-player
//!   flag spent at the ManaRefill step (the crystal_gain_pending
//!   precedent);
//! - the hero-damage hook (Emberroot Destroyer) — a damage-pipeline hook in
//!   rules.rs where the hero's health is actually reduced (no HeroDamaged
//!   trigger event exists).
//!
//! D2 simplifications registered in `docs/finished/fidelity-debt.md` §14.5:
//! every Discover is simplified to random (the W1–W4 convention); the
//! spell-school filters are unmodeled (Felfire Blaze's Fel, Inferno
//! Herald's Fire, Overheat's Nature, Scorching Winds' Fire, Scorchreaver's
//! Fel, Living Flame's Fire — the engine has no spell-school field); Sigil
//! of Cinder's "start of your next turn" fires immediately; the Smoldering
//! cards' "Upgrades each turn, but discards after N" play their first-turn
//! version only; Keeper of Flame's "destroyed in 3 turns" clause is
//! unmodeled (buff only); Light of the New Moon's return-to-hand counter is
//! approximated by the player's spell total and returns a fresh copy (the
//! Full Moon upgrade step unmodeled); Fyrakk's Immune-to-Fire and
//! "15 Mana of Fire spells" → 15 random 1-damage pings; Volcoross's
//! three-way choose-one → the largest affordable corpse option; Emberscarred
//! Whelp's "next turn only" crystal granted at the owner's ManaRefill;
//! Everburning Phoenix's end-of-turn deathrattle fires immediately; Magma
//! Hound's "and survives" check reads effective health at trigger time;
//! Emberroot Destroyer fires on any hero-health loss on the owner's turn;
//! Bursting Shot's "three random enemies" allows repeated targets (the Mad
//! Bomber ping convention).
//!
//! These consts are handwritten effect-wave implementations of the generated
//! expansion baselines: they never enter `ALL_CARDS` (the sampling pools
//! stay closed, decision D3) but are reachable through the `card_by_id`
//! chain (ALL_CARDS → HANDWRITTEN_EXPANSION_CARDS → EXPANSION_CARDS) and are
//! compared field-by-field against the generated baseline by the
//! `expansion_differential_gate` test (M0.4). FIR_907 Amirdrassil diverges
//! from its baseline exactly like the EDR_454/EDR_520 Locations
//! (`expansion_differential_rebalanced`).
//!
//! Keyword hooks (Rush/Lifesteal/AttackedMinion trigger) land in
//! `mod.rs::apply_card_keywords`; the spell trigger and deathrattle fields
//! ride the CardDef itself.

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;

/// FIR_777 Spirit of the Kaldorei — 2/1/3 Priest minion. Taunt, Lifesteal.
/// Battlecry: If you used your Hero Power this turn, gain +3/+3.
pub const SPIRIT_OF_THE_KALDOREI: CardDef = CardDef {
    id: "FIR_777",
    name: "Spirit of the Kaldorei",
    card_type: CardType::Minion,
    cost: 2,
    attack: 1,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsIfHeroPowerUsed {
        attack: 3,
        health: 3,
    }),
    deathrattle: None,
    taunt: true,
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

/// FIR_778 Avatar of Destruction — 9/9/9 Shaman minion. Taunt. Deathrattle:
/// Deal 9 damage to all enemy minions.
pub const AVATAR_OF_DESTRUCTION: CardDef = CardDef {
    id: "FIR_778",
    name: "Avatar of Destruction",
    card_type: CardType::Minion,
    cost: 9,
    attack: 9,
    health: 9,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DealDamageToAllEnemyMinions { damage: 9 }),
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

/// FIR_900 Cremate — 3-mana Death Knight spell. Discover a minion with a
/// Dark Gift. It costs (2) less. (simplified: Discover → random, the W2
/// convention; the discovered minion gets a random Dark Gift and joins the
/// hand costing (2) less, §14.5.)
pub const CREMATE: CardDef = CardDef {
    id: "FIR_900",
    name: "Cremate",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverWithDarkGiftCostReduction { reduction: 2 }),
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

/// FIR_901 Frostburn Matriarch — 5/4/4 Death Knight minion. Battlecry: If
/// you're holding a minion with a Dark Gift, summon two 4/4 Dragons with
/// Taunt.
pub const FROSTBURN_MATRIARCH: CardDef = CardDef {
    id: "FIR_901",
    name: "Frostburn Matriarch",
    card_type: CardType::Minion,
    cost: 5,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::SummonBroodlingsIfHoldingGift),
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

/// FIR_902 Sigil of Cinder — 2-mana Demon Hunter spell. At the start of
/// your next turn, deal 6 damage randomly split among all enemies.
/// (simplified: the "start of your next turn" timing is dropped — the 6
/// damage resolves immediately as random 1-damage pings, §14.5.)
pub const SIGIL_OF_CINDER: CardDef = CardDef {
    id: "FIR_902",
    name: "Sigil of Cinder",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DealDamageSplitAmongAllEnemies { amount: 6 }),
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

/// FIR_904 Felfire Blaze — 2/2/3 Demon Hunter minion. After you cast a Fel
/// spell, destroy this and deal 2 damage to all enemies. (simplified: the
/// Fel-spell filter is unmodeled — any friendly spell triggers it, §14.5.)
pub const FELFIRE_BLAZE: CardDef = CardDef {
    id: "FIR_904",
    name: "Felfire Blaze",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
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
    spell_trigger: Some(CardEffect::FelfireBlazeTrigger { damage: 2 }),
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// FIR_906 Overheat — 3-mana Druid spell. Give your minions +1/+1. Discard
/// a random Nature spell to give them +1/+1 more. (simplified: the
/// Nature-spell filter is unmodeled — any hand spell can be discarded,
/// §14.5.)
pub const OVERHEAT: CardDef = CardDef {
    id: "FIR_906",
    name: "Overheat",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::BuffFriendlyMinionsDiscardBonus {
        attack: 1,
        health: 1,
        bonus_attack: 1,
        bonus_health: 1,
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

/// FIR_907 Amirdrassil — 5-mana Druid Location (3 durability). Summon a
/// 1-Cost minion. Gain 1 Armor. Draw 1 card. Refresh 1 Mana. (simplified:
/// the "Improves each use!" escalation is unmodeled — every activation
/// resolves the first-tier effect, §14.5. The Location convention is the
/// EDR_454 precedent: health 0, durability from the official data, effect
/// in the battlecry slot.)
pub const AMIRDRASSIL: CardDef = CardDef {
    id: "FIR_907",
    name: "Amirdrassil",
    card_type: CardType::Location,
    cost: 5,
    attack: 0,
    health: 0,
    durability: 3,
    battlecry: Some(CardEffect::AmirdrassilActivate),
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

/// FIR_908 Charred Chameleon — 1/1/2 Druid minion. Battlecry: If you've
/// used your Hero Power this turn, give a friendly minion +1/+2 and Rush.
pub const CHARRED_CHAMELEON: CardDef = CardDef {
    id: "FIR_908",
    name: "Charred Chameleon",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::GiveMinionStatsRushIfHeroPowerUsed {
        attack: 1,
        health: 2,
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

/// FIR_909 Bursting Shot — 2-mana Hunter spell. Deal 2 damage to three
/// random enemies. (simplified: the three pings may hit the same target —
/// the Mad Bomber convention, §14.5.)
pub const BURSTING_SHOT: CardDef = CardDef {
    id: "FIR_909",
    name: "Bursting Shot",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DealDamageRandomly {
        amount: 2,
        count: 3,
        target: crate::core::effect::EffectTarget::AnyEnemy,
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

/// FIR_910 Scorching Winds — 3-mana Mage spell. Deal 3 damage. Discard a
/// random Fire spell to deal 3 more. (simplified: the Fire-spell filter is
/// unmodeled — any hand spell can be discarded, §14.5.)
pub const SCORCHING_WINDS: CardDef = CardDef {
    id: "FIR_910",
    name: "Scorching Winds",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DamageAndDiscardSpellMore { base: 3, bonus: 3 }),
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

/// FIR_911 Smoldering Grove — 2-mana Mage spell. Draw 1 card. (simplified:
/// the "Upgrades each turn, but discards after N" cycle is unmodeled — the
/// spell always plays its first-turn version, §14.5.)
pub const SMOLDERING_GROVE: CardDef = CardDef {
    id: "FIR_911",
    name: "Smoldering Grove",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
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

/// FIR_913 Inferno Herald — 4/3/6 Mage minion. After you cast a Fire spell,
/// get a random Elemental and reduce its Cost by (3). (simplified: the
/// Fire-spell filter is unmodeled — any friendly spell triggers it, §14.5.)
pub const INFERNO_HERALD: CardDef = CardDef {
    id: "FIR_913",
    name: "Inferno Herald",
    card_type: CardType::Minion,
    cost: 4,
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
    spell_trigger: Some(CardEffect::InfernoHeraldTrigger { reduction: 3 }),
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// FIR_914 Smoldering Strength — 1-mana Paladin spell. Give a friendly
/// minion +1/+1. (simplified: the "Upgrades each turn, but discards after
/// N" cycle is unmodeled — the spell always plays its first-turn version,
/// §14.5.)
pub const SMOLDERING_STRENGTH: CardDef = CardDef {
    id: "FIR_914",
    name: "Smoldering Strength",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::GainStats {
        attack: 1,
        health: 1,
        target: crate::core::effect::EffectTarget::FriendlyMinion,
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

/// FIR_916 Smoldering Ascent — 2-mana Priest spell. Deal 1 damage to all
/// enemy minions. (simplified: the "Upgrades each turn, but discards after
/// N" cycle is unmodeled — the spell always plays its first-turn version,
/// §14.5.)
pub const SMOLDERING_ASCENT: CardDef = CardDef {
    id: "FIR_916",
    name: "Smoldering Ascent",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DealDamageToAllEnemyMinions { damage: 1 }),
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

/// FIR_918 Light of the New Moon — 3-mana Priest spell. Give a minion
/// +3/+3. (Cast 3 spells to return this to your hand when played.)
/// (simplified: the return counter is approximated by the player's spell
/// total — the W4a Wish of the New Moon pattern — and the returned card is
/// a fresh copy; the Light of the Full Moon upgrade step is unmodeled,
/// §14.5.)
pub const LIGHT_OF_THE_NEW_MOON: CardDef = CardDef {
    id: "FIR_918",
    name: "Light of the New Moon",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::BuffMinionReturnIfSpellsCast {
        attack: 3,
        health: 3,
        threshold: 3,
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

/// FIR_919 Everburning Phoenix — 4/3/2 Rogue minion. Costs (1) less for
/// each card you've played this turn (the cost.rs discount). Deathrattle:
/// At end of turn, get another Phoenix. (simplified: the end-of-turn
/// timing is dropped — the deathrattle adds a fresh Phoenix immediately,
/// §14.5.)
pub const EVERBURNING_PHOENIX: CardDef = CardDef {
    id: "FIR_919",
    name: "Everburning Phoenix",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::AddCardToHand { card_id: "FIR_919" }),
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

/// FIR_920 Smoke Bomb — 2-mana Rogue spell. Discover a Combo, Battlecry, or
/// Stealth minion with a Dark Gift. (simplified: Discover → random, the W2
/// convention — a random minion carrying Combo, Battlecry or Stealth gets a
/// random Dark Gift and joins the hand, §14.5.)
pub const SMOKE_BOMB: CardDef = CardDef {
    id: "FIR_920",
    name: "Smoke Bomb",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverComboBattlecryStealthWithDarkGift),
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

/// FIR_921 Petal Picker — 3/2/3 Neutral minion. Battlecry: If you've
/// Imbued your Hero Power twice, draw 2 cards.
pub const PETAL_PICKER: CardDef = CardDef {
    id: "FIR_921",
    name: "Petal Picker",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::DrawIfImbuedTwice { count: 2 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Draenei),
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

/// FIR_922 Cindersword — 1-mana Rogue weapon (1/2). Battlecry: If you're
/// holding a minion with a Dark Gift, gain +3 Attack.
pub const CINDERSWORD: CardDef = CardDef {
    id: "FIR_922",
    name: "Cindersword",
    card_type: CardType::Weapon,
    cost: 1,
    attack: 1,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::GainWeaponAttackIfHoldingGift { amount: 3 }),
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

/// FIR_923 Flames of the Firelord — 2-mana Shaman spell. Deal 4 damage to a
/// random enemy minion. If you're holding a card that costs (8) or more,
/// deal 8 instead.
pub const FLAMES_OF_THE_FIRELORD: CardDef = CardDef {
    id: "FIR_923",
    name: "Flames of the Firelord",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DamageRandomEnemyMinionHoldingCostGE {
        base: 4,
        upgraded: 8,
        threshold: 8,
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

/// FIR_924 Shadowflame Stalker — 4/4/3 Warlock minion. Battlecry: Discover
/// a Demon with a Dark Gift. Get a copy of it. (simplified: Discover →
/// random, the W2 convention — a random Demon gets a random Dark Gift and a
/// copy joins the hand, §14.5.)
pub const SHADOWFLAME_STALKER: CardDef = CardDef {
    id: "FIR_924",
    name: "Shadowflame Stalker",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverDemonWithDarkGiftCopy),
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

/// FIR_927 Emberscarred Whelp — 3/3/2 Shaman minion. Battlecry: Discover a
/// 5-Cost card. Gain 1 Mana Crystal next turn only. (simplified: Discover →
/// random, the W1–W4 convention; the "next turn only" crystal is granted at
/// the owner's ManaRefill, §14.5.)
pub const EMBERSCARRED_WHELP: CardDef = CardDef {
    id: "FIR_927",
    name: "Emberscarred Whelp",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverCostCardGainTempMana { cost: 5, mana: 1 }),
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

/// FIR_928 Keeper of Flame — 5/5/5 Warrior minion. Battlecry: Give all
/// minions in your hand +3/+3. They are destroyed in 3 turns. (simplified:
/// the "destroyed in 3 turns" clause is unmodeled — the buff only, §14.5.)
pub const KEEPER_OF_FLAME: CardDef = CardDef {
    id: "FIR_928",
    name: "Keeper of Flame",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::BuffAllHandMinions {
        attack: 3,
        health: 3,
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

/// FIR_929 Living Flame — 2/3/2 Neutral minion. Deathrattle: Draw a Fire
/// spell. (simplified: the Fire-spell filter is unmodeled — the deathrattle
/// draws any card, §14.5.)
pub const LIVING_FLAME: CardDef = CardDef {
    id: "FIR_929",
    name: "Living Flame",
    card_type: CardType::Minion,
    cost: 2,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DrawCard { count: 1 }),
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

/// FIR_939 Shadowflame Suffusion — 2-mana Warrior spell. Deal 2 damage.
/// Discover a Warrior minion with a Dark Gift. (simplified: Discover →
/// random, the W2 convention — a random Warrior minion gets a random Dark
/// Gift and joins the hand, §14.5.)
pub const SHADOWFLAME_SUFFUSION: CardDef = CardDef {
    id: "FIR_939",
    name: "Shadowflame Suffusion",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DamageAndDiscoverWarriorWithGift { damage: 2 }),
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

/// FIR_940 Zaqali Flamemancer — 6/4/4 Neutral minion. Battlecry: If every
/// card in your hand is of a different Cost, reduce their Costs by (2).
pub const ZAQALI_FLAMEMANCER: CardDef = CardDef {
    id: "FIR_940",
    name: "Zaqali Flamemancer",
    card_type: CardType::Minion,
    cost: 6,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::ReduceHandCostIfAllDistinct { reduction: 2 }),
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

/// FIR_941 Searing Reflection — 7-mana Paladin spell. Draw a minion. Summon
/// an 8/8 copy of it with Divine Shield.
pub const SEARING_REFLECTION: CardDef = CardDef {
    id: "FIR_941",
    name: "Searing Reflection",
    card_type: CardType::Spell,
    cost: 7,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DrawMinionSummonDivineShieldCopy),
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

/// FIR_951 Volcoross — 8/5/5 Death Knight minion. Rush, Taunt. Battlecry:
/// Choose to spend 10, 20, or 30 Corpses to gain that many stats.
/// (simplified: the three-way choose-one is resolved to the largest
/// affordable corpse option — spend N for +N/+N, §14.5.)
pub const VOLCOROSS: CardDef = CardDef {
    id: "FIR_951",
    name: "Volcoross",
    card_type: CardType::Minion,
    cost: 8,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::VolcorossBattlecry),
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

/// FIR_952 Scorchreaver — 4/4/4 Demon Hunter minion. Battlecry: Discover a
/// Fel spell. Reduce the Cost of Fel spells in your hand by (1).
/// (simplified: Discover → random and the Fel-spell filter unmodeled — any
/// spell qualifies, §14.5.)
pub const SCORCHREAVER: CardDef = CardDef {
    id: "FIR_952",
    name: "Scorchreaver",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverSpellReduceHandSpells { reduction: 1 }),
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

/// FIR_953 Magma Hound — 8/5/8 Hunter minion. Rush. After this attacks a
/// minion and survives, deal this minion's Attack damage split among all
/// enemies. (simplified: the survive check reads effective health at trigger
/// time — a hound already dead from the attack splashes nothing, §14.5. The
/// AttackedMinion trigger is registered in apply_card_keywords.)
pub const MAGMA_HOUND: CardDef = CardDef {
    id: "FIR_953",
    name: "Magma Hound",
    card_type: CardType::Minion,
    cost: 8,
    attack: 5,
    health: 8,
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

/// FIR_954 Conflagrate — 1-mana Warlock spell. Deal 5 damage to a minion.
/// Its owner draws a card.
pub const CONFLAGRATE: CardDef = CardDef {
    id: "FIR_954",
    name: "Conflagrate",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DamageMinionOwnerDraws { damage: 5 }),
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

/// FIR_955 Emberroot Destroyer — 3/3/3 Warlock minion. Whenever your hero
/// takes damage on your turn, deal 3 damage to a random enemy minion.
/// (simplified: no HeroDamaged trigger event exists — the damage-pipeline
/// hook in rules.rs fires where the hero's health is actually reduced, on
/// the owner's turn only, §14.5.)
pub const EMBERROOT_DESTROYER: CardDef = CardDef {
    id: "FIR_955",
    name: "Emberroot Destroyer",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
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

/// FIR_956 Dragon Turtle — 4/3/6 Warrior minion. Battlecry: If you're
/// holding a minion with a Dark Gift, give your hero +3 Attack this turn
/// and 6 Armor.
pub const DRAGON_TURTLE: CardDef = CardDef {
    id: "FIR_956",
    name: "Dragon Turtle",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::GainHeroAttackArmorIfHoldingGift {
        attack: 3,
        armor: 6,
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

/// FIR_958 Tindral Sageswift — 4/4/3 Neutral minion. Deathrattle: Deal 1
/// damage to all enemies. If it's your opponent's turn, deal 4 damage
/// instead.
pub const TINDRAL_SAGESWIFT: CardDef = CardDef {
    id: "FIR_958",
    name: "Tindral Sageswift",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::DeathrattleDamageAllEnemiesTurnScaled {
        base: 1,
        boosted: 4,
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

/// FIR_959 Fyrakk the Blazing — 10/7/7 Neutral minion. Immune to Fire
/// spells. Battlecry: Cast 15 Mana worth of Fire spells at random enemies.
/// (simplified: the Immune-to-Fire clause is unmodeled and "15 Mana worth
/// of Fire spells" is approximated as 15 random 1-damage pings across the
/// enemy characters, §14.5.)
pub const FYRAKK_THE_BLAZING: CardDef = CardDef {
    id: "FIR_959",
    name: "Fyrakk the Blazing",
    card_type: CardType::Minion,
    cost: 10,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::DealDamageRandomly {
        amount: 1,
        count: 15,
        target: crate::core::effect::EffectTarget::AnyEnemy,
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

/// FIR_960 Tending Dragonkin — 5/5/4 Hunter minion. Battlecry: Copy the
/// lowest Cost Beast in your hand.
pub const TENDING_DRAGONKIN: CardDef = CardDef {
    id: "FIR_960",
    name: "Tending Dragonkin",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::CopyLowestCostBeastInHand),
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

/// FIR_961 Ashleaf Pixie — 2/3/2 Paladin minion. Battlecry: If you're
/// holding a spell that costs (5) or more, gain Divine Shield and Lifesteal.
pub const ASHLEAF_PIXIE: CardDef = CardDef {
    id: "FIR_961",
    name: "Ashleaf Pixie",
    card_type: CardType::Minion,
    cost: 2,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::GainDivineShieldLifestealIfHoldingSpellGE { cost: 5 }),
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

// ---------------------------------------------------------------------------
// M1-W5 tokens (handwritten only — no generated baselines)
// ---------------------------------------------------------------------------

/// FIR_901t Frostburn Broodling — the 4/4 Dragon with Taunt Frostburn
/// Matriarch summons (two of them while the owner holds a dark-gift
/// minion).
pub const FROSTBURN_BROODLING: CardDef = CardDef {
    id: "FIR_901t",
    name: "Frostburn Broodling",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
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
