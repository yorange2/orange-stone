//! 卡牌效果定义 — 编译时常量的 CardEffect 和 EffectTarget。
//!
//! Phase 2 支持的卡牌效果：伤害、抽牌、召唤、buff。
//! 效果作为 `Copy` 枚举常量存储在 `CardDef` 和 `Battlecry`/`Deathrattle` 组件中。

/// 效果目标选择器。
///
/// 执行效果时，引擎根据此枚举选择目标实体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectTarget {
    /// 随机敌方角色（英雄或随从）
    AnyEnemy,
    /// 随机敌方随从
    AnyEnemyMinion,
    /// 所有敌方随从
    AllEnemyMinions,
    /// 敌方英雄
    EnemyHero,
    /// 自身（buff 类效果）
    Self_,
    /// 所有友方随从
    AllFriendlyMinions,
}

/// 卡牌效果 — 触发时执行的动作。
///
/// 实现 `Copy` 以作为组件存储。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardEffect {
    /// 造成 N 点伤害（DealDamage）
    DealDamage {
        /// 伤害数值
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 抽 N 张牌
    DrawCard {
        /// 抽牌数量
        count: u32,
    },
    /// 召唤一个随从
    SummonMinion {
        /// 要召唤的卡牌 ID
        card_id: &'static str,
    },
    /// 获得 +N/+M (buff 自身或友方)
    GainStats {
        /// 攻击力增量
        attack: i32,
        /// 生命值增量
        health: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 装备武器
    EquipWeapon {
        /// 要装备的武器卡牌 ID
        card_id: &'static str,
    },
    /// 获得护甲
    GainArmor {
        /// 护甲值
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 将一个随从移回手牌
    ReturnToHand {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 增加随从的法力消耗（冰冻陷阱效果）
    IncreaseCost {
        /// 法力消耗增量
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
}
