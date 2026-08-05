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
use std::collections::VecDeque;

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
        /// 显式目标（`Action::PlayCard` 传入；`None` 时引擎随机选择）
        target: Option<Entity>,
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
    /// 攻击结算 — 统一伤害管线中的攻击步骤。
    ///
    /// 在 `AttackDeclared`（含奥秘反应点）之后处理：将攻击方伤害
    /// 与防御方反击入队。攻击方伤害在入队时计算（武器在
    /// `AttackDeclared` 中可能被摧毁，伤害必须包含攻击时的武器加成）；
    /// 反击伤害在结算时根据当前状态重新计算（奥秘重定向后的新目标
    /// 自动获得反击，无需逐个卡牌特殊处理）。
    ResolveAttack {
        /// 攻击方
        attacker: Entity,
        /// 防御方
        defender: Entity,
        /// 攻击方造成的伤害（入队时计算）
        attacker_damage: i32,
        /// 攻击方是否免疫反击（角斗士的长弓 — 英雄攻击时免疫）
        retaliation_immune: bool,
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
    /// 武器被装备
    WeaponEquipped {
        /// 装备武器的玩家
        player: PlayerId,
        /// 武器实体
        weapon: Entity,
    },
    /// 武器被摧毁（耐久归零或被替换）
    WeaponDestroyed {
        /// 失去武器的玩家
        player: PlayerId,
        /// 被摧毁的武器实体
        weapon: Entity,
    },
    /// 法术被施放
    SpellCast {
        /// 施放法术的玩家
        player: PlayerId,
        /// 法术实体
        spell: Entity,
    },
    /// 英雄技能被使用
    HeroPowerActivated {
        /// 使用技能的玩家
        player: PlayerId,
        /// 英雄实体
        hero: Entity,
    },
    /// 奥秘被揭示并触发
    SecretRevealed {
        /// 拥有奥秘的玩家
        player: PlayerId,
        /// 奥秘实体
        secret: Entity,
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
///
/// 内部使用三个 `VecDeque`（每个优先级一个），
/// 入队和出队都是 O(1) — 事件循环是引擎的最热路径。
#[derive(Debug, Default, Clone)]
pub struct EventQueue {
    buckets: [VecDeque<Event>; 3],
}

impl EventQueue {
    /// 创建一个空的事件队列。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buckets: [
                const { VecDeque::new() },
                const { VecDeque::new() },
                const { VecDeque::new() },
            ],
        }
    }

    /// 以 Normal 优先级入队一个事件。
    pub fn push(&mut self, event: Event) {
        self.push_with_priority(event, Priority::Normal);
    }

    /// 以指定优先级入队一个事件。
    ///
    /// 追加到同优先级队列的末尾（保持 FIFO）。
    pub fn push_with_priority(&mut self, event: Event, priority: Priority) {
        self.buckets[priority as usize].push_back(event);
    }

    /// 取出并移除队首事件（最高优先级中的最先入队者）。
    ///
    /// 按优先级从高到低检查各桶，返回第一个非空桶的队首。O(1)。
    pub fn pop_front(&mut self) -> Option<Event> {
        for bucket in &mut self.buckets {
            if let Some(event) = bucket.pop_front() {
                return Some(event);
            }
        }
        None
    }

    /// 返回 `true` 如果队列为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buckets.iter().all(VecDeque::is_empty)
    }

    /// 返回队列中的事件数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.buckets.iter().map(VecDeque::len).sum()
    }

    /// 清空队列并返回所有事件（按处理顺序）。
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::with_capacity(self.len());
        for bucket in &mut self.buckets {
            events.extend(bucket.drain(..));
        }
        events
    }

    /// 重定向队列中第一个匹配 `ResolveAttack { attacker, defender }` 的攻击。
    ///
    /// 攻击重定向奥秘（误导、崇高牺牲）的统一原语：攻击的伤害与反击
    /// 都在 `ResolveAttack` 结算时按当前状态计算，因此只需替换该事件的
    /// 防御方即可完整重定向（新目标的自动反击由结算逻辑处理）。
    /// 返回是否找到并重定向。
    pub fn redirect_attack(
        &mut self,
        attacker: Entity,
        old_defender: Entity,
        new_defender: Entity,
    ) -> bool {
        // 按处理顺序（优先级从高到低）扫描各桶，找到第一个匹配
        for bucket in &mut self.buckets {
            for event in bucket {
                if let Event::ResolveAttack {
                    attacker: a,
                    defender: d,
                    attacker_damage,
                    retaliation_immune,
                } = *event
                {
                    if a == attacker && d == old_defender {
                        *event = Event::ResolveAttack {
                            attacker,
                            defender: new_defender,
                            attacker_damage,
                            retaliation_immune,
                        };
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 将所有满足谓词的 `DamageDealt` 事件重定向到 `new_target`。
    ///
    /// 谓词接收事件的 source 和 target。用于法术扭曲者等需要按来源
    /// 批量重定向的奥秘（法术伤害在效果解析时按源法术入队，奥秘在此
    /// 统一拦截）。返回重定向的数量。
    pub fn redirect_damages(
        &mut self,
        predicate: impl Fn(Entity, Entity) -> bool,
        new_target: Entity,
    ) -> usize {
        let mut count = 0;
        for bucket in &mut self.buckets {
            for event in bucket {
                if let Event::DamageDealt { source, target, .. } = event {
                    if predicate(*source, *target) {
                        *target = new_target;
                        count += 1;
                    }
                }
            }
        }
        count
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
            target: None,
        });
        q.push(Event::CardPlayed {
            player: PlayerId::Player1,
            card: Entity::new(2, 0),
            target: None,
        });
        q.push(Event::CardPlayed {
            player: PlayerId::Player1,
            card: Entity::new(3, 0),
            target: None,
        });

        assert_eq!(
            q.pop_front(),
            Some(Event::CardPlayed {
                player: PlayerId::Player1,
                card: Entity::new(1, 0),
                target: None
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::CardPlayed {
                player: PlayerId::Player1,
                card: Entity::new(2, 0),
                target: None
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::CardPlayed {
                player: PlayerId::Player1,
                card: Entity::new(3, 0),
                target: None
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

    #[test]
    fn interleaved_priorities_preserve_fifo() {
        let mut q = EventQueue::new();
        // 交错入队：Lowest → Normal → Highest → Normal → Lowest
        q.push_with_priority(
            Event::MinionDied {
                minion: Entity::new(1, 0),
            },
            Priority::Lowest,
        );
        q.push_with_priority(
            Event::DamageDealt {
                source: Entity::new(2, 0),
                target: Entity::new(3, 0),
                amount: 1,
            },
            Priority::Normal,
        );
        q.push_with_priority(
            Event::GameOver {
                winner: PlayerId::Player1,
            },
            Priority::Highest,
        );
        q.push_with_priority(
            Event::DamageDealt {
                source: Entity::new(4, 0),
                target: Entity::new(5, 0),
                amount: 2,
            },
            Priority::Normal,
        );
        q.push_with_priority(
            Event::MinionDied {
                minion: Entity::new(6, 0),
            },
            Priority::Lowest,
        );

        // 顺序：Highest、Normal FIFO、Lowest FIFO
        assert_eq!(
            q.pop_front(),
            Some(Event::GameOver {
                winner: PlayerId::Player1
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::DamageDealt {
                source: Entity::new(2, 0),
                target: Entity::new(3, 0),
                amount: 1
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::DamageDealt {
                source: Entity::new(4, 0),
                target: Entity::new(5, 0),
                amount: 2
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::MinionDied {
                minion: Entity::new(1, 0)
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::MinionDied {
                minion: Entity::new(6, 0)
            })
        );
        assert!(q.is_empty());
    }

    #[test]
    fn redirect_attack_matches_processing_order() {
        let mut q = EventQueue::new();
        // 同优先级中，先入队的先处理 — 重定向应命中第一个匹配
        let first = Entity::new(1, 0);
        let second = Entity::new(2, 0);
        let attacker = Entity::new(3, 0);
        q.push(Event::ResolveAttack {
            attacker,
            defender: first,
            attacker_damage: 3,
            retaliation_immune: false,
        });
        q.push(Event::ResolveAttack {
            attacker,
            defender: second,
            attacker_damage: 3,
            retaliation_immune: false,
        });

        assert!(q.redirect_attack(attacker, second, Entity::new(9, 0)));
        // 第二个攻击被重定向，第一个保持不变（attacker_damage/retaliation_immune 也保留）
        assert_eq!(
            q.pop_front(),
            Some(Event::ResolveAttack {
                attacker,
                defender: first,
                attacker_damage: 3,
                retaliation_immune: false,
            })
        );
        assert_eq!(
            q.pop_front(),
            Some(Event::ResolveAttack {
                attacker,
                defender: Entity::new(9, 0),
                attacker_damage: 3,
                retaliation_immune: false,
            })
        );
    }

    #[test]
    fn redirect_attack_preserves_attack_fields() {
        let mut q = EventQueue::new();
        let attacker = Entity::new(5, 0);
        q.push(Event::ResolveAttack {
            attacker,
            defender: Entity::new(6, 0),
            attacker_damage: 7,
            retaliation_immune: true,
        });

        assert!(q.redirect_attack(attacker, Entity::new(6, 0), Entity::new(8, 0)));
        assert_eq!(
            q.pop_front(),
            Some(Event::ResolveAttack {
                attacker,
                defender: Entity::new(8, 0),
                attacker_damage: 7,
                retaliation_immune: true,
            })
        );
    }

    #[test]
    fn redirect_attack_no_match() {
        let mut q = EventQueue::new();
        q.push(Event::ResolveAttack {
            attacker: Entity::new(1, 0),
            defender: Entity::new(2, 0),
            attacker_damage: 1,
            retaliation_immune: false,
        });
        assert!(!q.redirect_attack(Entity::new(9, 0), Entity::new(2, 0), Entity::new(3, 0)));
        assert!(!q.redirect_attack(Entity::new(1, 0), Entity::new(9, 0), Entity::new(3, 0)));
        assert_eq!(q.len(), 1);
    }
}
