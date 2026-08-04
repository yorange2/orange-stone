//! Action — 玩家可以执行的操作。
//!
//! 每个 Action 代表玩家在游戏中可以做出的一个决定。
//! Action 提交给 `GameEngine::apply`，引擎负责验证和执行。

use crate::core::entity::Entity;

/// 玩家动作 — 游戏中的合法操作。
///
/// Phase 1 支持：出牌（白板随从）、攻击、结束回合。
/// HeroPower 在 Phase 1 中返回 `EngineError::Unimplemented`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// 打出一张随从牌（从手牌到战场）。
    PlayCard {
        /// 要打出的卡牌实体
        card: Entity,
    },
    /// 用己方随从/英雄攻击敌方随从/英雄。
    Attack {
        /// 攻击方实体
        attacker: Entity,
        /// 防御方实体（敌方随从或敌方英雄）
        defender: Entity,
    },
    /// 结束当前玩家的回合。
    EndTurn,
    /// 使用英雄技能（Phase 2+ 实现）。
    HeroPower {
        /// 使用技能的英雄实体
        hero: Entity,
    },
}
