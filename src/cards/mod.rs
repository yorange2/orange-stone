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

use crate::core::component::{Poison, Stealth};
use crate::core::entity::Entity;
use crate::core::world::World;
use def::CardDef;

/// 在实体上应用特殊关键词组件（剧毒、潜行等）。
///
/// 这些关键词不新增 `CardDef` 字段（避免大面积结构体改动），
/// 而是按卡牌 ID 在此集中映射。召唤随从（`trigger::resolve_summon`）
/// 和构建卡牌（`GameBuilder::spawn_minion`）时调用。
pub(crate) fn apply_card_keywords(world: &mut World, entity: Entity, card_def: &CardDef) {
    if card_def.id == "ROGUE_022" {
        // 耐心的刺客 — 潜行 + 剧毒
        world.set_poison(entity, Poison);
        world.set_stealth(entity, Stealth);
    }
}
