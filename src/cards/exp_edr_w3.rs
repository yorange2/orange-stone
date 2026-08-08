//! 2025–2026 expansions M1-W3 cards (exp_edr_w3) — the Emerald Dream (EDR)
//! choose-one wave: 12 choose-one cards + 6 tokens.
//!
//! The choose-one mechanic (see `engine/rules.rs`): playing a card with a
//! `choose_one_effect` surfaces a `ChoiceKind::ChooseOne` pending choice
//! (P3 — real branch resolution, exposed through `legal_actions`); option 0
//! resolves the `battlecry` slot (branch 1), option 1 the `choose_one_effect`
//! slot (branch 2). Discover cards in this wave (Symbiosis, Spark of Life)
//! stay simplified to a random pick (the registered Discover simplification,
//! fidelity-debt §14); Reforestation's 蓄力-style "hold for 3 turns" is
//! omitted and Wyvern's Slumber's Dormant is approximated with a plain
//! can't-attack token — see §14.2.
//!
//! These consts are handwritten effect-wave implementations of the generated
//! expansion baselines: they never enter `ALL_CARDS` (the sampling pools
//! stay closed, decision D3) but are reachable through the `card_by_id`
//! chain (ALL_CARDS → HANDWRITTEN_EXPANSION_CARDS → EXPANSION_CARDS) and are
//! compared field-by-field against the generated baseline by the
//! `expansion_differential_gate` test (M0.4). Simplifications registered in
//! `docs/finished/fidelity-debt.md` §14.2 (2025–2026 expansions).
//!
//! Card data: `cards/cards.json` (EDR_* ids); card texts verified against
//! the official Emerald Dream set (2026-08-08).

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::EffectTarget;

/// EDR_233 Spirits of the Forest — 5-mana Shaman spell. Choose One: summon
/// three 2/3 Wolves with Taunt, or two 4/3 Falcons with Windfury.
pub const SPIRITS_OF_THE_FOREST: CardDef = CardDef {
    id: "EDR_233",
    name: "Spirits of the Forest",
    card_type: CardType::Spell,
    cost: 5,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::SummonMultipleMinions {
        card_id: "EDR_233t1",
        count: 3,
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
    choose_one_effect: Some(CardEffect::SummonMultipleMinions {
        card_id: "EDR_233t2",
        count: 2,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_257 Lightmender — 4-mana 3/3 Paladin minion. Taunt. Choose One:
/// +3 Attack and Divine Shield, or +3 Health and Lifesteal.
pub const LIGHTMENDER: CardDef = CardDef {
    id: "EDR_257",
    name: "Lightmender",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsAndGrantDivineShield {
        attack: 3,
        health: 0,
        target: EffectTarget::Self_,
    }),
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
    choose_one_effect: Some(CardEffect::GainStatsAndGrantLifesteal {
        attack: 0,
        health: 3,
        target: EffectTarget::Self_,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_263 Grace of the Greatwolf — 4-mana Hunter spell. Choose One: deal 4
/// damage to the enemy hero, or summon two 3/2 Wolves with Rush.
pub const GRACE_OF_THE_GREATWOLF: CardDef = CardDef {
    id: "EDR_263",
    name: "Grace of the Greatwolf",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DealDamage {
        amount: 4,
        target: EffectTarget::EnemyHero,
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
    choose_one_effect: Some(CardEffect::SummonMultipleMinions {
        card_id: "EDR_263t",
        count: 2,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_273 Symbiosis — 1-mana Druid spell. Discover a Choose One card from
/// another class. (simplified: Discover is a random pick over the fixed
/// `OTHER_CLASS_CHOOSE_ONE_POOL` table of the non-Druid EDR choose-one cards
/// — fidelity-debt §14.2; the pool formula from the design brief yields an
/// empty set because every in-window choose-one card is Druid.)
pub const SYMBIOSIS: CardDef = CardDef {
    id: "EDR_273",
    name: "Symbiosis",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::AddRandomOtherClassChooseOneCard),
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

/// EDR_463 Twilight Influence — 2-mana Priest spell. Choose One: destroy a
/// minion with 3 or less Attack (either side), or summon a random 2-Cost
/// minion.
pub const TWILIGHT_INFLUENCE: CardDef = CardDef {
    id: "EDR_463",
    name: "Twilight Influence",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DestroyMinion {
        target: EffectTarget::AnyMinionAttackLE(3),
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
    choose_one_effect: Some(CardEffect::SummonRandomMinionOfCost { cost: 2 }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_490 Sleep Paralysis — 5-mana Warlock spell. Choose One: summon two
/// 3/6 Demons with Taunt that can't attack, or destroy an enemy minion.
pub const SLEEP_PARALYSIS: CardDef = CardDef {
    id: "EDR_490",
    name: "Sleep Paralysis",
    card_type: CardType::Spell,
    cost: 5,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::SummonMultipleMinions {
        card_id: "EDR_490t",
        count: 2,
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
    choose_one_effect: Some(CardEffect::DestroyMinion {
        target: EffectTarget::AnyEnemyMinion,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_525 Barbed Thorn — 3-mana 1/3 Rogue weapon. Choose One: gain
/// Poisonous this turn, or gain "Deathrattle: deal 2 damage to all enemies"
/// (the weapon deathrattle fires when the weapon breaks or is replaced —
/// `engine/rules.rs` WeaponDestroyed path).
pub const BARBED_THORN: CardDef = CardDef {
    id: "EDR_525",
    name: "Barbed Thorn",
    card_type: CardType::Weapon,
    cost: 3,
    attack: 1,
    health: 0,
    durability: 3,
    battlecry: Some(CardEffect::GrantPoisonousThisTurn),
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
    choose_one_effect: Some(CardEffect::GrantWeaponDeathrattleAllEnemies { damage: 2 }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_570 Ominous Nightmares — 1-mana Warrior spell. Choose One: deal 1
/// damage to all minions, or give a damaged minion +2/+2.
pub const OMINOUS_NIGHTMARES: CardDef = CardDef {
    id: "EDR_570",
    name: "Ominous Nightmares",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DamageAllMinions { damage: 1 }),
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
    choose_one_effect: Some(CardEffect::GainStats {
        attack: 2,
        health: 2,
        target: EffectTarget::DamagedMinion,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_813 Morbid Swarm — 1-mana Death Knight spell. Choose One: summon two
/// 1/1 Ants, or spend 2 Corpses to deal 4 damage to a minion (without the
/// corpses the branch is a no-op, matching the spend-corpses precedent).
pub const MORBID_SWARM: CardDef = CardDef {
    id: "EDR_813",
    name: "Morbid Swarm",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::SummonMultipleMinions {
        card_id: "EDR_813t",
        count: 2,
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
    choose_one_effect: Some(CardEffect::SpendCorpsesDamageMinion { cost: 2, damage: 4 }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_820 Wyvern's Slumber — 3-mana Demon Hunter spell. Choose One: summon
/// two Dormant Dreadseeds, or deal 2 damage to all minions. (simplified: the
/// engine has no Dormant; the Dreadseeds are plain 0/3 can't-attack tokens —
/// fidelity-debt §14.2.)
pub const WYVERNS_SLUMBER: CardDef = CardDef {
    id: "EDR_820",
    name: "Wyvern's Slumber",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::SummonMultipleMinions {
        card_id: "EDR_820t",
        count: 2,
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
    choose_one_effect: Some(CardEffect::DamageAllMinions { damage: 2 }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_843 Reforestation — 2-mana Druid spell. Choose One: draw a spell, or
/// draw a minion. (simplified: the real card's "Hold this for 3 turns to do
/// both" 蓄力-style mechanic is omitted — fidelity-debt §14.2.)
pub const REFORESTATION: CardDef = CardDef {
    id: "EDR_843",
    name: "Reforestation",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::DrawCardByType {
        count: 1,
        card_type: CardType::Spell,
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
    choose_one_effect: Some(CardEffect::DrawCardByType {
        count: 1,
        card_type: CardType::Minion,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_872 Spark of Life — 1-mana Mage spell. Choose One: Discover a Mage
/// spell, or Discover a Druid spell. (simplified: Discover is a random pick
/// from the in-window class-spell pools — fidelity-debt §14.2.)
pub const SPARK_OF_LIFE: CardDef = CardDef {
    id: "EDR_872",
    name: "Spark of Life",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::AddRandomMageSpells { count: 1 }),
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
    choose_one_effect: Some(CardEffect::AddRandomDruidSpell),
    combo_effect: None,
    attack_equals_health: false,
};

// ============================================================
// Tokens (handwritten only — not in the generated expansion baselines;
// the official set has no data rows for these token ids)
// ============================================================

/// EDR_233t1 Forest Wolf — Spirits of the Forest's 2/3 Taunt token.
pub const FOREST_WOLF: CardDef = CardDef {
    id: "EDR_233t1",
    name: "Forest Wolf",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: None,
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

/// EDR_233t2 Falcon — Spirits of the Forest's 4/3 Windfury token.
pub const FALCON: CardDef = CardDef {
    id: "EDR_233t2",
    name: "Falcon",
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
    race: Some(Race::Beast),
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

/// EDR_263t Greatwolf — Grace of the Greatwolf's 3/2 Rush token. (Rush is
/// applied by `apply_card_keywords`.)
pub const GREATWOLF: CardDef = CardDef {
    id: "EDR_263t",
    name: "Greatwolf",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
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

/// EDR_490t Nightmare Demon — Sleep Paralysis's 3/6 Taunt Demon token that
/// can't attack.
pub const NIGHTMARE_DEMON: CardDef = CardDef {
    id: "EDR_490t",
    name: "Nightmare Demon",
    card_type: CardType::Minion,
    cost: 6,
    attack: 3,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
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
    cant_attack: true,
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

/// EDR_813t Ant — Morbid Swarm's 1/1 token.
pub const ANT: CardDef = CardDef {
    id: "EDR_813t",
    name: "Ant",
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

/// EDR_820t Dreadseed — Wyvern's Slumber's Dormant Dreadseed token.
/// (simplified: the engine has no Dormant; the token is a plain 0/3
/// can't-attack minion that never wakes — fidelity-debt §14.2.)
pub const DREADSEED: CardDef = CardDef {
    id: "EDR_820t",
    name: "Dreadseed",
    card_type: CardType::Minion,
    cost: 0,
    attack: 0,
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
    cant_attack: true,
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
