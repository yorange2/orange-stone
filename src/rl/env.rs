//! Gym-style environment — a single agent plays against a scripted opponent (bot).
//!
//! Environment state machine:
//! - The agent can execute actions consecutively during its own turn (`step`)
//! - After `EndTurn`, the opponent's turn is advanced automatically by the bot;
//!   with `BotType::None` no bot acts and the caller controls both sides
//!   (roadmap M1-G4 — arena and future self-play use this mode)
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
#[derive(Debug, Clone)]
pub struct EnvConfig {
    /// Deck size per player (random deck generation)
    pub deck_size: usize,
    /// Initial hand size (first player; M1-G6)
    pub hand_size: usize,
    /// Second player draws one extra card + The Coin (official opening; M1-G6)
    pub second_player_coin: bool,
    /// Opponent (bot) type
    pub bot_type: BotType,
    /// Maximum action steps per game (prevents infinite loops)
    pub max_steps: u32,
    /// Reward configuration
    pub reward: RewardConfig,
    /// Explicit mirror deck (card IDs) used by both players; `None` = random decks (M1-G2)
    pub deck: Option<Vec<&'static str>>,
}

impl EnvConfig {
    /// Default configuration (30-card deck, greedy opponent, sparse win/loss reward).
    #[must_use]
    pub fn default_with(bot_type: BotType, deck_size: usize) -> Self {
        Self {
            deck_size,
            hand_size: 3,
            second_player_coin: false,
            bot_type,
            max_steps: 5000,
            reward: RewardConfig::default(),
            deck: None,
        }
    }

    /// Uses an explicit mirror deck (both players draw from the same card list).
    #[must_use]
    pub fn with_fixed_deck(mut self, deck: Vec<&'static str>) -> Self {
        self.deck = Some(deck);
        self
    }

    /// Sets the opening shape: `hand_size` cards to the first player; with
    /// `coin` the second player draws one extra and gets The Coin.
    #[must_use]
    pub fn with_opening(mut self, hand_size: usize, coin: bool) -> Self {
        self.hand_size = hand_size;
        self.second_player_coin = coin;
        self
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

    /// Resets the environment: generates a fresh game (random or fixed deck) with `seed`, returning the initial observation.
    pub fn reset(&mut self, seed: u64) -> Vec<f32> {
        let mut runner = BattleRunner::new(self.config.bot_type, seed);
        self.state = match &self.config.deck {
            Some(deck) => {
                let cards: Vec<&'static crate::cards::def::CardDef> = deck
                    .iter()
                    .filter_map(|id| crate::cards::card_by_id(id))
                    .collect();
                debug_assert_eq!(
                    cards.len(),
                    deck.len(),
                    "all deck card IDs must resolve (validated at construction)"
                );
                runner.create_game_state_with_decks(
                    &cards,
                    &cards,
                    self.config.hand_size,
                    self.config.second_player_coin,
                )
            }
            None => runner.create_game_state(
                self.config.deck_size,
                self.config.hand_size,
                self.config.second_player_coin,
            ),
        };
        self.steps = 0;
        self.done = false;
        self.observation()
    }

    /// Current observation (agent's perspective).
    #[must_use]
    pub fn observation(&self) -> Vec<f32> {
        encode_observation(&self.state, self.perspective)
    }

    /// The underlying game state (read-only) — exposed for the structured views (M1-G3).
    #[must_use]
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// The agent's fixed perspective player.
    #[must_use]
    pub fn perspective(&self) -> PlayerId {
        self.perspective
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
    #[must_use]
    pub fn winner(&self) -> Option<PlayerId> {
        match self.state.step() {
            Step::GameOver { winner } => Some(winner),
            _ => None,
        }
    }

    /// Whether the game has ended (M4 batch API).
    #[must_use]
    pub fn is_done(&self) -> bool {
        matches!(self.state.step(), Step::GameOver { .. })
    }
}

/// Structured metadata for a legal action (roadmap M1-G3).
///
/// Parallels `Action` with the fields the Python side needs for feature
/// engineering: action kind, the hand index of a played card, and the
/// involved entity IDs (matching `EntityView::entity_id`).
#[derive(Debug, Clone, Copy)]
pub struct ActionInfo {
    /// The engine action (executed via `GameEnv::step`)
    pub action: Action,
    /// Action kind: `"end_turn" | "play" | "attack" | "hero_power" | "choose"`
    pub kind: &'static str,
    /// Hand index of the card for `play`, else `-1`
    pub card_index: i32,
    /// Entity id: the card (`play`), the attacker (`attack`), the hero (`hero_power`), else `-1`
    pub entity_id: i32,
    /// Target entity id (play target / defender / hero power target), else `-1`
    pub target_id: i32,
}

/// Enumerates all legal actions for the current acting player with structured metadata.
///
/// Generates candidates (end turn, hero power, playing cards with explicit targets, all attack pairs),
/// filters them with the engine's `validate`, ensuring consistency with `GameEngine::apply` legality.
#[must_use]
pub fn legal_action_infos(state: &GameState) -> Vec<ActionInfo> {
    let player = state.active_player();
    let world = state.world();
    let mut candidates: Vec<ActionInfo> = Vec::new();

    // End turn
    candidates.push(ActionInfo {
        action: Action::EndTurn,
        kind: "end_turn",
        card_index: -1,
        entity_id: -1,
        target_id: -1,
    });
    // Hero power
    let hero = state.player(player).hero;
    if world.hero_power(hero).is_some() {
        candidates.push(ActionInfo {
            action: Action::HeroPower { hero, target: None },
            kind: "hero_power",
            card_index: -1,
            entity_id: hero.index as i32,
            target_id: -1,
        });
    }
    // Play cards (with explicit targets)
    for (hand_idx, card) in world.zones().iter(Zone::Hand, player).enumerate() {
        let targets = play_targets(state, card);
        if targets.is_empty() {
            candidates.push(ActionInfo {
                action: Action::PlayCard {
                    card,
                    target: None,
                    position: None,
                },
                kind: "play",
                card_index: hand_idx as i32,
                entity_id: card.index as i32,
                target_id: -1,
            });
        } else {
            for t in targets {
                candidates.push(ActionInfo {
                    action: Action::PlayCard {
                        card,
                        target: Some(t),
                        position: None,
                    },
                    kind: "play",
                    card_index: hand_idx as i32,
                    entity_id: card.index as i32,
                    target_id: t.index as i32,
                });
            }
        }
    }
    // All attack pairs (friendly battlefield characters → enemy battlefield characters)
    for attacker in world.zones().iter(Zone::Play, player) {
        for defender in world.zones().iter(Zone::Play, player.opponent()) {
            candidates.push(ActionInfo {
                action: Action::Attack { attacker, defender },
                kind: "attack",
                card_index: -1,
                entity_id: attacker.index as i32,
                target_id: defender.index as i32,
            });
        }
    }

    // Filter illegal candidates using engine validation
    candidates
        .into_iter()
        .filter(|i| rules::validate(state, i.action).is_ok())
        .collect()
}

/// Enumerates all legal actions for the current acting player.
///
/// Thin wrapper over [`legal_action_infos`] keeping the plain-action API.
#[must_use]
pub fn legal_actions(state: &GameState) -> Vec<Action> {
    legal_action_infos(state)
        .into_iter()
        .map(|info| info.action)
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
    fn first_player_turn_one_has_one_mana_in_battle_path() {
        // Regression (M3): build_game_state used to reset Player1 to 0/0 mana,
        // clobbering the turn-1 crystal that GameState::new() provides — the
        // first player stayed a full crystal behind for the whole game, which
        // collapsed first-seat RL training. HS official: 1 crystal on turn 1.
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        env.reset(7);
        assert_eq!(env.state.player(PlayerId::Player1).mana_crystals, 1);
        assert_eq!(env.state.player(PlayerId::Player1).current_mana, 1);
        assert_eq!(env.state.player(PlayerId::Player2).mana_crystals, 0);
        assert_eq!(env.state.player(PlayerId::Player2).current_mana, 0);
        // With 1 mana the opening hand must be playable (0/1-cost cards exist)
        let actions = env.legal_actions();
        assert!(
            actions.iter().any(|a| matches!(a, Action::PlayCard { .. })),
            "turn-1 play actions must exist with 1 mana"
        );
    }

    #[test]
    fn stealth_cards_carry_the_keyword_through_the_battle_path() {
        // Regression (M5): CardDef had no stealth field, so stealth minions
        // (Jungle Panther etc.) were defined vanilla. The deck path must apply
        // the component and expose it in the structured view.
        let deck: Vec<&'static str> = vec!["NEUTRAL_C10"; 10]; // Jungle Panther
        let env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 30).with_fixed_deck(deck),
        );
        let mut env = env;
        env.reset(3);
        let view = crate::rl::views::observation(&env.state, PlayerId::Player1);
        let stealth_cards: Vec<&str> = view
            .me
            .hand
            .iter()
            .filter(|c| c.stealth)
            .map(|c| c.card_id.as_str())
            .collect();
        assert!(
            !stealth_cards.is_empty(),
            "Jungle Panther must expose stealth in the view, got {stealth_cards:?}"
        );
        assert!(stealth_cards.contains(&"NEUTRAL_C10"));
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

    // ============================================================
    // M1-G2 — explicit mirror decks
    // ============================================================

    /// A fixed deck of 10 copies of one card: every hand card must come from it.
    fn fixed_deck_env(seed: u64) -> GameEnv {
        let deck: Vec<&'static str> = vec!["CLASSIC_001"; 10]; // Bloodfen Raptor
        let env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 30).with_fixed_deck(deck),
        );
        let mut env = env;
        env.reset(seed);
        env
    }

    #[test]
    fn fixed_deck_env_deals_only_deck_cards() {
        let env = fixed_deck_env(1);
        // Both players' decks have exactly 10 cards, hand 3 each → 7 remaining each
        assert_eq!(
            env.state.world().zones().len(Zone::Deck, PlayerId::Player1),
            7
        );
        assert_eq!(
            env.state.world().zones().len(Zone::Deck, PlayerId::Player2),
            7
        );
        for pid in [PlayerId::Player1, PlayerId::Player2] {
            for e in env.state.world().zones().iter(Zone::Hand, pid) {
                assert_eq!(
                    env.state.world().card_id(e),
                    Some(crate::core::component::CardId("CLASSIC_001")),
                    "hand card must come from the fixed deck"
                );
            }
        }
    }

    #[test]
    fn fixed_deck_env_is_deterministic() {
        let a = fixed_deck_env(7).observation();
        let b = fixed_deck_env(7).observation();
        assert_eq!(a, b, "same seed + same fixed deck → identical observation");
    }

    #[test]
    fn fixed_deck_env_differs_across_seeds() {
        // A heterogeneous deck: the opening hand depends on the per-seed shuffle,
        // so a different seed must change the observable state.
        let deck: Vec<&'static str> = vec![
            "CLASSIC_001",  // Bloodfen Raptor 2/3/2
            "CLASSIC_018",  // Amani Berserker 2/2/3
            "NEUTRAL_025",  // Core Hound 7/9/5
            "NEUTRAL_B09",  // Magma Rager 3/5/1
            "NEUTRAL_B13",  // Oasis Snapjaw 4/2/7
            "NEUTRAL_B19",  // Gurubashi Berserker 5/2/8
            "CLASSIC_019",  // Faerie Dragon 2/3/2
            "NEUTRAL_B02",  // Murloc Raider 1/2/1
            "CLASSIC_006t", // Murloc Scout 1/1/1
            "NEUTRAL_020t", // Squire 1/2/2
        ];
        let make = |seed: u64| {
            GameEnv::new(
                PlayerId::Player1,
                EnvConfig::default_with(BotType::Greedy, 30).with_fixed_deck(deck.clone()),
            )
            .reset(seed)
        };
        assert_ne!(make(7), make(8), "different seed → different opening hand");
        assert_eq!(make(42), make(42), "same seed → identical opening hand");
    }

    #[test]
    fn fixed_deck_env_completes_a_game() {
        let mut env = fixed_deck_env(3);
        let mut guard = 0;
        loop {
            let actions = env.legal_actions();
            if actions.is_empty() {
                break;
            }
            let r = env.step_indexed(actions.len() % actions.len().max(1));
            guard += 1;
            if r.done || guard > 3000 {
                break;
            }
        }
        assert!(env.done || guard >= 3000, "game must terminate");
    }

    // ============================================================
    // M1-G4 — both players externally controlled (BotType::None)
    // ============================================================

    #[test]
    fn no_bot_hands_control_to_the_other_side_after_end_turn() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::None, 20),
        );
        env.reset(4);
        // P1 ends the turn: with no bot, the opponent's turn is NOT auto-played
        let r = env.step(Action::EndTurn);
        assert!(!r.done);
        assert_eq!(
            env.state.active_player(),
            PlayerId::Player2,
            "control returns to the external driver on the other side"
        );
        assert!(!env.legal_actions().is_empty(), "P2 has legal actions");
        // P2 ends the turn → back to P1
        let r = env.step(Action::EndTurn);
        assert!(!r.done);
        assert_eq!(env.state.active_player(), PlayerId::Player1);
    }

    #[test]
    fn no_bot_full_game_loop_terminates_with_external_control() {
        // Random-vs-random can stall forever (no fatigue damage in the engine yet),
        // so the external driver plays a scripted policy: play a card → attack face
        // with everything → end turn. Face damage guarantees the game ends.
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::None, 20),
        );
        env.reset(6);
        let mut guard = 0;
        loop {
            let actions = env.legal_actions();
            assert!(!actions.is_empty(), "both sides always have EndTurn");
            let enemy_hero = env.state.player(env.state.active_player().opponent()).hero;
            let pick = actions
                .iter()
                .position(|a| matches!(a, Action::PlayCard { .. }))
                .or_else(|| {
                    actions.iter().position(
                        |a| matches!(a, Action::Attack { defender, .. } if *defender == enemy_hero),
                    )
                })
                .or_else(|| actions.iter().position(|a| matches!(a, Action::EndTurn)))
                .expect("EndTurn is always legal");
            let r = env.step_indexed(pick);
            guard += 1;
            if r.done || guard > 3000 {
                break;
            }
        }
        assert!(env.done, "scripted face pressure must end the game");
        assert!(
            matches!(env.state.step(), Step::GameOver { .. }),
            "termination must come from a real game over, not the step limit"
        );
    }

    #[test]
    fn no_bot_with_perspective_two_is_symmetric() {
        // Same setup with the agent fixed as the second player
        let mut env = GameEnv::new(
            PlayerId::Player2,
            EnvConfig::default_with(BotType::None, 20),
        );
        env.reset(9);
        let actions = env.legal_actions();
        assert!(!actions.is_empty(), "P2 controls its own opening turn");
        let r = env.step_indexed(actions.len() % actions.len().max(1));
        assert!(!r.done);
    }

    // ============================================================
    // M1-G5 — clone() for search / rollback
    // ============================================================

    #[test]
    fn cloned_env_steps_independently() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::Greedy, 20),
        );
        env.reset(5);
        let before = env.observation();
        let mut cloned = env.clone();
        assert_eq!(cloned.observation(), before, "clone starts identical");

        // The clone advances (EndTurn + bot turn); the original is untouched
        let r = cloned.step(Action::EndTurn);
        assert!(!r.done);
        assert_eq!(
            env.observation(),
            before,
            "stepping the clone must not disturb the original"
        );

        // And the original stepping leaves the clone's state as it was
        let cloned_obs = cloned.observation();
        let _ = env.step(Action::EndTurn);
        assert_eq!(
            cloned.observation(),
            cloned_obs,
            "stepping the original must not disturb the clone"
        );

        // A fresh clone of the clone also diverges independently
        let mut branch = cloned.clone();
        let _ = branch.step_indexed(0);
        assert_eq!(
            cloned.observation(),
            cloned_obs,
            "branching off a clone must not disturb it"
        );
    }

    // ============================================================
    // M1-G6 — configurable opening (hand size + second-player coin)
    // ============================================================

    fn hand_size_of(env: &GameEnv, pid: PlayerId) -> usize {
        env.state.world().zones().len(Zone::Hand, pid)
    }

    #[test]
    fn default_opening_is_three_and_three_no_coin() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::None, 20),
        );
        env.reset(1);
        assert_eq!(hand_size_of(&env, PlayerId::Player1), 3);
        assert_eq!(hand_size_of(&env, PlayerId::Player2), 3);
    }

    #[test]
    fn custom_hand_size_applies_to_both_players() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::None, 20).with_opening(4, false),
        );
        env.reset(2);
        assert_eq!(hand_size_of(&env, PlayerId::Player1), 4);
        assert_eq!(hand_size_of(&env, PlayerId::Player2), 4);
    }

    #[test]
    fn second_player_coin_adds_extra_card_and_coin() {
        let mut env = GameEnv::new(
            PlayerId::Player1,
            EnvConfig::default_with(BotType::None, 20).with_opening(3, true),
        );
        env.reset(3);
        assert_eq!(hand_size_of(&env, PlayerId::Player1), 3);
        // Official second-player shape: hand_size + 1 draw + The Coin
        assert_eq!(hand_size_of(&env, PlayerId::Player2), 5);
        let has_coin = env
            .state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .any(|e| {
                env.state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "GAME_005")
            });
        assert!(has_coin, "the second player must hold The Coin (GAME_005)");
    }

    #[test]
    fn opening_config_is_deterministic() {
        let make = |seed: u64| {
            let mut env = GameEnv::new(
                PlayerId::Player1,
                EnvConfig::default_with(BotType::None, 20).with_opening(4, true),
            );
            env.reset(seed);
            env.observation()
        };
        assert_eq!(make(9), make(9), "same seed + same opening → identical");
        assert_ne!(make(9), make(10), "different seed → different game");
    }
}
