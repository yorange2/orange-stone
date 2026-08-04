//! 卡牌定义 — 数据驱动的卡牌数据。
//!
//! 每张卡牌由 `CardDef` 描述其基本属性和效果。
//! Phase 2 支持战吼、亡语、嘲讽等关键词效果。
//!
//! 未来 Phase 会扩展为从 JSON/YAML 加载。

use crate::core::component::CardType;
use crate::core::effect::{CardEffect, EffectTarget};

/// 卡牌静态定义 — 描述一张卡牌的基本属性和效果。
///
/// 所有字段都是 `'static` 生命周期（编译时常量）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDef {
    /// 卡牌唯一标识符（如 "ORANGE_001"）
    pub id: &'static str,
    /// 卡牌名称
    pub name: &'static str,
    /// 卡牌类型
    pub card_type: CardType,
    /// 法力消耗
    pub cost: i32,
    /// 攻击力
    pub attack: i32,
    /// 生命值
    pub health: i32,
    /// 战吼效果（Phase 2+）
    pub battlecry: Option<CardEffect>,
    /// 亡语效果（Phase 2+）
    pub deathrattle: Option<CardEffect>,
    /// 是否有嘲讽（Phase 2+）
    pub taunt: bool,
}

// ============================================================
// 基础白板随从（Phase 1）
// ============================================================

/// 小精灵 — 0 费 1/1
pub const WISP: CardDef = CardDef {
    id: "ORANGE_001",
    name: "Wisp",
    card_type: CardType::Minion,
    cost: 0,
    attack: 1,
    health: 1,
    battlecry: None,
    deathrattle: None,
    taunt: false,
};

/// 淡水鳄 — 2 费 2/3
pub const RIVER_CROCOLISK: CardDef = CardDef {
    id: "ORANGE_002",
    name: "River Crocolisk",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
    battlecry: None,
    deathrattle: None,
    taunt: false,
};

/// 冰风雪人 — 4 费 4/5
pub const CHILLWIND_YETI: CardDef = CardDef {
    id: "ORANGE_003",
    name: "Chillwind Yeti",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 5,
    battlecry: None,
    deathrattle: None,
    taunt: false,
};

/// 石拳食人魔 — 6 费 6/7
pub const BOULDERFIST_OGRE: CardDef = CardDef {
    id: "ORANGE_004",
    name: "Boulderfist Ogre",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 7,
    battlecry: None,
    deathrattle: None,
    taunt: false,
};

/// 作战傀儡 — 7 费 7/7
pub const WAR_GOLEM: CardDef = CardDef {
    id: "ORANGE_005",
    name: "War Golem",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
    battlecry: None,
    deathrattle: None,
    taunt: false,
};

// ============================================================
// 关键词 Demo 卡牌（Phase 2）
// ============================================================

/// 精灵弓箭手 — 1 费 1/1，战吼：对一个随机敌方角色造成 1 点伤害
pub const ELVEN_ARCHER: CardDef = CardDef {
    id: "ORANGE_010",
    name: "Elven Archer",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
    battlecry: Some(CardEffect::DealDamage {
        amount: 1,
        target: EffectTarget::AnyEnemy,
    }),
    deathrattle: None,
    taunt: false,
};

/// 战利品贮藏者 — 2 费 2/1，亡语：抽一张牌
pub const LOOT_HOARDER: CardDef = CardDef {
    id: "ORANGE_011",
    name: "Loot Hoarder",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 1,
    battlecry: None,
    deathrattle: Some(CardEffect::DrawCard { count: 1 }),
    taunt: false,
};

/// 闪金镇步兵 — 1 费 1/2，嘲讽
pub const GOLDSHIRE_FOOTMAN: CardDef = CardDef {
    id: "ORANGE_012",
    name: "Goldshire Footman",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 2,
    battlecry: None,
    deathrattle: None,
    taunt: true,
};

/// 破碎残阳祭司 — 3 费 3/2，战吼：使一个友方随从获得 +1/+1
pub const SHATTERED_SUN_CLERIC: CardDef = CardDef {
    id: "ORANGE_013",
    name: "Shattered Sun Cleric",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 2,
    battlecry: Some(CardEffect::GainStats {
        attack: 1,
        health: 1,
        target: EffectTarget::Self_,
    }),
    deathrattle: None,
    taunt: false,
};

/// 新手工程师 — 2 费 1/1，战吼：抽一张牌
pub const NOVICE_ENGINEER: CardDef = CardDef {
    id: "ORANGE_014",
    name: "Novice Engineer",
    card_type: CardType::Minion,
    cost: 2,
    attack: 1,
    health: 1,
    battlecry: Some(CardEffect::DrawCard { count: 1 }),
    deathrattle: None,
    taunt: false,
};

/// 所有基本白板随从的列表。
pub const BASIC_MINIONS: &[CardDef] = &[
    WISP,
    RIVER_CROCOLISK,
    CHILLWIND_YETI,
    BOULDERFIST_OGRE,
    WAR_GOLEM,
];

/// 所有有效果的 Demo 卡牌列表（Phase 2）。
pub const DEMO_MINIONS: &[CardDef] = &[
    ELVEN_ARCHER,
    LOOT_HOARDER,
    GOLDSHIRE_FOOTMAN,
    SHATTERED_SUN_CLERIC,
    NOVICE_ENGINEER,
];

/// 所有已定义的卡牌。
pub const ALL_CARDS: &[CardDef] = &[
    WISP,
    RIVER_CROCOLISK,
    CHILLWIND_YETI,
    BOULDERFIST_OGRE,
    WAR_GOLEM,
    ELVEN_ARCHER,
    LOOT_HOARDER,
    GOLDSHIRE_FOOTMAN,
    SHATTERED_SUN_CLERIC,
    NOVICE_ENGINEER,
];

/// 根据 ID 查找卡牌定义。
#[must_use]
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    ALL_CARDS.iter().find(|c| c.id == id)
}
