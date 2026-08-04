//! GameEngine — action→event→resolve 循环的编排层。
//!
//! `GameEngine` 是一个无状态的单元结构体，负责：
//! 1. 调用 `rules::validate` 检查合法性
//! 2. 调用 `rules::enqueue` 生成初始事件
//! 3. 循环调用 `rules::apply_event` 直到事件队列为空
//! 4. 返回完整的事件日志（用于回放和调试）
//!
//! # 示例
//!
//! ```rust
//! use orange_stone::engine::game::GameEngine;
//! use orange_stone::core::state::GameState;
//! use orange_stone::core::action::Action;
//!
//! let engine = GameEngine::new();
//! let mut state = GameState::new();
//! let result = engine.apply(&mut state, Action::EndTurn);
//! assert!(result.is_ok());
//! ```

use crate::core::action::Action;
use crate::core::event::{Event, EventQueue};
use crate::core::state::GameState;
use crate::engine::rules::{self, EngineError};

/// 游戏引擎 — 无状态，纯逻辑编排。
#[derive(Debug, Default, Clone, Copy)]
pub struct GameEngine;

impl GameEngine {
    /// 创建一个新的游戏引擎实例。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 验证、入队并完全解析一个玩家动作。
    ///
    /// 返回按解析顺序排列的完整事件日志（可用于回放）。
    ///
    /// # 错误
    ///
    /// 如果验证失败，返回 `EngineError`，状态**不会**被修改。
    pub fn apply(&self, state: &mut GameState, action: Action) -> Result<Vec<Event>, EngineError> {
        // 1. 验证（只读）
        rules::validate(state, action)?;

        // 2. Action → 初始事件
        let mut queue = EventQueue::new();
        rules::enqueue(state, action, &mut queue)?;

        // 3. 事件循环
        let mut log = Vec::new();
        while let Some(event) = queue.pop_front() {
            rules::apply_event(state, event, &mut queue)?;
            log.push(event);
        }

        Ok(log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::action::Action;
    use crate::core::component::{Attack, AttacksUsed, CardType, Health};
    use crate::core::player::PlayerId;
    use crate::core::state::GameState;
    use crate::core::zone::Zone;

    fn add_minion_to_board(
        state: &mut GameState,
        player: PlayerId,
        atk: i32,
        hp: i32,
    ) -> crate::core::entity::Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(0));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        e
    }

    fn add_minion_to_hand(
        state: &mut GameState,
        player: PlayerId,
        atk: i32,
        hp: i32,
    ) -> crate::core::entity::Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(3));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        e
    }

    #[test]
    fn play_card_produces_events() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);

        let log = engine.apply(&mut state, Action::PlayCard { card }).unwrap();

        // 应该产生 CardPlayed 和 MinionSummoned
        assert_eq!(log.len(), 2);
        assert!(matches!(log[0], Event::CardPlayed { .. }));
        assert!(matches!(log[1], Event::MinionSummoned { .. }));

        // 卡牌应该在战场上
        assert_eq!(state.world().zone(card), Some(Zone::Play));
    }

    #[test]
    fn end_turn_switches_player() {
        let engine = GameEngine::new();
        let mut state = GameState::new();

        let log = engine.apply(&mut state, Action::EndTurn).unwrap();

        assert!(matches!(log[0], Event::TurnEnded { .. }));
        assert!(matches!(log[1], Event::TurnStarted { .. }));
        assert_eq!(state.active_player(), PlayerId::Player2);
        assert_eq!(state.turn(), 2);
    }

    #[test]
    fn illegal_action_preserves_state() {
        let engine = GameEngine::new();
        let mut state = GameState::new();

        // 试图攻击一个不存在的实体
        let result = engine.apply(
            &mut state,
            Action::Attack {
                attacker: crate::core::entity::Entity::new(999, 0),
                defender: crate::core::entity::Entity::new(998, 0),
            },
        );

        assert!(result.is_err());
        // 状态应该保持不变
        assert_eq!(state.turn(), 1);
        assert_eq!(state.phase(), crate::core::state::Phase::Main);
    }

    #[test]
    fn hero_death_ends_game() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let log = engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: hero,
                },
            )
            .unwrap();

        // 最后的 event 应该是 GameOver
        let last_event = log.last().unwrap();
        assert!(matches!(last_event, Event::GameOver { .. }));
        assert_eq!(
            state.phase(),
            crate::core::state::Phase::GameOver {
                winner: PlayerId::Player1
            }
        );
    }

    #[test]
    fn game_over_rejects_further_actions() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: hero,
                },
            )
            .unwrap();

        // 游戏已结束，下一步操作应该被拒绝
        let result = engine.apply(&mut state, Action::EndTurn);
        assert_eq!(result, Err(EngineError::GameAlreadyOver));
    }

    #[test]
    fn hero_power_unimplemented() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let hero = state.player(PlayerId::Player1).hero;
        let result = engine.apply(&mut state, Action::HeroPower { hero });
        assert_eq!(result, Err(EngineError::Unimplemented));
    }
}
