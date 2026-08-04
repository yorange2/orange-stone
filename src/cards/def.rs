//! 卡牌定义 — 数据驱动的卡牌数据。
//!
//! 每张卡牌由 `CardDef` 描述其基本属性。Phase 1 只包含白板随从
//! （无效果），用于测试基本的随从交易逻辑。
//!
//! 未来 Phase 会扩展为从 JSON/YAML 加载，并支持战吼、亡语等效果定义。

use crate::core::component::CardType;

/// 卡牌静态定义 — 描述一张卡牌的基本属性。
///
/// 所有字段都是 `'static` 生命周期（编译时常量），
/// Phase 2+ 将支持从外部数据文件加载。
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
    /// 攻击力（法术和武器暂支持 0）
    pub attack: i32,
    /// 生命值
    pub health: i32,
}

// 以下是基础白板随从卡牌定义，用于测试。
// 使用炉石经典的白板随从作为测试卡牌。
// 数据来源：炉石传说基本/经典系列。

/// 小精灵 — 0 费 1/1
pub const WISP: CardDef = CardDef {
    id: "ORANGE_001",
    name: "Wisp",
    card_type: CardType::Minion,
    cost: 0,
    attack: 1,
    health: 1,
};

/// 淡水鳄 — 2 费 2/3
pub const RIVER_CROCOLISK: CardDef = CardDef {
    id: "ORANGE_002",
    name: "River Crocolisk",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
};

/// 冰风雪人 — 4 费 4/5
pub const CHILLWIND_YETI: CardDef = CardDef {
    id: "ORANGE_003",
    name: "Chillwind Yeti",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 5,
};

/// 石拳食人魔 — 6 费 6/7
pub const BOULDERFIST_OGRE: CardDef = CardDef {
    id: "ORANGE_004",
    name: "Boulderfist Ogre",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 7,
};

/// 作战傀儡 — 7 费 7/7
pub const WAR_GOLEM: CardDef = CardDef {
    id: "ORANGE_005",
    name: "War Golem",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
};

/// 所有基本白板随从的列表。
pub const BASIC_MINIONS: &[CardDef] = &[
    WISP,
    RIVER_CROCOLISK,
    CHILLWIND_YETI,
    BOULDERFIST_OGRE,
    WAR_GOLEM,
];

/// 根据 ID 查找卡牌定义。
#[must_use]
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    BASIC_MINIONS.iter().find(|c| c.id == id)
}
