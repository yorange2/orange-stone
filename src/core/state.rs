//! GameState — 不可变游戏状态 + Copy-on-Write。
//!
//! `GameState` 将实际的游戏数据包裹在 `Arc<Inner>` 中。
//! Clone 是 O(1) 的引用计数增加，创建分支（如 MCTS 搜索）几乎零开销。
//! 首次修改通过 `Arc::make_mut` 进行克隆（仅在引用被共享时），
//! 未被共享时的修改则在原数据上进行。
//!
//! # 示例
//!
//! ```rust
//! use orange_stone::core::state::GameState;
//!
//! let mut parent = GameState::new();
//! let mut branch = parent.clone();  // Arc refcount bump
//! // 对 branch 修改时自动 CoW，parent 不受影响
//! ```

use crate::core::component::{Attack, AttacksUsed, CardType, Cost, Health};
use crate::core::player::{Player, PlayerId};
use crate::core::world::World;
use crate::core::zone::Zone;
use crate::sim::rng::GameRng;
use std::sync::Arc;

/// 游戏阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// 主要阶段 — 玩家可以出牌、攻击、结束回合
    Main,
    /// 回合结束阶段（Phase 1 中仅作标记）
    End,
    /// 游戏结束
    GameOver {
        /// 获胜方
        winner: PlayerId,
    },
}

/// 游戏状态的内部数据 — 通过 `Arc` 共享。
///
/// 包含完整的 World、玩家信息和游戏元数据。
#[derive(Debug, Clone)]
pub struct Inner {
    /// ECS World（实体和组件）
    pub world: World,
    /// 两个玩家的状态
    pub players: [Player; 2],
    /// 当前回合数（从 1 开始计数，每次 TurnStarted 递增）
    pub turn: u32,
    /// 当前游戏阶段
    pub phase: Phase,
    /// 当前行动玩家
    pub active_player: PlayerId,
    /// 随机数生成器（可复现）
    pub rng: GameRng,
}

/// 不可变游戏状态，支持 Copy-on-Write。
///
/// Clone 是 O(1)。变异通过内部的 `Arc::make_mut` 触发 CoW：
/// - 若仅有一个引用 → 原地修改
/// - 若被多个引用共享 → 克隆 Inner 后再修改
#[derive(Debug, Clone)]
pub struct GameState {
    inner: Arc<Inner>,
}

impl GameState {
    /// 创建一个新的初始游戏状态。
    ///
    /// 包含两个玩家及其英雄实体（30 HP, 0 Attack），每人法力水晶初始为 0。
    /// 牌库初始为空（通过 GameBuilder 填充）。
    /// 游戏从 Player1 的回合 1 开始，阶段为 Main。
    /// RNG seed 固定为 12345。
    #[must_use]
    pub fn new() -> Self {
        let mut world = World::new();

        // 创建英雄实体
        let hero1 = world.spawn();
        world.set_health(hero1, Health(30));
        world.set_attack(hero1, Attack(0));
        world.set_cost(hero1, Cost(0));
        world.set_card_type(hero1, CardType::Hero);
        world.set_player(hero1, PlayerId::Player1);
        world.set_attacks_used(hero1, AttacksUsed(0));
        world.set_zone(hero1, Zone::Play);
        world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player1, hero1);

        let hero2 = world.spawn();
        world.set_health(hero2, Health(30));
        world.set_attack(hero2, Attack(0));
        world.set_cost(hero2, Cost(0));
        world.set_card_type(hero2, CardType::Hero);
        world.set_player(hero2, PlayerId::Player2);
        world.set_attacks_used(hero2, AttacksUsed(0));
        world.set_zone(hero2, Zone::Play);
        world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player2, hero2);

        let inner = Inner {
            world,
            players: [
                Player::new(PlayerId::Player1, hero1, 0),
                Player::new(PlayerId::Player2, hero2, 0),
            ],
            turn: 1,
            phase: Phase::Main,
            active_player: PlayerId::Player1,
            rng: GameRng::new(12345),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    /// 获取 World 的只读引用（共享访问，无开销）。
    #[must_use]
    pub fn world(&self) -> &World {
        &self.inner.world
    }

    /// 获取指定玩家的状态。
    #[must_use]
    pub fn player(&self, id: PlayerId) -> &Player {
        &self.inner.players[id.index()]
    }

    /// 获取当前游戏阶段。
    #[must_use]
    pub fn phase(&self) -> Phase {
        self.inner.phase
    }

    /// 设置游戏阶段。
    /// 需要通过 `make_mut` 获取可变引用（由 `GameEngine` 或 `GameBuilder` 调用）。
    pub fn set_phase(&mut self, phase: Phase) {
        let inner = self.make_mut();
        inner.phase = phase;
    }

    /// 获取当前行动玩家。
    #[must_use]
    pub fn active_player(&self) -> PlayerId {
        self.inner.active_player
    }

    /// 设置当前行动玩家。
    pub fn set_active_player(&mut self, player: PlayerId) {
        self.make_mut().active_player = player;
    }

    /// 获取当前回合数。
    #[must_use]
    pub fn turn(&self) -> u32 {
        self.inner.turn
    }

    /// 设置当前回合数。
    pub fn set_turn(&mut self, turn: u32) {
        self.make_mut().turn = turn;
    }

    /// 获取 RNG 的只读引用。
    #[must_use]
    pub fn rng(&self) -> &GameRng {
        &self.inner.rng
    }

    /// 获取 RNG 的可变引用（触发 CoW）。
    #[must_use]
    pub fn rng_mut(&mut self) -> &mut GameRng {
        &mut self.make_mut().rng
    }

    /// 获取 Inner 的可变引用，触发 CoW。
    ///
    /// 如果 `Arc` 被共享（`strong_count > 1`），`Arc::make_mut` 会克隆
    /// 整个 `Inner` 然后返回对克隆体的独占引用。
    /// 如果只有一个引用，则直接返回原地数据。
    ///
    /// 此方法是 CoW 的核心。调用者应注意：
    /// - 在一次事件处理中将多个变更合并到一个 `make_mut` 调用下
    /// - 先完成所有只读操作，再调用 `make_mut`（避免借用冲突）
    #[must_use]
    pub fn make_mut(&mut self) -> &mut Inner {
        Arc::make_mut(&mut self.inner)
    }

    /// 获取 World 的可变引用（便捷方法，触发 CoW）。
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.make_mut().world
    }

    /// 返回当前 Inner 的 Arc 引用计数（用于测试和调试）。
    #[cfg(test)]
    #[must_use]
    fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// 获取共享 Inner 的只读引用（用于内部比较和测试）。
    #[cfg(test)]
    fn inner_ref(&self) -> &Inner {
        &self.inner
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_two_heroes() {
        let state = GameState::new();
        let world = state.world();
        let hero1 = state.player(PlayerId::Player1).hero;
        let hero2 = state.player(PlayerId::Player2).hero;
        assert_eq!(world.health(hero1), Some(Health(30)));
        assert_eq!(world.health(hero2), Some(Health(30)));
        assert_eq!(world.attack(hero1), Some(Attack(0)));
        assert_eq!(world.card_type(hero1), Some(CardType::Hero));
    }

    #[test]
    fn new_state_initial_values() {
        let state = GameState::new();
        assert_eq!(state.turn(), 1);
        assert_eq!(state.phase(), Phase::Main);
        assert_eq!(state.active_player(), PlayerId::Player1);
    }

    #[test]
    fn clone_is_independent() {
        let a = GameState::new();
        let mut b = a.clone();

        // 两者共享同一个 Arc，refcount 都是 2
        assert_eq!(a.ref_count(), 2);
        assert_eq!(b.ref_count(), 2);

        // 修改 b
        b.set_turn(5);
        assert_eq!(b.turn(), 5);
        // a 不受影响
        assert_eq!(a.turn(), 1);

        // 再次修改 b
        b.set_turn(10);
        assert_eq!(b.turn(), 10);
    }

    #[test]
    fn clone_then_modify_parent_still_independent() {
        let mut parent = GameState::new();
        let mut child = parent.clone();

        parent.set_turn(3);
        child.set_turn(7);

        assert_eq!(parent.turn(), 3);
        assert_eq!(child.turn(), 7);
    }

    #[test]
    fn siblings_diverge_independently() {
        let mut parent = GameState::new();
        let mut sibling_a = parent.clone();
        let mut sibling_b = parent.clone();

        sibling_a.set_turn(2);
        sibling_b.set_turn(3);
        parent.set_turn(4);

        assert_eq!(sibling_a.turn(), 2);
        assert_eq!(sibling_b.turn(), 3);
        assert_eq!(parent.turn(), 4);
    }

    #[test]
    fn second_mutation_no_reclone() {
        // 独占引用时不应重复克隆
        let mut state = GameState::new();
        assert_eq!(state.ref_count(), 1);

        // 第一次修改
        let inner_ptr_before: *const Inner = state.inner_ref();
        state.set_turn(2);
        let inner_ptr_after: *const Inner = state.inner_ref();
        // 独占引用时，Arc::make_mut 不应该分配新的内存
        assert_eq!(inner_ptr_before, inner_ptr_after);

        // 第二次修改
        state.set_turn(3);
        let inner_ptr_final: *const Inner = state.inner_ref();
        assert_eq!(inner_ptr_after, inner_ptr_final);
    }

    #[test]
    fn world_mutation_through_state() {
        let mut state = GameState::new();
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().set_health(hero, Health(25));
        assert_eq!(state.world().health(hero), Some(Health(25)));
    }

    #[test]
    fn clone_preserves_world_snapshot() {
        let mut state = GameState::new();
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().set_health(hero, Health(20));

        // 克隆快照
        let snapshot = state.clone();

        // 继续修改原状态
        state.world_mut().set_health(hero, Health(10));

        // 快照不受影响
        assert_eq!(state.world().health(hero), Some(Health(10)));
        assert_eq!(snapshot.world().health(hero), Some(Health(20)));
    }
}
