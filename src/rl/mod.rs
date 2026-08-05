//! 强化学习接口 — Gym 风格环境、观察空间与奖励函数。
//!
//! `GameEnv` 是单 agent 与脚本对手对弈的 gym 环境：
//! - `obs.rs` 将 `GameState` 张量化为固定长度观察（168 维）
//! - `reward.rs` 提供可配置的稀疏/密集奖励
//! - `env.rs` 提供 `reset`/`step`/`legal_actions` 循环
//!
//! Python 绑定（`py_bind`，需 `py` feature）在其上封装 `PyGameEnv`。

pub mod env;
pub mod obs;
pub mod reward;
