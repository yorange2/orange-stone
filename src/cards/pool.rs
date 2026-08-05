//! 随机卡池 — 从 Classic 池中按条件过滤抽样。
//!
//! 满足卡池封闭性：所有抽样池都是 `ALL_CARDS`（全部为 Classic 卡牌）
//! 或内置 token 池的过滤子集，不会引入 Classic 系列之外的卡牌。
//!
//! 种族（野兽/恶魔）未建模，因此这些池按卡牌 ID 硬编码；
//! 传说/职业过滤基于 `sets` 中的分组列表动态计算。

use crate::cards::def::{CardDef, card_by_id};
use crate::core::component::CardType;
use crate::core::effect::RandomPool;
use crate::sim::rng::GameRng;

/// 野兽池 — Classic 野兽卡牌 ID。
pub const BEAST_POOL: &[&str] = &[
    "NEUTRAL_B08", // Bloodfen Raptor / Ironfur Grizzly
    "NEUTRAL_B03", // River Crocolisk
    "NEUTRAL_B11", // Silverback Patriarch
    "NEUTRAL_C04", // Stonetusk Boar / Young Dragonhawk
    "NEUTRAL_T14", // Stranglethorn Tiger
    "NEUTRAL_C10", // Jungle Panther
    "NEUTRAL_B13", // Oasis Snapjaw
    "NEUTRAL_R20", // Stampeding Kodo
    "NEUTRAL_T05", // River Crocolisk (Tier 1)
    "HUNTER_010",  // Timber Wolf
    "HUNTER_006",  // Savannah Highmane
    "HUNTER_011",  // King Krush
    "HUNTER_014",  // Starving Buzzard
    "HUNTER_016",  // Tundra Rhino
];

/// 恶魔池 — Classic 恶魔卡牌 ID。
pub const DEMON_POOL: &[&str] = &[
    "WARLOCK_004", // Voidwalker
    "WARLOCK_002", // Flame Imp
    "WARLOCK_007", // Doomguard
    "WARLOCK_011", // Dread Infernal
    "WARLOCK_012", // Pit Lord
    "WARLOCK_016", // Felstalker
    "WARLOCK_019", // Felguard
    "WARLOCK_022", // Void Terror
];

/// 梦境卡池 — Classic 内置 token（伊瑟拉）。
pub const DREAM_POOL: &[&str] = &[
    "NEUTRAL_T21a", // Emerald Drake
    "NEUTRAL_T21b", // Laughing Sister
    "NEUTRAL_T21c", // Dream
    "NEUTRAL_T21d", // Nightmare
    "NEUTRAL_T21e", // Ysera Awakens
];

/// 动物伙伴池 — Classic 内置 token。
pub const COMPANION_POOL: &[&str] = &["HUNTER_023a", "HUNTER_023b", "HUNTER_023c"];

/// 从 ID 池中随机抽取一张卡牌定义。
pub(crate) fn random_from_pool(pool: &[&str], rng: &mut GameRng) -> Option<&'static CardDef> {
    if pool.is_empty() {
        return None;
    }
    let idx = rng.next_usize(pool.len());
    card_by_id(pool[idx])
}

/// 按卡池类型随机抽取一张卡牌定义。
pub(crate) fn random_card(rng: &mut GameRng, pool: RandomPool) -> Option<&'static CardDef> {
    match pool {
        RandomPool::Beast => random_from_pool(BEAST_POOL, rng),
        RandomPool::Demon => random_from_pool(DEMON_POOL, rng),
        RandomPool::Dream => random_from_pool(DREAM_POOL, rng),
        RandomPool::Companion => random_from_pool(COMPANION_POOL, rng),
        RandomPool::Legendary => random_filtered(rng, |c| {
            c.card_type == CardType::Minion
                && crate::cards::sets::LEGENDARY_CLASSIC
                    .iter()
                    .any(|l| l.id == c.id)
        }),
        RandomPool::MageSpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell
                && crate::cards::sets::MAGE_CLASSIC
                    .iter()
                    .any(|m| m.id == c.id)
        }),
        RandomPool::ShadowSpell => random_filtered(rng, |c| {
            c.card_type == CardType::Spell && c.name.contains("Shadow")
        }),
        RandomPool::OtherClass => random_filtered(rng, |c| {
            // 偷窃：另一职业的随机卡牌 — 简化为非盗贼卡牌
            !crate::cards::sets::ROGUE_CLASSIC
                .iter()
                .any(|r| r.id == c.id)
        }),
    }
}

/// 从全卡池中按谓词过滤后随机抽取。
fn random_filtered(
    rng: &mut GameRng,
    predicate: impl Fn(&CardDef) -> bool,
) -> Option<&'static CardDef> {
    let pool: Vec<&CardDef> = crate::cards::sets::ALL_CARDS
        .iter()
        .filter(|c| predicate(c))
        .collect();
    if pool.is_empty() {
        return None;
    }
    let idx = rng.next_usize(pool.len());
    Some(pool[idx])
}
