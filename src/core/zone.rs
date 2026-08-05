//! 区域（Zone）系统 — 卡牌在游戏中所处的位置。
//!
//! 炉石传说有 5 个区域：牌库 (Deck)、手牌 (Hand)、战场 (Play)、
//! 坟墓场 (Graveyard)、暂离区 (SetAside)。
//! 每个区域维护一个有序的实体列表（手牌和战场顺序有意义）。
use serde::{Deserialize, Serialize};

use crate::core::player::PlayerId;

/// 区域类型。
///
/// 实体在同一时刻只能在一个区域中。区域转移通过 `World::move_to_zone` 进行。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    /// 牌库 — 卡牌初始所在地，Phase 2+ 加入抽牌逻辑
    Deck,
    /// 手牌 — 可以打出的区域
    Hand,
    /// 战场 — 随从和英雄所在区域
    Play,
    /// 坟墓场 — 死亡随从/使用过的法术去的地方
    Graveyard,
    /// 暂离区 — 临时移出游戏的区域（Phase 2+ 用于奥秘等）
    SetAside,
}

/// 区域移动错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneError {
    /// 实体已被销毁（generation 不匹配）
    EntityGone,
    /// 缺少判断所属玩家所需的 PlayerId 组件
    MissingPlayer,
    /// 缺少当前 Zone 组件（不应出现，说明状态不一致）
    MissingZone,
}

/// 所有区域的实体列表。
///
/// 牌库、手牌、战场、坟墓场是每个玩家独立维护的；暂离区是共享的。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zones {
    deck: [Vec<crate::core::entity::Entity>; 2],
    hand: [Vec<crate::core::entity::Entity>; 2],
    play: [Vec<crate::core::entity::Entity>; 2],
    graveyard: [Vec<crate::core::entity::Entity>; 2],
    set_aside: Vec<crate::core::entity::Entity>,
}

impl Zones {
    /// 创建一个空的区域表。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            deck: [const { Vec::new() }, const { Vec::new() }],
            hand: [const { Vec::new() }, const { Vec::new() }],
            play: [const { Vec::new() }, const { Vec::new() }],
            graveyard: [const { Vec::new() }, const { Vec::new() }],
            set_aside: Vec::new(),
        }
    }

    /// 获取某个区域的可变引用。
    fn vec_mut(&mut self, zone: Zone, player: PlayerId) -> &mut Vec<crate::core::entity::Entity> {
        match zone {
            Zone::Deck => &mut self.deck[player.index()],
            Zone::Hand => &mut self.hand[player.index()],
            Zone::Play => &mut self.play[player.index()],
            Zone::Graveyard => &mut self.graveyard[player.index()],
            Zone::SetAside => &mut self.set_aside,
        }
    }

    /// 获取某个区域的只读引用。
    fn vec_ref(&self, zone: Zone, player: PlayerId) -> &[crate::core::entity::Entity] {
        match zone {
            Zone::Deck => &self.deck[player.index()],
            Zone::Hand => &self.hand[player.index()],
            Zone::Play => &self.play[player.index()],
            Zone::Graveyard => &self.graveyard[player.index()],
            Zone::SetAside => &self.set_aside,
        }
    }

    /// 将一个实体插入到指定区域的末尾。
    pub fn insert(&mut self, zone: Zone, player: PlayerId, entity: crate::core::entity::Entity) {
        self.vec_mut(zone, player).push(entity);
    }

    /// 从指定区域移除一个实体（保持顺序）。
    ///
    /// 使用 `Vec::remove`（非 swap-remove）以保持手牌/战场顺序。
    pub fn remove(&mut self, zone: Zone, player: PlayerId, entity: crate::core::entity::Entity) {
        let vec = self.vec_mut(zone, player);
        if let Some(pos) = vec.iter().position(|&e| e == entity) {
            vec.remove(pos);
        }
    }

    /// 从所有区域尝试移除实体（用于 despawn）。
    ///
    /// 返回 `true` 如果确实移除了该实体。
    pub fn remove_from_all(&mut self, entity: crate::core::entity::Entity) -> bool {
        // 使用索引避免借用检查器问题（不能在数组字面量中同时 borrow 多个可变引用）
        let mut removed = false;
        for player_idx in 0..2usize {
            if let Some(pos) = self.deck[player_idx].iter().position(|&e| e == entity) {
                self.deck[player_idx].remove(pos);
                removed = true;
                break;
            }
            if let Some(pos) = self.hand[player_idx].iter().position(|&e| e == entity) {
                self.hand[player_idx].remove(pos);
                removed = true;
                break;
            }
            if let Some(pos) = self.play[player_idx].iter().position(|&e| e == entity) {
                self.play[player_idx].remove(pos);
                removed = true;
                break;
            }
            if let Some(pos) = self.graveyard[player_idx].iter().position(|&e| e == entity) {
                self.graveyard[player_idx].remove(pos);
                removed = true;
                break;
            }
        }
        if !removed {
            if let Some(pos) = self.set_aside.iter().position(|&e| e == entity) {
                self.set_aside.remove(pos);
                removed = true;
            }
        }
        removed
    }

    /// 遍历指定区域的所有实体（按顺序）。
    pub fn iter(
        &self,
        zone: Zone,
        player: PlayerId,
    ) -> impl Iterator<Item = crate::core::entity::Entity> + '_ {
        self.vec_ref(zone, player).iter().copied()
    }

    /// 返回指定区域的实体数量。
    #[must_use]
    pub fn len(&self, zone: Zone, player: PlayerId) -> usize {
        self.vec_ref(zone, player).len()
    }

    /// 返回 `true` 如果指定的区域为空。
    #[must_use]
    pub fn is_empty(&self, zone: Zone, player: PlayerId) -> bool {
        self.len(zone, player) == 0
    }

    /// 返回指定区域的所有实体（按顺序的拷贝）。
    #[must_use]
    pub fn entities(&self, zone: Zone, player: PlayerId) -> Vec<crate::core::entity::Entity> {
        self.vec_ref(zone, player).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::Entity;

    fn e(idx: u32) -> Entity {
        Entity::new(idx, 0)
    }

    #[test]
    fn insert_and_iter_hand() {
        let mut zones = Zones::new();
        zones.insert(Zone::Hand, PlayerId::Player1, e(1));
        zones.insert(Zone::Hand, PlayerId::Player1, e(2));
        let hand: Vec<_> = zones.iter(Zone::Hand, PlayerId::Player1).collect();
        assert_eq!(hand, vec![e(1), e(2)]);
    }

    #[test]
    fn remove_preserves_order() {
        let mut zones = Zones::new();
        zones.insert(Zone::Play, PlayerId::Player1, e(10));
        zones.insert(Zone::Play, PlayerId::Player1, e(20));
        zones.insert(Zone::Play, PlayerId::Player1, e(30));
        zones.remove(Zone::Play, PlayerId::Player1, e(20));
        let play: Vec<_> = zones.iter(Zone::Play, PlayerId::Player1).collect();
        assert_eq!(play, vec![e(10), e(30)]);
    }

    #[test]
    fn zones_are_per_player() {
        let mut zones = Zones::new();
        zones.insert(Zone::Hand, PlayerId::Player1, e(1));
        zones.insert(Zone::Hand, PlayerId::Player2, e(2));
        assert!(zones.iter(Zone::Hand, PlayerId::Player1).eq([e(1)]));
        assert!(zones.iter(Zone::Hand, PlayerId::Player2).eq([e(2)]));
    }

    #[test]
    fn remove_from_all_finds_entity() {
        let mut zones = Zones::new();
        zones.insert(Zone::Play, PlayerId::Player1, e(42));
        assert!(zones.remove_from_all(e(42)));
        assert!(zones.is_empty(Zone::Play, PlayerId::Player1));
    }

    #[test]
    fn remove_from_all_missing() {
        let mut zones = Zones::new();
        assert!(!zones.remove_from_all(e(99)));
    }
}
