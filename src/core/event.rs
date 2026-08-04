//! 事件系统 — Event 枚举、EventQueue 和优先级。
//!
//! 游戏规则引擎通过事件驱动：Action 产生 Event，Event 可能触发更多 Event，
//! 直到队列清空。事件既是"发生了什么"的记录（用于回放），
//! 也是状态变更的驱动（`apply_event` 修改 GameState）。
//!
//! Phase 2+ 的 Trigger 系统在 `apply_event` 处插入，匹配事件类型并
//! 将触发效果入队。

use crate::core::entity::Entity;
use crate::core::player::PlayerId;

/// 游戏事件 — 规则引擎处理的原子事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// 回合开始 — 重置攻击次数，更新 active_player 等
    TurnStarted {
        /// 进入回合的玩家
        player: PlayerId,
    },
    /// 回合结束
    TurnEnded {
        /// 结束回合的玩家
        player: PlayerId,
    },
    /// 卡牌被打出
    CardPlayed {
        /// 打出卡牌的玩家
        player: PlayerId,
        /// 被打出的卡牌实体
        card: Entity,
    },
    /// 随从被召唤到战场
    MinionSummoned {
        /// 召唤者
        player: PlayerId,
        /// 被召唤的随从实体
        minion: Entity,
    },
    /// 攻击被宣告
    AttackDeclared {
        /// 攻击方
        attacker: Entity,
        /// 防御方
        defender: Entity,
    },
    /// 伤害被造成
    DamageDealt {
        /// 伤害来源
        source: Entity,
        /// 伤害目标
        target: Entity,
        /// 伤害数值
        amount: i32,
    },
    /// 随从死亡
    MinionDied {
        /// 死亡的随从实体
        minion: Entity,
    },
    /// 卡牌被抽到手中
    CardDrawn {
        /// 抽牌玩家
        player: PlayerId,
        /// 被抽到的卡牌实体
        card: Entity,
    },
    /// 游戏结束
    GameOver {
        /// 获胜方
        winner: PlayerId,
    },
}

/// 事件优先级 — 数值越小越先处理。
///
/// Phase 1 中所有事件都是 Normal 优先级。
/// Phase 2+ 的 Secret/Trigger 会在不同优先级入队。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Priority {
    /// 最高优先级（Phase 2+ 用于奥秘触发等）
    Highest = 0,
    /// 标准优先级
    Normal = 1,
    /// 最低优先级
    Lowest = 2,
}

/// 事件队列 — 按优先级排序的 FIFO 队列。
///
/// 同优先级内保持入队顺序（FIFO），这保证了游戏逻辑的确定性。
/// 不适用 `BinaryHeap`，因为它不保证同优先级元素的顺序。
#[derive(Debug, Default, Clone)]
pub struct EventQueue {
    items: Vec<(Priority, Event)>,
}

impl EventQueue {
    /// 创建一个空的事件队列。
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 以 Normal 优先级入队一个事件。
    pub fn push(&mut self, event: Event) {
        self.push_with_priority(event, Priority::Normal);
    }

    /// 以指定优先级入队一个事件。
    ///
    /// 插入到同优先级元素的末尾（保持 FIFO）。
    pub fn push_with_priority(&mut self, event: Event, priority: Priority) {
        // 找到同优先级的最后一个位置 + 1
        let pos = self
            .items
            .iter()
            .rposition(|(p, _)| *p <= priority)
            .map_or(0, |i| i + 1);
        self.items.insert(pos, (priority, event));
    }

    /// 取出并移除队首事件（最高优先级中的最先入队者）。
    pub fn pop_front(&mut self) -> Option<Event> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0).1)
        }
    }

    /// 返回 `true` 如果队列为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 返回队列中的事件数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 清空队列并返回所有事件（按处理顺序）。
    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.items)
            .into_iter()
            .map(|(_, e)| e)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_within_same_priority() {
        let mut q = EventQueue::new();
        q.push(Event::TurnStarted {
            player: PlayerId::Player1,
        });
        q.push(Event::AttackDeclared {
            attacker: Entity::new(1, 0),
            defender: Entity::new(2, 0),
        });
        q.push(Event::TurnEnded {
            player: PlayerId::Player1,
        });

        assert_eq!(
            q.pop_front(),
            Some(Event::TurnStarted {
                player: PlayerId::Player1
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::AttackDeclared {
                attacker: Entity::new(1, 0),
                defender: Entity::new(2, 0)
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::TurnEnded {
                player: PlayerId::Player1
            })
        );
        assert!(q.is_empty());
    }

    #[test]
    fn priority_ordering() {
        let mut q = EventQueue::new();
        q.push_with_priority(
            Event::MinionDied {
                minion: Entity::new(10, 0),
            },
            Priority::Lowest,
        );
        q.push_with_priority(
            Event::GameOver {
                winner: PlayerId::Player1,
            },
            Priority::Highest,
        );
        q.push_with_priority(
            Event::DamageDealt {
                source: Entity::new(1, 0),
                target: Entity::new(2, 0),
                amount: 3,
            },
            Priority::Normal,
        );

        // Highest 先
        assert_eq!(
            q.pop_front(),
            Some(Event::GameOver {
                winner: PlayerId::Player1
            })
        );
        // 然后 Normal
        assert_eq!(
            q.pop_front(),
            Some(Event::DamageDealt {
                source: Entity::new(1, 0),
                target: Entity::new(2, 0),
                amount: 3
            })
        );
        // 最后 Lowest
        assert_eq!(
            q.pop_front(),
            Some(Event::MinionDied {
                minion: Entity::new(10, 0)
            })
        );
    }

    #[test]
    fn fifo_within_same_priority_stable() {
        let mut q = EventQueue::new();
        q.push(Event::CardPlayed {
            player: PlayerId::Player1,
            card: Entity::new(1, 0),
        });
        q.push(Event::CardPlayed {
            player: PlayerId::Player1,
            card: Entity::new(2, 0),
        });
        q.push(Event::CardPlayed {
            player: PlayerId::Player1,
            card: Entity::new(3, 0),
        });

        assert_eq!(
            q.pop_front(),
            Some(Event::CardPlayed {
                player: PlayerId::Player1,
                card: Entity::new(1, 0)
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::CardPlayed {
                player: PlayerId::Player1,
                card: Entity::new(2, 0)
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::CardPlayed {
                player: PlayerId::Player1,
                card: Entity::new(3, 0)
            })
        );
    }

    #[test]
    fn drain_returns_all_in_order() {
        let mut q = EventQueue::new();
        q.push(Event::TurnStarted {
            player: PlayerId::Player2,
        });
        q.push(Event::TurnEnded {
            player: PlayerId::Player2,
        });
        let events = q.drain();
        assert_eq!(events.len(), 2);
        assert!(q.is_empty());
    }

    #[test]
    fn empty_queue_pop_returns_none() {
        let mut q = EventQueue::new();
        assert_eq!(q.pop_front(), None);
    }
}
