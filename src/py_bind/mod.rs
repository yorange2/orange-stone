//! Python 绑定 — PyO3 扩展模块 `orange_stone`。
//!
//! 通过 `cargo build --features py` / maturin 构建。暴露 Gym 风格环境：
//!
//! ```python
//! import orange_stone as os
//! env = os.GameEnv(seed=42)
//! obs = env.reset(seed=1)          # 168 维观察向量
//! actions = env.legal_actions()    # [(索引, 描述), ...]
//! obs, reward, done, winner = env.step(action_index)
//! ```
//!
//! 动作空间是逐回合的合法动作索引列表（`legal_actions()` 返回），
//! 索引在同一个回合内稳定；`EndTurn` 后环境自动推进对手回合。

use crate::core::player::PlayerId;
use crate::rl::env::{EnvConfig, GameEnv};
use crate::rl::obs::OBS_LEN;
use crate::sim::battle::BotType;
use pyo3::prelude::*;

/// Gym 风格环境的 Python 包装。
#[pyclass(name = "GameEnv")]
struct PyGameEnv {
    env: GameEnv,
}

#[pymethods]
impl PyGameEnv {
    /// 创建环境。`perspective` 为 agent 视角（0 = 先手，1 = 后手）。
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

    /// 观察向量长度（固定 168）。
    #[staticmethod]
    fn obs_len() -> usize {
        OBS_LEN
    }

    /// 重置环境，返回初始观察。
    fn reset(&mut self, seed: u64) -> Vec<f32> {
        self.env.reset(seed)
    }

    /// 当前观察（agent 视角）。
    fn observation(&self) -> Vec<f32> {
        self.env.observation()
    }

    /// 合法动作列表：`[(索引, 可读描述), ...]`。
    fn legal_actions(&self) -> Vec<(usize, String)> {
        self.env
            .legal_actions()
            .into_iter()
            .enumerate()
            .map(|(i, a)| (i, format!("{a:?}")))
            .collect()
    }

    /// 执行第 `action_index` 个合法动作。
    ///
    /// 返回 `(observation, reward, done, winner)`；`winner` 为 `None`
    /// 表示未结束或平局。
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

/// `orange_stone` 扩展模块入口。
#[pymodule]
fn orange_stone(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGameEnv>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
