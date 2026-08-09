//! Aura system — querying and managing Aura components.
//!
//! Auras are computed dynamically: base entity stats are not modified;
//! instead, when querying `effective_attack` / `effective_health`, all
//! living `Aura` sources are iterated and their buffs accumulated.
//!
//! Aura computation lives in `World::effective_attack` and
//! `World::effective_health` (in `src/core/world.rs`), along with helper
//! functions such as `aura_applies_to` and `is_adjacent`.
//!
//! This module currently provides aura-related helper types and utility
//! functions. It may later be extended with aura update events and
//! incremental computation optimizations.

use crate::core::component::{Aura, AuraEffect, AuraTarget};

/// Computes the total attack bonus from auras applied to the target entity.
///
/// Iterates all living aura sources on the battlefield and returns the sum
/// of attack bonuses from all matching auras.
/// Used internally by `World::effective_attack`.
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
        if aura_applies_to_entity(**aura, *source, aura_player, target, target_player, world) {
            bonus += aura_attack_value(aura.effect);
        }
    }
    bonus
}

/// Computes the total health bonus from auras applied to the target entity.
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
        if aura_applies_to_entity(**aura, *source, aura_player, target, target_player, world) {
            bonus += aura_health_value(aura.effect);
        }
    }
    bonus
}

/// Checks whether an aura effect applies to the target entity.
fn aura_applies_to_entity(
    aura: Aura,
    aura_source: crate::core::entity::Entity,
    aura_player: crate::core::player::PlayerId,
    target: crate::core::entity::Entity,
    target_player: crate::core::player::PlayerId,
    world: &crate::core::world::World,
) -> bool {
    use crate::core::component::CardType;

    // M4-W1 — FriendlyHero scope (Azshara's "Your hero has Windfury"):
    // the hero-targeting aura is decided before the minion gate below.
    if matches!(aura.target, AuraTarget::FriendlyHero) {
        return target_player == aura_player
            && world.card_type(target) == Some(CardType::Hero)
            && world.is_alive(target);
    }

    // The target must be a living minion
    if world.card_type(target) != Some(CardType::Minion) {
        return false;
    }
    if !world.is_alive(target) {
        return false;
    }

    match aura.target {
        AuraTarget::AllFriendlyMinions => target_player == aura_player,
        AuraTarget::OtherFriendlyMinions => target_player == aura_player && target != aura_source,
        AuraTarget::FriendlyRace(race) => {
            target_player == aura_player && world.has_race(target, race)
        }
        AuraTarget::OtherFriendlyRace(race) => {
            target_player == aura_player && target != aura_source && world.has_race(target, race)
        }
        AuraTarget::AdjacentMinions => {
            if target_player != aura_player || target == aura_source {
                return false;
            }
            is_adjacent_to(aura_source, target, aura_player, world)
        }
        AuraTarget::AllEnemyMinions => target_player != aura_player,
        AuraTarget::FriendlyHero => false,
    }
}

/// Checks whether two entities are adjacent on the battlefield.
fn is_adjacent_to(
    source: crate::core::entity::Entity,
    target: crate::core::entity::Entity,
    player: crate::core::player::PlayerId,
    world: &crate::core::world::World,
) -> bool {
    use crate::core::component::CardType;
    use crate::core::small_list::SmallList;
    use crate::core::zone::Zone;

    let minions: SmallList<crate::core::entity::Entity> = world
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

/// Returns the attack bonus value of an aura effect.
const fn aura_attack_value(effect: AuraEffect) -> i32 {
    match effect {
        AuraEffect::GainStats { attack, .. } => attack,
        AuraEffect::GainAttack(a) => a,
        AuraEffect::GainHealth(_) => 0,
        AuraEffect::ReduceSpellCost(_) => 0,
        AuraEffect::ReduceMinionCost { .. } => 0,
        AuraEffect::GrantCharge => 0,
        AuraEffect::FirstMinionDiscount { .. } => 0,
        AuraEffect::IncreaseMinionCost { .. } => 0,
        AuraEffect::IncreaseMinionCostFriendly { .. } => 0,
        AuraEffect::ChargeWithWeapon => 0,
        AuraEffect::DoubleTriggers => 0,
        AuraEffect::RewindKeepsBothOutcomes => 0,
        AuraEffect::GrantWindfury => 0,
    }
}

/// Returns the health bonus value of an aura effect.
const fn aura_health_value(effect: AuraEffect) -> i32 {
    match effect {
        AuraEffect::GainStats { health, .. } => health,
        AuraEffect::GainAttack(_) => 0,
        AuraEffect::GainHealth(h) => h,
        AuraEffect::ReduceSpellCost(_) => 0,
        AuraEffect::ReduceMinionCost { .. } => 0,
        AuraEffect::GrantCharge => 0,
        AuraEffect::FirstMinionDiscount { .. } => 0,
        AuraEffect::IncreaseMinionCost { .. } => 0,
        AuraEffect::IncreaseMinionCostFriendly { .. } => 0,
        AuraEffect::ChargeWithWeapon => 0,
        AuraEffect::DoubleTriggers => 0,
        AuraEffect::RewindKeepsBothOutcomes => 0,
        AuraEffect::GrantWindfury => 0,
    }
}
