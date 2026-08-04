//! 卡牌定义模块 — 基本卡牌数据。
//!
//! 包含 CardDef 结构体、vanilla! 宏，以及所有怀旧系列卡牌定义。
//! 所有卡牌常量通过 `def` 模块统一 re-export，
//! 外部代码可通过 `crate::cards::def::*` 访问。

pub mod classic_druid;
pub mod classic_hunter;
pub mod classic_legendary;
pub mod classic_mage;
pub mod classic_neutral;
pub mod classic_paladin;
pub mod classic_priest;
pub mod classic_rogue;
pub mod classic_shaman;
pub mod classic_warlock;
pub mod classic_warrior;
pub mod def;
pub mod sets;
