//! Reward functions — configurable per-step rewards and terminal rewards.
//!
//! The default configuration is sparse win/loss rewards (`win`/`loss`); dense shaping can be enabled
//! (enemy hero damage, minion kills, own minion losses) to speed up learning.

use crate::core::player::PlayerId;
use crate::core::state::{GameState, Step};
use crate::core::zone::Zone;

/// Reference hero health used by the health-scaled loss (the simplified
/// hearthstone engine's `HERO_HEALTH`).
pub const HERO_HEALTH: f32 = 30.0;

/// Terminal reward policy (roadmap M1-G7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalReward {
    /// Sparse: win +1 / loss −1 / draw 0 (default)
    Sparse,
    /// Simplified-hearthstone style: win +1 / draw 0 / loss −(winner's
    /// remaining health)/30 — losing to a nearly-dead opponent is a mild
    /// penalty, losing untouched is a full −1
    ScaledByWinnerHealth,
}

/// Reward configuration — weights for each component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RewardConfig {
    /// Win reward
    pub win: f32,
    /// Loss penalty
    pub loss: f32,
    /// Draw reward
    pub draw: f32,
    /// Reward per point of enemy hero health lost (0 disables dense rewards)
    pub enemy_hero_damage: f32,
    /// Reward per enemy minion killed
    pub enemy_minion_kill: f32,
    /// Penalty per own minion that dies
    pub own_minion_loss: f32,
    /// Invalid action penalty
    pub invalid_action: f32,
    /// How the terminal reward is computed (M1-G7)
    pub terminal: TerminalReward,
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
            terminal: TerminalReward::Sparse,
        }
    }
}

/// Terminal reward — computed from the winner and the perspective player.
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

/// Health-scaled loss (M1-G7): the penalty scales with the winner's remaining
/// health — `−remaining/30`, clamped to [−1, 0].
#[must_use]
fn scaled_loss(state: &GameState, winner: PlayerId) -> f32 {
    let hp = state
        .world()
        .effective_health(state.player(winner).hero)
        .map_or(0, |h| h.0);
    -(hp as f32 / HERO_HEALTH).clamp(0.0, 1.0)
}

/// Computes the final reward when the game ends.
#[must_use]
pub fn final_reward(config: &RewardConfig, state: &GameState, perspective: PlayerId) -> f32 {
    match state.step() {
        Step::GameOver { winner } => match config.terminal {
            TerminalReward::Sparse => terminal_reward(config, Some(winner), perspective),
            TerminalReward::ScaledByWinnerHealth => {
                if winner == perspective {
                    config.win
                } else {
                    scaled_loss(state, winner)
                }
            }
        },
        _ => config.draw,
    }
}

/// Counts entities in `zone` belonging to `player` (deduplicated by CardId, used for kill detection).
fn count_in_zone(state: &GameState, zone: Zone, player: PlayerId) -> usize {
    state
        .world()
        .zones()
        .iter(zone, player)
        .filter(|&e| state.world().player(e) == Some(player))
        .count()
}

/// Per-step reward — computed from the state change (excluding terminal reward).
///
/// `before`/`after` are the states before and after the action. Reward components:
/// - Enemy hero health lost × `enemy_hero_damage`
/// - Number of minions that entered the enemy graveyard × `enemy_minion_kill`
/// - Number of minions that entered the friendly graveyard × `own_minion_loss`
///
/// Note: card entities in the graveyard keep their `PlayerId` component, which allows attribution.
#[must_use]
pub fn step_reward(
    config: &RewardConfig,
    before: &GameState,
    after: &GameState,
    perspective: PlayerId,
) -> f32 {
    let enemy = perspective.opponent();

    // Enemy hero health lost
    let before_enemy_hp = before
        .world()
        .effective_health(before.player(enemy).hero)
        .map_or(0, |h| h.0);
    let after_enemy_hp = after
        .world()
        .effective_health(after.player(enemy).hero)
        .map_or(0, |h| h.0);
    let hero_damage = (before_enemy_hp - after_enemy_hp).max(0) as f32;

    // Minions newly killed this step: graveyard count difference (by owner)
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

    // ============================================================
    // M1-G7 — health-scaled terminal reward (simplified-hearthstone style)
    // ============================================================

    fn game_over_state(winner: PlayerId, winner_hp: i32) -> GameState {
        let mut builder = GameBuilder::new();
        builder.hero_health(winner, winner_hp);
        builder.step(Step::GameOver { winner });
        builder.build()
    }

    #[test]
    fn scaled_terminal_reward_win_draw_loss() {
        let cfg = RewardConfig {
            terminal: TerminalReward::ScaledByWinnerHealth,
            ..RewardConfig::default()
        };
        // Win → +1 regardless of the winner's health
        let state = game_over_state(PlayerId::Player1, 30);
        assert_eq!(final_reward(&cfg, &state, PlayerId::Player1), 1.0);
        // Draw (no game over) → 0
        let mut builder = GameBuilder::new();
        let state = builder.build();
        assert_eq!(final_reward(&cfg, &state, PlayerId::Player1), 0.0);
    }

    #[test]
    fn scaled_loss_uses_winner_remaining_health() {
        let cfg = RewardConfig {
            terminal: TerminalReward::ScaledByWinnerHealth,
            ..RewardConfig::default()
        };
        // Lost to a 30-HP winner: full penalty −1
        let state = game_over_state(PlayerId::Player2, 30);
        assert_eq!(final_reward(&cfg, &state, PlayerId::Player1), -1.0);
        // Lost to a 15-HP winner: −0.5
        let state = game_over_state(PlayerId::Player2, 15);
        assert!((final_reward(&cfg, &state, PlayerId::Player1) + 0.5).abs() < 1e-6);
        // Lost to a 2-HP winner: −2/30
        let state = game_over_state(PlayerId::Player2, 2);
        assert!((final_reward(&cfg, &state, PlayerId::Player1) + 2.0 / 30.0).abs() < 1e-6);
        // Clamped: a winner with negative/absurd health never flips the sign
        let state = game_over_state(PlayerId::Player2, -5);
        assert_eq!(final_reward(&cfg, &state, PlayerId::Player1), 0.0);
    }

    #[test]
    fn default_terminal_reward_is_sparse() {
        assert_eq!(
            RewardConfig::default().terminal,
            TerminalReward::Sparse,
            "default must stay sparse for backward compatibility"
        );
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
        // Enemy has a 1 HP minion; we have a 30-attack minion
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 30, 10, 5);
        let victim = builder.add_custom_minion_to_board(PlayerId::Player2, 1, 1, 1);
        let mut state = builder.build();

        // Attack: 1 minion kill + 0 hero damage
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

        // Attack the hero: 30 damage (hero at 30 HP dies immediately → game over)
        // Use a fresh game to test the hero damage component (25 HP)
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
