//! 规则引擎 — 验证、事件入队、事件应用。
//!
//! 三个核心函数：
//! - `validate()` — 只读检查 action 在当前状态下的合法性
//! - `enqueue()` — 将 action 转化为初始事件并入队
//! - `apply_event()` — 执行单个事件，可能产生新事件并入队
//!
//! 所有函数都是纯函数式风格，通过 `GameState` 参数与状态交互。

use crate::core::action::Action;
use crate::core::component::{
    Attack, AttacksUsed, CardType, Cost, Durability, Health, HeroPowerUsed, Secret,
};
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue, Priority};
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Phase};
use crate::core::zone::Zone;
use crate::engine::trigger;

/// 引擎错误 — action 不能被执行的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineError {
    /// 不是你的回合
    NotYourTurn,
    /// 试图打出对手的牌
    NotYourCard,
    /// 卡牌不在手牌中
    CardNotInHand,
    /// 不是随从或武器（PlayCard 只能打出随从或武器）
    NotPlayable,
    /// 战场已满（7 个随从上限）
    BoardFull,
    /// 无效的目标（攻击己方或目标不存在）
    InvalidTarget,
    /// 攻击者不在战场上
    NotOnBoard,
    /// 本回合攻击次数已耗尽
    AttacksExhausted,
    /// 实体已销毁（过期 handle）
    EntityGone(Entity),
    /// 游戏已经结束
    GameAlreadyOver,
    /// 法力不足
    NotEnoughMana,
    /// 必须先攻击嘲讽随从
    MustAttackTaunt,
    /// 英雄技能本回合已使用过
    HeroPowerAlreadyUsed,
    /// 功能尚未实现（Phase 2+）
    Unimplemented,
}

/// 战场随从数量上限。
pub const MAX_BOARD_SIZE: usize = 7;

/// 验证 action 在当前状态下的合法性（只读）。
///
/// 返回 `Ok(())` 或 `Err(EngineError)`。
pub fn validate(state: &GameState, action: Action) -> Result<(), EngineError> {
    // 游戏结束后拒绝所有操作
    if matches!(state.phase(), Phase::GameOver { .. }) {
        return Err(EngineError::GameAlreadyOver);
    }

    match action {
        Action::PlayCard { card, target: _ } => validate_play_card(state, card),
        Action::Attack { attacker, defender } => validate_attack(state, attacker, defender),
        Action::EndTurn => validate_end_turn(state),
        Action::HeroPower { hero } => validate_hero_power(state, hero),
    }
}

/// 验证出牌操作。
fn validate_play_card(state: &GameState, card: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // 检查实体存活和组件存在
    check_entity(world, card)?;

    let card_player = world.player(card).ok_or(EngineError::EntityGone(card))?;
    if card_player != active {
        return Err(EngineError::NotYourCard);
    }

    // 必须是随从、武器或法术
    let card_type = world.card_type(card).ok_or(EngineError::NotPlayable)?;
    if card_type != CardType::Minion
        && card_type != CardType::Weapon
        && card_type != CardType::Spell
    {
        return Err(EngineError::NotPlayable);
    }

    // 必须在手牌中
    let zone = world.zone(card).ok_or(EngineError::CardNotInHand)?;
    if zone != Zone::Hand {
        return Err(EngineError::CardNotInHand);
    }

    // 检查法力（使用有效费用：光环减免 + 肯瑞托法师的一次性免费奥秘）
    let cost = effective_play_cost(state, card, active);
    if cost.0 > state.player(active).current_mana {
        return Err(EngineError::NotEnoughMana);
    }

    // 检查战场上限（武器不占随从位）
    if card_type == CardType::Minion {
        let board_count = count_board_minions(world, active);
        if board_count >= MAX_BOARD_SIZE {
            return Err(EngineError::BoardFull);
        }
    }

    Ok(())
}

/// 计算打出卡牌的实际费用：基础费用 - 光环减免；
/// 奥秘卡牌额外享受肯瑞托法师的一次性免费。
fn effective_play_cost(state: &GameState, card: Entity, player: PlayerId) -> Cost {
    let mut cost = state.world().effective_cost(card).unwrap_or_default();
    let is_secret = state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.secret.is_some());
    if is_secret && state.player(player).next_secret_free {
        cost = Cost(0);
    }
    cost
}

/// 验证攻击操作。
fn validate_attack(
    state: &GameState,
    attacker: Entity,
    defender: Entity,
) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // 检查 phase
    if state.phase() != Phase::Main {
        return Err(EngineError::InvalidTarget);
    }

    // 检查实体存活
    check_entity(world, attacker)?;
    check_entity(world, defender)?;

    // 攻击者必须是己方角色（随从或英雄），在战场上
    let attacker_player = world
        .player(attacker)
        .ok_or(EngineError::EntityGone(attacker))?;
    if attacker_player != active {
        return Err(EngineError::InvalidTarget);
    }
    let attacker_type = world.card_type(attacker).ok_or(EngineError::NotOnBoard)?;
    match attacker_type {
        CardType::Minion => {}
        CardType::Hero => {
            // 英雄攻击必须有武器或临时攻击力加成
            let has_weapon = state.player(attacker_player).weapon.is_some();
            let has_temp_attack = state.player(attacker_player).temp_attack_bonus > 0;
            if !has_weapon && !has_temp_attack {
                return Err(EngineError::InvalidTarget);
            }
        }
        _ => return Err(EngineError::NotOnBoard),
    }
    let attacker_zone = world.zone(attacker).ok_or(EngineError::NotOnBoard)?;
    if attacker_zone != Zone::Play {
        return Err(EngineError::NotOnBoard);
    }

    // 检查攻击次数（考虑风怒）
    let max_atks = world.max_attacks(attacker);
    if world
        .attacks_used(attacker)
        .is_some_and(|a| a.is_exhausted_with(max_atks))
    {
        return Err(EngineError::AttacksExhausted);
    }

    // 不能攻击的随从（如拉格纳罗斯）
    if world.cant_attack(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // 攻击力必须 > 0（考虑武器和光环）
    let total_atk = compute_attacker_damage(state, attacker);
    if total_atk <= 0 {
        return Err(EngineError::InvalidTarget);
    }

    // 防御者必须是敌方目标（己方英雄或己方随从不行）
    let defender_player = world.player(defender).ok_or(EngineError::InvalidTarget)?;
    if defender_player == active {
        return Err(EngineError::InvalidTarget);
    }

    // 防御者必须是在战场上的随从或英雄
    let defender_type = world
        .card_type(defender)
        .ok_or(EngineError::InvalidTarget)?;
    let defender_zone = world.zone(defender).ok_or(EngineError::InvalidTarget)?;
    if defender_zone != Zone::Play {
        return Err(EngineError::InvalidTarget);
    }
    match defender_type {
        CardType::Minion | CardType::Hero => {}
        _ => return Err(EngineError::InvalidTarget),
    }

    // 嘲讽检查：如果敌方场上有嘲讽随从，必须攻击嘲讽
    let enemy = active.opponent();
    let has_taunt = world
        .zones()
        .iter(Zone::Play, enemy)
        .any(|e| world.taunt(e).is_some());
    if has_taunt {
        // defender 必须是嘲讽随从
        if world.taunt(defender).is_none() {
            return Err(EngineError::MustAttackTaunt);
        }
    }

    // 潜行检查：不能攻击敌方潜行角色
    if world.stealth(defender).is_some() && defender_player != active {
        return Err(EngineError::InvalidTarget);
    }

    Ok(())
}

/// 验证结束回合操作。
fn validate_end_turn(state: &GameState) -> Result<(), EngineError> {
    if state.phase() != Phase::Main {
        return Err(EngineError::NotYourTurn);
    }
    Ok(())
}

/// 验证英雄技能使用。
fn validate_hero_power(state: &GameState, hero: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // 检查实体存活
    check_entity(world, hero)?;

    // 必须是英雄
    if world.card_type(hero) != Some(CardType::Hero) {
        return Err(EngineError::InvalidTarget);
    }

    // 必须是己方英雄
    let hero_player = world.player(hero).ok_or(EngineError::EntityGone(hero))?;
    if hero_player != active {
        return Err(EngineError::NotYourTurn);
    }

    // 阶段检查
    if state.phase() != Phase::Main {
        return Err(EngineError::NotYourTurn);
    }

    // 检查本回合是否已使用
    if world.hero_power_used(hero).is_some_and(|u| u.0) {
        return Err(EngineError::HeroPowerAlreadyUsed);
    }

    // 检查法力
    let hero_power = world.hero_power(hero);
    let cost = hero_power.map(|hp| hp.cost).unwrap_or(2);
    if cost > state.player(active).current_mana {
        return Err(EngineError::NotEnoughMana);
    }

    Ok(())
}

/// 计算攻击者的总伤害（基础攻击 + 光环 + 武器加成）。
fn compute_attacker_damage(state: &GameState, attacker: Entity) -> i32 {
    let world = state.world();
    let base = world.effective_attack(attacker).unwrap_or(Attack(0));
    let weapon_bonus = if world.card_type(attacker) == Some(CardType::Hero) {
        world
            .player(attacker)
            .and_then(|pid| state.player(pid).weapon)
            .and_then(|w| world.attack(w))
            .unwrap_or(Attack(0))
            .0
    } else {
        0
    };
    base.0 + weapon_bonus
}

/// 检查实体是否存活，不稳定则返回 `EntityGone`。
fn check_entity(world: &crate::core::world::World, entity: Entity) -> Result<(), EngineError> {
    if world.is_alive(entity) {
        Ok(())
    } else {
        Err(EngineError::EntityGone(entity))
    }
}

/// 统计某个玩家战场上的随从数量。
fn count_board_minions(world: &crate::core::world::World, player: PlayerId) -> usize {
    world
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| world.card_type(e) == Some(CardType::Minion))
        .count()
}

// ============================================================
// 事件入队
// ============================================================

/// 根据 action 生成初始事件并入队（只读）。
pub fn enqueue(
    state: &GameState,
    action: Action,
    queue: &mut EventQueue,
) -> Result<(), EngineError> {
    match action {
        Action::PlayCard { card, target } => {
            let player = state.active_player();
            queue.push(Event::CardPlayed {
                player,
                card,
                target,
            });
            let card_type = state.world().card_type(card);
            if card_type == Some(CardType::Minion) {
                queue.push(Event::MinionSummoned {
                    player,
                    minion: card,
                });
            }
        }
        Action::Attack { attacker, defender } => {
            let world = state.world();
            queue.push(Event::AttackDeclared { attacker, defender });
            // 攻击结算入队为单一管线事件：伤害数值在 `ResolveAttack`
            // 处理时计算，但攻击方伤害必须在入队时确定 — 武器在
            // `AttackDeclared` 中可能被摧毁，攻击伤害必须包含武器加成。
            let attacker_total_atk = compute_attacker_damage(state, attacker);
            // 反击免疫（角斗士的长弓：英雄攻击时免疫，不反击）
            let retaliation_immune = world.card_type(attacker) == Some(CardType::Hero)
                && world.player(attacker).is_some_and(|pid| {
                    state.player(pid).weapon.is_some_and(|w| {
                        world.card_id(w).is_some_and(|c| {
                            c.0 == crate::cards::classic_hunter::GLADIATORS_LONGBOW_ID
                        })
                    })
                });
            queue.push(Event::ResolveAttack {
                attacker,
                defender,
                attacker_damage: attacker_total_atk,
                retaliation_immune,
            });
        }
        Action::EndTurn => {
            let active = state.active_player();
            let opponent = active.opponent();
            queue.push(Event::TurnEnded { player: active });
            queue.push(Event::TurnStarted { player: opponent });
        }
        Action::HeroPower { hero } => {
            let active = state.active_player();
            queue.push(Event::HeroPowerActivated {
                player: active,
                hero,
            });
        }
    }
    Ok(())
}

// ============================================================
// 事件应用（修改状态）
// ============================================================

/// 死亡检查 — 伤害管线（`DamageDealt`）的最后一步。
///
/// 目标的有效生命值 ≤ 0 时：英雄入队游戏结束（最高优先级），
/// 随从入队 `MinionDied`。
fn queue_death_events(
    state: &GameState,
    queue: &mut EventQueue,
    target: Entity,
    card_type: Option<CardType>,
) {
    let effective_hp = state.world().effective_health(target);
    if !effective_hp.is_some_and(|h| h.is_dead()) {
        return;
    }
    match card_type {
        Some(CardType::Hero) => {
            let loser = state.world().player(target);
            if let Some(player) = loser {
                let winner = player.opponent();
                queue.push_with_priority(Event::GameOver { winner }, Priority::Highest);
            }
        }
        Some(CardType::Minion) => {
            queue.push(Event::MinionDied { minion: target });
        }
        _ => {}
    }
}

/// 应用单个事件，可能产生新的事件并入队。
///
/// 这是事件循环的核心。每个事件只被处理一次；
/// 如果事件产生了新事件，它们会被加入队列并依次处理。
pub fn apply_event(
    state: &mut GameState,
    event: Event,
    queue: &mut EventQueue,
) -> Result<(), EngineError> {
    match event {
        Event::TurnStarted { player } => {
            // 腐蚀术：在你的回合开始时消灭被腐蚀的随从
            let corrupted: Vec<Entity> = state.make_mut().players[player.index()]
                .corrupted
                .drain(..)
                .collect();
            for entity in corrupted {
                if !state.world().is_alive(entity)
                    || state.world().card_type(entity) != Some(CardType::Minion)
                {
                    continue;
                }
                let hp = state.world().health(entity).unwrap_or(Health(1));
                queue.push(Event::DamageDealt {
                    source: entity,
                    target: entity,
                    amount: hp.0.max(1),
                });
            }

            // 先收集需要重置攻击次数的实体（需要只读 borrow）
            let player_entities: Vec<Entity> = state
                .world()
                .iter_player()
                .filter(|(_, pid)| **pid == player)
                .map(|(e, _)| e)
                .collect();
            let new_turn = state.turn() + 1;

            // 然后分步进行所有修改
            {
                let world = state.world_mut();
                for entity in &player_entities {
                    world.set_attacks_used(*entity, AttacksUsed(0));
                    // 重置英雄技能使用标记
                    world.set_hero_power_used(*entity, HeroPowerUsed(false));
                    // 清除冻结
                    world.remove_freeze(*entity);
                }
            }
            state.set_active_player(player);
            state.set_turn(new_turn);
            state.set_phase(Phase::Main);

            // 法力水晶增长和回满
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.mana_crystals = (p.mana_crystals + 1).min(10);
                p.current_mana = p.mana_crystals;
                p.cards_played_this_turn = 0;
            }

            // 抽一张牌
            trigger::draw_card(state, queue, player);
        }
        Event::TurnEnded { player } => {
            state.set_phase(Phase::End);
            // 清除临时攻击力加成
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                let hero = p.hero;
                let bonus = p.temp_attack_bonus;
                if bonus > 0 {
                    let cur_atk = inner.world.attack(hero).unwrap_or(Attack(0));
                    inner
                        .world
                        .set_attack(hero, Attack((cur_atk.0 - bonus).max(0)));
                    p.temp_attack_bonus = 0;
                }
            }
            // 清除临时攻击减益和死亡记录
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.died_this_turn.clear();
                // 清除所有实体的临时攻击减益
                let debuff_entities: Vec<Entity> = inner
                    .world
                    .iter_temp_attack_debuff()
                    .map(|(e, _)| e)
                    .collect();
                for e in debuff_entities {
                    inner.world.remove_temp_attack_debuff(e);
                }
                // 清除所有实体的临时免疫（狂野怒火 — 直到回合结束）
                let immune_entities: Vec<Entity> =
                    inner.world.iter_immune().map(|(e, _)| e).collect();
                for e in immune_entities {
                    inner.world.remove_immune(e);
                }
            }
            // 归还被临时控制的随从（暗影狂乱 — 直到回合结束）
            let controlled: Vec<(Entity, PlayerId)> = state.make_mut().players[player.index()]
                .controlled_this_turn
                .drain(..)
                .collect();
            for (entity, original_owner) in controlled {
                if state.world().is_alive(entity) {
                    trigger::transfer_minion(state, entity, original_owner);
                }
            }
            // 清除命令怒吼的最低生命值效果（直到回合结束）
            {
                let inner = state.make_mut();
                inner.players[player.index()].minion_min_health = 0;
            }
            // 触发回合结束效果（先收集再逐个处理）
            let end_turn_effects: Vec<(Entity, crate::core::effect::CardEffect)> = state
                .world()
                .iter_end_turn_effect()
                .filter(|(e, _)| {
                    state.world().zone(*e) == Some(Zone::Play)
                        && state.world().is_alive(*e)
                        && state.world().player(*e) == Some(player)
                })
                .map(|(e, ete)| (e, ete.0))
                .collect();
            for (source, effect) in end_turn_effects {
                trigger::resolve_effect(state, queue, source, player, effect, None);
            }
        }
        Event::CardPlayed {
            player,
            card,
            target,
        } => {
            // 扣除法力（使用有效费用：光环减免 + 肯瑞托法师的一次性免费奥秘）
            let cost = effective_play_cost(state, card, player);
            let card_type = state.world().card_type(card);
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.current_mana -= cost.0;
                p.cards_played_this_turn += 1;
            }
            // 检测连击：本回合已打出其他牌 (cards_played > 1 因为刚递增了)
            let combo_active = state.player(player).cards_played_this_turn > 1;
            if card_type == Some(CardType::Spell) {
                // 奥秘卡牌：挂载奥秘组件并放入 SetAside 区域（触发条件满足时揭示）。
                // 奥秘效果存储在 battlecry 槽位（与法术牌惯例一致）。
                let secret_trigger = state
                    .world()
                    .card_id(card)
                    .and_then(|cid| crate::cards::def::card_by_id(cid.0))
                    .and_then(|def| def.secret);
                if let Some(trigger) = secret_trigger {
                    // 肯瑞托法师的一次性免费奥秘被消耗
                    let secret_effect = state.world().battlecry(card).map(|b| b.0);
                    let inner = state.make_mut();
                    inner.players[player.index()].next_secret_free = false;
                    if let Some(effect) = secret_effect {
                        inner.world.set_secret(card, Secret { trigger, effect });
                    }
                    state
                        .world_mut()
                        .move_to_zone(card, Zone::SetAside)
                        .map_err(|_| EngineError::EntityGone(card))?;
                    queue.push(Event::SpellCast {
                        player,
                        spell: card,
                    });
                } else {
                    // 法术牌：解析效果（支持抉择随机选择和连击），然后移入坟墓场
                    let chosen_effect = if combo_active {
                        // 连击：优先使用 combo_effect
                        state
                            .world()
                            .combo_effect(card)
                            .map(|c| c.0)
                            .or_else(|| state.world().battlecry(card).map(|b| b.0))
                    } else if state.world().choose_one_effect(card).is_some() {
                        // 抉择：随机选一个
                        let has_main = state.world().battlecry(card).is_some();
                        let has_alt = state.world().choose_one_effect(card).is_some();
                        if has_main && has_alt {
                            if state.rng_mut().next_usize(2) == 0 {
                                state.world().battlecry(card).map(|b| b.0)
                            } else {
                                state.world().choose_one_effect(card).map(|c| c.0)
                            }
                        } else {
                            state
                                .world()
                                .battlecry(card)
                                .map(|b| b.0)
                                .or_else(|| state.world().choose_one_effect(card).map(|c| c.0))
                        }
                    } else {
                        state.world().battlecry(card).map(|b| b.0)
                    };
                    // 连击回手（头部爆裂）：效果解析后留在手牌而非进入坟墓场
                    let returns_to_hand = matches!(
                        chosen_effect,
                        Some(crate::core::effect::CardEffect::DealDamageAndReturnToHand { .. })
                    );
                    if let Some(effect) = chosen_effect {
                        trigger::resolve_effect(state, queue, card, player, effect, target);
                    }
                    // 触发法术施放事件
                    queue.push(Event::SpellCast {
                        player,
                        spell: card,
                    });
                    if !returns_to_hand {
                        state
                            .world_mut()
                            .move_to_zone(card, Zone::Graveyard)
                            .map_err(|_| EngineError::EntityGone(card))?;
                    }
                }
            } else if card_type == Some(CardType::Weapon) {
                // 武器牌：先销毁旧武器，然后装备新武器
                let old_weapon = state.player(player).weapon;
                if let Some(old) = old_weapon {
                    let inner = state.make_mut();
                    inner.players[player.index()].weapon = None;
                    queue.push(Event::WeaponDestroyed {
                        player,
                        weapon: old,
                    });
                }
                // 装备新武器：卡牌移到战场作为武器实体
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
                let inner = state.make_mut();
                inner.players[player.index()].weapon = Some(card);
                queue.push(Event::WeaponEquipped {
                    player,
                    weapon: card,
                });
                // 武器战吼（连击感知）：装备后解析，如毁灭之刃
                let weapon_effect = if combo_active {
                    state
                        .world()
                        .combo_effect(card)
                        .map(|c| c.0)
                        .or_else(|| state.world().battlecry(card).map(|b| b.0))
                } else {
                    state.world().battlecry(card).map(|b| b.0)
                };
                if let Some(effect) = weapon_effect {
                    trigger::resolve_effect(state, queue, card, player, effect, None);
                }
            } else {
                // 卡牌从手牌移到战场
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
            }
            // 过载触发器：打出带过载的牌时，触发友方随从的过载效果（无羁元素）
            if state.world().overload(card).is_some() {
                let overload_triggers: Vec<(Entity, crate::core::effect::CardEffect)> = state
                    .world()
                    .iter_overload_trigger()
                    .filter(|(e, _)| {
                        state.world().zone(*e) == Some(Zone::Play)
                            && state.world().is_alive(*e)
                            && state.world().player(*e) == Some(player)
                    })
                    .map(|(e, ot)| (e, ot.0))
                    .collect();
                for (source, effect) in overload_triggers {
                    trigger::resolve_effect(state, queue, source, player, effect, None);
                }
            }
        }
        Event::MinionSummoned { player, minion } => {
            // 召唤失调：非冲锋随从本回合不能攻击
            if state.world().charge(minion).is_none() {
                state.world_mut().set_attacks_used(minion, AttacksUsed(1));
            }
            // 检查战吼组件（支持连击）
            let combo_active = state.player(player).cards_played_this_turn > 1;
            let chosen_effect = if combo_active {
                state
                    .world()
                    .combo_effect(minion)
                    .map(|c| c.0)
                    .or_else(|| state.world().battlecry(minion).map(|b| b.0))
            } else {
                state.world().battlecry(minion).map(|b| b.0)
            };
            if let Some(effect) = chosen_effect {
                trigger::resolve_effect(state, queue, minion, player, effect, None);
            }
            // 检查召唤触发（友方随从被召唤时，场上其他随从的 summon_trigger）
            let summon_triggers: Vec<(Entity, crate::core::effect::CardEffect)> = state
                .world()
                .iter_summon_trigger()
                .filter(|(e, _)| {
                    *e != minion  // 不包括自身
                        && state.world().zone(*e) == Some(Zone::Play)
                        && state.world().is_alive(*e)
                        && state.world().player(*e) == Some(player)
                })
                .map(|(e, st)| (e, st.0))
                .collect();
            for (source, effect) in summon_triggers {
                trigger::resolve_effect(state, queue, source, player, effect, None);
            }
        }
        Event::AttackDeclared { attacker, .. } => {
            // 冻结检查：被冻结的角色不能攻击
            if state.world().freeze(attacker).is_some() {
                return Err(EngineError::InvalidTarget);
            }

            // 先读取攻击者类型和武器信息（只读 borrow）
            let is_hero = state.world().card_type(attacker) == Some(CardType::Hero);
            let attacker_player = state.world().player(attacker);
            let weapon_info: Option<(PlayerId, Entity)> = if is_hero {
                attacker_player.and_then(|pid| state.player(pid).weapon.map(|w| (pid, w)))
            } else {
                None
            };

            // 标记攻击者已使用过攻击次数
            {
                let world = state.world_mut();
                let used = world.attacks_used(attacker).unwrap_or(AttacksUsed(0));
                world.set_attacks_used(attacker, AttacksUsed(used.0 + 1));
            }

            // 武器耐久减 1
            if let Some((player, weapon)) = weapon_info {
                let dur = state.world().durability(weapon).unwrap_or(Durability(0));
                let new_dur = Durability(dur.0 - 1);
                state.world_mut().set_durability(weapon, new_dur);
                if new_dur.0 <= 0 {
                    // 武器摧毁
                    let inner = state.make_mut();
                    inner.players[player.index()].weapon = None;
                    queue.push(Event::WeaponDestroyed { player, weapon });
                }
            }
        }
        Event::ResolveAttack {
            attacker,
            defender,
            attacker_damage,
            retaliation_immune,
        } => {
            // 攻击伤害入队（数值在入队时已确定）
            queue.push(Event::DamageDealt {
                source: attacker,
                target: defender,
                amount: attacker_damage,
            });
            // 防御方反击：按结算时的当前状态计算攻击力 — 攻击被奥秘
            // 重定向后，新防御者的反击自动生效，无需逐个卡牌特殊处理。
            if !retaliation_immune && state.world().card_type(defender) == Some(CardType::Minion) {
                let atk = state
                    .world()
                    .effective_attack(defender)
                    .unwrap_or(Attack(0));
                if atk.0 > 0 {
                    queue.push(Event::DamageDealt {
                        source: defender,
                        target: attacker,
                        amount: atk.0,
                    });
                }
            }
        }
        Event::DamageDealt {
            target,
            amount,
            source,
        } => {
            // 统一伤害管线：免疫 → 圣盾 → 护甲 → 生命值 → 死亡检查
            // 免疫：伤害被完全忽略（攻击仍被消耗）
            if state.world().immune(target).is_some() {
                return Ok(());
            }
            // 圣盾吸收：如果目标有圣盾，移除圣盾，伤害归零
            if state.world().divine_shield(target).is_some() {
                state.world_mut().remove_divine_shield(target);
                return Ok(());
            }

            // 获取目标的卡牌类型
            let card_type = state.world().card_type(target);

            // 英雄先扣护甲，剩余伤害穿透到生命值
            if card_type == Some(CardType::Hero) {
                let target_player = state.world().player(target);
                if let Some(pid) = target_player {
                    let armor = state.player(pid).armor;
                    if armor > 0 {
                        let absorbed = amount.min(armor);
                        let remaining = amount - absorbed;
                        let inner = state.make_mut();
                        inner.players[pid.index()].armor -= absorbed;
                        if remaining <= 0 {
                            // 伤害全部被护甲吸收
                            return Ok(());
                        }
                        // 剩余伤害继续穿透到生命值
                        let current_health = state.world().health(target);
                        let Some(hp) = current_health else {
                            return Ok(());
                        };
                        let new_hp = Health(hp.0 - remaining);
                        state.world_mut().set_health(target, new_hp);
                        queue_death_events(state, queue, target, card_type);
                        return Ok(());
                    }
                }
            }

            // 无护甲或非英雄：直接扣生命值
            let current_health = state.world().health(target);
            let Some(hp) = current_health else {
                // 目标不存在（已死或已移除），跳过
                return Ok(());
            };
            let mut new_hp = Health(hp.0 - amount);
            // 剧毒：带剧毒的来源对随从造成伤害时直接将其消灭（圣盾判定已优先返回）
            if state.world().poison(source).is_some()
                && card_type == Some(CardType::Minion)
                && amount > 0
            {
                new_hp = Health(0);
            }
            // 命令怒吼：本回合随从生命值不能低于 1
            if card_type == Some(CardType::Minion)
                && state
                    .world()
                    .player(target)
                    .is_some_and(|pid| state.player(pid).minion_min_health > 0)
            {
                let min_hp = state
                    .world()
                    .player(target)
                    .map(|pid| state.player(pid).minion_min_health)
                    .unwrap_or(0);
                new_hp = Health(new_hp.0.max(min_hp));
            }

            // 通过 CoW 修改状态
            state.world_mut().set_health(target, new_hp);

            // 死亡检查（使用有效生命值考虑光环加成）
            queue_death_events(state, queue, target, card_type);
        }
        Event::MinionDied { minion } => {
            // 先检查亡语效果（实体还在战场上）
            let deathrattle_effect = state.world().deathrattle(minion);
            let owner = state.world().player(minion);

            // 如果是亡语效果，入队（之后处理，保持当前实体状态）
            if let (Some(dr), Some(owner)) = (deathrattle_effect, owner) {
                trigger::resolve_effect(state, queue, minion, owner, dr.0, None);
            }

            // 检查死亡触发（其他友方随从的 death_trigger）
            if let Some(owner) = owner {
                let death_triggers: Vec<(Entity, crate::core::effect::CardEffect)> = state
                    .world()
                    .iter_death_trigger()
                    .filter(|(e, _)| {
                        *e != minion  // 不包括死亡的随从自身
                            && state.world().zone(*e) == Some(Zone::Play)
                            && state.world().is_alive(*e)
                            && state.world().player(*e) == Some(owner)
                    })
                    .map(|(e, dt)| (e, dt.0))
                    .collect();
                for (source, effect) in death_triggers {
                    trigger::resolve_effect(state, queue, source, owner, effect, None);
                }
            }

            // 随从移到坟墓场（保留实体和组件，用于回放和 Phase 2+ 的坟场效果）
            state
                .world_mut()
                .move_to_zone(minion, Zone::Graveyard)
                .map_err(|_| EngineError::EntityGone(minion))?;
            // 记录死亡用于复活效果
            if let Some(owner) = owner {
                let inner = state.make_mut();
                inner.players[owner.index()].died_this_turn.push(minion);
            }
        }
        Event::CardDrawn { .. } => {
            // 通知事件 — 卡牌已在 draw_card 中移到手牌
        }
        Event::HeroPowerActivated { player, hero } => {
            // 扣除法力
            let cost = state
                .world()
                .hero_power(hero)
                .map(|hp| hp.cost)
                .unwrap_or(2);
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.current_mana = (p.current_mana - cost).max(0);
            }
            // 标记已使用
            state
                .world_mut()
                .set_hero_power_used(hero, HeroPowerUsed(true));
            // 解析英雄技能效果
            if let Some(hp_def) = state.world().hero_power(hero) {
                let effect = hp_def.effect;
                trigger::resolve_effect(state, queue, hero, player, effect, None);
            }
        }
        Event::WeaponEquipped { .. } => {
            // 通知事件 — 武器已在 resolve_equip_weapon 中创建
        }
        Event::WeaponDestroyed { .. } => {
            // 通知事件 — 武器已在 AttackDeclared 或 equip 时移除
        }
        Event::SecretRevealed { .. } => {
            // 通知事件 — 奥秘效果通过 trigger 系统解析
        }
        Event::SpellCast { player, spell: _ } => {
            // 法术触发效果 — 检查场上所有友方随从的 spell_trigger 组件
            let spell_triggers: Vec<(Entity, crate::core::effect::CardEffect)> = state
                .world()
                .iter_spell_trigger()
                .filter(|(e, _)| {
                    state.world().zone(*e) == Some(Zone::Play)
                        && state.world().is_alive(*e)
                        && state.world().player(*e) == Some(player)
                })
                .map(|(e, st)| (e, st.0))
                .collect();
            for (source, effect) in spell_triggers {
                trigger::resolve_effect(state, queue, source, player, effect, None);
            }
        }
        Event::GameOver { winner } => {
            state.set_phase(Phase::GameOver { winner });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{Attack, CardType, Health};
    use crate::core::entity::Entity;
    use crate::core::state::GameState;
    use crate::core::zone::Zone;

    /// 辅助函数：创建一个带有完整组件的随从在战场上的状态
    fn add_minion_to_board(state: &mut GameState, player: PlayerId, atk: i32, hp: i32) -> Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(0));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        e
    }

    /// 辅助函数：添加随从到手牌
    fn add_minion_to_hand(state: &mut GameState, player: PlayerId, atk: i32, hp: i32) -> Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(3));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        // 设置足够的法力用于测试
        state.make_mut().players[player.index()].current_mana = 10;
        e
    }

    #[test]
    fn validate_play_card_legal() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert!(result.is_ok(), "valid play should succeed: {result:?}");
    }

    #[test]
    fn validate_play_card_not_your_card() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player2, 2, 3);
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert_eq!(result, Err(EngineError::NotYourCard));
    }

    #[test]
    fn validate_play_card_not_in_hand() {
        let mut state = GameState::new();
        let card = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        // 在战场上，不在手牌
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert_eq!(result, Err(EngineError::CardNotInHand));
    }

    #[test]
    fn validate_play_card_board_full() {
        let mut state = GameState::new();
        // 填满 7 个随从
        for _ in 0..7 {
            add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        }
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert_eq!(result, Err(EngineError::BoardFull));
    }

    #[test]
    fn validate_attack_legal() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert!(result.is_ok(), "valid attack should succeed: {result:?}");
    }

    #[test]
    fn validate_attack_own_minion() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player1, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert_eq!(result, Err(EngineError::InvalidTarget));
    }

    #[test]
    fn validate_attack_hero_legal() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = state.player(PlayerId::Player2).hero;
        let result = validate(&state, Action::Attack { attacker, defender });
        assert!(result.is_ok(), "attacking hero should be valid: {result:?}");
    }

    #[test]
    fn validate_attack_twice() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        state.world_mut().set_attacks_used(attacker, AttacksUsed(1));
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert_eq!(result, Err(EngineError::AttacksExhausted));
    }

    #[test]
    fn validate_stale_entity() {
        let state = GameState::new();
        let entity = Entity::new(999, 0); // 不存在的实体
        let result = validate(
            &state,
            Action::PlayCard {
                card: entity,
                target: None,
            },
        );
        assert_eq!(result, Err(EngineError::EntityGone(entity)));
    }

    #[test]
    fn apply_trade_deals_damage_both_ways() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 5);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 3);

        let mut queue = EventQueue::new();
        enqueue(&state, Action::Attack { attacker, defender }, &mut queue).unwrap();

        // 处理所有事件
        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        // defender 受到 3 伤害: 3 → 0, 死亡
        assert_eq!(
            state.world().health(defender),
            Some(Health(0)),
            "defender should have taken 3 damage"
        );
        assert_eq!(
            state.world().zone(defender),
            Some(Zone::Graveyard),
            "defender should be dead in graveyard"
        );

        // attacker 受到 2 伤害: 5 → 3, 存活
        assert_eq!(
            state.world().health(attacker),
            Some(Health(3)),
            "attacker should have taken 2 damage"
        );
        assert_eq!(
            state.world().attacks_used(attacker),
            Some(AttacksUsed(1)),
            "attacker should have used attack"
        );
    }

    #[test]
    fn damage_hero() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 5, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let mut queue = EventQueue::new();
        enqueue(
            &state,
            Action::Attack {
                attacker,
                defender: hero,
            },
            &mut queue,
        )
        .unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(state.world().health(hero), Some(Health(25)));
        // 攻击者不应对英雄受到伤害（英雄不还手）
        assert_eq!(state.world().health(attacker), Some(Health(3)));
    }

    #[test]
    fn game_over_on_hero_death() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let mut queue = EventQueue::new();
        enqueue(
            &state,
            Action::Attack {
                attacker,
                defender: hero,
            },
            &mut queue,
        )
        .unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(
            state.phase(),
            Phase::GameOver {
                winner: PlayerId::Player1
            }
        );
    }

    #[test]
    fn attacker_dies_still_deals_damage() {
        // 攻击者在交易中死亡，但伤害仍然造成（同时结算）
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 4, 1);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 5, 10);

        let mut queue = EventQueue::new();
        enqueue(&state, Action::Attack { attacker, defender }, &mut queue).unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        // defender 受到 4 伤害: 10 → 6
        assert_eq!(state.world().health(defender), Some(Health(6)));
        // attacker 受到 5 伤害: 1 → -4 → 死亡
        assert_eq!(state.world().health(attacker), Some(Health(-4)));
        assert_eq!(state.world().zone(attacker), Some(Zone::Graveyard));
        assert_eq!(state.world().zone(defender), Some(Zone::Play)); // defender 存活
    }

    #[test]
    fn reset_attacks_on_turn_start() {
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        state.world_mut().set_attacks_used(minion, AttacksUsed(1));

        let mut queue = EventQueue::new();
        queue.push(Event::TurnStarted {
            player: PlayerId::Player1,
        });

        // 应用事件
        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(state.world().attacks_used(minion), Some(AttacksUsed(0)));
    }
}
