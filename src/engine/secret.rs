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

use crate::core::component::{Secret, SecretTrigger};
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
            // 解析奥秘效果
            crate::engine::trigger::resolve_effect(state, queue, *entity, *player, secret.effect);
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
            matches!(event, Event::CardPlayed { player, .. } if *player != owner)
        }
        SecretTrigger::WhenEnemyMinionAttacksHero => {
            matches_enemy_minion_attacks_hero(event, state, owner)
        }
    }
}

/// 检查敌方随从是否攻击己方英雄。
fn matches_enemy_minion_attacks_hero(
    event: &Event,
    state: &GameState,
    owner: PlayerId,
) -> bool {
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
    use crate::core::component::CardType;
    // 攻击者是敌方英雄
    state.world().card_type(*attacker) == Some(CardType::Hero)
        && state.world().player(*attacker).is_some_and(|p| p != owner)
}
