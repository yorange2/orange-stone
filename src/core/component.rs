//! 组件定义 — ECS 中的数据类型。
//!
//! 每个组件都是一个 Copy 类型，存储在对应的 `SparseSet` 中。
//! newtype 包装确保类型安全（不能用 Attack 替代 Health）。

use std::ops::{Add, AddAssign, Sub, SubAssign};

macro_rules! impl_arith {
    ($t:ty) => {
        impl Add for $t {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }
        impl AddAssign for $t {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl Sub for $t {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }
        impl SubAssign for $t {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
    };
}

/// 生命值组件。
///
/// 当 Health ≤ 0 时，实体视为死亡（英雄死亡 → 游戏结束，随从死亡 → 移入坟墓场）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Health(pub i32);
impl From<i32> for Health {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl Health {
    /// 返回 `true` 如果生命值 ≤ 0（实体已死亡）。
    #[must_use]
    pub const fn is_dead(self) -> bool {
        self.0 <= 0
    }
}
impl_arith!(Health);

/// 攻击力组件。
///
/// 英雄和某些随从可以有 0 攻击力。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Attack(pub i32);
impl From<i32> for Attack {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Attack);

/// 法力消耗组件。
///
/// Phase 1 中预留，暂不使用法力水晶系统。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cost(pub i32);
impl From<i32> for Cost {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Cost);

/// 卡牌类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    /// 随从 — 可以站场、攻击和被攻击
    Minion,
    /// 英雄 — 每个玩家有一个，死亡则游戏结束
    Hero,
    /// 武器（Phase 2+）
    Weapon,
    /// 法术（Phase 2+）
    Spell,
}

/// 本回合已攻击次数。
///
/// 每个随从/英雄每回合最多攻击 1 次（Phase 2+ 的风怒会增加到 2）。
/// 回合开始时由 `TurnStarted` 事件重置为 0。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AttacksUsed(pub u8);
impl From<u8> for AttacksUsed {
    fn from(v: u8) -> Self {
        Self(v)
    }
}
impl AttacksUsed {
    /// 返回 `true` 如果已耗尽本回合的攻击次数（默认 1 次）。
    /// 传入 `max_attacks` 以支持风怒（2 次）。
    #[must_use]
    pub const fn is_exhausted_with(self, max_attacks: u8) -> bool {
        self.0 >= max_attacks
    }
}

/// 战吼组件 — 随从被召唤时触发。
///
/// `Battlecry` 在 `MinionSummoned` 事件处理时被检测，
/// 触发效果通过 `CardEffect` 定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Battlecry(pub crate::core::effect::CardEffect);

/// 亡语组件 — 随从死亡时触发。
///
/// `Deathrattle` 在 `MinionDied` 事件处理时被检测（在移入坟墓场之前），
/// 触发效果通过 `CardEffect` 定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Deathrattle(pub crate::core::effect::CardEffect);

/// 嘲讽组件 — 敌方必须优先攻击此随从。
///
/// 如果敌方战场上有任何带 `Taunt` 的随从，
/// 攻击者不能选择英雄或非嘲讽随从作为攻击目标。
/// 多个嘲讽随从可以自由选择攻击哪个。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Taunt;

/// 武器耐久度 — 英雄攻击时消耗 1 点，归零时摧毁武器。
///
/// 耐久度在武器实体的 `Weapon` 组件中。英雄攻击宣言后，
/// 武器耐久 -1；耐久降至 0 时，武器被摧毁。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Durability(pub i32);
impl From<i32> for Durability {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Durability);

/// 英雄护甲 — 吸收伤害，先于生命值扣除。
///
/// 英雄受到伤害时，先扣除护甲值。护甲降至 0 后，
/// 剩余伤害由生命值承受。护甲不能为负。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Armor(pub i32);
impl From<i32> for Armor {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Armor);

/// 英雄技能定义 — 英雄可使用的主动技能。
///
/// 大多数英雄技能消耗 2 点法力，每回合限用一次。
/// 效果通过 `CardEffect` 定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeroPowerDef {
    /// 法力消耗（通常为 2）
    pub cost: i32,
    /// 技能效果
    pub effect: crate::core::effect::CardEffect,
}

/// 本回合是否已使用过英雄技能。
///
/// 回合开始时重置为 `false`，使用英雄技能后设为 `true`。
/// 使用 `bool` 而非次数计数器，因为英雄技能每回合只能使用一次。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HeroPowerUsed(pub bool);

/// 光环效果 — 持续影响符合条件实体的被动效果。
///
/// 光环不修改实体的基础属性，而是在查询时动态叠加 buff。
/// 光环源死亡或移出战场后，效果自动消失。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Aura {
    /// 光环效果类型
    pub effect: AuraEffect,
    /// 影响范围
    pub target: AuraTarget,
}

/// 光环效果类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuraEffect {
    /// +N/+M 增益
    GainStats {
        /// 攻击力增量
        attack: i32,
        /// 生命值增量
        health: i32,
    },
    /// +N 攻击力
    GainAttack(i32),
    /// +N 生命值
    GainHealth(i32),
}

/// 光环影响范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuraTarget {
    /// 相邻随从（左右各一个）
    AdjacentMinions,
    /// 其他友方随从（不含自身）
    OtherFriendlyMinions,
    /// 所有友方随从（含自身）
    AllFriendlyMinions,
    /// 所有敌方随从
    AllEnemyMinions,
}

/// 奥秘 — 面朝下挂载的被动触发法术。
///
/// 奥秘卡牌打出后进入 `SetAside` 区域（对对手隐藏）。
/// 当触发条件满足时，奥秘被揭示并执行效果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Secret {
    /// 触发条件
    pub trigger: SecretTrigger,
    /// 触发后执行的效果
    pub effect: crate::core::effect::CardEffect,
}

/// 奥秘触发条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretTrigger {
    /// 己方角色（英雄或随从）被攻击后
    AfterFriendlyAttacked,
    /// 敌方随从被打出后
    AfterEnemyMinionPlayed,
    /// 敌方英雄攻击后
    AfterEnemyHeroAttacks,
    /// 己方回合开始时
    OnFriendlyTurnStart,
    /// 随从死亡后
    AfterMinionDied,
    /// 敌方施放法术后（Counter 类奥秘）
    WhenEnemySpellCast,
}

/// 圣盾 — 吸收一次伤害后消失。
///
/// 当带圣盾的角色受到伤害时，圣盾移除，伤害完全被吸收。
/// 圣盾不叠加（一个实体只能有一个圣盾）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DivineShield;

/// 风怒 — 每回合可攻击 2 次。
///
/// 拥有风怒的角色每回合最多攻击 2 次（而非默认的 1 次）。
/// `AttacksUsed.is_exhausted()` 检查时需根据是否拥有风怒判断。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Windfury;

/// 冲锋 — 召唤当回合即可攻击。
///
/// 当随从被召唤时，如果有冲锋组件，则不标记为已攻击
/// （即 `AttacksUsed` 保持为 0，允许立即攻击）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Charge;

/// 法术伤害加成 — 增加法术造成的伤害。
///
/// 场上所有友方 `SpellDamage` 值累加，加到法术伤害上。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SpellDamage(pub i32);
impl From<i32> for SpellDamage {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(SpellDamage);

/// 冻结 — 角色被冻结，跳过下一次攻击机会。
///
/// 冻结的角色在回合开始时解冻（清除 Freeze 组件）。
/// 如果在被冻结的回合，角色无法攻击。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Freeze;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_dead() {
        assert!(Health(0).is_dead());
        assert!(Health(-1).is_dead());
        assert!(!Health(1).is_dead());
    }

    #[test]
    fn health_arithmetic() {
        let mut h = Health(10);
        h += Health(5);
        assert_eq!(h, Health(15));
        h -= Health(3);
        assert_eq!(h, Health(12));
        let diff = Health(10) - Health(3);
        assert_eq!(diff, Health(7));
    }

    #[test]
    fn attacks_used_is_exhausted() {
        assert!(AttacksUsed(1).is_exhausted_with(1));
        assert!(AttacksUsed(2).is_exhausted_with(2));
        assert!(!AttacksUsed(1).is_exhausted_with(2));
        assert!(!AttacksUsed(0).is_exhausted_with(1));
    }
}
