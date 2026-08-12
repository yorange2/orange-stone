//! Python bindings — the `orange_stone` PyO3 extension module.
//!
//! Built via `cargo build --features py` / maturin. Exposes the Gym-style environment:
//!
//! ```python
//! import orange_stone as os
//! env = os.GameEnv(seed=42)
//! obs = env.reset(seed=1)          # 177-dim observation vector
//! actions = env.legal_actions()    # [(index, description), ...]
//! obs, reward, done, winner = env.step(action_index)
//! ```
//!
//! The action space is the per-turn list of legal action indices (returned by `legal_actions()`);
//! indices are stable within a turn; after `EndTurn`, the environment advances the opponent's turn automatically.

mod views;

use crate::core::player::PlayerId;
use crate::rl::env::{EnvConfig, GameEnv};
use crate::rl::obs::OBS_LEN;
use crate::sim::battle::BotType;
use pyo3::prelude::*;
use views::{PyActionView, PyObservation};

/// Builds an `EnvConfig` from the Python-facing parameters (shared by
/// `GameEnv` and `BatchEnv`; M1-G2/G4/G6/G7 parsing).
fn build_env_config(
    deck_size: usize,
    deck: Option<Vec<String>>,
    bot: &str,
    hand_size: usize,
    second_player_coin: bool,
    terminal_reward: &str,
) -> PyResult<EnvConfig> {
    let bot_type = match bot {
        "greedy" => BotType::Greedy,
        "smart" => BotType::Smart,
        "none" => BotType::None,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "unknown bot type: {other} (expected greedy | smart | none)"
            )));
        }
    };
    let terminal = match terminal_reward {
        "sparse" => crate::rl::reward::TerminalReward::Sparse,
        "health_scaled" => crate::rl::reward::TerminalReward::ScaledByWinnerHealth,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "unknown terminal_reward: {other} (expected sparse | health_scaled)"
            )));
        }
    };
    let mut config =
        EnvConfig::default_with(bot_type, deck_size).with_opening(hand_size, second_player_coin);
    config.reward.terminal = terminal;
    if let Some(ids) = deck {
        let mut resolved = Vec::with_capacity(ids.len());
        for id in &ids {
            let Some(card) = crate::cards::card_by_id(id) else {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "unknown card id: {id}"
                )));
            };
            resolved.push(card.id);
        }
        config = config.with_fixed_deck(resolved);
    }
    Ok(config)
}

/// Python wrapper for the Gym-style environment.
#[pyclass(name = "GameEnv")]
struct PyGameEnv {
    env: GameEnv,
}

#[pymethods]
impl PyGameEnv {
    /// Creates the environment. `perspective` is the agent's perspective (0 = first player, 1 = second player).
    ///
    /// `deck` (optional) is a list of card IDs used as a mirror deck for both players
    /// (M1-G2); `None`/omitted generates random decks of `deck_size` cards.
    ///
    /// `bot` (optional) selects the opponent: `"greedy"` / `"smart"` play the
    /// opponent's turn automatically; `"none"` (M1-G4) leaves both sides to
    /// the caller — after `EndTurn` the other player becomes externally steppable.
    ///
    /// `hand_size` / `second_player_coin` (optional, M1-G6) shape the opening:
    /// the first player draws `hand_size`; with the coin the second player
    /// draws `hand_size + 1` and gets The Coin.
    ///
    /// `terminal_reward` (optional, M1-G7): `"sparse"` (default, win +1 / loss
    /// −1 / draw 0) or `"health_scaled"` (simplified-hearthstone style: loss
    /// penalty scales with the winner's remaining health, 0..−1).
    #[new]
    #[pyo3(signature = (seed, perspective=0, deck_size=30, deck=None, bot="greedy", hand_size=3, second_player_coin=false, terminal_reward="sparse"))]
    fn new(
        seed: u64,
        perspective: u8,
        deck_size: usize,
        deck: Option<Vec<String>>,
        bot: &str,
        hand_size: usize,
        second_player_coin: bool,
        terminal_reward: &str,
    ) -> PyResult<Self> {
        let perspective = match perspective {
            0 => PlayerId::Player1,
            _ => PlayerId::Player2,
        };
        let config = build_env_config(
            deck_size,
            deck,
            bot,
            hand_size,
            second_player_coin,
            terminal_reward,
        )?;
        let mut env = GameEnv::new(perspective, config);
        env.reset(seed);
        Ok(Self { env })
    }

    /// Observation vector length (177 since the D3 observation extension —
    /// the legacy 168 features plus the new-mechanic block).
    #[staticmethod]
    fn obs_len() -> usize {
        OBS_LEN
    }

    /// All card IDs in the full classic pool (M5 deck building).
    #[staticmethod]
    fn all_card_ids() -> Vec<String> {
        // D3 cut-over (2026-08-09): the RL pool is now Standard — the
        // handwritten expansion cards are sampled alongside Classic-era +
        // Core (mirror of `pool::sampling_cards`; generated stat-only
        // baselines stay out). The Python side filters debt/tokens.
        crate::cards::sets::ALL_CARDS
            .iter()
            .chain(crate::cards::sets::HANDWRITTEN_EXPANSION_CARDS)
            .map(|c| c.id.to_string())
            .collect()
    }

    /// Pool-open card IDs — cards whose resolution reads the opponent's
    /// hand/deck or copies a cast spell (Mind Vision, Thoughtsteal, Mindgames,
    /// Lorewalker Cho). The RL pool can exclude them with one flag
    /// (`full_pool(include_pool_open=False)`) if a second set ever lands.
    #[staticmethod]
    fn pool_open_card_ids() -> Vec<String> {
        crate::cards::sets::POOL_OPEN_CARDS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Non-collectible card IDs (official JSON flag): tokens, enchantments
    /// and other generated cards — things a constructed deck can never hold.
    /// The RL pool filters them (the `t`-suffix rule alone misses enchantments
    /// like CORE_EX1_506a / CORE_CATA_006e).
    #[staticmethod]
    fn non_collectible_card_ids() -> Vec<String> {
        crate::cards::sets::ALL_CARDS
            .iter()
            .chain(crate::cards::sets::HANDWRITTEN_EXPANSION_CARDS)
            .filter(|c| !crate::cards::generated::is_collectible(c.id))
            .map(|c| c.id.to_string())
            .collect()
    }

    /// Resets the environment, returning the initial observation.
    ///
    /// Releases the GIL while the engine works (M4): the engine is pure Rust
    /// with per-game RNG, so parallel threads stepping independent envs are
    /// safe — the GIL only guards the Python object itself.
    fn reset(&mut self, py: Python<'_>, seed: u64) -> Vec<f32> {
        py.allow_threads(|| self.env.reset(seed))
    }

    /// Current observation (agent's perspective).
    fn observation(&self, py: Python<'_>) -> Vec<f32> {
        py.allow_threads(|| self.env.observation())
    }

    /// Legal actions: `[(index, readable description), ...]`.
    fn legal_actions(&self, py: Python<'_>) -> Vec<(usize, String)> {
        py.allow_threads(|| {
            self.env
                .legal_actions()
                .into_iter()
                .enumerate()
                .map(|(i, a)| (i, format!("{a:?}")))
                .collect()
        })
    }

    /// Executes the `action_index`-th legal action.
    ///
    /// Returns `(observation, reward, done, winner)`; `winner` is `None`
    /// if the game is not over or it's a draw.
    fn step(&mut self, py: Python<'_>, action_index: usize) -> (Vec<f32>, f64, bool, Option<u8>) {
        let result = py.allow_threads(|| self.env.step_indexed(action_index));
        let winner = result.winner.map(|p| p.index() as u8);
        (
            result.observation,
            result.reward as f64,
            result.done,
            winner,
        )
    }

    /// Structured observation (M1-G3): full view of heroes, mana, hands, boards.
    fn structured_observation(&self, py: Python<'_>) -> PyObservation {
        // The view mirror is pure Rust (rviews); build it with the GIL released
        // and convert to PyO3 objects afterwards (they need the GIL).
        let view = py.allow_threads(|| {
            crate::rl::views::observation(
                self.env.game_state(),
                self.env.perspective(),
                self.env.is_done(),
            )
        });
        PyObservation::from(&view)
    }

    /// Structured legal actions (M1-G3): same order as `legal_actions()`, each
    /// with kind / card_index / entity_id / target_id / description.
    fn structured_legal_actions(&self, py: Python<'_>) -> Vec<PyActionView> {
        let views = py.allow_threads(|| crate::rl::views::action_views(self.env.game_state()));
        views.iter().map(PyActionView::from).collect()
    }

    /// Deep copy of the environment (M1-G5): independent state and RNG, so
    /// search / rollback can branch from Python — the CoW `GameState` makes
    /// this cheap.
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
        }
    }
}

/// Batched environments (roadmap M4): N independent games driven in one call.
///
/// The whole batch iterates inside a single `allow_threads` region, amortizing
/// the Python↔Rust crossing; per-game RNG keeps every env deterministic and
/// thread-safe, so batches of batches can also run on a thread pool.
///
/// `perspectives` (optional, one per env) pins each env's reward perspective;
/// the **structured observations always use the active player's view** — the
/// RL batch sees whoever is acting, and each env's reward stays seat-fixed.
#[pyclass(name = "BatchEnv")]
struct PyBatchEnv {
    envs: Vec<GameEnv>,
}

#[pymethods]
impl PyBatchEnv {
    #[new]
    #[pyo3(signature = (seeds, deck=None, perspectives=None, bot="none", deck_size=30, hand_size=3, second_player_coin=true, terminal_reward="sparse"))]
    fn new(
        seeds: Vec<u64>,
        deck: Option<Vec<String>>,
        perspectives: Option<Vec<u8>>,
        bot: &str,
        deck_size: usize,
        hand_size: usize,
        second_player_coin: bool,
        terminal_reward: &str,
    ) -> PyResult<Self> {
        let config = build_env_config(
            deck_size,
            deck,
            bot,
            hand_size,
            second_player_coin,
            terminal_reward,
        )?;
        let mut envs = Vec::with_capacity(seeds.len());
        for (i, seed) in seeds.iter().enumerate() {
            let perspective = match perspectives.as_ref().and_then(|p| p.get(i)) {
                Some(0) | None => PlayerId::Player1,
                _ => PlayerId::Player2,
            };
            let mut env = GameEnv::new(perspective, config.clone());
            env.reset(*seed);
            envs.push(env);
        }
        Ok(Self { envs })
    }

    /// Number of environments in the batch.
    fn len(&self) -> usize {
        self.envs.len()
    }

    /// Resets every env with the given seeds; returns the initial 177-dim
    /// observation of each env (its own perspective).
    fn reset(&mut self, py: Python<'_>, seeds: Vec<u64>) -> Vec<Vec<f32>> {
        py.allow_threads(|| {
            self.envs
                .iter_mut()
                .zip(seeds)
                .map(|(env, seed)| env.reset(seed))
                .collect()
        })
    }

    /// Resets one env (M4 batched training: a finished game reopens without
    /// touching the others).
    fn reset_one(&mut self, py: Python<'_>, index: usize, seed: u64) -> Vec<f32> {
        py.allow_threads(|| self.envs[index].reset(seed))
    }

    /// Current 177-dim observation per env (each env's own perspective).
    fn observation(&self, py: Python<'_>) -> Vec<Vec<f32>> {
        py.allow_threads(|| self.envs.iter().map(|env| env.observation()).collect())
    }

    /// Legal actions per env: `[(index, description), ...]` (the active player).
    fn legal_actions(&self, py: Python<'_>) -> Vec<Vec<(usize, String)>> {
        py.allow_threads(|| {
            self.envs
                .iter()
                .map(|env| {
                    env.legal_actions()
                        .into_iter()
                        .enumerate()
                        .map(|(i, a)| (i, format!("{a:?}")))
                        .collect()
                })
                .collect()
        })
    }

    /// Steps every env with its own action index.
    ///
    /// Returns per-env `(observation, reward, done, winner)`; rewards are each
    /// env's own perspective (the seat it was built with).
    fn step(
        &mut self,
        py: Python<'_>,
        indices: Vec<usize>,
    ) -> (Vec<Vec<f32>>, Vec<f64>, Vec<bool>, Vec<Option<u8>>) {
        py.allow_threads(|| {
            let mut obs = Vec::with_capacity(self.envs.len());
            let mut rewards = Vec::with_capacity(self.envs.len());
            let mut done = Vec::with_capacity(self.envs.len());
            let mut winners = Vec::with_capacity(self.envs.len());
            for (env, index) in self.envs.iter_mut().zip(indices) {
                let result = env.step_indexed(index);
                obs.push(result.observation);
                rewards.push(result.reward as f64);
                done.push(result.done);
                winners.push(result.winner.map(|p| p.index() as u8));
            }
            (obs, rewards, done, winners)
        })
    }

    /// Per-env `done` flags.
    fn done(&self, py: Python<'_>) -> Vec<bool> {
        py.allow_threads(|| self.envs.iter().map(|env| env.is_done()).collect())
    }

    /// Per-env winner (`None` = not over / draw); `0`-based player index.
    fn winners(&self, py: Python<'_>) -> Vec<Option<u8>> {
        py.allow_threads(|| {
            self.envs
                .iter()
                .map(|env| env.winner().map(|p| p.index() as u8))
                .collect()
        })
    }

    /// Active player per env (`0` = P1, `1` = P2) — the batch trainer needs it
    /// to know whose decision point each env is at.
    fn active_players(&self, py: Python<'_>) -> Vec<u8> {
        py.allow_threads(|| {
            self.envs
                .iter()
                .map(|env| env.game_state().active_player().index() as u8)
                .collect()
        })
    }

    /// Structured observations (M4): one per env, **active player's view**.
    fn structured_observations(&self, py: Python<'_>) -> Vec<PyObservation> {
        let views: Vec<_> = py.allow_threads(|| {
            self.envs
                .iter()
                .map(|env| {
                    crate::rl::views::observation(
                        env.game_state(),
                        env.game_state().active_player(),
                        env.is_done(),
                    )
                })
                .collect()
        });
        views.iter().map(PyObservation::from).collect()
    }

    /// Structured legal actions per env (active player; M1-G3 shape).
    fn structured_legal_actions(&self, py: Python<'_>) -> Vec<Vec<PyActionView>> {
        let views: Vec<_> = py.allow_threads(|| {
            self.envs
                .iter()
                .map(|env| crate::rl::views::action_views(env.game_state()))
                .collect()
        });
        views
            .iter()
            .map(|v| v.iter().map(PyActionView::from).collect())
            .collect()
    }
}

/// Bot-vs-bot battles on a fixed mirror deck, advanced in parallel by the
/// rayon batch simulator (roadmap M4 benchmark / engine-limit throughput).
///
/// Returns `[(winner, turns), ...]` in input order; `winner` is `None` when
/// the step cap was hit. Each game is deterministic per seed — identical to
/// single-threaded execution regardless of thread scheduling.
#[pyfunction]
#[pyo3(signature = (seeds, deck, bot="greedy", max_steps=5000))]
fn battle_batch(
    seeds: Vec<u64>,
    deck: Vec<String>,
    bot: &str,
    max_steps: u32,
) -> PyResult<Vec<(Option<u8>, u32)>> {
    let bot_type = match bot {
        "greedy" => BotType::Greedy,
        "smart" => BotType::Smart,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "unknown bot type: {other} (expected greedy | smart)"
            )));
        }
    };
    let cards: Vec<&'static crate::cards::def::CardDef> = deck
        .iter()
        .filter_map(|id| crate::cards::card_by_id(id))
        .collect();
    if cards.len() != deck.len() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "deck contains unknown card id",
        ));
    }
    // Per-game runner seeds make each battle's shuffle/opening deterministic.
    let mut states = Vec::with_capacity(seeds.len());
    for seed in seeds {
        let mut runner = crate::sim::battle::BattleRunner::new(bot_type, seed);
        let state = runner.create_game_state_with_decks(&cards, &cards, 3, true);
        states.push(state);
    }
    let sim = crate::sim::batch::BatchSimulator::new(bot_type, max_steps);
    let outcomes = sim.run(states);
    Ok(outcomes
        .into_iter()
        .map(|o| (o.winner.map(|p| p.index() as u8), o.turn))
        .collect())
}

/// Entry point of the `orange_stone` extension module.
#[pymodule]
fn orange_stone(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGameEnv>()?;
    m.add_class::<PyBatchEnv>()?;
    m.add_class::<views::PyEntityView>()?;
    m.add_class::<views::PyPlayerView>()?;
    m.add_class::<views::PyObservation>()?;
    m.add_class::<views::PyActionView>()?;
    m.add_function(wrap_pyfunction!(battle_batch, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
