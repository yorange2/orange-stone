//! Trigger 效果解析器 — 将 CardEffect 转化为事件入队。
//!
//! 当规则引擎检测到 Battlecry/Deathrattle 组件时，
//! 调用此模块的函数将效果解析为具体的游戏事件并入队。
//!
//! # 目标选择
//!
//! 效果的目标通过 `EffectTarget` 枚举指定：
//! - `AnyEnemy` → 随机敌方英雄或随从
//! - `AnyEnemyMinion` → 随机敌方随从
//! - `EnemyHero` → 敌方英雄
//! - `Self_` → 效果来源实体自身
//! - `AllEnemyMinions` → 所有敌方随从

use crate::core::component::{Attack, CardType, Cost, Durability, Freeze, Health};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue};
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// 将 CardEffect 解析为游戏事件并入队。
///
/// `source` 是效果来源实体（拥有该效果的随从）。
/// `owner` 是来源实体的所属玩家。
pub fn resolve_effect(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    effect: CardEffect,
) {
    match effect {
        CardEffect::DealDamage { amount, target } => {
            resolve_deal_damage(state, queue, source, owner, amount, target);
        }
        CardEffect::DrawCard { count } => {
            for _ in 0..count {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::SummonMinion { card_id } => {
            resolve_summon(state, queue, source, owner, card_id);
        }
        CardEffect::GainStats {
            attack,
            health,
            target,
        } => {
            resolve_gain_stats(state, queue, source, owner, attack, health, target);
        }
        CardEffect::EquipWeapon { card_id } => {
            resolve_equip_weapon(state, queue, owner, card_id);
        }
        CardEffect::GainArmor { amount, target } => {
            resolve_gain_armor(state, owner, amount, target);
        }
        CardEffect::ReturnToHand { target } => {
            resolve_return_to_hand(state, queue, owner, target);
        }
        CardEffect::IncreaseCost { amount, target } => {
            resolve_increase_cost(state, owner, amount, target);
        }
        CardEffect::DestroyMinion { target } => {
            resolve_destroy_minion(state, queue, owner, target);
        }
        CardEffect::SilenceMinion { target } => {
            resolve_silence(state, owner, target);
        }
        CardEffect::SetAttack { attack, target } => {
            resolve_set_attack(state, owner, attack, target);
        }
        CardEffect::RestoreHealth { amount, target } => {
            resolve_restore_health(state, owner, amount, target);
        }
        CardEffect::FreezeCharacter { target } => {
            resolve_freeze(state, owner, target);
        }
        CardEffect::GainManaCrystal { count } => {
            let inner = state.make_mut();
            let p = &mut inner.players[owner.index()];
            p.mana_crystals = (p.mana_crystals + count).min(10);
            p.current_mana += count;
        }
        CardEffect::DestroyWeapon => {
            let enemy = owner.opponent();
            if let Some(weapon) = state.player(enemy).weapon {
                let inner = state.make_mut();
                inner.players[enemy.index()].weapon = None;
                queue.push(Event::WeaponDestroyed {
                    player: enemy,
                    weapon,
                });
            }
        }
    }
}

/// 从牌库随机抽一张牌到手中。
pub fn draw_card(state: &mut GameState, queue: &mut EventQueue, player: PlayerId) {
    let deck_len = state.world().zones().len(Zone::Deck, player);
    if deck_len == 0 {
        // 牌库空，不抽牌（疲劳 Phase 3+）
        return;
    }

    // 从牌库随机选一张
    let idx = state.rng_mut().next_usize(deck_len);
    let card = state
        .world()
        .zones()
        .iter(Zone::Deck, player)
        .nth(idx)
        .expect("deck card should exist");

    // 移到手牌
    state
        .world_mut()
        .move_to_zone(card, Zone::Hand)
        .expect("card should be movable to hand");

    queue.push(Event::CardDrawn { player, card });
}

/// 解析伤害效果。
fn resolve_deal_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
) {
    let enemies = match target {
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::EnemyHero => {
            let hero = state.player(owner.opponent()).hero;
            vec![hero]
        }
        EffectTarget::AllEnemyMinions => {
            let minions = collect_enemy_minions(state, owner);
            for minion in &minions {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount,
                });
            }
            return;
        }
        EffectTarget::AllFriendlyMinions => {
            let minions = collect_friendly_minions(state, owner);
            for minion in &minions {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount,
                });
            }
            return;
        }
        EffectTarget::Self_ => {
            queue.push(Event::DamageDealt {
                source,
                target: source,
                amount,
            });
            return;
        }
        EffectTarget::AllMinions
        | EffectTarget::AllCharacters
        | EffectTarget::FriendlyHero
        | EffectTarget::DamagedEnemyMinion => {
            // 这些目标类型不在 DealDamage 中使用
            return;
        }
    };

    if enemies.is_empty() {
        return;
    }

    // 随机选择一个目标
    let idx = state.rng_mut().next_usize(enemies.len());
    let target_entity = enemies[idx];

    queue.push(Event::DamageDealt {
        source,
        target: target_entity,
        amount,
    });
}

/// 解析召唤随从效果。
fn resolve_summon(
    state: &mut GameState,
    queue: &mut EventQueue,
    _source: Entity,
    owner: PlayerId,
    card_id: &str,
) {
    // 查找卡牌定义
    let Some(card_def) = crate::cards::def::card_by_id(card_id) else {
        return;
    };

    // 检查战场上限
    let board_count = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .count();
    if board_count >= crate::engine::rules::MAX_BOARD_SIZE {
        return;
    }

    // 创建随从实体并放到战场
    let e = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(card_def.health));
        world.set_attack(e, Attack(card_def.attack));
        world.set_cost(e, crate::core::component::Cost(card_def.cost));
        world.set_card_type(e, card_def.card_type);
        world.set_player(e, owner);
        world.set_attacks_used(e, crate::core::component::AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, owner, e);
        // 设置光环、战吼、亡语、嘲讽（如果有）
        if let Some((aura_effect, aura_target)) = card_def.aura {
            world.set_aura(
                e,
                crate::core::component::Aura {
                    effect: aura_effect,
                    target: aura_target,
                },
            );
        }
        if let Some(bc) = card_def.battlecry {
            world.set_battlecry(e, crate::core::component::Battlecry(bc));
        }
        if let Some(dr) = card_def.deathrattle {
            world.set_deathrattle(e, crate::core::component::Deathrattle(dr));
        }
        if card_def.taunt {
            world.set_taunt(e, crate::core::component::Taunt);
        }
        e
    };

    // 入队 MinionSummoned 事件（触发战吼等效果）
    queue.push(Event::MinionSummoned {
        player: owner,
        minion: e,
    });
}

/// 解析 buff 效果。
fn resolve_gain_stats(
    state: &mut GameState,
    _queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    attack: i32,
    health: i32,
    target: EffectTarget,
) {
    match target {
        EffectTarget::Self_ => {
            let world = state.world_mut();
            let cur_atk = world.attack(source).unwrap_or(Attack(0));
            let cur_hp = world.health(source).unwrap_or(Health(0));
            world.set_attack(source, Attack(cur_atk.0 + attack));
            world.set_health(source, Health(cur_hp.0 + health));
        }
        EffectTarget::AllFriendlyMinions => {
            // 先收集，再获取可变引用
            let minions: Vec<Entity> = collect_friendly_minions(state, owner);
            let world = state.world_mut();
            for minion in &minions {
                let cur_atk = world.attack(*minion).unwrap_or(Attack(0));
                let cur_hp = world.health(*minion).unwrap_or(Health(0));
                world.set_attack(*minion, Attack(cur_atk.0 + attack));
                world.set_health(*minion, Health(cur_hp.0 + health));
            }
        }
        _ => {
            // Phase 2 不支持其他 buff 目标类型
        }
    }
}

/// 装备武器。
fn resolve_equip_weapon(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    card_id: &str,
) {
    let Some(card_def) = crate::cards::def::card_by_id(card_id) else {
        return;
    };

    // 如果已有武器，先摧毁旧武器
    let old_weapon = state.player(owner).weapon;
    if let Some(w) = old_weapon {
        queue.push(Event::WeaponDestroyed {
            player: owner,
            weapon: w,
        });
    }

    // 创建武器实体并更新 Player
    let inner = state.make_mut();
    let weapon = inner.world.spawn();
    inner.world.set_attack(weapon, Attack(card_def.attack));
    inner
        .world
        .set_durability(weapon, Durability(card_def.durability));
    inner.world.set_cost(weapon, Cost(card_def.cost));
    inner.world.set_card_type(weapon, CardType::Weapon);
    inner.world.set_player(weapon, owner);
    inner.world.set_zone(weapon, Zone::Play);
    inner.world.zones_mut().insert(Zone::Play, owner, weapon);
    inner.players[owner.index()].weapon = Some(weapon);

    queue.push(Event::WeaponEquipped {
        player: owner,
        weapon,
    });
}

/// 获得护甲。
fn resolve_gain_armor(state: &mut GameState, owner: PlayerId, amount: i32, target: EffectTarget) {
    match target {
        EffectTarget::Self_ => {
            let inner = state.make_mut();
            inner.players[owner.index()].armor += amount;
        }
        EffectTarget::EnemyHero => {
            let inner = state.make_mut();
            inner.players[owner.opponent().index()].armor += amount;
        }
        _ => {
            // 其他目标暂不支持
        }
    }
}

/// 将随从移回手牌。
fn resolve_return_to_hand(
    state: &mut GameState,
    _queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
) {
    let _enemy = owner.opponent();
    let minions = match target {
        EffectTarget::AnyEnemy => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };

    if minions.is_empty() {
        return;
    }

    let idx = state.rng_mut().next_usize(minions.len());
    let target_entity = minions[idx];

    // 移到手牌
    let _ = state.world_mut().move_to_zone(target_entity, Zone::Hand);
}

/// 增加随从的法力消耗（如冰冻陷阱效果）。
fn resolve_increase_cost(
    state: &mut GameState,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
) {
    let _enemy = owner.opponent();
    let minions = match target {
        EffectTarget::AnyEnemy => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };

    if minions.is_empty() {
        return;
    }

    let idx = state.rng_mut().next_usize(minions.len());
    let target_entity = minions[idx];

    let world = state.world_mut();
    let cur_cost = world.cost(target_entity).unwrap_or(Cost(0));
    world.set_cost(target_entity, Cost(cur_cost.0 + amount));
}

/// 消灭随从 — 造成等于当前生命值的伤害来确保击杀。
fn resolve_destroy_minion(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
) {
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::DamagedEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };
    for &m in &minions {
        let hp = state.world().health(m).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: m,
            target: m,
            amount: hp.0.max(1),
        });
    }
}

/// 沉默随从 — 移除所有效果组件。
fn resolve_silence(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_enemy_minions(state, owner));
            all
        }
        _ => return,
    };
    for &m in &minions {
        let world = state.world_mut();
        world.remove_taunt(m);
        world.remove_battlecry(m);
        world.remove_deathrattle(m);
        world.remove_aura(m);
        world.remove_divine_shield(m);
        world.remove_windfury(m);
        world.remove_charge(m);
        world.remove_spell_damage(m);
    }
}

/// 设置攻击力。
fn resolve_set_attack(state: &mut GameState, owner: PlayerId, attack: i32, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    state.world_mut().set_attack(minions[idx], Attack(attack));
}

/// 恢复生命值。
fn resolve_restore_health(
    state: &mut GameState,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
) {
    match target {
        EffectTarget::FriendlyHero => {
            let hero = state.player(owner).hero;
            let cur = state.world().health(hero).unwrap_or(Health(0));
            state
                .world_mut()
                .set_health(hero, Health((cur.0 + amount).min(30)));
        }
        EffectTarget::Self_ => {
            // source is the effect source, but here we use hero
            let hero = state.player(owner).hero;
            let cur = state.world().health(hero).unwrap_or(Health(0));
            state
                .world_mut()
                .set_health(hero, Health((cur.0 + amount).min(30)));
        }
        EffectTarget::AllFriendlyMinions => {
            let minions = collect_friendly_minions(state, owner);
            let world = state.world_mut();
            for &m in &minions {
                let cur = world.health(m).unwrap_or(Health(0));
                world.set_health(m, Health((cur.0 + amount).min(30)));
            }
        }
        _ => {}
    }
}

/// 冻结角色。
fn resolve_freeze(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let targets: Vec<Entity> = match target {
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AllEnemyMinions => collect_enemy_minions(state, owner),
        EffectTarget::AllCharacters => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_enemy_characters(state, owner));
            all
        }
        _ => return,
    };
    for &t in &targets {
        state.world_mut().set_freeze(t, Freeze);
    }
}

// ============================================================
// 辅助函数
// ============================================================

/// 收集敌方所有角色（英雄 + 随从）。
fn collect_enemy_characters(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    let enemy = owner.opponent();
    let mut chars: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| {
            let ct = state.world().card_type(e);
            ct == Some(CardType::Minion) || ct == Some(CardType::Hero)
        })
        .collect();
    chars.push(state.player(enemy).hero);
    // 去重
    chars.sort_by_key(|e| (e.index, e.generation));
    chars.dedup();
    chars
}

/// 收集敌方所有随从。
fn collect_enemy_minions(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    let enemy = owner.opponent();
    state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect()
}

/// 收集友方所有随从。
fn collect_friendly_minions(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect()
}
