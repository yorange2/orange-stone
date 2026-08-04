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
    /// 返回 `true` 如果已耗尽本回合的攻击次数。
    #[must_use]
    pub const fn is_exhausted(self) -> bool {
        self.0 >= 1
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
        assert!(AttacksUsed(1).is_exhausted());
        assert!(AttacksUsed(2).is_exhausted());
        assert!(!AttacksUsed(0).is_exhausted());
    }
}
