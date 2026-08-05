//! 批量模拟 — 并行推进多个对局实例（RL 批量推理的前置）。
//!
//! RL 训练场景需要同时运行大量对局：每个 `GameState` 携带独立的 RNG，
//! 在 rayon 线程池上并行推进，互不干扰。每个对局的结果与单线程运行时
//! 完全一致（每局的 RNG 与动作序列决定结果，与线程调度无关）。

use crate::core::player::PlayerId;
use crate::core::state::{GameState, Phase};
use crate::engine::game::GameEngine;
use crate::sim::battle::{BattleRunner, BotDelegate, BotType};
use rayon::prelude::*;

/// 单局批量模拟的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchOutcome {
    /// 胜者（`None` 表示达到步数上限仍未分出胜负）
    pub winner: Option<PlayerId>,
    /// 实际执行的动作步数
    pub steps: u32,
    /// 结束时的回合数
    pub turn: u32,
}

/// 批量模拟器 — 在多个对局上并行运行机器人驱动的自对弈。
#[derive(Debug, Clone, Copy)]
pub struct BatchSimulator {
    bot_type: BotType,
    bot: BotDelegate,
    max_steps: u32,
}

impl BatchSimulator {
    /// 创建一个批量模拟器。
    ///
    /// `max_steps` — 每局的最大动作步数（防止死循环）。
    #[must_use]
    pub fn new(bot_type: BotType, max_steps: u32) -> Self {
        Self {
            bot_type,
            bot: BotDelegate::new(bot_type),
            max_steps,
        }
    }

    /// 并行推进所有对局，返回与输入顺序一致的结果列表。
    ///
    /// 每个对局独立推进；`par_iter` 保持输出顺序，因此给定相同输入
    /// 与每局 seed，结果完全可复现。
    pub fn run(&self, games: Vec<GameState>) -> Vec<BatchOutcome> {
        games
            .into_par_iter()
            .map(|mut state| self.run_one(&mut state))
            .collect()
    }

    /// 生成并并行运行 `count` 场完整对局。
    ///
    /// 每局使用独立种子（`seed + i`）生成随机牌组与对局 RNG，
    /// 互不相关。返回按序号排序的结果。
    pub fn run_battles(&self, seed: u64, deck_size: usize, count: usize) -> Vec<BatchOutcome> {
        let states: Vec<GameState> = (0..count)
            .map(|i| {
                let mut runner = BattleRunner::new(self.bot_type, seed.wrapping_add(i as u64));
                runner.create_game_state(deck_size)
            })
            .collect();
        self.run(states)
    }

    /// 推进单个对局直到结束或达到步数上限。
    fn run_one(&self, state: &mut GameState) -> BatchOutcome {
        let engine = GameEngine::new();
        let mut steps = 0u32;
        loop {
            if steps >= self.max_steps || matches!(state.phase(), Phase::GameOver { .. }) {
                break;
            }
            let actions = self.bot.decide_actions(state);
            if actions.is_empty() {
                // 无合法动作（无牌可出、无可攻击目标）— 回合无法推进
                break;
            }
            let mut applied = 0;
            for action in &actions {
                if engine.apply(state, *action).is_ok() {
                    steps += 1;
                    applied += 1;
                }
                if matches!(state.phase(), Phase::GameOver { .. }) {
                    break;
                }
            }
            // 本回合所有动作都被拒绝 → 状态不再变化，结束推进
            if applied == 0 {
                break;
            }
        }
        let winner = match state.phase() {
            Phase::GameOver { winner } => Some(winner),
            _ => None,
        };
        BatchOutcome {
            winner,
            steps,
            turn: state.turn(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::game::GameBuilder;

    #[test]
    fn batch_of_battles_all_finish() {
        let sim = BatchSimulator::new(BotType::Greedy, 5000);
        let outcomes = sim.run_battles(42, 30, 8);
        assert_eq!(outcomes.len(), 8);
        for (i, o) in outcomes.iter().enumerate() {
            assert!(o.steps > 0, "game {i} should make progress");
            assert!(o.turn >= 1);
            // 最多 5000 步内必然结束或有胜者
            if o.winner.is_none() {
                assert_eq!(o.steps, 5000, "game {i} hit the step cap");
            }
        }
    }

    #[test]
    fn batch_is_deterministic() {
        let sim = BatchSimulator::new(BotType::Greedy, 3000);
        let a = sim.run_battles(7, 20, 4);
        let b = sim.run_battles(7, 20, 4);
        assert_eq!(a, b, "same seeds must produce identical outcomes");
    }

    #[test]
    fn batch_states_are_independent() {
        // 同一模板克隆出的对局互相隔离（CoW）：推进一批不影响模板
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(
            crate::core::player::PlayerId::Player1,
            &crate::cards::def::BLOODFEN_RAPTOR,
        );
        let template = builder.build();
        let games = vec![template.clone(), template.clone(), template.clone()];

        let sim = BatchSimulator::new(BotType::Greedy, 200);
        let outcomes = sim.run(games);

        assert_eq!(outcomes.len(), 3);
        // 模板未被修改：手牌仍有一张牌，英雄 30 HP
        let world = template.world();
        assert_eq!(
            world.health(template.player(PlayerId::Player1).hero),
            Some(crate::core::component::Health(30))
        );
        assert_eq!(
            world
                .zones()
                .len(crate::core::zone::Zone::Hand, PlayerId::Player1),
            1
        );
    }
}
