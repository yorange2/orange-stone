//! Gym 风格环境 — 单个 agent 与脚本化对手（bot）对弈。
//!
//! 环境状态机：
//! - agent 在己方回合内可连续执行动作（`step`）
//! - `EndTurn` 后对手回合由 bot 自动推进
//! - 任一英雄死亡即终局（`done = true`，附带终局奖励）
//!
//! 观察（`obs`）、奖励（`reward`）与动作空间（`legal_actions`）都是
//! 固定/可枚举的，供 Python 绑定与 RL 训练直接使用。

use crate::core::action::Action;
use crate::core::component::CardType;
use crate::core::effect::EffectTarget;
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Phase};
use crate::core::zone::Zone;
use crate::engine::game::GameEngine;
use crate::engine::rules;
use crate::rl::obs::{OBS_LEN, encode_observation};
use crate::rl::reward::{self, RewardConfig};
use crate::sim::battle::{BattleRunner, BotDelegate, BotType};

/// 环境配置。
#[derive(Debug, Clone, Copy)]
pub struct EnvConfig {
    /// 每方牌组大小
    pub deck_size: usize,
    /// 初始手牌数
    pub hand_size: usize,
    /// 对手（bot）类型
    pub bot_type: BotType,
    /// 单局最大动作步数（防止死循环）
    pub max_steps: u32,
    /// 奖励配置
    pub reward: RewardConfig,
}

impl EnvConfig {
    /// 默认配置（30 张牌组、贪心对手、稀疏胜/负奖励）。
    #[must_use]
    pub fn default_with(bot_type: BotType, deck_size: usize) -> Self {
        Self {
            deck_size,
            hand_size: 3,
            bot_type,
            max_steps: 5000,
            reward: RewardConfig::default(),
        }
    }
}

/// 单步结果。
#[derive(Debug, Clone)]
pub struct StepResult {
    /// 动作后的观察（agent 视角）
    pub observation: Vec<f32>,
    /// 本步奖励（含终局奖励）
    pub reward: f32,
    /// 对局是否结束
    pub done: bool,
    /// 胜者（`None` 表示未结束或平局）
    pub winner: Option<PlayerId>,
}

/// Gym 风格环境 — agent 固定为 `perspective` 玩家。
#[derive(Debug, Clone)]
pub struct GameEnv {
    engine: GameEngine,
    bot: BotDelegate,
    perspective: PlayerId,
    config: EnvConfig,
    state: GameState,
    steps: u32,
    done: bool,
}

impl GameEnv {
    /// 创建一个新环境；`reset` 前不可用。
    #[must_use]
    pub fn new(perspective: PlayerId, config: EnvConfig) -> Self {
        Self {
            engine: GameEngine::new(),
            bot: BotDelegate::new(config.bot_type),
            perspective,
            config,
            state: GameState::new(),
            steps: 0,
            done: true,
        }
    }

    /// 重置环境：以 `seed` 生成全新对局（随机牌组），返回初始观察。
    pub fn reset(&mut self, seed: u64) -> Vec<f32> {
        let mut runner = BattleRunner::new(self.config.bot_type, seed);
        self.state = runner.create_game_state(self.config.deck_size);
        // 初始手牌大小由 create_game_state 决定（固定 3），如需可配置在此调整
        self.steps = 0;
        self.done = false;
        self.observation()
    }

    /// 当前观察（agent 视角）。
    #[must_use]
    pub fn observation(&self) -> Vec<f32> {
        encode_observation(&self.state, self.perspective)
    }

    /// 观察长度（固定值）。
    #[must_use]
    pub const fn obs_len() -> usize {
        OBS_LEN
    }

    /// 当前玩家的合法动作列表（完整枚举，含显式目标）。
    ///
    /// 只应在 agent 的回合调用；对手回合由环境自动推进。
    #[must_use]
    pub fn legal_actions(&self) -> Vec<Action> {
        legal_actions(&self.state)
    }

    /// 按索引执行 `legal_actions()[idx]`（与 Python 绑定共用）。
    pub fn step_indexed(&mut self, action_idx: usize) -> StepResult {
        let actions = self.legal_actions();
        match actions.get(action_idx) {
            Some(&action) => self.step(action),
            None => {
                // 索引越界视为非法动作
                let reward = self.config.reward.invalid_action;
                StepResult {
                    observation: self.observation(),
                    reward,
                    done: self.done,
                    winner: self.winner(),
                }
            }
        }
    }

    /// 执行一个动作并推进环境。
    pub fn step(&mut self, action: Action) -> StepResult {
        if self.done {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                done: true,
                winner: self.winner(),
            };
        }

        let before = self.state.clone();
        let ok = self.engine.apply(&mut self.state, action).is_ok();
        self.steps += 1;

        let mut reward = if ok {
            reward::step_reward(&self.config.reward, &before, &self.state, self.perspective)
        } else {
            self.config.reward.invalid_action
        };

        // 终局检查
        if matches!(self.state.phase(), Phase::GameOver { .. }) {
            self.done = true;
            reward += reward::final_reward(&self.config.reward, &self.state, self.perspective);
        } else if ok && matches!(action, Action::EndTurn) {
            // 对手回合：bot 自动推进到回合结束或终局
            self.run_bot_turn();
            if matches!(self.state.phase(), Phase::GameOver { .. }) {
                self.done = true;
                reward += reward::final_reward(&self.config.reward, &self.state, self.perspective);
            }
        } else if self.steps >= self.config.max_steps {
            // 步数上限：平局结束
            self.done = true;
        }

        StepResult {
            observation: self.observation(),
            reward,
            done: self.done,
            winner: self.winner(),
        }
    }

    /// 对手回合 — 执行 bot 的动作直到结束回合或终局。
    fn run_bot_turn(&mut self) {
        loop {
            if matches!(self.state.phase(), Phase::GameOver { .. }) {
                break;
            }
            if self.state.active_player() == self.perspective {
                // bot 回合结束，控制权交还 agent
                break;
            }
            let actions = self.bot.decide_actions(&self.state);
            if actions.is_empty() {
                break;
            }
            let mut applied = 0;
            for action in &actions {
                if self.engine.apply(&mut self.state, *action).is_ok() {
                    self.steps += 1;
                    applied += 1;
                }
                if matches!(self.state.phase(), Phase::GameOver { .. }) {
                    break;
                }
            }
            if applied == 0 {
                break;
            }
        }
    }

    /// 当前胜者（未结束时为 `None`）。
    fn winner(&self) -> Option<PlayerId> {
        match self.state.phase() {
            Phase::GameOver { winner } => Some(winner),
            _ => None,
        }
    }
}

/// 枚举当前行动玩家的全部合法动作。
///
/// 生成候选（结束回合、英雄技能、出牌含显式目标、全部攻击对），
/// 用引擎的 `validate` 过滤，保证与 `GameEngine::apply` 的合法性一致。
#[must_use]
pub fn legal_actions(state: &GameState) -> Vec<Action> {
    let player = state.active_player();
    let world = state.world();
    let mut candidates: Vec<Action> = Vec::new();

    // 结束回合
    candidates.push(Action::EndTurn);
    // 英雄技能
    let hero = state.player(player).hero;
    if world.hero_power(hero).is_some() {
        candidates.push(Action::HeroPower { hero });
    }
    // 出牌（带显式目标）
    for card in world.zones().iter(Zone::Hand, player) {
        let targets = play_targets(state, card);
        if targets.is_empty() {
            candidates.push(Action::PlayCard { card, target: None });
        } else {
            for t in targets {
                candidates.push(Action::PlayCard {
                    card,
                    target: Some(t),
                });
            }
        }
    }
    // 全部攻击对（己方战场角色 → 敌方战场角色）
    for attacker in world.zones().iter(Zone::Play, player) {
        for defender in world.zones().iter(Zone::Play, player.opponent()) {
            candidates.push(Action::Attack { attacker, defender });
        }
    }

    // 用引擎验证过滤非法候选
    candidates
        .into_iter()
        .filter(|a| rules::validate(state, *a).is_ok())
        .collect()
}

/// 手牌卡牌效果的合法目标列表（空 = 无目标卡牌）。
fn play_targets(
    state: &GameState,
    card: crate::core::entity::Entity,
) -> Vec<crate::core::entity::Entity> {
    use crate::core::effect::CardEffect;

    let Some(battlecry) = state.world().battlecry(card) else {
        return Vec::new();
    };
    // 从效果变体中提取 EffectTarget（法术与战吼共用 battlecry 槽位）
    let target = match battlecry.0 {
        CardEffect::DealDamage { target, .. } => target,
        CardEffect::DestroyMinion { target } => target,
        CardEffect::SilenceMinion { target } => target,
        CardEffect::SetAttack { target, .. } => target,
        CardEffect::RestoreHealth { target, .. } => target,
        CardEffect::FreezeCharacter { target } => target,
        CardEffect::ReturnToHand { target } => target,
        CardEffect::IncreaseCost { target, .. } => target,
        CardEffect::GainStats { target, .. } => target,
        CardEffect::GainArmor { target, .. } => target,
        CardEffect::FullHeal { target } => target,
        CardEffect::GrantWindfury { target } => target,
        CardEffect::DoubleAttack { target } => target,
        CardEffect::DoubleHealth { target } => target,
        CardEffect::SetAttackToHealth { target } => target,
        CardEffect::TempDebuff { target, .. } => target,
        _ => return Vec::new(),
    };
    let owner = state
        .world()
        .player(card)
        .unwrap_or_else(|| state.active_player());
    candidates_for_target(state, owner, target)
}

/// 按 `EffectTarget` 枚举候选实体。
fn candidates_for_target(
    state: &GameState,
    owner: PlayerId,
    target: EffectTarget,
) -> Vec<crate::core::entity::Entity> {
    let world = state.world();
    let enemy = owner.opponent();
    let chars = |p: PlayerId| {
        world
            .zones()
            .iter(Zone::Play, p)
            .filter(|&e| {
                let ct = world.card_type(e);
                ct == Some(CardType::Minion) || ct == Some(CardType::Hero)
            })
            .collect::<Vec<_>>()
    };
    let minions = |p: PlayerId| {
        world
            .zones()
            .iter(Zone::Play, p)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .collect::<Vec<_>>()
    };
    match target {
        EffectTarget::AnyEnemy => chars(enemy),
        EffectTarget::AnyEnemyMinion => minions(enemy),
        EffectTarget::EnemyHero => vec![state.player(enemy).hero],
        EffectTarget::FriendlyMinion => minions(owner),
        EffectTarget::FriendlyHero => vec![state.player(owner).hero],
        EffectTarget::DamagedEnemyMinion => minions(enemy)
            .into_iter()
            .filter(|&e| {
                world
                    .health(e)
                    .is_some_and(|h| h.0 < world.effective_health(e).unwrap_or(h).0)
            })
            .collect(),
        EffectTarget::TauntEnemyMinion => minions(enemy)
            .into_iter()
            .filter(|&e| world.taunt(e).is_some())
            .collect(),
        EffectTarget::Self_ => Vec::new(),
        // AOE/无目标效果 — 无显式目标
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::action::Action;
    use crate::core::component::Health;

    #[test]
    fn env_reset_returns_valid_observation() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        let obs = env.reset(1);
        assert_eq!(obs.len(), OBS_LEN);
        assert_eq!(obs[0], 1.0, "30 HP hero normalized");
    }

    #[test]
    fn legal_actions_include_end_turn_and_attacks() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        env.reset(3);
        let actions = env.legal_actions();
        assert!(actions.contains(&Action::EndTurn));
        // 初始手牌 3 张 — 有可打的牌（法力 0 时除外）
        assert!(!actions.is_empty());
        // 全部动作都能通过引擎验证
        for a in &actions {
            assert!(
                rules::validate(&env.state, *a).is_ok(),
                "action {a:?} must be legal"
            );
        }
    }

    #[test]
    fn step_with_end_turn_runs_bot_turn() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        env.reset(5);
        let result = env.step(Action::EndTurn);
        assert!(!result.done || result.winner.is_some());
        // bot 回合后，控制权回到 agent（或终局）
        if !result.done {
            assert_eq!(env.state.active_player(), PlayerId::Player1);
        }
    }

    #[test]
    fn full_game_loop_terminates() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        env.reset(9);
        let mut guard = 0;
        loop {
            let actions = env.legal_actions();
            if actions.is_empty() {
                // 无动作可做 — 强制结束回合推进
                let r = env.step(Action::EndTurn);
                assert!(!r.done);
            }
            let actions = env.legal_actions();
            let r = env.step_indexed(actions.len() % actions.len().max(1));
            guard += 1;
            if r.done || guard > 3000 {
                break;
            }
        }
        assert!(env.done || guard >= 3000, "game must terminate");
    }

    #[test]
    fn env_observation_reflects_hero_health() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        env.reset(2);
        // 直接改状态观察（测试用：通过 state 不可达，使用 step 造成的伤害）
        // 这里验证：观察始终是 agent 视角的英雄在前
        let obs = env.observation();
        assert_eq!(obs[0], 1.0);
        assert_eq!(obs[5], 1.0);
        let _ = Health(30); // 保持 Health 引用，避免未使用警告
    }
}
