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
    Attack, AttacksUsed, CardType, Durability, Health, HeroPowerUsed,
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
    /// 不是随从（PlayCard 只能打出随从）
    NotAMinion,
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
        Action::PlayCard { card } => validate_play_card(state, card),
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

    // 必须是随从
    let card_type = world.card_type(card).ok_or(EngineError::NotAMinion)?;
    if card_type != CardType::Minion {
        return Err(EngineError::NotAMinion);
    }

    // 必须在手牌中
    let zone = world.zone(card).ok_or(EngineError::CardNotInHand)?;
    if zone != Zone::Hand {
        return Err(EngineError::CardNotInHand);
    }

    // 检查法力
    let cost = world.cost(card).unwrap_or_default();
    if cost.0 > state.player(active).current_mana {
        return Err(EngineError::NotEnoughMana);
    }

    // 检查战场上限
    let board_count = count_board_minions(world, active);
    if board_count >= MAX_BOARD_SIZE {
        return Err(EngineError::BoardFull);
    }

    Ok(())
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
            // 英雄攻击必须有武器
            let has_weapon = state.player(attacker_player).weapon.is_some();
            if !has_weapon {
                return Err(EngineError::InvalidTarget);
            }
        }
        _ => return Err(EngineError::NotOnBoard),
    }
    let attacker_zone = world.zone(attacker).ok_or(EngineError::NotOnBoard)?;
    if attacker_zone != Zone::Play {
        return Err(EngineError::NotOnBoard);
    }

    // 检查攻击次数
    if world
        .attacks_used(attacker)
        .is_some_and(|a| a.is_exhausted())
    {
        return Err(EngineError::AttacksExhausted);
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
        Action::PlayCard { card } => {
            let player = state.active_player();
            queue.push(Event::CardPlayed { player, card });
            queue.push(Event::MinionSummoned {
                player,
                minion: card,
            });
        }
        Action::Attack { attacker, defender } => {
            let world = state.world();
            queue.push(Event::AttackDeclared { attacker, defender });
            // 计算攻击者的有效攻击力（含光环加成和武器加成）
            let attacker_total_atk = compute_attacker_damage(state, attacker);
            queue.push(Event::DamageDealt {
                source: attacker,
                target: defender,
                amount: attacker_total_atk,
            });
            // 如果防御者是随从，它也会反击
            if world.card_type(defender) == Some(CardType::Minion) {
                let defender_atk = world
                    .effective_attack(defender)
                    .unwrap_or(Attack(0));
                if defender_atk.0 > 0 {
                    queue.push(Event::DamageDealt {
                        source: defender,
                        target: attacker,
                        amount: defender_atk.0,
                    });
                }
            }
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
            }

            // 抽一张牌
            trigger::draw_card(state, queue, player);
        }
        Event::TurnEnded { .. } => {
            state.set_phase(Phase::End);
        }
        Event::CardPlayed { player, card } => {
            // 扣除法力
            let cost = state.world().cost(card).unwrap_or_default();
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.current_mana -= cost.0;
            }
            // 卡牌从手牌移到战场
            state
                .world_mut()
                .move_to_zone(card, Zone::Play)
                .map_err(|_| EngineError::EntityGone(card))?;
        }
        Event::MinionSummoned { player, minion } => {
            // 检查战吼组件
            if let Some(battlecry) = state.world().battlecry(minion) {
                let effect = battlecry.0;
                trigger::resolve_effect(state, queue, minion, player, effect);
            }
        }
        Event::AttackDeclared { attacker, .. } => {
            // 先读取攻击者类型和武器信息（只读 borrow）
            let is_hero = state.world().card_type(attacker) == Some(CardType::Hero);
            let attacker_player = state.world().player(attacker);
            let weapon_info: Option<(PlayerId, Entity)> = if is_hero {
                attacker_player.and_then(|pid| {
                    state.player(pid).weapon.map(|w| (pid, w))
                })
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
        Event::DamageDealt {
            target,
            amount,
            source: _,
        } => {
            // 获取目标的卡牌类型
            let card_type = state.world().card_type(target);

            // 如果目标是英雄，先扣护甲
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

                        let effective_hp =
                            state.world().effective_health(target).unwrap_or(new_hp);
                        if effective_hp.is_dead() {
                            let winner = pid.opponent();
                            queue.push_with_priority(
                                Event::GameOver { winner },
                                Priority::Highest,
                            );
                        }
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
            let new_hp = Health(hp.0 - amount);

            // 通过 CoW 修改状态
            state.world_mut().set_health(target, new_hp);

            // 检查死亡（使用有效生命值考虑光环加成）
            let effective_hp = state.world().effective_health(target).unwrap_or(new_hp);
            if effective_hp.is_dead() {
                match card_type {
                    Some(CardType::Hero) => {
                        // 英雄死亡 → 游戏结束
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
        }
        Event::MinionDied { minion } => {
            // 先检查亡语效果（实体还在战场上）
            let deathrattle_effect = state.world().deathrattle(minion);
            let owner = state.world().player(minion);

            // 如果是亡语效果，入队（之后处理，保持当前实体状态）
            if let (Some(dr), Some(owner)) = (deathrattle_effect, owner) {
                trigger::resolve_effect(state, queue, minion, owner, dr.0);
            }

            // 随从移到坟墓场（保留实体和组件，用于回放和 Phase 2+ 的坟场效果）
            state
                .world_mut()
                .move_to_zone(minion, Zone::Graveyard)
                .map_err(|_| EngineError::EntityGone(minion))?;
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
                trigger::resolve_effect(state, queue, hero, player, effect);
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
        let result = validate(&state, Action::PlayCard { card });
        assert!(result.is_ok(), "valid play should succeed: {result:?}");
    }

    #[test]
    fn validate_play_card_not_your_card() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player2, 2, 3);
        let result = validate(&state, Action::PlayCard { card });
        assert_eq!(result, Err(EngineError::NotYourCard));
    }

    #[test]
    fn validate_play_card_not_in_hand() {
        let mut state = GameState::new();
        let card = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        // 在战场上，不在手牌
        let result = validate(&state, Action::PlayCard { card });
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
        let result = validate(&state, Action::PlayCard { card });
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
        let result = validate(&state, Action::PlayCard { card: entity });
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
