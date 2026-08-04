//! 玩家定义 — PlayerId 和 Player 状态。
//!
//! 每个玩家有一个 `PlayerId`（通过组件挂载到实体上）
//! 和一个 `Player` 结构体（存储在 `GameState` 中）。

/// 玩家标识符。
///
/// 用 `#[repr(u8)]` 确保可以高效索引数组（`[T; 2]`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PlayerId {
    /// 先手玩家
    Player1 = 0,
    /// 后手玩家
    Player2 = 1,
}

impl PlayerId {
    /// 玩家数量。
    pub const COUNT: usize = 2;

    /// 返回 `usize` 索引，用于数组下标。
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// 返回对手的 `PlayerId`。
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

/// 玩家状态 — 非实体级别的玩家数据。
///
/// 英雄本身是一个实体（`CardType::Hero`），存储在 World 中。
/// `Player` 则持有对英雄实体的引用以及将来会加入的法力水晶等状态。
#[derive(Debug, Clone)]
pub struct Player {
    /// 玩家 ID
    pub id: PlayerId,
    /// 指向英雄实体的句柄
    pub hero: crate::core::entity::Entity,
}

impl Player {
    /// 创建一个新的玩家状态。
    #[must_use]
    pub const fn new(id: PlayerId, hero: crate::core::entity::Entity) -> Self {
        Self { id, hero }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_index() {
        assert_eq!(PlayerId::Player1.index(), 0);
        assert_eq!(PlayerId::Player2.index(), 1);
    }

    #[test]
    fn player_id_opponent() {
        assert_eq!(PlayerId::Player1.opponent(), PlayerId::Player2);
        assert_eq!(PlayerId::Player2.opponent(), PlayerId::Player1);
    }
}
