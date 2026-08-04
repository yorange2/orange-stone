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
//! ## Phase 1 (当前)
//!
//! 核心框架：ECS、GameState、基础 Action/Event、白板随从对战

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod cards;
pub mod core;
pub mod engine;
pub mod sim;
