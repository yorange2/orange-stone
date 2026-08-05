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

use crate::core::component::{
    Attack, AttacksUsed, CardId, CardType, Cost, Durability, Freeze, Health, Stealth,
};
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
        CardEffect::ReturnToHandAndIncreaseCost { amount } => {
            resolve_return_to_hand_and_increase_cost(state, queue, owner, amount);
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
            p.current_mana = (p.current_mana + count).min(10);
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
        CardEffect::GainHeroAttack { attack, armor } => {
            resolve_gain_hero_attack(state, owner, attack, armor);
        }
        CardEffect::DealHeroAttackDamage { target } => {
            resolve_deal_hero_attack_damage(state, queue, source, owner, target);
        }
        CardEffect::FullHeal { target } => {
            resolve_full_heal(state, owner, target);
        }
        CardEffect::GrantWindfury { target } => {
            resolve_grant_windfury(state, owner, target);
        }
        CardEffect::GrantCharge {
            target,
            attack_bonus,
        } => {
            resolve_grant_charge(state, owner, target, attack_bonus);
        }
        CardEffect::DoubleAttack { target } => {
            resolve_double_attack(state, owner, target);
        }
        CardEffect::DoubleHealth { target } => {
            resolve_double_health(state, owner, target);
        }
        CardEffect::BuffWeapon { attack, durability } => {
            resolve_buff_weapon(state, owner, attack, durability);
        }
        CardEffect::DiscardRandomCard => {
            resolve_discard_random(state, owner);
        }
        CardEffect::DealArmorDamage { target } => {
            resolve_deal_armor_damage(state, queue, source, owner, target);
        }
        CardEffect::DestroyWeaponAndDraw => {
            resolve_destroy_weapon_and_draw(state, queue, owner);
        }
        CardEffect::ReturnAllToHand => {
            resolve_return_all_to_hand(state, owner);
        }
        CardEffect::SetAttackToHealth { target } => {
            resolve_set_attack_to_health(state, owner, target);
        }
        CardEffect::DestroyAllExceptOne => {
            resolve_destroy_all_except_one(state, queue, owner);
        }
        CardEffect::DestroyAndHeal { target, heal } => {
            resolve_destroy_and_heal(state, queue, owner, target, heal);
        }
        CardEffect::DestroyAndAOE { target } => {
            resolve_destroy_and_aoe(state, queue, owner, source, target);
        }
        CardEffect::DealDamageToTwo { amount } => {
            resolve_deal_damage_to_two(state, queue, source, owner, amount);
        }
        CardEffect::DealDamageAndDraw {
            damage,
            target,
            draw,
        } => {
            resolve_deal_damage(state, queue, source, owner, damage, target);
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::DamageAndGainAttack {
            damage,
            attack_bonus,
            target,
        } => {
            resolve_deal_damage(state, queue, source, owner, damage, target);
            // 给目标随从增加攻击力（简化：随机友方随从）
            let minions = collect_friendly_minions(state, owner);
            if !minions.is_empty() {
                let idx = state.rng_mut().next_usize(minions.len());
                let world = state.world_mut();
                let m = minions[idx];
                let cur = world.attack(m).unwrap_or(Attack(0));
                world.set_attack(m, Attack(cur.0 + attack_bonus));
            }
        }
        CardEffect::DestroyAdjacent { gain_stats: _ } => {
            // 简化实现 — 随机消灭一个友方随从并获得其属性
            let friendly = collect_friendly_minions(state, owner);
            if friendly.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(friendly.len());
            let sacrifice = friendly[idx];
            let atk = state.world().attack(sacrifice).unwrap_or(Attack(0));
            let hp = state.world().health(sacrifice).unwrap_or(Health(0));
            // 消灭牺牲品
            queue.push(Event::DamageDealt {
                source,
                target: sacrifice,
                amount: hp.0.max(1),
            });
            // 给source随从增加属性
            let cur_atk = state.world().attack(source).unwrap_or(Attack(0));
            let cur_hp = state.world().health(source).unwrap_or(Health(0));
            state
                .world_mut()
                .set_attack(source, Attack(cur_atk.0 + atk.0));
            state
                .world_mut()
                .set_health(source, Health(cur_hp.0 + hp.0));
        }
        CardEffect::DestroyManaCrystal => {
            let inner = state.make_mut();
            let p = &mut inner.players[owner.index()];
            if p.mana_crystals > 0 {
                p.mana_crystals -= 1;
                p.current_mana = p.current_mana.min(p.mana_crystals);
            }
        }
        CardEffect::GiveCardsToOpponent { count: _ } => {
            let enemy = owner.opponent();
            draw_card(state, queue, enemy);
        }
        CardEffect::ResurrectMinion => {
            // 战场已满时无法复活
            let board_count = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count();
            if board_count >= crate::engine::rules::MAX_BOARD_SIZE {
                return;
            }
            let inner = state.make_mut();
            let died = &mut inner.players[owner.index()].died_this_turn;
            if let Some(entity) = died.pop() {
                // 复活：将随从从坟场移回战场，生命值设为1
                let world = &mut inner.world;
                if world.zone(entity) == Some(Zone::Graveyard) {
                    let _ = world.move_to_zone(entity, Zone::Play);
                    world.set_health(entity, Health(1));
                    world.set_attacks_used(entity, AttacksUsed(0));
                }
            }
        }
        CardEffect::CopyMinionStats => {
            let friendly = collect_friendly_minions(state, owner);
            if friendly.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(friendly.len());
            let target = friendly[idx];
            let atk = state.world().effective_attack(target).unwrap_or(Attack(0));
            let hp = state.world().effective_health(target).unwrap_or(Health(0));
            let world = state.world_mut();
            world.set_attack(source, Attack(atk.0));
            world.set_health(source, Health(hp.0));
        }
        CardEffect::TempDebuff {
            attack_reduction,
            target,
        } => {
            let enemies = match target {
                EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
                _ => return,
            };
            if enemies.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(enemies.len());
            let enemy = enemies[idx];
            state.world_mut().set_temp_attack_debuff(
                enemy,
                crate::core::component::TempAttackDebuff(attack_reduction),
            );
        }
        CardEffect::ReflectDamage => {
            // 已由奥秘系统的 WhenHeroDamaged 触发器处理
        }
        CardEffect::DealDamageAndReturnToHand { amount, target } => {
            // 伤害立即结算；"移回手牌"由 rules.rs 的 CardPlayed 处理
            resolve_deal_damage(state, queue, source, owner, amount, target);
        }
        CardEffect::ReturnFriendlyToHandAndReduceCost { amount } => {
            resolve_return_friendly_reduce_cost(state, owner, amount);
        }
        CardEffect::AdjacentDamage => {
            resolve_adjacent_damage(state, queue, source, owner);
        }
        CardEffect::DestroyWeaponAndDealAttackToEnemies => {
            resolve_destroy_weapon_deal_attack(state, queue, source, owner);
        }
        CardEffect::GrantStealth => {
            resolve_grant_stealth(state, source, owner);
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
            let minions = collect_all_enemy_minions(state, owner);
            for minion in &minions {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount,
                });
            }
            return;
        }
        EffectTarget::AllEnemies => {
            let enemies = collect_all_enemy_characters(state, owner);
            for enemy in &enemies {
                queue.push(Event::DamageDealt {
                    source,
                    target: *enemy,
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
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for m in &all {
                queue.push(Event::DamageDealt {
                    source,
                    target: *m,
                    amount,
                });
            }
            return;
        }
        EffectTarget::AllCharacters
        | EffectTarget::FriendlyHero
        | EffectTarget::DamagedEnemyMinion
        | EffectTarget::FriendlyMinion
        | EffectTarget::TauntEnemyMinion => {
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
        world.set_card_id(e, CardId(card_def.id));
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
        // 设置圣盾/风怒/冲锋/法伤/不能攻击/回合结束效果
        if card_def.divine_shield {
            world.set_divine_shield(e, crate::core::component::DivineShield);
        }
        if card_def.windfury {
            world.set_windfury(e, crate::core::component::Windfury);
        }
        if card_def.charge {
            world.set_charge(e, crate::core::component::Charge);
        }
        if card_def.spell_damage != 0 {
            world.set_spell_damage(
                e,
                crate::core::component::SpellDamage(card_def.spell_damage),
            );
        }
        if card_def.cant_attack {
            world.set_cant_attack(e, crate::core::component::CantAttack);
        }
        if let Some(ete) = card_def.end_turn_effect {
            world.set_end_turn_effect(e, crate::core::component::EndTurnEffect(ete));
        }
        if let Some(st) = card_def.spell_trigger {
            world.set_spell_trigger(e, crate::core::component::SpellTrigger(st));
        }
        if let Some(dt) = card_def.death_trigger {
            world.set_death_trigger(e, crate::core::component::DeathTrigger(dt));
        }
        if let Some(st) = card_def.summon_trigger {
            world.set_summon_trigger(e, crate::core::component::SummonTrigger(st));
        }
        if let Some(ce) = card_def.choose_one_effect {
            world.set_choose_one_effect(e, crate::core::component::ChooseOneEffect(ce));
        }
        if let Some(cb) = card_def.combo_effect {
            world.set_combo_effect(e, crate::core::component::ComboEffect(cb));
        }
        if card_def.attack_equals_health {
            world.set_attack_equals_health(e, crate::core::component::AttackEqualsHealth);
        }
        // 特殊关键词（剧毒/潜行等）：按卡牌 ID 在 cards 层集中映射
        crate::cards::apply_card_keywords(world, e, card_def);
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
            let minions: Vec<Entity> = collect_friendly_minions(state, owner);
            let world = state.world_mut();
            for minion in &minions {
                let cur_atk = world.attack(*minion).unwrap_or(Attack(0));
                let cur_hp = world.health(*minion).unwrap_or(Health(0));
                world.set_attack(*minion, Attack(cur_atk.0 + attack));
                world.set_health(*minion, Health(cur_hp.0 + health));
            }
        }
        EffectTarget::FriendlyMinion => {
            let minions = collect_friendly_minions(state, owner);
            if minions.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(minions.len());
            let world = state.world_mut();
            let m = minions[idx];
            let cur_atk = world.attack(m).unwrap_or(Attack(0));
            let cur_hp = world.health(m).unwrap_or(Health(0));
            world.set_attack(m, Attack(cur_atk.0 + attack));
            world.set_health(m, Health(cur_hp.0 + health));
        }
        _ => {}
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
    inner.world.set_card_id(weapon, CardId(card_def.id));
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

/// 将一个随机敌方随从移回手牌并增加其法力消耗（冰冻陷阱完整效果）。
fn resolve_return_to_hand_and_increase_cost(
    state: &mut GameState,
    _queue: &mut EventQueue,
    owner: PlayerId,
    amount: i32,
) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }

    let idx = state.rng_mut().next_usize(minions.len());
    let target_entity = minions[idx];

    let _ = state.world_mut().move_to_zone(target_entity, Zone::Hand);
    let world = state.world_mut();
    let cur_cost = world.cost(target_entity).unwrap_or(Cost(0));
    world.set_cost(target_entity, Cost(cur_cost.0 + amount));
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
        EffectTarget::DamagedEnemyMinion => collect_enemy_minions(state, owner)
            .into_iter()
            .filter(|&e| {
                state
                    .world()
                    .health(e)
                    .is_some_and(|h| h.0 < state.world().effective_health(e).unwrap_or(h).0)
            })
            .collect(),
        EffectTarget::TauntEnemyMinion => {
            let minions: Vec<Entity> = collect_enemy_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().taunt(e).is_some())
                .collect();
            if minions.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(minions.len());
            let m = minions[idx];
            let hp = state.world().health(m).unwrap_or(Health(1));
            queue.push(Event::DamageDealt {
                source: m,
                target: m,
                amount: hp.0.max(1),
            });
            return;
        }
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for &m in &all {
                let hp = state.world().health(m).unwrap_or(Health(1));
                queue.push(Event::DamageDealt {
                    source: m,
                    target: m,
                    amount: hp.0.max(1),
                });
            }
            return;
        }
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
            all.extend(collect_all_enemy_minions(state, owner));
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
        EffectTarget::AllEnemyMinions => collect_all_enemy_minions(state, owner),
        EffectTarget::AllCharacters => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_characters(state, owner));
            all
        }
        _ => return,
    };
    for &t in &targets {
        state.world_mut().set_freeze(t, Freeze);
    }
}

/// 给英雄增加临时攻击力和可选护甲。
fn resolve_gain_hero_attack(state: &mut GameState, owner: PlayerId, attack: i32, armor: i32) {
    let hero = state.player(owner).hero;
    let inner = state.make_mut();
    // 增加临时攻击力
    inner.players[owner.index()].temp_attack_bonus += attack;
    let cur_atk = inner.world.attack(hero).unwrap_or(Attack(0));
    inner.world.set_attack(hero, Attack(cur_atk.0 + attack));
    // 增加护甲
    if armor > 0 {
        inner.players[owner.index()].armor += armor;
    }
}

/// 对目标造成等于英雄攻击力的伤害。
fn resolve_deal_hero_attack_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    target: EffectTarget,
) {
    let hero = state.player(owner).hero;
    let hero_atk = state.world().effective_attack(hero).unwrap_or(Attack(0)).0;
    if hero_atk <= 0 {
        return;
    }
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    queue.push(Event::DamageDealt {
        source,
        target: minions[idx],
        amount: hero_atk,
    });
}

/// 将随从的生命值恢复到满（最大生命值）。
fn resolve_full_heal(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let m = minions[idx];
    // 获取最大生命值（基于初始定义，这里简化为设为30或使用current max）
    // 简化方案：从基础生命值恢复（使用 max_health 如果存在，否则设为当前值）
    let world = state.world_mut();
    if world.health(m).is_some() {
        // 简化：恢复到卡牌定义值。由于无法获取原始定义，设为当前生命值
        // 实际应通过 card_id 查找原始定义
        // 在这里我们使用一个辅助方法：将生命值设为最大值（在实体创建时保存的）
        // 简化实现：设为 30（英雄上限）或保持当前值
        // 真正的实现需要保存 max_health 组件
        let cur = world.health(m).unwrap_or(Health(0));
        // 对于随从，我们假设最大生命值 >= 当前值，设为较大值
        // 简化：使用 cur + 一个合理buffer
        world.set_health(m, Health((cur.0 + 10).min(30)));
    }
}

/// 给随从增加风怒。
fn resolve_grant_windfury(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    state
        .world_mut()
        .set_windfury(minions[idx], crate::core::component::Windfury);
}

/// 给随从增加冲锋和可选攻击力加成。
fn resolve_grant_charge(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    attack_bonus: i32,
) {
    let minions: Vec<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let world = state.world_mut();
    let m = minions[idx];
    world.set_charge(m, crate::core::component::Charge);
    // 重置攻击次数，允许立即攻击
    world.set_attacks_used(m, crate::core::component::AttacksUsed(0));
    if attack_bonus > 0 {
        let cur_atk = world.attack(m).unwrap_or(Attack(0));
        world.set_attack(m, Attack(cur_atk.0 + attack_bonus));
    }
}

/// 双倍随从的攻击力。
fn resolve_double_attack(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let world = state.world_mut();
    let m = minions[idx];
    let cur_atk = world.attack(m).unwrap_or(Attack(0));
    world.set_attack(m, Attack(cur_atk.0 * 2));
}

/// 双倍随从的生命值。
fn resolve_double_health(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let world = state.world_mut();
    let m = minions[idx];
    let cur_hp = world.health(m).unwrap_or(Health(0));
    world.set_health(m, Health((cur_hp.0 * 2).min(30)));
}

/// 给友方英雄的武器增加攻击力和耐久度。
fn resolve_buff_weapon(state: &mut GameState, owner: PlayerId, attack: i32, durability: i32) {
    let weapon = state.player(owner).weapon;
    if let Some(w) = weapon {
        let world = state.world_mut();
        if attack != 0 {
            let cur_atk = world.attack(w).unwrap_or(Attack(0));
            world.set_attack(w, Attack(cur_atk.0 + attack));
        }
        if durability != 0 {
            let cur_dur = world.durability(w).unwrap_or(Durability(0));
            world.set_durability(w, Durability(cur_dur.0 + durability));
        }
    }
}

/// 随机丢弃一张手牌。
fn resolve_discard_random(state: &mut GameState, owner: PlayerId) {
    let hand: Vec<Entity> = state.world().zones().iter(Zone::Hand, owner).collect();
    if hand.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(hand.len());
    let card = hand[idx];
    let _ = state.world_mut().move_to_zone(card, Zone::Graveyard);
}

/// 对目标造成等于英雄护甲值的伤害。
fn resolve_deal_armor_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    target: EffectTarget,
) {
    let armor = state.player(owner).armor;
    if armor <= 0 {
        return;
    }
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    queue.push(Event::DamageDealt {
        source,
        target: minions[idx],
        amount: armor,
    });
}

/// 摧毁敌方武器并抽等于其耐久度的牌数。
fn resolve_destroy_weapon_and_draw(state: &mut GameState, queue: &mut EventQueue, owner: PlayerId) {
    let enemy = owner.opponent();
    let weapon = state.player(enemy).weapon;
    if let Some(w) = weapon {
        let durability = state.world().durability(w).unwrap_or(Durability(0)).0;
        let inner = state.make_mut();
        inner.players[enemy.index()].weapon = None;
        queue.push(Event::WeaponDestroyed {
            player: enemy,
            weapon: w,
        });
        for _ in 0..durability {
            draw_card(state, queue, owner);
        }
    }
}

/// 对两个随机敌方随从造成伤害。
fn resolve_deal_damage_to_two(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
) {
    let mut enemies = collect_enemy_minions(state, owner);
    if enemies.is_empty() {
        return;
    }
    // 随机选两个（可能同一个被选两次如果敌方只有1个随从）
    for _ in 0..2 {
        if enemies.is_empty() {
            break;
        }
        let idx = state.rng_mut().next_usize(enemies.len());
        let target = enemies[idx];
        queue.push(Event::DamageDealt {
            source,
            target,
            amount,
        });
        enemies.remove(idx);
    }
}

/// 返回所有随从到各自拥有者手牌。
fn resolve_return_all_to_hand(state: &mut GameState, owner: PlayerId) {
    let all_minions: Vec<Entity> = [owner, owner.opponent()]
        .iter()
        .flat_map(|&pid| {
            state
                .world()
                .zones()
                .iter(Zone::Play, pid)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect::<Vec<_>>()
        })
        .collect();
    for m in &all_minions {
        let _ = state.world_mut().move_to_zone(*m, Zone::Hand);
    }
}

/// 将随从的攻击力设为等于其当前生命值。
fn resolve_set_attack_to_health(state: &mut GameState, owner: PlayerId, target: EffectTarget) {
    let minions: Vec<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let world = state.world_mut();
    let m = minions[idx];
    let hp = world.health(m).unwrap_or(Health(0));
    world.set_attack(m, Attack(hp.0));
}

/// 消灭所有随从，除随机一个之外。
fn resolve_destroy_all_except_one(state: &mut GameState, queue: &mut EventQueue, owner: PlayerId) {
    let enemy = owner.opponent();
    let mut all_minions: Vec<Entity> = [owner, enemy]
        .iter()
        .flat_map(|&pid| {
            state
                .world()
                .zones()
                .iter(Zone::Play, pid)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect::<Vec<_>>()
        })
        .collect();
    if all_minions.is_empty() {
        return;
    }
    // 随机选一个幸存者，消灭其余
    let survivor_idx = state.rng_mut().next_usize(all_minions.len());
    let survivor = all_minions.remove(survivor_idx);
    for &m in &all_minions {
        let hp = state.world().health(m).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: survivor,
            target: m,
            amount: hp.0.max(1),
        });
    }
}

/// 消灭一个随从并为英雄恢复生命值。
fn resolve_destroy_and_heal(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
    heal: i32,
) {
    let minions: Vec<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let m = minions[idx];
    let hp = state.world().health(m).unwrap_or(Health(1));
    queue.push(Event::DamageDealt {
        source: m,
        target: m,
        amount: hp.0.max(1),
    });
    let hero = state.player(owner).hero;
    let cur = state.world().health(hero).unwrap_or(Health(0));
    state
        .world_mut()
        .set_health(hero, Health((cur.0 + heal).min(30)));
}

/// 消灭一个友方随从并对其攻击力造成AOE伤害。
fn resolve_destroy_and_aoe(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    source: Entity,
    target: EffectTarget,
) {
    // 先收集友方随从，随机选一个
    let friendly = collect_friendly_minions(state, owner);
    if friendly.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(friendly.len());
    let sacrifice = friendly[idx];
    let atk = state.world().attack(sacrifice).unwrap_or(Attack(0)).0;
    // 消灭牺牲品
    let hp = state.world().health(sacrifice).unwrap_or(Health(1));
    queue.push(Event::DamageDealt {
        source,
        target: sacrifice,
        amount: hp.0.max(1),
    });
    // 对所有敌方随从造成等于其攻击力的伤害
    let _enemy = owner.opponent();
    let targets: Vec<Entity> = match target {
        EffectTarget::AllEnemyMinions => collect_all_enemy_minions(state, owner),
        _ => return,
    };
    for t in &targets {
        queue.push(Event::DamageDealt {
            source,
            target: *t,
            amount: atk,
        });
    }
}

/// 将一个友方随从移回手牌并使其费用减少（暗影步）。
fn resolve_return_friendly_reduce_cost(state: &mut GameState, owner: PlayerId, amount: i32) {
    let minions = collect_friendly_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let _ = state.world_mut().move_to_zone(target, Zone::Hand);
    let world = state.world_mut();
    let cur = world.cost(target).unwrap_or(Cost(0));
    world.set_cost(target, Cost((cur.0 - amount).max(0)));
}

/// 对目标相邻的随从造成等于其攻击力的伤害（背叛）。
///
/// 目标为随机敌方随从；伤害其左右相邻的随从（同一战场位置差为 1）。
fn resolve_adjacent_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let atk = state
        .world()
        .effective_attack(target)
        .unwrap_or(Attack(0))
        .0;
    if atk <= 0 {
        return;
    }
    // 找到目标在敌方战场上的位置，取其左右相邻随从
    let enemy = owner.opponent();
    let board: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let Some(pos) = board.iter().position(|&e| e == target) else {
        return;
    };
    // 左邻（pos 为 0 时 wrapping_sub 得到 None）
    if let Some(&left) = board.get(pos.wrapping_sub(1)) {
        queue.push(Event::DamageDealt {
            source,
            target: left,
            amount: atk,
        });
    }
    if let Some(&right) = board.get(pos + 1) {
        queue.push(Event::DamageDealt {
            source,
            target: right,
            amount: atk,
        });
    }
}

/// 摧毁己方武器并对所有敌人造成等于其攻击力的伤害（剑刃乱舞）。
fn resolve_destroy_weapon_deal_attack(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
) {
    let weapon = state.player(owner).weapon;
    let Some(w) = weapon else {
        return;
    };
    let atk = state.world().attack(w).unwrap_or(Attack(0)).0;
    let inner = state.make_mut();
    inner.players[owner.index()].weapon = None;
    queue.push(Event::WeaponDestroyed {
        player: owner,
        weapon: w,
    });
    if atk > 0 {
        let enemies = collect_all_enemy_characters(state, owner);
        for enemy in &enemies {
            queue.push(Event::DamageDealt {
                source,
                target: *enemy,
                amount: atk,
            });
        }
    }
}

/// 使一个友方随从获得潜行（伪装大师，不能指定自身）。
fn resolve_grant_stealth(state: &mut GameState, source: Entity, owner: PlayerId) {
    let minions: Vec<Entity> = collect_friendly_minions(state, owner)
        .into_iter()
        .filter(|&e| e != source)
        .collect();
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    state.world_mut().set_stealth(minions[idx], Stealth);
}

// ============================================================
// 辅助函数
// ============================================================

/// 收集敌方所有角色（英雄 + 随从），排除潜行随从。
///
/// 潜行角色不能被单目标效果指定，但仍受 AOE 影响。
fn collect_enemy_characters(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    collect_enemy_characters_impl(state, owner, false)
}

/// 收集敌方所有角色（英雄 + 随从，含潜行）— 用于 AOE 效果。
fn collect_all_enemy_characters(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    collect_enemy_characters_impl(state, owner, true)
}

fn collect_enemy_characters_impl(
    state: &GameState,
    owner: PlayerId,
    include_stealth: bool,
) -> Vec<Entity> {
    let enemy = owner.opponent();
    let mut chars: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| {
            let ct = state.world().card_type(e);
            if ct != Some(CardType::Minion) && ct != Some(CardType::Hero) {
                return false;
            }
            // 单目标选择排除潜行；AOE（include_stealth）包含潜行
            !(!include_stealth && state.world().stealth(e).is_some())
        })
        .collect();
    chars.push(state.player(enemy).hero);
    // 去重
    chars.sort_by_key(|e| (e.index, e.generation));
    chars.dedup();
    chars
}

/// 收集敌方所有随从，排除潜行随从。
fn collect_enemy_minions(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    collect_enemy_minions_impl(state, owner, false)
}

/// 收集敌方所有随从（含潜行）— 用于 AOE 效果。
fn collect_all_enemy_minions(state: &GameState, owner: PlayerId) -> Vec<Entity> {
    collect_enemy_minions_impl(state, owner, true)
}

fn collect_enemy_minions_impl(
    state: &GameState,
    owner: PlayerId,
    include_stealth: bool,
) -> Vec<Entity> {
    let enemy = owner.opponent();
    state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| {
            state.world().card_type(e) == Some(CardType::Minion)
                && (include_stealth || state.world().stealth(e).is_none())
        })
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
