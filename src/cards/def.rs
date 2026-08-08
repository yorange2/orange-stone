//! Card definitions — data-driven card data.
//!
//! Contains the CardDef struct, the vanilla! macro, and re-exports of all card constants.

#![allow(missing_docs)]

use crate::core::component::{AuraEffect, AuraTarget, CardType, SecretTrigger};
use crate::core::effect::CardEffect;

use super::sets::ALL_CARDS;

/// Static card definition — describes a card's basic attributes and effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDef {
    pub id: &'static str,
    pub name: &'static str,
    pub card_type: CardType,
    pub cost: i32,
    pub attack: i32,
    pub health: i32,
    pub durability: i32,
    pub battlecry: Option<CardEffect>,
    pub deathrattle: Option<CardEffect>,
    pub taunt: bool,
    pub stealth: bool,
    pub elusive: bool,
    /// Minion race / tribe (Beast / Murloc / Demon); `None` for non-tribe minions
    pub race: Option<crate::core::component::Race>,
    pub hero_power: Option<CardEffect>,
    pub aura: Option<(AuraEffect, AuraTarget)>,
    pub secret: Option<SecretTrigger>,
    pub divine_shield: bool,
    pub windfury: bool,
    pub charge: bool,
    pub spell_damage: i32,
    /// Cannot attack actively (e.g., Ragnaros)
    pub cant_attack: bool,
    /// End-of-turn effect
    pub end_turn_effect: Option<CardEffect>,
    /// Start-of-turn effect
    pub start_turn_effect: Option<CardEffect>,
    /// Spell effect (only for spell cards; triggers when played)
    pub spell_effect: Option<CardEffect>,
    /// Spell-trigger effect — triggers when a friendly spell is cast (this minion must be on board)
    pub spell_trigger: Option<CardEffect>,
    /// Death-trigger effect — triggers when a friendly minion dies
    pub death_trigger: Option<CardEffect>,
    /// Summon-trigger effect — triggers when a friendly minion is summoned
    pub summon_trigger: Option<CardEffect>,
    /// Choose One effect — the alternate effect of Druid "Choose One" cards
    pub choose_one_effect: Option<CardEffect>,
    /// Combo effect — triggers for Rogue "Combo" cards after another card was played this turn
    pub combo_effect: Option<CardEffect>,
    /// Attack always equals Health (Lightspawn trait)
    pub attack_equals_health: bool,
}

/// Macro: simplifies vanilla minion definitions.
/// Exported to the crate root via `#[macro_export]`; submodules import it with `use crate::vanilla;`.
#[macro_export]
macro_rules! vanilla {
    ($id:expr, $name:expr, $cost:expr, $atk:expr, $hp:expr) => {
        CardDef {
            id: $id,
            name: $name,
            card_type: CardType::Minion,
            cost: $cost,
            attack: $atk,
            health: $hp,
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
        }
    };
}

// ============================================================
// Re-export all card constants for backward compatibility
// External code can access them via paths like `crate::cards::def::CHILLWIND_YETI`
// ============================================================

pub use super::classic_druid::*;
pub use super::classic_hunter::*;
pub use super::classic_legendary::*;
pub use super::classic_mage::*;
pub use super::classic_neutral::*;
pub use super::classic_paladin::*;
pub use super::classic_priest::*;
pub use super::classic_rogue::*;
pub use super::classic_shaman::*;
pub use super::classic_warlock::*;
pub use super::classic_warrior::*;
pub use super::core_w1::*;
pub use super::core_w2::*;
pub use super::core_w3a::*;
pub use super::core_w3b::*;
pub use super::core_w3c::*;
pub use super::core_w3d::*;
pub use super::core_w4a::*;
pub use super::core_w4b::*;

/// Look up a card definition by card ID.
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    ALL_CARDS.iter().find(|c| c.id == id)
}
