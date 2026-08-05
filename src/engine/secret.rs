//! 奥秘系统 — Secret 组件的检查与触发逻辑。
//!
//! 奥秘卡牌打出后进入 `Zone::SetAside`（对对手隐藏）。
//! 每次事件处理后，遍历所有 SetAside 中的奥秘，
//! 检查触发条件是否匹配。匹配的奥秘被揭示并执行效果。
//!
//! # 触发顺序
//!
//! 同玩家多个奥秘同时触发时，按打出顺序（SetAside 中的顺序）依次触发。
//! 不同玩家的奥秘，主玩家（当前 active player）的先触发。

use crate::core::component::{CardType, Secret, SecretTrigger};
use crate::core::effect::CardEffect;
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue};
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// 检查所有秘密，触发匹配当前事件的奥秘。
///
/// 在 `apply_event` 处理完每个事件后调用。
/// 返回被揭示的奥秘数量。
pub fn check_secrets(state: &mut GameState, queue: &mut EventQueue, event: &Event) -> usize {
    let active = state.active_player();

    // 收集所有 SetAside 中的奥秘（按打出的顺序，主玩家优先）
    let secrets: Vec<(Entity, PlayerId, Secret)> = {
        let world = state.world();
        let mut active_secrets = Vec::new();
        let mut opponent_secrets = Vec::new();
        // SetAside 是共享区域，需要手动按玩家分类
        for entity in world.zones().iter(Zone::SetAside, active) {
            if let Some(secret) = world.secret(entity) {
                if let Some(owner) = world.player(entity) {
                    if owner == active {
                        active_secrets.push((entity, owner, secret));
                    } else {
                        opponent_secrets.push((entity, owner, secret));
                    }
                }
            }
        }
        // 主玩家先触发
        active_secrets.extend(opponent_secrets);
        active_secrets
    };

    let mut triggered = 0;

    for (entity, player, secret) in &secrets {
        if matches_trigger(secret.trigger, event, state, *player) {
            // 奥秘揭示：从 SetAside 移到 Graveyard
            let _ = state.world_mut().move_to_zone(*entity, Zone::Graveyard);
            queue.push(Event::SecretRevealed {
                player: *player,
                secret: *entity,
            });
            // 解析奥秘效果（部分效果需要触发事件上下文，如狙击/误导）
            resolve_secret_effect(state, queue, event, *entity, *player, secret.effect);
            triggered += 1;
        }
    }

    triggered
}

/// 检查奥秘触发条件是否匹配当前事件。
fn matches_trigger(
    trigger: SecretTrigger,
    event: &Event,
    state: &GameState,
    owner: PlayerId,
) -> bool {
    match trigger {
        SecretTrigger::AfterFriendlyAttacked => {
            // 己方角色被攻击
            matches_after_friendly_attacked(event, state, owner)
        }
        SecretTrigger::AfterEnemyMinionPlayed => {
            matches!(event, Event::MinionSummoned { player, .. } if *player != owner)
        }
        SecretTrigger::AfterEnemyHeroAttacks => {
            // 敌方英雄攻击
            matches_after_enemy_hero_attacks(event, state, owner)
        }
        SecretTrigger::OnFriendlyTurnStart => {
            matches!(event, Event::TurnStarted { player } if *player == owner)
        }
        SecretTrigger::AfterMinionDied => {
            // 任意随从死亡（Phase 5 可限制为敌方随从）
            matches!(event, Event::MinionDied { .. })
        }
        SecretTrigger::WhenEnemySpellCast => {
            // 敌方施放法术（法术卡牌被打出）
            matches!(event, Event::CardPlayed { player, card } if *player != owner && state.world().card_type(*card) == Some(CardType::Spell))
        }
        SecretTrigger::WhenEnemyMinionAttacksHero => {
            matches_enemy_minion_attacks_hero(event, state, owner)
        }
        SecretTrigger::WhenEnemyAttacksHero => {
            // 敌方任意角色（随从或英雄）攻击己方英雄
            matches_when_enemy_attacks_hero(event, state, owner)
        }
        SecretTrigger::WhenEnemyAttacks => {
            // 敌方任意角色发起攻击
            matches!(event, Event::AttackDeclared { attacker, .. } if state.world().player(*attacker).is_some_and(|p| p != owner))
        }
        SecretTrigger::WhenFriendlyMinionDamaged => {
            // 己方随从受到伤害
            matches!(event, Event::DamageDealt { target, amount, .. } if *amount > 0 && state.world().player(*target).is_some_and(|p| p == owner) && state.world().card_type(*target) == Some(CardType::Minion))
        }
    }
}

/// 检查敌方随从是否攻击己方英雄。
fn matches_enemy_minion_attacks_hero(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    use crate::core::component::CardType;
    if let Event::AttackDeclared { attacker, defender } = event {
        let hero = state.player(owner).hero;
        *defender == hero
            && state.world().card_type(*attacker) == Some(CardType::Minion)
            && state.world().player(*attacker) == Some(owner.opponent())
    } else {
        false
    }
}

/// 检查 AfterFriendlyAttacked 触发条件。
fn matches_after_friendly_attacked(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    let Event::AttackDeclared { defender, .. } = event else {
        return false;
    };
    // 防御者是己方角色
    state.world().player(*defender).is_some_and(|p| p == owner)
}

/// 检查 AfterEnemyHeroAttacks 触发条件。
fn matches_after_enemy_hero_attacks(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    let Event::AttackDeclared { attacker, .. } = event else {
        return false;
    };
    // 攻击者是敌方英雄
    state.world().card_type(*attacker) == Some(CardType::Hero)
        && state.world().player(*attacker).is_some_and(|p| p != owner)
}

/// 检查敌方任意角色是否攻击己方英雄（误导触发条件）。
fn matches_when_enemy_attacks_hero(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    let Event::AttackDeclared { attacker, defender } = event else {
        return false;
    };
    let hero = state.player(owner).hero;
    *defender == hero && state.world().player(*attacker).is_some_and(|p| p != owner)
}

/// 解析奥秘效果。
///
/// 部分奥秘效果依赖触发事件上下文（狙击需要刚被打出的随从，
/// 误导/崇高牺牲/法术扭曲者需要重定向队列中的待结算事件），
/// 在此处理；其余效果委托给 `trigger::resolve_effect`。
fn resolve_secret_effect(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    entity: Entity,
    player: PlayerId,
    effect: CardEffect,
) {
    match effect {
        CardEffect::DamagePlayedMinion { amount } => {
            // 狙击：对刚被打出的随从造成伤害
            if let Event::MinionSummoned { minion, .. } = event {
                queue.push(Event::DamageDealt {
                    source: entity,
                    target: *minion,
                    amount,
                });
            }
        }
        CardEffect::RedirectAttackToRandomCharacter => {
            resolve_misdirection(state, queue, event, player);
        }
        CardEffect::SummonAndRedirectAttack { card_id } => {
            resolve_noble_sacrifice(state, queue, event, player, card_id);
        }
        CardEffect::SummonSpellbender => {
            resolve_spellbender(state, queue, event, player);
        }
        _ => crate::engine::trigger::resolve_effect(state, queue, entity, player, effect),
    }
}

/// 误导：将攻击重定向到另一个随机角色（包括攻击者自身，排除己方英雄）。
fn resolve_misdirection(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    owner: PlayerId,
) {
    let Event::AttackDeclared { attacker, .. } = event else {
        return;
    };
    let hero = state.player(owner).hero;
    // 收集双方所有角色（英雄 + 随从，含潜行），排除己方英雄
    let mut candidates: Vec<Entity> = [owner, owner.opponent()]
        .iter()
        .flat_map(|&pid| {
            state
                .world()
                .zones()
                .iter(Zone::Play, pid)
                .filter(|&e| {
                    let ct = state.world().card_type(e);
                    ct == Some(CardType::Minion) || ct == Some(CardType::Hero)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    candidates.retain(|&e| e != hero);
    if candidates.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(candidates.len());
    let new_target = candidates[idx];
    // 重定向待结算的攻击伤害
    if queue.redirect_damage(*attacker, hero, new_target) {
        // 若新目标是随从，它也会反击攻击者
        if state.world().card_type(new_target) == Some(CardType::Minion) {
            let atk = state
                .world()
                .effective_attack(new_target)
                .unwrap_or(crate::core::component::Attack(0))
                .0;
            if atk > 0 {
                queue.push(Event::DamageDealt {
                    source: new_target,
                    target: *attacker,
                    amount: atk,
                });
            }
        }
    }
}

/// 崇高牺牲：召唤防御者作为攻击的新目标。
///
/// 原防御者为随从时替换其反击；为英雄时新增防御者的反击。
fn resolve_noble_sacrifice(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    owner: PlayerId,
    card_id: &str,
) {
    let Event::AttackDeclared { attacker, defender } = event else {
        return;
    };
    let Some(defender_minion) =
        crate::engine::trigger::resolve_summon(state, queue, *defender, owner, card_id)
    else {
        // 战场已满无法召唤防御者
        return;
    };
    // 重定向攻击伤害到防御者
    queue.redirect_damage(*attacker, *defender, defender_minion);
    // 反击处理
    let defender_atk = state
        .world()
        .attack(defender_minion)
        .unwrap_or(crate::core::component::Attack(0))
        .0;
    if state.world().card_type(*defender) == Some(CardType::Minion) {
        // 原防御者是随从：替换其反击为防御者的反击
        queue.replace_damage(
            *defender,
            *attacker,
            Event::DamageDealt {
                source: defender_minion,
                target: *attacker,
                amount: defender_atk,
            },
        );
    } else {
        // 原防御者是英雄：新增防御者的反击
        queue.push(Event::DamageDealt {
            source: defender_minion,
            target: *attacker,
            amount: defender_atk,
        });
    }
}

/// 法术扭曲者：召唤 1/3 随从，将敌方法术对随从的待结算伤害重定向到它。
fn resolve_spellbender(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    owner: PlayerId,
) {
    let Event::CardPlayed { card, .. } = event else {
        return;
    };
    let Some(spellbender) =
        crate::engine::trigger::resolve_summon(state, queue, *card, owner, "MAGE_019t")
    else {
        return;
    };
    // 重定向来源为该法术、目标是随从的待结算伤害
    queue.redirect_damages(
        |s, t| s == *card && state.world().card_type(t) == Some(CardType::Minion),
        spellbender,
    );
}
