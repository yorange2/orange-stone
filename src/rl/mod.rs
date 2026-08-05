//! Reinforcement learning interface — Gym-style environment, observation space, and reward functions.
//!
//! `GameEnv` is a gym environment where a single agent plays against a scripted opponent:
//! - `obs.rs` tensorizes `GameState` into a fixed-length observation (168 dimensions)
//! - `reward.rs` provides configurable sparse/dense rewards
//! - `env.rs` provides the `reset`/`step`/`legal_actions` loop
//!
//! The Python bindings (`py_bind`, requires the `py` feature) wrap `PyGameEnv` on top of it.

pub mod env;
pub mod obs;
pub mod reward;
