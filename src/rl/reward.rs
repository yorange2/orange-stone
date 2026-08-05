//! 奖励函数 — 可配置的单步奖励与终局奖励。
//!
//! 默认配置是稀疏的胜/负奖励（`win`/`loss`）；可开启密集整形
//! （敌方英雄伤害、击杀随从、己方随从损失），加速学习。

use crate::core::player::PlayerId;
use crate::core::state::{GameState, Phase};
use crate::core::zone::Zone;

/// 奖励配置 — 各分量的权重。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RewardConfig {
    /// 获胜奖励
    pub win: f32,
    /// 失败惩罚
    pub loss: f32,
    /// 平局奖励
    pub draw: f32,
    /// 敌方英雄每损失 1 点生命值的奖励（0 表示关闭密集奖励）
    pub enemy_hero_damage: f32,
    /// 敌方每死亡 1 个随从的奖励
    pub enemy_minion_kill: f32,
    /// 己方每死亡 1 个随从的惩罚
    pub own_minion_loss: f32,
    /// 非法动作惩罚
    pub invalid_action: f32,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            win: 1.0,
            loss: -1.0,
            draw: 0.0,
            enemy_hero_damage: 0.0,
            enemy_minion_kill: 0.0,
            own_minion_loss: 0.0,
            invalid_action: -0.1,
        }
    }
}

/// 终局奖励 — 根据胜者与视角玩家计算。
#[must_use]
pub fn terminal_reward(
    config: &RewardConfig,
    winner: Option<PlayerId>,
    perspective: PlayerId,
) -> f32 {
    match winner {
        Some(w) if w == perspective => config.win,
        Some(_) => config.loss,
        None => config.draw,
    }
}

/// 计算游戏结束时的最终奖励。
#[must_use]
pub fn final_reward(config: &RewardConfig, state: &GameState, perspective: PlayerId) -> f32 {
    match state.phase() {
        Phase::GameOver { winner } => terminal_reward(config, Some(winner), perspective),
        _ => config.draw,
    }
}

/// 统计 `zone` 中属于 `player` 的实体数量（按 CardId 去重计数，用于击杀检测）。
fn count_in_zone(state: &GameState, zone: Zone, player: PlayerId) -> usize {
    state
        .world()
        .zones()
        .iter(zone, player)
        .filter(|&e| state.world().player(e) == Some(player))
        .count()
}

/// 单步奖励 — 根据状态变化计算（不含终局奖励）。
///
/// `before`/`after` 是动作前后的状态。奖励分量：
/// - 敌方英雄生命值损失 × `enemy_hero_damage`
/// - 进入敌方坟墓场的随从数 × `enemy_minion_kill`
/// - 进入己方坟墓场的随从数 × `own_minion_loss`
///
/// 注意：坟场中的卡牌实体保留 `PlayerId` 组件，可据此归属。
#[must_use]
pub fn step_reward(
    config: &RewardConfig,
    before: &GameState,
    after: &GameState,
    perspective: PlayerId,
) -> f32 {
    let enemy = perspective.opponent();

    // 敌方英雄生命损失
    let before_enemy_hp = before
        .world()
        .health(before.player(enemy).hero)
        .map_or(0, |h| h.0);
    let after_enemy_hp = after
        .world()
        .health(after.player(enemy).hero)
        .map_or(0, |h| h.0);
    let hero_damage = (before_enemy_hp - after_enemy_hp).max(0) as f32;

    // 本步新死亡的随从：坟场数量差（按归属玩家）
    let enemy_deaths = (count_in_zone(after, Zone::Graveyard, enemy)
        - count_in_zone(before, Zone::Graveyard, enemy)) as f32;
    let own_deaths = (count_in_zone(after, Zone::Graveyard, perspective)
        - count_in_zone(before, Zone::Graveyard, perspective)) as f32;

    hero_damage * config.enemy_hero_damage
        + enemy_deaths * config.enemy_minion_kill
        + own_deaths * config.own_minion_loss
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::action::Action;
    use crate::engine::game::GameEngine;
    use crate::sim::game::GameBuilder;

    #[test]
    fn terminal_reward_win_loss_draw() {
        let cfg = RewardConfig::default();
        assert_eq!(
            terminal_reward(&cfg, Some(PlayerId::Player1), PlayerId::Player1),
            1.0
        );
        assert_eq!(
            terminal_reward(&cfg, Some(PlayerId::Player2), PlayerId::Player1),
            -1.0
        );
        assert_eq!(terminal_reward(&cfg, None, PlayerId::Player1), 0.0);
    }

    #[test]
    fn step_reward_hero_damage_and_kills() {
        let cfg = RewardConfig {
            enemy_hero_damage: 0.1,
            enemy_minion_kill: 0.5,
            own_minion_loss: -0.5,
            ..RewardConfig::default()
        };
        let engine = GameEngine::new();
        let mut builder = GameBuilder::new();
        // 敌方一个 1 HP 随从；我方一个 30 攻随从
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 30, 10, 5);
        let victim = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
        let mut state = builder.build();

        // 攻击：击杀 1 随从 + 0 英雄伤害
        let before = state.clone();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: victim,
                },
            )
            .unwrap();
        let r = step_reward(&cfg, &before, &state, PlayerId::Player1);
        assert!((r - 0.5).abs() < 1e-6, "kill reward only: {r}");

        // 攻击英雄：30 伤害（英雄 30 HP 直接死亡 → 终局）
        // 用新对局测试英雄伤害分量（生命 25）
        let mut builder = GameBuilder::new();
        builder.hero_health(PlayerId::Player2, 25);
        let attacker2 = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        let mut state = builder.build();
        let before = state.clone();
        let enemy_hero = state.player(PlayerId::Player2).hero;
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker: attacker2,
                    defender: enemy_hero,
                },
            )
            .unwrap();
        let r = step_reward(&cfg, &before, &state, PlayerId::Player1);
        assert!((r - 0.3).abs() < 1e-6, "3 hero damage x 0.1: {r}");
    }

    #[test]
    fn own_minion_loss_is_penalized() {
        let cfg = RewardConfig {
            own_minion_loss: -0.5,
            ..RewardConfig::default()
        };
        let engine = GameEngine::new();
        let mut builder = GameBuilder::new();
        builder.active_player(PlayerId::Player2);
        let victim = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 1, 1);
        let killer = builder.add_custom_minion_to_board(PlayerId::Player2, 5, 5, 5);
        let mut state = builder.build();

        let before = state.clone();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker: killer,
                    defender: victim,
                },
            )
            .unwrap();
        let r = step_reward(&cfg, &before, &state, PlayerId::Player1);
        assert!((r - -0.5).abs() < 1e-6, "own minion death penalty: {r}");
    }
}
