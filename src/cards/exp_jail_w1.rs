//! M5-W1 cards (exp_jail_w1) — Escape from Violet Hold sub-roadmap W1,
//! the rule-override framework V1 + the first Rulebreakers batch
//! (2025–2026 expansions master roadmap M5-W1, PR #161).
//!
//! Implementation decisions (verified against `cards/cards.json` and the
//! W0 generated baselines 2026-08-09; the fidelity rows are
//! fidelity-debt.md §27, en + zh):
//! - **The Start-of-Game cards** (Hogger JAIL_384, Azalina JAIL_430,
//!   Godfrey JAIL_509, Neth'rek JAIL_860, Mug'Zee JAIL_800, Beatrix
//!   JAIL_397) carry NO def-level effect — the setup phase resolves them
//!   through the `cards::start_of_game` registry (the V1 hook in
//!   `GameBuilder::build`). Aya's "You always go second" is the builder
//!   flip (`aya_flip`); her counterfeit pick is a BATTLECRY, the
//!   choose-one three-branch minion (battlecry = Jade Coin, choose-one =
//!   Grimy Coin, third branch = Kabal Coin).
//! - **The Prepare cards** (Vanessa JAIL_407, Tras'tath JAIL_721, Moragg
//!   JAIL_906) are keyword-only — `cards::prepare::is_prepare_card`
//!   gates `Action::Prepare`; Vanessa's after-play trigger and Tras'tath's
//!   Demon-summon trigger ride `apply_card_keywords` attachments.
//! - **Rush** rides `apply_card_keywords` (Tras'tath, Moragg), **Charge**
//!   the `CardDef` field (Warptooth, The Living Plague).
//! - The Living Plague's hero-damage redirection and Warptooth's
//!   four-damage summon live in the damage/attack pipeline (rules.rs).

use crate::cards::def::CardDef;
use crate::core::component::{CardType, Race};
use crate::core::effect::{CardEffect, EffectTarget};

// ---------------------------------------------------------------------
// The Start-of-Game block
// ---------------------------------------------------------------------

/// JAIL_384 Chainbreaker Hogger — 8-cost 10/10 Taunt. "Start of Game:
/// Duplicate all other Legendary cards in your deck."
pub const CHAINBREAKER_HOGGER: CardDef = CardDef {
    id: "JAIL_384",
    name: "Chainbreaker Hogger",
    card_type: CardType::Minion,
    cost: 8,
    attack: 10,
    health: 10,
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

/// JAIL_430 Azalina Soulsever — 7-cost 7/7 Undead. "Your starting Health
/// is 40. Your deck is 20 cards, plus 20 copied from your enemy.
/// Battlecry: Draw until your hand is full." — the Start-of-Game half
/// resolves in the setup phase; the battlecry draws through the F-A11
/// cap.
pub const AZALINA_SOULSEVER: CardDef = CardDef {
    id: "JAIL_430",
    name: "Azalina Soulsever",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::DrawUntilHandFull),
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

/// JAIL_509 Godfrey the Betrayer — 4-cost 5/4 Undead. "Start of Game:
/// Overdrawn cards return to your hand when you have space. They cost
/// (1) less." — the F-A11 burn override.
pub const GODFREY_THE_BETRAYER: CardDef = CardDef {
    id: "JAIL_509",
    name: "Godfrey the Betrayer",
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

/// JAIL_860 Chef Neth'rek — 3-cost 3/3. "Start of Game: If your deck
/// only has cards that cost (3) or less, set your Mana to 10 after five
/// turns!"
pub const CHEF_NETHREK: CardDef = CardDef {
    id: "JAIL_860",
    name: "Chef Neth'rek",
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

/// JAIL_800 Mug'Zee — 6-cost 6/7. "Start of Game: If your deck has no
/// other minions, get Mug's Hero Power. If it has no spells, get
/// Zee's!" — the passive hero powers live in the cost pipeline (Mug's
/// Magic) and the CardPlayed counter (Zee's Might).
pub const MUGZEE: CardDef = CardDef {
    id: "JAIL_800",
    name: "Mug'Zee",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
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

/// JAIL_397 Commander Beatrix — 5-cost 5/6 Taunt. "While building your
/// deck, pick a 2-Cost minion. Ten copies join your deck!" — simplified
/// to a Start-of-Game random 2-Cost minion ×10 (§27).
pub const COMMANDER_BEATRIX: CardDef = CardDef {
    id: "JAIL_397",
    name: "Commander Beatrix",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 6,
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

// ---------------------------------------------------------------------
// The Prepare block
// ---------------------------------------------------------------------

/// JAIL_407 Vanessa the Ringleader — 7-cost 4/3. "Prepare. After you
/// play a card, get a random Battlecry minion. It costs (2) less." —
/// the after-play trigger rides an apply_card_keywords attachment.
pub const VANESSA_THE_RINGLEADER: CardDef = CardDef {
    id: "JAIL_407",
    name: "Vanessa the Ringleader",
    card_type: CardType::Minion,
    cost: 7,
    attack: 4,
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

/// JAIL_721 Tras'tath, Soul Parasite — 6-cost 3/3. "Prepare, Rush. After
/// you summon a Demon, gain its stats." — the summon trigger rides an
/// apply_card_keywords attachment (Demon race filter); Rush rides the
/// keyword list.
pub const TRASTATHS_SOUL_PARASITE: CardDef = CardDef {
    id: "JAIL_721",
    name: "Tras'tath, Soul Parasite",
    card_type: CardType::Minion,
    cost: 6,
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

/// JAIL_906 Moragg — 9-cost 6/5 Demon. "Prepare. Deathrattle: Summon a
/// random Demon from your deck. Give it 'Deathrattle: Summon Moragg.'"
pub const MORAGG: CardDef = CardDef {
    id: "JAIL_906",
    name: "Moragg",
    card_type: CardType::Minion,
    cost: 9,
    attack: 6,
    health: 5,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::MoraggDeathrattle),
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

// ---------------------------------------------------------------------
// Aya, Lotus Kingpin + the upgraded counterfeit tokens
// ---------------------------------------------------------------------

/// JAIL_504 Aya, Lotus Kingpin — 6-cost 6/3. "You always go second.
/// Battlecry: Pick an upgraded counterfeit to replace your Coins this
/// game. Get two." — the flip is the builder-level `aya_flip`; the
/// battlecry is the choose-one three-branch pick (battlecry slot = Jade
/// Coin, choose-one slot = Grimy Coin, the cards-side table = Kabal
/// Coin).
pub const AYA_LOTUS_KINGPIN: CardDef = CardDef {
    id: "JAIL_504",
    name: "Aya, Lotus Kingpin",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::AyaUpgradeCoins {
        card_id: "JAIL_504t",
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
    choose_one_effect: Some(CardEffect::AyaUpgradeCoins {
        card_id: "JAIL_504t2",
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// JAIL_504t Jade Coin — 0-cost (SPELL). "Gain 1 Mana Crystal this turn
/// only. Summon a 1/1 Jade Golem." (the Golem's scaling is simplified
/// to a fixed 1/1, §27.)
pub const JADE_COIN: CardDef = CardDef {
    id: "JAIL_504t",
    name: "Jade Coin",
    card_type: CardType::Spell,
    cost: 0,
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
    spell_effect: Some(CardEffect::JadeCoin),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// JAIL_504t2 Grimy Coin — 0-cost (SPELL). "Gain 1 Mana Crystal this
/// turn only. Deal 2 damage to a random enemy minion."
pub const GRIMY_COIN: CardDef = CardDef {
    id: "JAIL_504t2",
    name: "Grimy Coin",
    card_type: CardType::Spell,
    cost: 0,
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
    spell_effect: Some(CardEffect::GrimyCoin),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// JAIL_504t3 Kabal Coin — 0-cost (SPELL). "Gain 1 Mana Crystal this
/// turn only. Get a random 1-Cost Kazakus Potion." — the potion pool is
/// simplified to the 1-Cost spell pool (§27).
pub const KABAL_COIN: CardDef = CardDef {
    id: "JAIL_504t3",
    name: "Kabal Coin",
    card_type: CardType::Spell,
    cost: 0,
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
    spell_effect: Some(CardEffect::KabalCoin),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// JAIL_504tj Jade Golem — 1-cost 1/1. The fixed-size Jade Golem the
/// Jade Coin summons (the scaling simplified, §27).
pub const JADE_GOLEM: CardDef = CardDef {
    id: "JAIL_504tj",
    name: "Jade Golem",
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

// ---------------------------------------------------------------------
// The plain-effect batch
// ---------------------------------------------------------------------

/// JAIL_118 V'ama, Looming Death — 9-cost 5/5. "Battlecry: Destroy all
/// non-Paladin minions." — the class filter is dropped (the Sinestra §23
/// precedent), so it destroys ALL minions (§27).
pub const VAMA_LOOMING_DEATH: CardDef = CardDef {
    id: "JAIL_118",
    name: "V'ama, Looming Death",
    card_type: CardType::Minion,
    cost: 9,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::DestroyMinion {
        target: EffectTarget::AllMinions,
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

/// JAIL_122 Jailhouse Manastorm — 5-cost 5/5. "Battlecry: After you cast
/// a spell this game, summon a random minion of the same Cost." —
/// simplified to while-alive (§27): the SpellCast hook checks the board.
pub const JAILHOUSE_MANASTORM: CardDef = CardDef {
    id: "JAIL_122",
    name: "Jailhouse Manastorm",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::ManastormSetAfterSpell),
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

/// JAIL_319 The Skeleton Key — 1-cost (SPELL). "Discover a spell, or
/// refresh your options (20% chance to take 5 damage each refresh!)." —
/// the refresh half is simplified away (§27): a plain discover-a-spell.
pub const THE_SKELETON_KEY: CardDef = CardDef {
    id: "JAIL_319",
    name: "The Skeleton Key",
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
    spell_effect: Some(CardEffect::DiscoverAnySpell),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// JAIL_421 Warptooth — 4-cost 3/3 Beast, Charge. "If four friendly
/// characters take damage on one of your turns, summon this from hand
/// or deck." — the damage-pipeline hook (rules.rs).
pub const WARPTOOTH: CardDef = CardDef {
    id: "JAIL_421",
    name: "Warptooth",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
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
    windfury: false,
    charge: true,
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

/// JAIL_443 The Living Plague — 6-cost 8/8 Undead, Charge. "Instead of
/// damaging heroes, this shuffles that many Blights into their deck
/// that deal 2 when drawn." — the hero-damage redirection in the
/// ResolveAttack handler; the Blight (JAIL_443t) is the cast-when-drawn
/// simplification (§27).
pub const THE_LIVING_PLAGUE: CardDef = CardDef {
    id: "JAIL_443",
    name: "The Living Plague",
    card_type: CardType::Minion,
    cost: 6,
    attack: 8,
    health: 8,
    durability: 0,
    battlecry: None,
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
    charge: true,
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

/// JAIL_443t Blight — 1-cost (SPELL). "Deal 2 damage to your own hero."
/// The cast-when-drawn simplification of the Blight the Living Plague
/// shuffles (§27): the deal-2-on-draw fires as a playable spell.
pub const BLIGHT: CardDef = CardDef {
    id: "JAIL_443t",
    name: "Blight",
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
    spell_effect: Some(CardEffect::DamageSelfHero { damage: 2 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// JAIL_446 Blood Doctor Thal'ena — 4-cost 4/5 Undead. "Battlecry: Get
/// a second Hero Power that costs Corpses." — simplified (§27): the
/// second hero power is a swap — the hero power becomes Vampyr's Kiss
/// (3 mana, "Give a minion +3 Attack"), costing 3 Corpses instead of
/// Mana.
pub const BLOOD_DOCTOR_THALENA: CardDef = CardDef {
    id: "JAIL_446",
    name: "Blood Doctor Thal'ena",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::ThalenaSecondHeroPower),
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

/// JAIL_448 Karov the Broken — 6-cost 6/6 Taunt. "Deathrattle: Get
/// three 1/1 copies of random Legendary minions. They cost (1)."
pub const KAROV_THE_BROKEN: CardDef = CardDef {
    id: "JAIL_448",
    name: "Karov the Broken",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::KarovThreeLegendaryCopies),
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
