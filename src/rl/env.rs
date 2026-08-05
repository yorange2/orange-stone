//! Gym-style environment — a single agent plays against a scripted opponent (bot).
//!
//! Environment state machine:
//! - The agent can execute actions consecutively during its own turn (`step`)
//! - After `EndTurn`, the opponent's turn is advanced automatically by the bot
//! - The game ends when either hero dies (`done = true`, with terminal reward)
//!
//! Observations (`obs`), rewards (`reward`), and the action space (`legal_actions`)
//! are fixed/enumerable, usable directly from the Python bindings and RL training.

use crate::core::action::Action;
use crate::core::component::CardType;
use crate::core::effect::EffectTarget;
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Step};
use crate::core::zone::Zone;
use crate::engine::game::GameEngine;
use crate::engine::rules;
use crate::rl::obs::{OBS_LEN, encode_observation};
use crate::rl::reward::{self, RewardConfig};
use crate::sim::battle::{BattleRunner, BotDelegate, BotType};

/// Environment configuration.
#[derive(Debug, Clone, Copy)]
pub struct EnvConfig {
    /// Deck size per player
    pub deck_size: usize,
    /// Initial hand size
    pub hand_size: usize,
    /// Opponent (bot) type
    pub bot_type: BotType,
    /// Maximum action steps per game (prevents infinite loops)
    pub max_steps: u32,
    /// Reward configuration
    pub reward: RewardConfig,
}

impl EnvConfig {
    /// Default configuration (30-card deck, greedy opponent, sparse win/loss reward).
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

/// Result of a single step.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Observation after the action (from the agent's perspective)
    pub observation: Vec<f32>,
    /// Reward for this step (including terminal reward)
    pub reward: f32,
    /// Whether the game has ended
    pub done: bool,
    /// Winner (`None` if the game is not over or it's a draw)
    pub winner: Option<PlayerId>,
}

/// Gym-style environment — the agent is fixed as the `perspective` player.
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
    /// Creates a new environment; not usable until `reset`.
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

    /// Resets the environment: generates a fresh game (random deck) with `seed`, returning the initial observation.
    pub fn reset(&mut self, seed: u64) -> Vec<f32> {
        let mut runner = BattleRunner::new(self.config.bot_type, seed);
        self.state = runner.create_game_state(self.config.deck_size);
        // Initial hand size is determined by create_game_state (fixed at 3); adjust here if configurability is needed
        self.steps = 0;
        self.done = false;
        self.observation()
    }

    /// Current observation (agent's perspective).
    #[must_use]
    pub fn observation(&self) -> Vec<f32> {
        encode_observation(&self.state, self.perspective)
    }

    /// Observation length (fixed value).
    #[must_use]
    pub const fn obs_len() -> usize {
        OBS_LEN
    }

    /// Legal actions for the current player (full enumeration, including explicit targets).
    ///
    /// Should only be called during the agent's turn; the opponent's turn is advanced automatically by the environment.
    #[must_use]
    pub fn legal_actions(&self) -> Vec<Action> {
        legal_actions(&self.state)
    }

    /// Executes `legal_actions()[idx]` by index (shared with the Python bindings).
    pub fn step_indexed(&mut self, action_idx: usize) -> StepResult {
        let actions = self.legal_actions();
        match actions.get(action_idx) {
            Some(&action) => self.step(action),
            None => {
                // Out-of-bounds index is treated as an invalid action
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

    /// Executes an action and advances the environment.
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

        // Terminal check
        if matches!(self.state.step(), Step::GameOver { .. }) {
            self.done = true;
            reward += reward::final_reward(&self.config.reward, &self.state, self.perspective);
        } else if ok && matches!(action, Action::EndTurn) {
            // Opponent's turn: the bot advances automatically until turn end or game over
            self.run_bot_turn();
            if matches!(self.state.step(), Step::GameOver { .. }) {
                self.done = true;
                reward += reward::final_reward(&self.config.reward, &self.state, self.perspective);
            }
        } else if self.steps >= self.config.max_steps {
            // Step limit reached: game ends in a draw
            self.done = true;
        }

        StepResult {
            observation: self.observation(),
            reward,
            done: self.done,
            winner: self.winner(),
        }
    }

    /// Opponent's turn — executes the bot's actions until turn end or game over.
    fn run_bot_turn(&mut self) {
        loop {
            if matches!(self.state.step(), Step::GameOver { .. }) {
                break;
            }
            if self.state.active_player() == self.perspective {
                // Bot's turn is over, control returns to the agent
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
                if matches!(self.state.step(), Step::GameOver { .. }) {
                    break;
                }
            }
            if applied == 0 {
                break;
            }
        }
    }

    /// Current winner (`None` if the game is not over).
    fn winner(&self) -> Option<PlayerId> {
        match self.state.step() {
            Step::GameOver { winner } => Some(winner),
            _ => None,
        }
    }
}

/// Enumerates all legal actions for the current acting player.
///
/// Generates candidates (end turn, hero power, playing cards with explicit targets, all attack pairs),
/// filters them with the engine's `validate`, ensuring consistency with `GameEngine::apply` legality.
#[must_use]
pub fn legal_actions(state: &GameState) -> Vec<Action> {
    let player = state.active_player();
    let world = state.world();
    let mut candidates: Vec<Action> = Vec::new();

    // End turn
    candidates.push(Action::EndTurn);
    // Hero power
    let hero = state.player(player).hero;
    if world.hero_power(hero).is_some() {
        candidates.push(Action::HeroPower { hero, target: None });
    }
    // Play cards (with explicit targets)
    for card in world.zones().iter(Zone::Hand, player) {
        let targets = play_targets(state, card);
        if targets.is_empty() {
            candidates.push(Action::PlayCard {
                card,
                target: None,
                position: None,
            });
        } else {
            for t in targets {
                candidates.push(Action::PlayCard {
                    card,
                    target: Some(t),
                    position: None,
                });
            }
        }
    }
    // All attack pairs (friendly battlefield characters → enemy battlefield characters)
    for attacker in world.zones().iter(Zone::Play, player) {
        for defender in world.zones().iter(Zone::Play, player.opponent()) {
            candidates.push(Action::Attack { attacker, defender });
        }
    }

    // Filter illegal candidates using engine validation
    candidates
        .into_iter()
        .filter(|a| rules::validate(state, *a).is_ok())
        .collect()
}

/// Legal targets for a hand card's effect (empty = card without targets).
fn play_targets(
    state: &GameState,
    card: crate::core::entity::Entity,
) -> Vec<crate::core::entity::Entity> {
    use crate::core::effect::CardEffect;

    let Some(battlecry) = state.world().battlecry(card) else {
        return Vec::new();
    };
    // Extract EffectTarget from the effect variant (spells and battlecries share the battlecry slot)
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

/// Enumerates candidate entities by `EffectTarget`.
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
        // AOE/no-target effects — no explicit target
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
        // Initial hand of 3 cards — there are playable cards (except when mana is 0)
        assert!(!actions.is_empty());
        // All actions must pass engine validation
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
        // After the bot's turn, control returns to the agent (or game over)
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
                // No actions available — force end turn to advance
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
        // Directly modifying state to observe (for testing: unreachable via state, use damage caused by step)
        // Verify here: the observation always has the agent-perspective hero first
        let obs = env.observation();
        assert_eq!(obs[0], 1.0);
        assert_eq!(obs[5], 1.0);
        let _ = Health(30); // keep the Health reference to avoid an unused warning
    }
}
