//! 卡牌定义 — 数据驱动的卡牌数据。
//!
//! 每张卡牌由 `CardDef` 描述其基本属性和效果。
//! Phase 2 支持战吼、亡语、嘲讽等关键词效果。
//! Phase 3 支持武器、英雄技能、光环、奥秘。
//!
//! 未来 Phase 会扩展为从 JSON/YAML 加载。

use crate::core::component::{AuraEffect, AuraTarget, CardType, SecretTrigger};
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
    /// 武器耐久（0 表示非武器）
    pub durability: i32,
    /// 战吼效果（Phase 2+）
    pub battlecry: Option<CardEffect>,
    /// 亡语效果（Phase 2+）
    pub deathrattle: Option<CardEffect>,
    /// 是否有嘲讽（Phase 2+）
    pub taunt: bool,
    /// 英雄技能效果（Phase 3）
    pub hero_power: Option<CardEffect>,
    /// 光环效果（Phase 3）
    pub aura: Option<(AuraEffect, AuraTarget)>,
    /// 奥秘触发条件（Phase 3，仅 Spell 且为奥秘时有效）
    pub secret: Option<SecretTrigger>,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
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
    durability: 0,
    hero_power: None,
    aura: None,
    secret: None,
};

// ============================================================
// 武器 Demo 卡牌（Phase 3）
// ============================================================

/// 炽炎战斧 — 2 费 3/2 武器
pub const FIERY_WAR_AXE: CardDef = CardDef {
    id: "ORANGE_020",
    name: "Fiery War Axe",
    card_type: CardType::Weapon,
    cost: 2,
    attack: 3,
    health: 0,
    durability: 2,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: None,
    secret: None,
};

/// 奥金斧 — 5 费 5/2 武器
pub const ARCANITE_REAPER: CardDef = CardDef {
    id: "ORANGE_021",
    name: "Arcanite Reaper",
    card_type: CardType::Weapon,
    cost: 5,
    attack: 5,
    health: 0,
    durability: 2,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: None,
    secret: None,
};

// ============================================================
// 英雄技能 Demo（Phase 3）
// ============================================================

/// 火焰冲击 — 2 费，对任一敌人造成 1 点伤害
pub const FIREBLAST: CardDef = CardDef {
    id: "ORANGE_030",
    name: "Fireblast",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: Some(CardEffect::DealDamage {
        amount: 1,
        target: EffectTarget::AnyEnemy,
    }),
    aura: None,
    secret: None,
};

/// 生命分流 — 2 费，抽 1 张牌，对自己造成 2 点伤害
pub const LIFE_TAP: CardDef = CardDef {
    id: "ORANGE_031",
    name: "Life Tap",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: Some(CardEffect::DrawCard { count: 1 }),
    aura: None,
    secret: None,
};

// ============================================================
// 光环 Demo 卡牌（Phase 3）
// ============================================================

/// 恐狼前锋 — 2 费 2/2，相邻随从 +1 攻击力
pub const DIRE_WOLF_ALPHA: CardDef = CardDef {
    id: "ORANGE_040",
    name: "Dire Wolf Alpha",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: Some((AuraEffect::GainAttack(1), AuraTarget::AdjacentMinions)),
    secret: None,
};

/// 团队领袖 — 3 费 2/3，你的其他随从 +1 攻击力
pub const RAID_LEADER: CardDef = CardDef {
    id: "ORANGE_041",
    name: "Raid Leader",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: Some((AuraEffect::GainAttack(1), AuraTarget::OtherFriendlyMinions)),
    secret: None,
};

/// 暴风城勇士 — 7 费 6/6，你的其他随从 +1/+1
pub const STORMWIND_CHAMPION: CardDef = CardDef {
    id: "ORANGE_042",
    name: "Stormwind Champion",
    card_type: CardType::Minion,
    cost: 7,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: Some((
        AuraEffect::GainStats {
            attack: 1,
            health: 1,
        },
        AuraTarget::OtherFriendlyMinions,
    )),
    secret: None,
};

// ============================================================
// 奥秘 Demo 卡牌（Phase 3）
// ============================================================

/// 爆炸陷阱 — 2 费奥秘，敌人攻击己方英雄后，对所有敌人造成 2 点伤害
pub const EXPLOSIVE_TRAP: CardDef = CardDef {
    id: "ORANGE_050",
    name: "Explosive Trap",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: None,
    secret: Some(SecretTrigger::AfterEnemyHeroAttacks),
};

/// 冰冻陷阱 — 2 费奥秘，敌人攻击己方后，将其移回手牌并 +2 费
pub const FREEZING_TRAP: CardDef = CardDef {
    id: "ORANGE_051",
    name: "Freezing Trap",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    hero_power: None,
    aura: None,
    secret: Some(SecretTrigger::AfterFriendlyAttacked),
};

// ============================================================
// 卡牌列表
// ============================================================

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

/// 武器卡牌列表（Phase 3）。
pub const WEAPON_CARDS: &[CardDef] = &[FIERY_WAR_AXE, ARCANITE_REAPER];

/// 英雄技能卡牌列表（Phase 3）。
pub const HERO_POWER_CARDS: &[CardDef] = &[FIREBLAST, LIFE_TAP];

/// 光环卡牌列表（Phase 3）。
pub const AURA_CARDS: &[CardDef] = &[DIRE_WOLF_ALPHA, RAID_LEADER, STORMWIND_CHAMPION];

/// 奥秘卡牌列表（Phase 3）。
pub const SECRET_CARDS: &[CardDef] = &[EXPLOSIVE_TRAP, FREEZING_TRAP];

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
    FIERY_WAR_AXE,
    ARCANITE_REAPER,
    DIRE_WOLF_ALPHA,
    RAID_LEADER,
    STORMWIND_CHAMPION,
    EXPLOSIVE_TRAP,
    FREEZING_TRAP,
];

/// 根据 ID 查找卡牌定义。
#[must_use]
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    ALL_CARDS.iter().find(|c| c.id == id)
}
