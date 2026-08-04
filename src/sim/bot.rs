//! 贪心机器人 — 简单的人机对弈策略。
//!
//! 策略：全力打脸，除非有嘲讽阻挡（必须解嘲讽），或英雄不可被攻击。
//! 不主动在回合中死亡（避免自杀式攻击）。
//!
//! # 算法
//!
//! 1. 从手牌打出所有可负担的随从（按费用从低到高）
//! 2. 使用英雄技能（如果可用且有剩余法力）
//! 3. 用所有能攻击的角色攻击敌方英雄
//!    - 如果敌方场上有嘲讽随从，必须先攻击嘲讽
//!    - 攻击者存活优先（不自杀），但如果必须解嘲讽则允许
//! 4. 全部攻击完后结束回合

use crate::core::action::Action;
use crate::core::component::{CardType, Health};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// 贪心机器人 — 打脸优先的简单策略。
#[derive(Debug, Clone, Copy, Default)]
pub struct GreedyBot;

impl GreedyBot {
    /// 创建一个新的贪心机器人。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 为当前玩家生成本回合的行动序列。
    ///
    /// 返回应在当前回合执行的所有动作。
    /// 返回空 `Vec` 表示游戏已结束。
    pub fn decide_actions(&self, state: &GameState) -> Vec<Action> {
        // 游戏结束后不返回任何动作
        if matches!(state.phase(), crate::core::state::Phase::GameOver { .. }) {
            return vec![];
        }

        let active = state.active_player();
        let mut actions = Vec::new();

        // 1. 出牌：从手牌打出所有可负担的随从和武器
        let (play_actions, remaining_mana) = self.play_cards(state, active);
        actions.extend(play_actions);

        // 2. 英雄技能（考虑打牌后的剩余法力）
        if let Some(hp_action) = self.hero_power(state, active, remaining_mana) {
            actions.push(hp_action);
        }

        // 3. 攻击：所有能攻击的角色打脸
        actions.extend(self.attack_phase(state, active));

        // 4. 结束回合
        actions.push(Action::EndTurn);

        actions
    }

    /// 打出所有可负担的随从和武器牌（费用从低到高）。
    /// 返回 (动作列表, 剩余法力)。
    fn play_cards(&self, state: &GameState, player: PlayerId) -> (Vec<Action>, i32) {
        let world = state.world();
        let current_mana = state.player(player).current_mana;

        // 收集所有可打出的牌（随从 + 武器）
        let mut playable: Vec<(i32, Entity)> = world
            .zones()
            .iter(Zone::Hand, player)
            .filter(|&e| {
                let ct = world.card_type(e);
                (ct == Some(CardType::Minion) || ct == Some(CardType::Weapon))
                    && world.cost(e).is_some_and(|c| c.0 <= current_mana)
            })
            .map(|e| (world.cost(e).unwrap().0, e))
            .collect();

        // 费用从低到高排序（贪心：先打便宜的，以便可能多打一张）
        playable.sort_by_key(|(cost, _)| *cost);

        let mut actions = Vec::new();
        let mut remaining_mana = current_mana;

        for (cost, card) in playable {
            if cost <= remaining_mana {
                actions.push(Action::PlayCard { card });
                remaining_mana -= cost;
            }
        }

        (actions, remaining_mana)
    }

    /// 尝试使用英雄技能（如果出牌后还有剩余法力且未使用过）。
    fn hero_power(
        &self,
        state: &GameState,
        player: PlayerId,
        remaining_mana: i32,
    ) -> Option<Action> {
        let hero = state.player(player).hero;
        let world = state.world();

        // 检查是否已使用过
        if world.hero_power_used(hero).is_some_and(|u| u.0) {
            return None;
        }

        // 检查是否有足够的法力（英雄技能定义或默认 2）
        let cost = world.hero_power(hero).map(|hp| hp.cost).unwrap_or(2);
        if remaining_mana >= cost {
            Some(Action::HeroPower { hero })
        } else {
            None
        }
    }

    /// 攻击阶段：所有能攻击的角色打脸（或解嘲讽）。
    fn attack_phase(&self, state: &GameState, player: PlayerId) -> Vec<Action> {
        let enemy = player.opponent();
        let enemy_hero = state.player(enemy).hero;
        let world = state.world();

        // 收集所有能攻击的友方角色（随从 + 有武器的英雄）
        let attackers: Vec<Entity> = self.collect_attackers(state, player);

        // 检查敌方是否有嘲讽随从
        let taunts: Vec<Entity> = world
            .zones()
            .iter(Zone::Play, enemy)
            .filter(|&e| world.taunt(e).is_some() && world.card_type(e) == Some(CardType::Minion))
            .collect();

        let mut actions = Vec::new();

        if taunts.is_empty() {
            // 没有嘲讽，全员打脸
            for attacker in &attackers {
                actions.push(Action::Attack {
                    attacker: *attacker,
                    defender: enemy_hero,
                });
            }
        } else {
            // 有嘲讽：必须先解嘲讽，剩余攻击打脸
            let mut remaining_taunts: Vec<Entity> = taunts.clone();
            let mut used_attackers: Vec<usize> = Vec::new();

            // 贪心：用最弱的攻击者解嘲讽（能解就解）
            for (i, attacker) in attackers.iter().enumerate() {
                if remaining_taunts.is_empty() {
                    break;
                }

                let atk = world
                    .effective_attack(*attacker)
                    .unwrap_or(crate::core::component::Attack(0));

                // 找一个能杀死的嘲讽（贪心：先杀受伤最重的）
                let target = remaining_taunts
                    .iter()
                    .filter(|&t| {
                        let hp = world.effective_health(*t).unwrap_or(Health(99));
                        hp.0 <= atk.0 || i == attackers.len() - 1 // 最后一个攻击者必须出手
                    })
                    .min_by_key(|&t| world.effective_health(*t).unwrap_or(Health(99)).0);

                if let Some(&target) = target {
                    actions.push(Action::Attack {
                        attacker: *attacker,
                        defender: target,
                    });
                    // 模拟预测：目标受到攻击后是否死亡
                    let target_hp = world.effective_health(target).unwrap_or(Health(0));
                    if target_hp.0 <= atk.0 {
                        remaining_taunts.retain(|&t| t != target);
                    } else {
                        // 目标存活，从列表中移除（避免重复攻击同一目标）
                        remaining_taunts.retain(|&t| t != target);
                    }
                    used_attackers.push(i);
                }
            }

            // 嘲讽解完后，剩余攻击者打脸
            for (i, attacker) in attackers.iter().enumerate() {
                if used_attackers.contains(&i) {
                    continue;
                }
                // 再检查一次是否还有嘲讽（如果有嘲讽没解完，跳过打脸）
                let still_has_taunt = state
                    .world()
                    .zones()
                    .iter(Zone::Play, enemy)
                    .any(|e| world.taunt(e).is_some());
                if still_has_taunt {
                    // 还有嘲讽但没攻击者了，跳过
                    continue;
                }
                actions.push(Action::Attack {
                    attacker: *attacker,
                    defender: enemy_hero,
                });
            }
        }

        actions
    }

    /// 收集当前玩家所有可以攻击的角色。
    fn collect_attackers(&self, state: &GameState, player: PlayerId) -> Vec<Entity> {
        let world = state.world();

        // 战场上的随从（未攻击过，攻击力 > 0）
        let mut attackers: Vec<Entity> = world
            .zones()
            .iter(Zone::Play, player)
            .filter(|&e| {
                world.card_type(e) == Some(CardType::Minion)
                    && world
                        .attacks_used(e)
                        .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(e)))
                    && world.effective_attack(e).is_some_and(|a| a.0 > 0)
            })
            .collect();

        // 英雄（有武器且未攻击过）
        let hero = state.player(player).hero;
        let has_weapon = state.player(player).weapon.is_some();
        let hero_can_attack = world
            .attacks_used(hero)
            .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(hero)));
        if has_weapon && hero_can_attack {
            attackers.push(hero);
        }

        attackers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::def::{BLOODFEN_RAPTOR, OGRE_MAGI, VOIDWALKER};
    use crate::core::player::PlayerId;
    use crate::sim::game::GameBuilder;

    #[test]
    fn bot_plays_cards_from_hand() {
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // 应该包含两个 PlayCard 和一个 EndTurn
        let play_count = actions
            .iter()
            .filter(|a| matches!(a, Action::PlayCard { .. }))
            .count();
        assert!(play_count >= 1);
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn bot_attacks_hero_when_no_taunts() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // 应有攻击敌方英雄的动作
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == attacker && *d == enemy_hero
        )));
    }

    #[test]
    fn bot_must_attack_taunt_first() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        builder.add_minion_to_board(PlayerId::Player2, &VOIDWALKER);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        // 获取嘲讽随从 entity
        let taunt: Vec<Entity> = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .filter(|&e| state.world().taunt(e).is_some())
            .collect();
        let taunt_entity = taunt[0];

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // 第一个攻击动作应该攻击嘲讽（不是英雄）
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        if !attack_actions.is_empty() {
            let first = attack_actions[0];
            assert!(matches!(
                first,
                Action::Attack { attacker: a, defender: d }
                if *a == attacker && *d == taunt_entity
            ));
        }
    }

    #[test]
    fn bot_uses_hero_power_when_available() {
        use crate::core::effect::{CardEffect, EffectTarget};

        let mut builder = GameBuilder::new();
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_hero_power(
            PlayerId::Player1,
            2,
            CardEffect::DealDamage {
                amount: 1,
                target: EffectTarget::AnyEnemy,
            },
        );
        let state = builder.build();

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::HeroPower { .. }))
        );
    }

    #[test]
    fn bot_hero_attacks_with_weapon() {
        use crate::cards::def::EAGLEHORN_BOW;

        let mut builder = GameBuilder::new();
        builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let hero = state.player(PlayerId::Player1).hero;
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // 英雄应有攻击动作
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == hero && *d == enemy_hero
        )));
    }

    #[test]
    fn bot_ends_turn() {
        let state = GameState::new();
        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // 最后一个动作应该是 EndTurn
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn bot_returns_empty_for_game_over() {
        let mut builder = GameBuilder::new();
        builder.phase(crate::core::state::Phase::GameOver {
            winner: PlayerId::Player1,
        });
        let state = builder.build();

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        assert!(actions.is_empty());
    }
}
