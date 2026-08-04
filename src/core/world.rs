//! World — ECS 容器，管理所有实体和组件。
//!
//! World 是实体生命周期的唯一入口：
//! - `spawn()` 创建实体（分配槽位）
//! - `despawn()` 销毁实体（释放槽位，清除所有组件和区域引用）
//! - `move_to_zone()` 原子性地将实体从一个区域移动到另一个区域
//! - 组件通过生成的 accessor 方法访问
//!
//! 所有实体访问都经过 generation 检查，防止悬垂引用。

use crate::core::component::{
    Attack, AttacksUsed, Battlecry, CardType, Cost, Deathrattle, Health, Taunt,
};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::sparse_set::SparseSet;
use crate::core::zone::{Zone, ZoneError, Zones};

/// 区域移动错误 — move_to_zone 的可能失败模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// 实体已销毁
    EntityGone,
    /// 缺少 PlayerId 组件
    MissingPlayer,
    /// 缺少当前 Zone 组件
    MissingZone,
}

impl From<ZoneError> for MoveError {
    fn from(e: ZoneError) -> Self {
        match e {
            ZoneError::EntityGone => Self::EntityGone,
            ZoneError::MissingPlayer => Self::MissingPlayer,
            ZoneError::MissingZone => Self::MissingZone,
        }
    }
}

/// 生成组件访问方法的宏。
///
/// 为每种组件类型生成 get/set/remove/iter 四个方法。
macro_rules! component_accessors {
    ($field:ident, $t:ty, $get:ident, $set:ident, $remove:ident, $iter:ident) => {
        #[doc = concat!("获取实体的 `", stringify!($t), "` 组件。")]
        #[must_use]
        pub fn $get(&self, entity: Entity) -> Option<$t> {
            self.$field.get(entity)
        }

        #[doc = concat!("设置实体的 `", stringify!($t), "` 组件。")]
        pub fn $set(&mut self, entity: Entity, value: impl Into<$t>) {
            self.$field.insert(entity, value.into());
        }

        #[doc = concat!("移除实体的 `", stringify!($t), "` 组件。")]
        pub fn $remove(&mut self, entity: Entity) -> Option<$t> {
            self.$field.remove(entity)
        }

        #[doc = concat!("遍历所有拥有 `", stringify!($t), "` 组件的实体。")]
        pub fn $iter(&self) -> impl Iterator<Item = (Entity, &$t)> {
            self.$field.iter()
        }
    };
}

/// ECS World — 所有实体和组件的容器。
///
/// # 内部结构
///
/// - `generations`: 每个槽位的代际版本号（despawn 时递增）
/// - `free_list`: 可复用的空闲槽位（FIFO）
/// - 10 个组件稀疏集 + Zones 表
#[derive(Debug, Clone)]
pub struct World {
    /// 每个槽位的代际版本号，用于检测过期 Entity handle
    generations: Vec<u32>,
    /// 可复用的空闲槽位索引
    free_list: Vec<u32>,
    /// Health 组件存储
    health: SparseSet<Health>,
    /// Attack 组件存储
    attack: SparseSet<Attack>,
    /// Cost 组件存储
    cost: SparseSet<Cost>,
    /// CardType 组件存储
    card_type: SparseSet<CardType>,
    /// Zone 组件存储（实体的当前位置）
    zone_comp: SparseSet<Zone>,
    /// PlayerId 组件存储
    player: SparseSet<PlayerId>,
    /// AttacksUsed 组件存储
    attacks_used: SparseSet<AttacksUsed>,
    /// Battlecry 组件存储
    battlecry: SparseSet<Battlecry>,
    /// Deathrattle 组件存储
    deathrattle: SparseSet<Deathrattle>,
    /// Taunt 组件存储
    taunt: SparseSet<Taunt>,
    /// 区域表 — 每个 Zone 的有序实体列表
    zones: Zones,
}

impl World {
    /// 创建一个空的世界。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            health: SparseSet::new(),
            attack: SparseSet::new(),
            cost: SparseSet::new(),
            card_type: SparseSet::new(),
            zone_comp: SparseSet::new(),
            player: SparseSet::new(),
            attacks_used: SparseSet::new(),
            battlecry: SparseSet::new(),
            deathrattle: SparseSet::new(),
            taunt: SparseSet::new(),
            zones: Zones::new(),
        }
    }

    /// 生成一个新的实体并返回其句柄。
    ///
    /// 优先复用空闲槽位，否则扩展数组。
    pub fn spawn(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            let generation = self.generations[index as usize];
            Entity::new(index, generation)
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            Entity::new(index, 0)
        }
    }

    /// 检查实体是否还存活（generation 匹配）。
    #[must_use]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    /// 销毁实体：清除所有组件，从所有区域移除，递增 generation，归还槽位。
    ///
    /// Phase 1 中 despawn 仅用于清理（测试等场景）。
    /// 游戏内死亡使用 `move_to_zone(entity, Zone::Graveyard)` 而非 despawn。
    pub fn despawn(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        let idx = entity.index as usize;
        // 从所有区域移除
        self.zones.remove_from_all(entity);
        // 清除所有组件
        self.health.remove(entity);
        self.attack.remove(entity);
        self.cost.remove(entity);
        self.card_type.remove(entity);
        self.zone_comp.remove(entity);
        self.player.remove(entity);
        self.attacks_used.remove(entity);
        self.battlecry.remove(entity);
        self.deathrattle.remove(entity);
        self.taunt.remove(entity);
        // 提升 generation
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        // 归还槽位
        self.free_list.push(entity.index);
    }

    /// 将实体从一个区域移动到另一个区域。
    ///
    /// 这是区域转移的**唯一入口**，确保 Zone 组件和 Zones 表保持同步。
    ///
    /// # 错误
    ///
    /// - `MoveError::EntityGone` — 实体已销毁
    /// - `MoveError::MissingPlayer` — 缺少 PlayerId 组件，无法判断所属玩家
    /// - `MoveError::MissingZone` — 缺少当前 Zone 组件（状态不一致）
    pub fn move_to_zone(&mut self, entity: Entity, target: Zone) -> Result<(), MoveError> {
        if !self.is_alive(entity) {
            return Err(MoveError::EntityGone);
        }
        let player = self.player(entity).ok_or(MoveError::MissingPlayer)?;
        let current = self.zone(entity).ok_or(MoveError::MissingZone)?;

        // 从旧区域移除
        self.zones.remove(current, player, entity);
        // 插入新区域
        self.zones.insert(target, player, entity);
        // 更新 Zone 组件
        self.set_zone(entity, target);

        Ok(())
    }

    /// 获取 Zones 表的只读引用。
    #[must_use]
    pub fn zones(&self) -> &Zones {
        &self.zones
    }

    /// 获取 Zones 表的可变引用（用于测试/GameBuilder 直接操作区域）。
    ///
    /// ⚠️ 直接操作 Zones 表需要同时更新 Zone 组件，否则状态不一致。
    /// 优先使用 `move_to_zone`。
    pub fn zones_mut(&mut self) -> &mut Zones {
        &mut self.zones
    }

    // 为每种组件类型生成 accessor 方法
    component_accessors!(
        health,
        Health,
        health,
        set_health,
        remove_health,
        iter_health
    );
    component_accessors!(
        attack,
        Attack,
        attack,
        set_attack,
        remove_attack,
        iter_attack
    );
    component_accessors!(cost, Cost, cost, set_cost, remove_cost, iter_cost);
    component_accessors!(
        card_type,
        CardType,
        card_type,
        set_card_type,
        remove_card_type,
        iter_card_type
    );
    component_accessors!(zone_comp, Zone, zone, set_zone, remove_zone, iter_zone);
    component_accessors!(
        player,
        PlayerId,
        player,
        set_player,
        remove_player,
        iter_player
    );
    component_accessors!(
        attacks_used,
        AttacksUsed,
        attacks_used,
        set_attacks_used,
        remove_attacks_used,
        iter_attacks_used
    );
    component_accessors!(
        battlecry,
        Battlecry,
        battlecry,
        set_battlecry,
        remove_battlecry,
        iter_battlecry
    );
    component_accessors!(
        deathrattle,
        Deathrattle,
        deathrattle,
        set_deathrattle,
        remove_deathrattle,
        iter_deathrattle
    );
    component_accessors!(taunt, Taunt, taunt, set_taunt, remove_taunt, iter_taunt);
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_entity() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        assert_eq!(e.index, 0);
        assert_eq!(e.generation, 0);
    }

    #[test]
    fn stale_handle_after_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e);
        assert!(!world.is_alive(e));
    }

    #[test]
    fn slot_reuse_bumps_generation() {
        let mut world = World::new();
        let e1 = world.spawn();
        assert_eq!(e1.index, 0);
        assert_eq!(e1.generation, 0);
        world.despawn(e1);

        // 旧句柄应失效
        assert!(!world.is_alive(e1));

        // 复用槽位，generation 应该不同
        let e2 = world.spawn();
        assert_eq!(e2.index, 0);
        assert_eq!(e2.generation, 1); // generation bump
        assert!(world.is_alive(e2));
        assert!(!world.is_alive(e1)); // 旧句柄仍然失效
    }

    #[test]
    fn component_set_and_get() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_health(e, Health(30));
        world.set_attack(e, Attack(0));
        assert_eq!(world.health(e), Some(Health(30)));
        assert_eq!(world.attack(e), Some(Attack(0)));
    }

    #[test]
    fn component_missing_after_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_health(e, Health(5));
        world.despawn(e);
        // 注意：组件被清除但需要检查 is_alive 语义
        // despawn 后 generational check，但组件 remove 也是通过 generation
        // 由于 generation 已变，旧句柄查不到组件
        assert_eq!(world.health(e), None);
    }

    #[test]
    fn move_to_zone_consistency() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_player(e, PlayerId::Player1);
        world.set_zone(e, Zone::Hand);
        world.zones.insert(Zone::Hand, PlayerId::Player1, e);

        // 移到战场
        world.move_to_zone(e, Zone::Play).unwrap();
        assert_eq!(world.zone(e), Some(Zone::Play));
        assert!(world.zones.is_empty(Zone::Hand, PlayerId::Player1));
        let play_entities: Vec<_> = world.zones.iter(Zone::Play, PlayerId::Player1).collect();
        assert_eq!(play_entities, vec![e]);
    }

    #[test]
    fn move_to_zone_maintains_order() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        for &(e, pid) in &[(e1, PlayerId::Player1), (e2, PlayerId::Player1)] {
            world.set_player(e, pid);
            world.set_zone(e, Zone::Hand);
            world.set_health(e, Health(0));
            world.zones.insert(Zone::Hand, pid, e);
        }
        // 按顺序移到战场
        world.move_to_zone(e1, Zone::Play).unwrap();
        world.move_to_zone(e2, Zone::Play).unwrap();
        let play: Vec<_> = world.zones.iter(Zone::Play, PlayerId::Player1).collect();
        assert_eq!(play, vec![e1, e2]);
    }

    #[test]
    fn despawn_clears_zones() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_player(e, PlayerId::Player1);
        world.set_zone(e, Zone::Play);
        world.zones.insert(Zone::Play, PlayerId::Player1, e);

        world.despawn(e);
        assert!(world.zones.is_empty(Zone::Play, PlayerId::Player1));
    }

    #[test]
    fn iter_components() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.set_health(e1, Health(10));
        let e2 = world.spawn();
        world.set_health(e2, Health(20));
        let e3 = world.spawn();
        world.set_health(e3, Health(30));

        let mut healths: Vec<_> = world.iter_health().map(|(e, h)| (e.index, h.0)).collect();
        healths.sort_by_key(|(i, _)| *i);
        assert_eq!(healths, vec![(0, 10), (1, 20), (2, 30)]);
    }
}
