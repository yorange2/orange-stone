//! Python bindings — the `orange_stone` PyO3 extension module.
//!
//! Built via `cargo build --features py` / maturin. Exposes the Gym-style environment:
//!
//! ```python
//! import orange_stone as os
//! env = os.GameEnv(seed=42)
//! obs = env.reset(seed=1)          # 168-dim observation vector
//! actions = env.legal_actions()    # [(index, description), ...]
//! obs, reward, done, winner = env.step(action_index)
//! ```
//!
//! The action space is the per-turn list of legal action indices (returned by `legal_actions()`);
//! indices are stable within a turn; after `EndTurn`, the environment advances the opponent's turn automatically.

use crate::core::player::PlayerId;
use crate::rl::env::{EnvConfig, GameEnv};
use crate::rl::obs::OBS_LEN;
use crate::sim::battle::BotType;
use pyo3::prelude::*;

/// Python wrapper for the Gym-style environment.
#[pyclass(name = "GameEnv")]
struct PyGameEnv {
    env: GameEnv,
}

#[pymethods]
impl PyGameEnv {
    /// Creates the environment. `perspective` is the agent's perspective (0 = first player, 1 = second player).
    #[new]
    #[pyo3(signature = (seed, perspective=0, deck_size=30))]
    fn new(seed: u64, perspective: u8, deck_size: usize) -> PyResult<Self> {
        let perspective = match perspective {
            0 => PlayerId::Player1,
            _ => PlayerId::Player2,
        };
        let config = EnvConfig::default_with(BotType::Greedy, deck_size);
        let mut env = GameEnv::new(perspective, config);
        env.reset(seed);
        Ok(Self { env })
    }

    /// Observation vector length (fixed at 168).
    #[staticmethod]
    fn obs_len() -> usize {
        OBS_LEN
    }

    /// Resets the environment, returning the initial observation.
    fn reset(&mut self, seed: u64) -> Vec<f32> {
        self.env.reset(seed)
    }

    /// Current observation (agent's perspective).
    fn observation(&self) -> Vec<f32> {
        self.env.observation()
    }

    /// Legal actions: `[(index, readable description), ...]`.
    fn legal_actions(&self) -> Vec<(usize, String)> {
        self.env
            .legal_actions()
            .into_iter()
            .enumerate()
            .map(|(i, a)| (i, format!("{a:?}")))
            .collect()
    }

    /// Executes the `action_index`-th legal action.
    ///
    /// Returns `(observation, reward, done, winner)`; `winner` is `None`
    /// if the game is not over or it's a draw.
    fn step(&mut self, action_index: usize) -> (Vec<f32>, f64, bool, Option<u8>) {
        let result = self.env.step_indexed(action_index);
        let winner = result.winner.map(|p| p.index() as u8);
        (
            result.observation,
            result.reward as f64,
            result.done,
            winner,
        )
    }
}

/// Entry point of the `orange_stone` extension module.
#[pymodule]
fn orange_stone(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGameEnv>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
