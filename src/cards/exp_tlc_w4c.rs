//! 2025–2026 expansions M2-W4c cards (exp_tlc_w4c) — the Festival of the
//! Devilsaur miniset: the 38 DINO_* collectible cards (the Un'Goro
//! sub-roadmap's closing wave).
//!
//! Implementation decisions (per the M2-W4c spec, verified against
//! `/tmp/hs_full.json` 2026-08-09):
//! - **Full** implementations:
//!   - DINO_130 Longneck Egg — deathrattle summons a 3/3 Beast and gives
//!     the owner's minions +1/+1 (the buff lands even with a full board).
//!   - DINO_131 Possessed Animancer — deathrattle summons a random Beast
//!     from the owner's deck and gives it Lifesteal (a copy-summon — the
//!     deck is untouched, the Herenn §18 convention).
//!   - DINO_132 Asphyxiodon — Taunt; end-of-turn 5 damage to a random
//!     enemy minion.
//!   - DINO_136 Horn of Feasting — summons three 2/1 Raptors with Rush;
//!     Outcast gives them Immune for the rest of the turn (the official
//!     "Immune while attacking this turn" as the immune-until-end-of-turn
//!     shape — the Raptors can only attack this turn anyway).
//!   - DINO_137 Skittish Saucier — battlecry reduces the Cost of the
//!     adjacent hand cards by (1) (the play path records the card's hand
//!     position, `Player::last_played_hand_index`).
//!   - DINO_138 Diabolus Rex — Kindred (Demon): deal 6 damage to the
//!     opponent's left- and right-most minions.
//!   - DINO_400 Barricade Basher — "whenever you gain Armor, gain +2/+2
//!     and attack a random enemy minion": the armor-gain chokepoint
//!     (`grant_armor` in trigger.rs) fires the per-id board hook.
//!   - DINO_401 The Great Dracorex — Rush; after this attacks an enemy
//!     minion it damages ALL other enemy minions (the new
//!     `AttackedEnemyMinion` event carries the DEFENDER as the subject so
//!     the splash can exclude the attacked minion).
//!   - DINO_402 Bat Mask — set a friendly minion's stats to 1/1 and fill
//!     the board with copies of it (the copies are the set 1/1 — "copies
//!     of it" refers to the minion as set).
//!   - DINO_403 Devilsaur Mask / DINO_428 Behemoth Mask / DINO_429 Sheep
//!     Mask / DINO_432 Panther Mask — the set-stats mask family (the W4b
//!     set semantics: base stats written, damage and permanent
//!     enchantments cleared) plus Charge / Lifesteal + forced attack /
//!     attached deathrattle / Stealth + draw.
//!   - DINO_404 Firegill — Kindred (Elemental): give the owner's other
//!     minions Rush.
//!   - DINO_405 Hatching Ceremony — the +2/+2 to the owner's minions at
//!     the end of the owner's NEXT turn after the cast (a two-tick
//!     countdown on the player record).
//!   - DINO_406 Fire Breath — deal 4 damage (spell-power scaled) and give
//!     the owner's Elementals +1/+1.
//!   - DINO_408 Crystal Tusk — weapon: battlecry shuffles the left-most
//!     hand card into the deck at a random position; deathrattle draws 2.
//!   - DINO_411 Holy Eggbearer — battlecry draws a 0-Attack minion (a
//!     deck scan, the Sense Demons pattern; a deck with no 0-Attack
//!     minion draws nothing).
//!   - DINO_412 Tortotem — end of turn: get a random minion with multiple
//!     minion types (the fixed MULTI_TRIBE_MINION_POOL — the D2
//!     simplification, §19).
//!   - DINO_413 Chillspine Stegodon — battlecry deals 2 damage to two
//!     random enemy minions; Kindred (Elemental) and Freeze them (the
//!     `BattlecryModifier` replace pattern — active, the damage-and-freeze
//!     resolves as one effect on the SAME two minions).
//!   - DINO_416 Hollow Direhorn — Rush; after a friendly minion dies,
//!     spend 3 Corpses to gain Reborn (the SpendCorpses quest-progress
//!     pattern).
//!   - DINO_417 Soulrest Ceremony — give the owner's minions +1 Attack
//!     and Rush; they die at the end of the owner's turn (damaged to
//!     death through the normal death path so deathrattles fire).
//!   - DINO_419 Herbivore Assistant — battlecry: give a friendly Beast
//!     +2/+2 and Rush.
//!   - DINO_421 Seismopod — Taunt, Elusive; deathrattle gives all minions
//!     in the hand and deck +3/+3 (the wherever-buff machinery from W4b's
//!     Esho).
//!   - DINO_422 Ankylodon — Taunt; deathrattle summons two random 3-Cost
//!     Beasts that attack random enemies (the normal attack pipeline).
//!   - DINO_427 Costume Merchant — battlecry gets a random Mask (the
//!     fixed MASK_POOL — all five Masks are non-Rogue, so "from another
//!     class" is the full set); the Combo makes it cost (2) less.
//!   - DINO_434 Raptor-Nest Nurse — battlecry gets a random 1-Cost minion,
//!     deathrattle gets a random 1-Cost spell (the D2 pattern).
//!   - DINO_435 Crater Experiment — Kindred (Beast — the ALL tribe lands
//!     as Beast, §19): summon a copy of this.
//! - **Simplified** (fidelity-debt §19): DINO_407 Mirrex (no
//!   transform-while-in-hand — vanilla), DINO_409 Techysaurus (no "costs
//!   (1) less" discount — vanilla Taunt), DINO_410 The Egg of Khelos
//!   (the five-stage crack chain is skipped — the deathrattle summons the
//!   20/20 Khelos directly), DINO_414 Tribute Dance (the two "choose a
//!   minion" picks resolve as random picks), DINO_430 Beast Speaker Taka
//!   (the Discover resolves as an independent random Legendary Beast pick,
//!   and the deathrattle summons it as a second independent pick) — plus
//!   the D2 random Discover/random-summon simplifications: DINO_415
//!   (random Deathrattle minion >= (5), deathrattle triggered), DINO_424
//!   (random Legendary minion set to 10/10), DINO_426 (random 3-Cost
//!   minion as a 2/3 copy), DINO_431 (random Taunt minion >= (5)),
//!   DINO_433 (random 6-, 4-, and 2-Cost Taunt minions), DINO_412 (the
//!   fixed multi-tribe pool), DINO_427 (the fixed Mask pool).
//!
//! Card data: `/tmp/hs_full.json` (verified 2026-08-09). The wave's
//! collectible tokens — DINO_130t Little Longneck (3/3 Beast), DINO_136t
//! Ravenous Raptor (2/1 Beast with Rush) and DINO_410t Khelos (20/20
//! Beast with Taunt) — are handwritten here; the intermediate Egg stages
//! (DINO_410t2..t5) are skipped per the DINO_410 simplification.

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::EffectTarget;

// ============================================================
// Minions.
// ============================================================

/// DINO_130 Longneck Egg — 2-cost 0/2. Deathrattle: summon a 3/3 Beast
/// and give your minions +1/+1 (the buff lands even when the board is
/// full; the freshly summoned Beast is a friendly minion and receives the
/// buff too).
pub const LONGNECK_EGG: CardDef = CardDef {
    id: "DINO_130",
    name: "Longneck Egg",
    card_type: CardType::Minion,
    cost: 2,
    attack: 0,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonMinionAndBuffFriendlyMinions {
        card_id: "DINO_130t",
        attack: 1,
        health: 1,
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

/// DINO_131 Possessed Animancer — 5-cost 2/2. Deathrattle: summon a random
/// Beast from your deck and give it Lifesteal (a copy-summon — the deck is
/// untouched; a Beast-less deck summons nothing).
pub const POSSESSED_ANIMANCER: CardDef = CardDef {
    id: "DINO_131",
    name: "Possessed Animancer",
    card_type: CardType::Minion,
    cost: 5,
    attack: 2,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonRandomDeckBeastGiveLifesteal),
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

/// DINO_132 Asphyxiodon — 8-cost 6/12. Taunt. At the end of your turn,
/// deal 5 damage to a random enemy minion. The Demon tribe is the primary
/// race (the second tribe, Beast, lands via `apply_card_keywords`).
pub const ASPHYXIODON: CardDef = CardDef {
    id: "DINO_132",
    name: "Asphyxiodon",
    card_type: CardType::Minion,
    cost: 8,
    attack: 6,
    health: 12,
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
    cant_attack: false,
    end_turn_effect: Some(CardEffect::DealDamage {
        amount: 5,
        target: EffectTarget::AnyEnemyMinion,
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

/// DINO_137 Skittish Saucier — 3-cost 4/2. Battlecry: reduce the Cost of
/// the adjacent cards in your hand by (1) (the play path records the
/// card's hand position — `Player::last_played_hand_index` — so the
/// battlecry knows which hand cards are adjacent).
pub const SKITTISH_SAUCIER: CardDef = CardDef {
    id: "DINO_137",
    name: "Skittish Saucier",
    card_type: CardType::Minion,
    cost: 3,
    attack: 4,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::ReduceAdjacentHandCardCost { amount: 1 }),
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

/// DINO_138 Diabolus Rex — 6-cost 6/5. Kindred (Demon): deal 6 damage to
/// your opponent's left- and right-most minions. The Demon tribe is the
/// primary race (the second tribe, Beast, lands via
/// `apply_card_keywords`).
pub const DIABOLUS_REX: CardDef = CardDef {
    id: "DINO_138",
    name: "Diabolus Rex",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 5,
    durability: 0,
    battlecry: None,
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

/// DINO_400 Barricade Basher — 3-cost 4/3. Whenever you gain Armor, gain
/// +2/+2 and attack a random enemy minion (the armor-gain chokepoint
/// `grant_armor` fires the board hook keyed on this card id — the CardDef
/// itself is vanilla).
pub const BARRICADE_BASHER: CardDef = CardDef {
    id: "DINO_400",
    name: "Barricade Basher",
    card_type: CardType::Minion,
    cost: 3,
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

/// DINO_401 The Great Dracorex — 8-cost 5/12. Rush. After this attacks an
/// enemy minion, it damages ALL other enemy minions (the per-id
/// `AttackedEnemyMinion` trigger — the defender is the event subject so
/// the splash excludes the attacked minion; the splash amount is the
/// Dracorex's effective Attack).
pub const THE_GREAT_DRACOREX: CardDef = CardDef {
    id: "DINO_401",
    name: "The Great Dracorex",
    card_type: CardType::Minion,
    cost: 8,
    attack: 5,
    health: 12,
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

/// DINO_404 Firegill — 2-cost 3/2. Kindred (Elemental): give your other
/// minions Rush. The Elemental tribe is the primary race (the second
/// tribe, Murloc, lands via `apply_card_keywords`).
pub const FIREGILL: CardDef = CardDef {
    id: "DINO_404",
    name: "Firegill",
    card_type: CardType::Minion,
    cost: 2,
    attack: 3,
    health: 2,
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

/// DINO_407 Mirrex, the Crystalline — 3-cost 3/4 (registered
/// simplification §19: the official "while in your hand, this is a 3/4
/// copy of the last minion your opponent played" transform is unmodeled —
/// the CardDef is vanilla 3/4). The Elemental tribe is the primary race
/// (the second tribe, Beast, lands via `apply_card_keywords`).
pub const MIRREX_THE_CRYSTALLINE: CardDef = CardDef {
    id: "DINO_407",
    name: "Mirrex, the Crystalline",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 4,
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

/// DINO_409 Techysaurus — 7-cost 3/6. Taunt (registered simplification
/// §19: the official "costs (1) less for each card you played this game
/// that didn't start in your deck" discount is unmodeled). The Mechanical
/// tribe is the primary race (the second tribe, Beast, lands via
/// `apply_card_keywords`).
pub const TECHYSAURUS: CardDef = CardDef {
    id: "DINO_409",
    name: "Techysaurus",
    card_type: CardType::Minion,
    cost: 7,
    attack: 3,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: false,
    race: Some(Race::Mechanical),
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

/// DINO_410 The Egg of Khelos — 3-cost 0/3 (registered simplification
/// §19: the official five-stage crack chain — summoning successively more
/// cracked Eggs — is skipped; the deathrattle summons the 20/20 Khelos
/// directly).
pub const THE_EGG_OF_KHELOS: CardDef = CardDef {
    id: "DINO_410",
    name: "The Egg of Khelos",
    card_type: CardType::Minion,
    cost: 3,
    attack: 0,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonMinion {
        card_id: "DINO_410t",
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

/// DINO_411 Holy Eggbearer — 2-cost 1/2. Battlecry: draw a 0-Attack
/// minion (a deck scan — the first deck minion whose base Attack is 0
/// moves to the hand; a deck with no 0-Attack minion draws nothing).
pub const HOLY_EGGBEARER: CardDef = CardDef {
    id: "DINO_411",
    name: "Holy Eggbearer",
    card_type: CardType::Minion,
    cost: 2,
    attack: 1,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::DrawZeroAttackMinion),
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

/// DINO_412 Tortotem — 1-cost 0/3. At the end of your turn, get a random
/// minion with multiple minion types (the fixed MULTI_TRIBE_MINION_POOL —
/// the D2 simplification, §19).
pub const TORTOTEM: CardDef = CardDef {
    id: "DINO_412",
    name: "Tortotem",
    card_type: CardType::Minion,
    cost: 1,
    attack: 0,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Totem),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: Some(CardEffect::AddRandomMultiTribeMinion),
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_413 Chillspine Stegodon — 4-cost 3/4. Battlecry: deal 2 damage to
/// two random enemy minions; Kindred (Elemental): and Freeze them (the
/// `BattlecryModifier` replace pattern — the freeze lands on the SAME two
/// minions the damage hit). The Elemental tribe is the primary race (the
/// second tribe, Beast, lands via `apply_card_keywords`).
pub const CHILLSPINE_STEGODON: CardDef = CardDef {
    id: "DINO_413",
    name: "Chillspine Stegodon",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DealDamageToTwo { amount: 2 }),
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

/// DINO_416 Hollow Direhorn — 5-cost 5/4. Rush. After a friendly minion
/// dies, spend 3 Corpses to gain Reborn (the per-id `FriendlyMinionDied`
/// trigger; the spend counts toward the "Spend 15 Corpses" quest). The
/// Undead tribe is the primary race (the second tribe, Beast, lands via
/// `apply_card_keywords`).
pub const HOLLOW_DIREHORN: CardDef = CardDef {
    id: "DINO_416",
    name: "Hollow Direhorn",
    card_type: CardType::Minion,
    cost: 5,
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

/// DINO_419 Herbivore Assistant — 3-cost 3/2. Battlecry: give a friendly
/// Beast +2/+2 and Rush.
pub const HERBIVORE_ASSISTANT: CardDef = CardDef {
    id: "DINO_419",
    name: "Herbivore Assistant",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsAndGrantRush {
        attack: 2,
        health: 2,
        target: EffectTarget::FriendlyRace(Race::Beast),
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

/// DINO_421 Seismopod — 9-cost 9/9. Taunt, Elusive. Deathrattle: give all
/// minions in your hand and deck +3/+3 (the wherever-buff machinery from
/// W4b's Esho — base stat writes, no board minions, matching the card
/// text).
pub const SEISMOPOD: CardDef = CardDef {
    id: "DINO_421",
    name: "Seismopod",
    card_type: CardType::Minion,
    cost: 9,
    attack: 9,
    health: 9,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::BuffHandAndDeckMinions {
        attack: 3,
        health: 3,
    }),
    taunt: true,
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

/// DINO_422 Ankylodon — 6-cost 7/5. Taunt. Deathrattle: summon two random
/// 3-Cost Beasts; they attack random enemies (each summon is an
/// independent random pick; the attacks run through the normal attack
/// pipeline).
pub const ANKYLODON: CardDef = CardDef {
    id: "DINO_422",
    name: "Ankylodon",
    card_type: CardType::Minion,
    cost: 6,
    attack: 7,
    health: 5,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonTwoRandomCostBeastsAttackRandomEnemies { cost: 3 }),
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

/// DINO_427 Costume Merchant — 3-cost 2/4. Battlecry: get a random Mask
/// from another class (the fixed MASK_POOL — all five Masks are
/// non-Rogue, so the "from another class" filter leaves the full set).
/// Combo: it costs (2) less — the combo slot replaces the battlecry under
/// the standard combo rule.
pub const COSTUME_MERCHANT: CardDef = CardDef {
    id: "DINO_427",
    name: "Costume Merchant",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::AddRandomMaskCombo { reduction: 0 }),
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
    combo_effect: Some(CardEffect::AddRandomMaskCombo { reduction: 2 }),
    attack_equals_health: false,
};

/// DINO_430 Beast Speaker Taka — 7-cost 2/2 (registered simplification
/// §19: the official Discover resolves as two independent random Legendary
/// Beast picks — the battlecry gains the first pick's stats, the
/// deathrattle summons the second pick).
pub const BEAST_SPEAKER_TAKA: CardDef = CardDef {
    id: "DINO_430",
    name: "Beast Speaker Taka",
    card_type: CardType::Minion,
    cost: 7,
    attack: 2,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsOfRandomLegendaryBeast),
    deathrattle: Some(CardEffect::SummonRandomLegendaryBeast),
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

/// DINO_431 Atlasaurus — 8-cost 5/10. Taunt. Deathrattle: summon a random
/// Taunt minion that costs (5) or more (the D2 random Discover
/// simplification, §19).
pub const ATLASAURUS: CardDef = CardDef {
    id: "DINO_431",
    name: "Atlasaurus",
    card_type: CardType::Minion,
    cost: 8,
    attack: 5,
    health: 10,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonRandomTauntMinionCostGE { min_cost: 5 }),
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

/// DINO_434 Raptor-Nest Nurse — 1-cost 1/1. Battlecry: get a random 1-Cost
/// minion; deathrattle: get a random 1-Cost spell (the D2 random
/// simplification, §19).
pub const RAPTOR_NEST_NURSE: CardDef = CardDef {
    id: "DINO_434",
    name: "Raptor-Nest Nurse",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
    durability: 0,
    battlecry: Some(CardEffect::AddRandomOneCostMinion),
    deathrattle: Some(CardEffect::AddRandomOneCostSpell),
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

/// DINO_435 Crater Experiment — 5-cost 3/4. Kindred (Beast — the official
/// ALL tribe lands as Beast, the registered approximation §19): summon a
/// copy of this.
pub const CRATER_EXPERIMENT: CardDef = CardDef {
    id: "DINO_435",
    name: "Crater Experiment",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
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

// ============================================================
// Spells.
// ============================================================

/// DINO_136 Horn of Feasting — 4-cost Demon Hunter spell. Summon three
/// 2/1 Raptors with Rush. Outcast: give them Immune for the rest of this
/// turn (the official "Immune while attacking this turn" as the
/// immune-until-end-of-turn shape — the Raptors can only attack this turn
/// anyway, with Rush).
pub const HORN_OF_FEASTING: CardDef = CardDef {
    id: "DINO_136",
    name: "Horn of Feasting",
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
    spell_effect: Some(CardEffect::SummonRaptorsOutcast { count: 3 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_402 Bat Mask — 8-cost Warlock spell. Set a friendly minion's
/// stats to 1/1 and fill your board with copies of it (the copies are the
/// set 1/1 — "copies of it" refers to the minion as set; a full board
/// still sets the chosen minion, the flood is capped by the board limit).
pub const BAT_MASK: CardDef = CardDef {
    id: "DINO_402",
    name: "Bat Mask",
    card_type: CardType::Spell,
    cost: 8,
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
    spell_effect: Some(CardEffect::SetStatsAndFillBoardWithCopies {
        attack: 1,
        health: 1,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_403 Devilsaur Mask — 8-cost Hunter spell. Set a minion's stats to
/// 8/8 and give it Charge.
pub const DEVILSAUR_MASK: CardDef = CardDef {
    id: "DINO_403",
    name: "Devilsaur Mask",
    card_type: CardType::Spell,
    cost: 8,
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
    spell_effect: Some(CardEffect::SetStatsAndGrantCharge {
        attack: 8,
        health: 8,
        target: EffectTarget::AnyMinion,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_405 Hatching Ceremony — 3-cost Paladin spell. At the end of your
/// NEXT turn, give your minions +2/+2 (the two-tick countdown on the
/// player record — armed at 2 by the cast, decremented at each of the
/// owner's turn ends, the buff landing on the second).
pub const HATCHING_CEREMONY: CardDef = CardDef {
    id: "DINO_405",
    name: "Hatching Ceremony",
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
    spell_effect: Some(CardEffect::SetHatchingPending),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_406 Fire Breath — 3-cost Shaman spell. Deal 4 damage (spell-power
/// scaled) and give your Elementals +1/+1.
pub const FIRE_BREATH: CardDef = CardDef {
    id: "DINO_406",
    name: "Fire Breath",
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
    spell_effect: Some(CardEffect::DealDamageAndBuffFriendlyElementals {
        damage: 4,
        attack: 1,
        health: 1,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_414 Tribute Dance — 5-cost Mage spell (registered simplification
/// §19: the official "choose a minion, choose a different minion to
/// transform it into" resolves as two random picks — a random minion on
/// either side transforms into a random active-window minion card, the
/// real Hex transform semantics).
pub const TRIBUTE_DANCE: CardDef = CardDef {
    id: "DINO_414",
    name: "Tribute Dance",
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
    spell_effect: Some(CardEffect::TransformRandomMinionIntoRandomMinion),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_415 Story of Umbra — 7-cost Death Knight spell. Discover a
/// Deathrattle minion that costs (5) or more, summon it and trigger its
/// Deathrattle (the D2 random Discover simplification, §19 — "Deathrattle"
/// covers both the deathrattle component and the death trigger).
pub const STORY_OF_UMBRA: CardDef = CardDef {
    id: "DINO_415",
    name: "Story of Umbra",
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
    spell_effect: Some(CardEffect::SummonRandomDeathrattleMinionCostGEAndTrigger { min_cost: 5 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_417 Soulrest Ceremony — 1-cost Death Knight spell. Give your
/// minions +1 Attack and Rush; they die at the end of your turn (damaged
/// to death through the normal death path so deathrattles fire).
pub const SOULREST_CEREMONY: CardDef = CardDef {
    id: "DINO_417",
    name: "Soulrest Ceremony",
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
    spell_effect: Some(CardEffect::SoulrestMarkAndBuff),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_424 Hero's Welcome — 8-cost Paladin spell. Discover a Legendary
/// minion to summon and set its stats to 10/10 (the D2 random Discover
/// simplification, §19).
pub const HEROS_WELCOME: CardDef = CardDef {
    id: "DINO_424",
    name: "Hero's Welcome",
    card_type: CardType::Spell,
    cost: 8,
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
    spell_effect: Some(CardEffect::SummonRandomLegendaryMinionSetStats {
        attack: 10,
        health: 10,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_426 Ritual of Life — 2-cost Priest spell. Discover a 3-Cost
/// minion and summon a 2/3 copy of it (the D2 random Discover
/// simplification, §19 — the set is the W4b set semantics).
pub const RITUAL_OF_LIFE: CardDef = CardDef {
    id: "DINO_426",
    name: "Ritual of Life",
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
    spell_effect: Some(CardEffect::SummonRandomCostMinionSetStats {
        cost: 3,
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

/// DINO_428 Behemoth Mask — 7-cost Priest spell. Set a minion's stats to
/// 8/10 and give it Lifesteal; force a random enemy minion that can
/// attack to attack it (the forced attack runs through the normal attack
/// pipeline; no eligible enemy attacker is a no-op for the force).
pub const BEHEMOTH_MASK: CardDef = CardDef {
    id: "DINO_428",
    name: "Behemoth Mask",
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
    spell_effect: Some(CardEffect::SetStatsGrantLifestealForceAttack {
        attack: 8,
        health: 10,
        target: EffectTarget::AnyMinion,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_429 Sheep Mask — 4-cost Mage spell. Set a minion's stats to 1/1
/// and give it "Deathrattle: deal 2 damage to all minions" (a real
/// deathrattle — it fires when the masked minion dies).
pub const SHEEP_MASK: CardDef = CardDef {
    id: "DINO_429",
    name: "Sheep Mask",
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
    spell_effect: Some(CardEffect::SetStatsAttachDamageAllDeathrattle {
        attack: 1,
        health: 1,
        target: EffectTarget::AnyMinion,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_432 Panther Mask — 4-cost Druid spell. Set a minion's stats to
/// 5/4, give it Stealth and draw 2 cards.
pub const PANTHER_MASK: CardDef = CardDef {
    id: "DINO_432",
    name: "Panther Mask",
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
    spell_effect: Some(CardEffect::SetStatsGrantStealthAndDraw {
        attack: 5,
        health: 4,
        draw: 2,
        target: EffectTarget::AnyMinion,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// DINO_433 Guard Duty — 7-cost Warrior spell. Summon a random 6-, 4-,
/// and 2-Cost Taunt minion (the D2 random simplification, §19 — each pick
/// is independent, the same minion can be summoned twice).
pub const GUARD_DUTY: CardDef = CardDef {
    id: "DINO_433",
    name: "Guard Duty",
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
    spell_effect: Some(CardEffect::SummonRandomTauntMinionsOfCosts { a: 6, b: 4, c: 2 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

// ============================================================
// Weapons.
// ============================================================

/// DINO_408 Crystal Tusk — 2-cost 2/2 weapon. Battlecry: shuffle the
/// left-most card in your hand into your deck (at a random deck
/// position); deathrattle: draw 2 cards.
pub const CRYSTAL_TUSK: CardDef = CardDef {
    id: "DINO_408",
    name: "Crystal Tusk",
    card_type: CardType::Weapon,
    cost: 2,
    attack: 2,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::ShuffleLeftmostHandCardIntoDeck),
    deathrattle: Some(CardEffect::DrawCard { count: 2 }),
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
// Tokens.
// ============================================================

/// DINO_130t Little Longneck — the 3/3 Beast token summoned by Longneck
/// Egg's deathrattle.
pub const LITTLE_LONGNECK: CardDef = CardDef {
    id: "DINO_130t",
    name: "Little Longneck",
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

/// DINO_136t Ravenous Raptor — the 2/1 Beast with Rush token summoned by
/// Horn of Feasting.
pub const RAVENOUS_RAPTOR: CardDef = CardDef {
    id: "DINO_136t",
    name: "Ravenous Raptor",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
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

/// DINO_410t Khelos — the 20/20 Beast with Taunt hatched from The Egg of
/// Khelos (summoned directly by the Egg's deathrattle per the §19
/// simplification — the crack chain is skipped).
pub const KHELOS: CardDef = CardDef {
    id: "DINO_410t",
    name: "Khelos",
    card_type: CardType::Minion,
    cost: 10,
    attack: 20,
    health: 20,
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
