//! # Orange Stone — 炉石传说模拟器
//!
//! 用 Rust 编写的高性能炉石传说（Hearthstone）模拟器，专为强化学习训练而设计。
//!
//! ## 架构
//!
//! - **ECS (Entity Component System)**：卡牌效果作为可组合的 Component
//! - **不可变 GameState + Copy-on-Write**：低成本状态分支，MCTS 友好
//! - **事件驱动规则引擎**：Action → Event → Trigger → 新 Action
//!
//! ## Phase 3 (当前)
//!
//! 完整规则：武器、英雄技能、光环、奥秘、复杂卡牌交互时序

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod cards;
pub mod core;
pub mod engine;
pub mod rl;
pub mod sim;

// Python 绑定（需 `py` feature）
#[cfg(feature = "py")]
pub mod py_bind;
