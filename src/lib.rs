//! # Orange Stone — Hearthstone simulator
//!
//! A high-performance Hearthstone simulator written in Rust, designed specifically for reinforcement learning training.
//!
//! ## Architecture
//!
//! - **ECS (Entity Component System)**: card effects as composable Components
//! - **Immutable GameState + Copy-on-Write**: low-cost state branching, MCTS friendly
//! - **Event-driven rule engine**: Action → Event → Trigger → new Action
//!
//! ## Phase 3 (current)
//!
//! Full rules: weapons, hero powers, auras, secrets, complex card interaction sequencing

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod cards;
pub mod core;
pub mod engine;
pub mod rl;
pub mod sim;

// Python bindings (requires the `py` feature)
#[cfg(feature = "py")]
pub mod py_bind;
