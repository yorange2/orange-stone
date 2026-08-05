//! Batched simulation — advances multiple game instances in parallel (the basis for RL batched inference).
//!
//! RL training needs many games running at once: each `GameState` carries its own RNG,
//! and games advance in parallel on a rayon thread pool without interfering with one
//! another. Each game's result is identical to single-threaded execution (the per-game
//! RNG and action sequence determine the outcome, independent of thread scheduling).

use crate::core::player::PlayerId;
use crate::core::state::{GameState, Step};
use crate::engine::game::GameEngine;
use crate::sim::battle::{BattleRunner, BotDelegate, BotType};
use rayon::prelude::*;

/// Outcome of a single game in a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchOutcome {
    /// Winner (`None` means the step cap was reached without a victor)
    pub winner: Option<PlayerId>,
    /// Number of actions actually executed
    pub steps: u32,
    /// Turn number at the end
    pub turn: u32,
}

/// Batch simulator — runs bot-driven self-play across multiple games in parallel.
#[derive(Debug, Clone, Copy)]
pub struct BatchSimulator {
    bot_type: BotType,
    bot: BotDelegate,
    max_steps: u32,
}

impl BatchSimulator {
    /// Creates a batch simulator.
    ///
    /// `max_steps` — the maximum number of action steps per game (prevents infinite loops).
    #[must_use]
    pub fn new(bot_type: BotType, max_steps: u32) -> Self {
        Self {
            bot_type,
            bot: BotDelegate::new(bot_type),
            max_steps,
        }
    }

    /// Advances all games in parallel, returning results in input order.
    ///
    /// Each game advances independently; `par_iter` preserves output order, so given
    /// the same inputs and per-game seeds, the results are fully reproducible.
    pub fn run(&self, games: Vec<GameState>) -> Vec<BatchOutcome> {
        games
            .into_par_iter()
            .map(|mut state| self.run_one(&mut state))
            .collect()
    }

    /// Generates and runs `count` full games in parallel.
    ///
    /// Each game uses an independent seed (`seed + i`) for its random decks and game RNG,
    /// so games are uncorrelated. Results are returned in index order.
    pub fn run_battles(&self, seed: u64, deck_size: usize, count: usize) -> Vec<BatchOutcome> {
        let states: Vec<GameState> = (0..count)
            .map(|i| {
                let mut runner = BattleRunner::new(self.bot_type, seed.wrapping_add(i as u64));
                runner.create_game_state(deck_size, 3, false)
            })
            .collect();
        self.run(states)
    }

    /// Advances a single game until it ends or hits the step cap.
    fn run_one(&self, state: &mut GameState) -> BatchOutcome {
        let engine = GameEngine::new();
        let mut steps = 0u32;
        loop {
            if steps >= self.max_steps || matches!(state.step(), Step::GameOver { .. }) {
                break;
            }
            let actions = self.bot.decide_actions(state);
            if actions.is_empty() {
                // No legal actions (nothing playable, nothing to attack) — the turn cannot advance
                break;
            }
            let mut applied = 0;
            for action in &actions {
                if engine.apply(state, *action).is_ok() {
                    steps += 1;
                    applied += 1;
                }
                if matches!(state.step(), Step::GameOver { .. }) {
                    break;
                }
            }
            // All actions this turn were rejected — the state no longer changes, so stop
            if applied == 0 {
                break;
            }
        }
        let winner = match state.step() {
            Step::GameOver { winner } => Some(winner),
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
            // Must end or have a winner within at most 5000 steps
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
        // Games cloned from the same template are isolated (CoW): advancing a batch does not affect the template
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
        // Template was not modified: the hand still has one card and the hero still has 30 HP
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
