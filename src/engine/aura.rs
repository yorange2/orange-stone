//! 光环系统 — Aura 组件的查询和管理。
//!
//! 光环采用动态计算方案：不修改实体的基础属性，
//! 而是在查询 `effective_attack` / `effective_health` 时
//! 遍历所有存活的 `Aura` 源并累积叠加 buff。
//!
//! 光环的计算逻辑位于 `World::effective_attack` 和
//! `World::effective_health` 中（在 `src/core/world.rs`），
//! 以及辅助函数 `aura_applies_to`、`is_adjacent` 等。
//!
//! 本模块目前提供光环相关的辅助类型和工具函数。
//! 未来可能扩展为光环更新事件和增量计算优化。

use crate::core::component::{Aura, AuraEffect, AuraTarget};

/// 计算光环对目标实体的攻击力加成总和。
///
/// 遍历场上所有存活的光环源，返回所有匹配光环的攻击力加成之和。
/// 此函数供 `World::effective_attack` 内部使用。
#[must_use]
pub fn compute_aura_attack_bonus(
    auras: &[(crate::core::entity::Entity, &Aura)],
    target: crate::core::entity::Entity,
    target_player: crate::core::player::PlayerId,
    world: &crate::core::world::World,
) -> i32 {
    let mut bonus = 0i32;
    for (source, aura) in auras {
        if !world.is_alive(*source) {
            continue;
        }
        if world.zone(*source) != Some(crate::core::zone::Zone::Play) {
            continue;
        }
        let aura_player = match world.player(*source) {
            Some(p) => p,
            None => continue,
        };
        if aura_applies_to_entity(*aura, *source, aura_player, target, target_player, world) {
            bonus += aura_attack_value(aura.effect);
        }
    }
    bonus
}

/// 计算光环对目标实体的生命值加成总和。
#[must_use]
pub fn compute_aura_health_bonus(
    auras: &[(crate::core::entity::Entity, &Aura)],
    target: crate::core::entity::Entity,
    target_player: crate::core::player::PlayerId,
    world: &crate::core::world::World,
) -> i32 {
    let mut bonus = 0i32;
    for (source, aura) in auras {
        if !world.is_alive(*source) {
            continue;
        }
        if world.zone(*source) != Some(crate::core::zone::Zone::Play) {
            continue;
        }
        let aura_player = match world.player(*source) {
            Some(p) => p,
            None => continue,
        };
        if aura_applies_to_entity(*aura, *source, aura_player, target, target_player, world) {
            bonus += aura_health_value(aura.effect);
        }
    }
    bonus
}

/// 检查光环效果是否作用于目标实体。
fn aura_applies_to_entity(
    aura: Aura,
    aura_source: crate::core::entity::Entity,
    aura_player: crate::core::player::PlayerId,
    target: crate::core::entity::Entity,
    target_player: crate::core::player::PlayerId,
    world: &crate::core::world::World,
) -> bool {
    use crate::core::component::CardType;

    // 目标必须是存活的随从
    if world.card_type(target) != Some(CardType::Minion) {
        return false;
    }
    if !world.is_alive(target) {
        return false;
    }

    match aura.target {
        AuraTarget::AllFriendlyMinions => target_player == aura_player,
        AuraTarget::OtherFriendlyMinions => {
            target_player == aura_player && target != aura_source
        }
        AuraTarget::AdjacentMinions => {
            if target_player != aura_player || target == aura_source {
                return false;
            }
            is_adjacent_to(aura_source, target, aura_player, world)
        }
        AuraTarget::AllEnemyMinions => target_player != aura_player,
    }
}

/// 检查两个实体在战场上是否相邻。
fn is_adjacent_to(
    source: crate::core::entity::Entity,
    target: crate::core::entity::Entity,
    player: crate::core::player::PlayerId,
    world: &crate::core::world::World,
) -> bool {
    use crate::core::component::CardType;
    use crate::core::zone::Zone;

    let minions: Vec<crate::core::entity::Entity> = world
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| world.card_type(e) == Some(CardType::Minion) && world.is_alive(e))
        .collect();

    let source_pos = minions.iter().position(|&e| e == source);
    let target_pos = minions.iter().position(|&e| e == target);

    match (source_pos, target_pos) {
        (Some(s), Some(t)) => (s as isize - t as isize).unsigned_abs() == 1,
        _ => false,
    }
}

/// 返回光环效果的攻击力加成值。
const fn aura_attack_value(effect: AuraEffect) -> i32 {
    match effect {
        AuraEffect::GainStats { attack, .. } => attack,
        AuraEffect::GainAttack(a) => a,
        AuraEffect::GainHealth(_) => 0,
    }
}

/// 返回光环效果的生命值加成值。
const fn aura_health_value(effect: AuraEffect) -> i32 {
    match effect {
        AuraEffect::GainStats { health, .. } => health,
        AuraEffect::GainAttack(_) => 0,
        AuraEffect::GainHealth(h) => h,
    }
}
