//! 卡牌定义 — 数据驱动的卡牌数据。
//!
//! 包含 CardDef 结构体、vanilla! 宏，以及所有卡牌常量的 re-export。

#![allow(missing_docs)]

use crate::core::component::{AuraEffect, AuraTarget, CardType, SecretTrigger};
use crate::core::effect::CardEffect;

use super::sets::ALL_CARDS;

/// 卡牌静态定义 — 描述一张卡牌的基本属性和效果。
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
    pub hero_power: Option<CardEffect>,
    pub aura: Option<(AuraEffect, AuraTarget)>,
    pub secret: Option<SecretTrigger>,
    pub divine_shield: bool,
    pub windfury: bool,
    pub charge: bool,
    pub spell_damage: i32,
    /// 不能主动攻击（如拉格纳罗斯）
    pub cant_attack: bool,
    /// 回合结束效果
    pub end_turn_effect: Option<CardEffect>,
    /// 法术效果（仅对法术牌有效，打出时触发）
    pub spell_effect: Option<CardEffect>,
    /// 法术触发效果 — 当友方施放法术时触发此效果（此随从需在场上）
    pub spell_trigger: Option<CardEffect>,
}

/// 宏：简化白板随从定义。
/// 使用 `#[macro_export]` 导出到 crate root，子模块通过 `use crate::vanilla;` 导入。
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
            hero_power: None,
            aura: None,
            secret: None,
            divine_shield: false,
            windfury: false,
            charge: false,
            spell_damage: 0,
            cant_attack: false,
            end_turn_effect: None,
            spell_effect: None,
            spell_trigger: None,
        }
    };
}

// ============================================================
// Re-export 所有卡牌常量，保持向后兼容
// 外部代码可通过 `crate::cards::def::CHILLWIND_YETI` 等路径访问
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

/// 根据卡牌 ID 查找卡牌定义。
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    ALL_CARDS.iter().find(|c| c.id == id)
}
