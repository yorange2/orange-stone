//! 由 build.rs 生成的卡牌常量（数据源：`cards/classic_cards.json`）。
//!
//! 生成代码只包含静态属性与关键词；效果（战吼/亡语/光环/奥秘）仍需手写。
//! 生成的卡牌与手写常量的一致性由 `cards/mod.rs` 的测试验证。

#![allow(missing_docs)]

use crate::cards::def::CardDef;
use crate::core::component::CardType;

include!(concat!(env!("OUT_DIR"), "/cards_generated.rs"));
