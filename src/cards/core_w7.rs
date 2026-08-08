//! Core Set W7 cards (core-set-roadmap W7) — enrage finish. Both cards
//! ride the engine's read-based `Enrage` component (fidelity-debt §13):
//! the bonus applies while the minion is damaged and ends the moment it is
//! healed to full. The Enrage amounts are wired in `apply_card_keywords`
//! (mirroring the Classic pool's Grommash / Tauren Warrior).

use crate::cards::def::CardDef;
use crate::core::component::CardType;

/// CORE_EX1_414 Grommash Hellscream — 8费 Minion (Charge, Enrage +6)
pub const CORE_GROMMASH_HELLSCREAM: CardDef = CardDef {
    id: "CORE_EX1_414",
    name: "Grommash Hellscream",
    card_type: CardType::Minion,
    cost: 8,
    attack: 4,
    health: 9,
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

/// CORE_OG_218 Bloodhoof Brave — 4费 Minion (Taunt, Enrage +3)
pub const CORE_BLOODHOOF_BRAVE: CardDef = CardDef {
    id: "CORE_OG_218",
    name: "Bloodhoof Brave",
    card_type: CardType::Minion,
    cost: 4,
    attack: 2,
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
