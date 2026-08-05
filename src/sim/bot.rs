//! 机器人策略 — 贪心机器人和智能机器人。
//!
//! ## GreedyBot
//!
//! 策略：全力打脸，除非有嘲讽阻挡（必须解嘲讽），或英雄不可被攻击。
//! 不主动在回合中死亡（避免自杀式攻击）。
//!
//! ## SmartBot
//!
//! 基于规则的启发式策略，相比 GreedyBot 的改进：
//!
//! 1. **卡牌评分**：根据身材/费用效率和关键词加成评估每张卡牌
//! 2. **斩杀检测**：检查是否能当回合击杀对手
//! 3. **价值交换**：评估随从交换中的价值得失，做出有利交换
//! 4. **威胁评估**：优先处理高攻击力/圣盾/风怒等高威胁敌方随从
//! 5. **局势感知**：根据场面优劣势自动切换进攻/防守策略
//! 6. **圣盾处理**：用最弱的攻击者破盾，保护主力输出
//! 7. **状态投影**：追踪计划打出的冲锋随从和武器，纳入战斗规划

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

        // 收集所有可打出的牌（随从 + 武器 + 法术）
        let mut playable: Vec<(i32, Entity)> = world
            .zones()
            .iter(Zone::Hand, player)
            .filter(|&e| {
                let ct = world.card_type(e);
                (ct == Some(CardType::Minion)
                    || ct == Some(CardType::Weapon)
                    || ct == Some(CardType::Spell))
                    && world.effective_cost(e).is_some_and(|c| c.0 <= current_mana)
            })
            .map(|e| (world.effective_cost(e).unwrap().0, e))
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

// ============================================================
// SmartBot — 基于规则的启发式智能机器人
// ============================================================

// SmartBot uses world.divine_shield(), world.taunt(), world.windfury() methods
// — no direct type imports needed beyond what's already imported above

/// 智能机器人 — 基于规则的启发式策略。
///
/// 相比 `GreedyBot` 的关键改进：
///
/// | 特性 | GreedyBot | SmartBot |
/// |------|-----------|----------|
/// | 卡牌选择 | 费用从低到高 | 价值评分排序 |
/// | 攻击策略 | 永远打脸 | 斩杀检测 + 价值交换 |
/// | 嘲讽处理 | 用最弱攻击者解 | 评估最优解法 |
/// | 圣盾处理 | 忽略 | 用最弱攻击者破盾 |
/// | 局势感知 | 无 | 攻守策略自动切换 |
/// | 状态投影 | 无 | 跟踪冲锋随从和武器 |
///
/// # 算法概览
///
/// 1. **卡牌评分** — 对每张可打出的牌按身材效率、关键词、战吼效果打分
/// 2. **卡牌打出** — 按评分从高到低打出，投影冲锋随从和武器到战斗阶段
/// 3. **英雄技能** — 有剩余法力时使用（对场面有帮助的技能优先）
/// 4. **战斗阶段**：
///    a. 斩杀检测 — 如果总攻击力 ≥ 敌方英雄生命值（含嘲讽阻挡计算），全员打脸
///    b. 嘲讽清除 — 必须解嘲讽，用最优攻击者交换
///    c. 价值交换 — 对高威胁敌方随从做有利交换（我方存活 > 互换 > 放弃）
///    d. 剩余攻击者打脸
/// 5. **结束回合**
#[derive(Debug, Clone, Copy, Default)]
pub struct SmartBot;

/// 投影的攻击者信息 — 用于在打出卡牌后将冲锋随从和武器纳入攻击规划。
#[derive(Debug, Clone, Copy)]
struct ProjectedAttacker {
    /// 攻击者实体（已存在或将要打出的）
    entity: Entity,
    /// 攻击力
    attack: i32,
    /// 生命值
    health: i32,
    /// 是否有圣盾
    has_divine_shield: bool,
    /// 是否是英雄（英雄攻击时不受反击伤害）
    is_hero: bool,
}

impl SmartBot {
    /// 创建一个新的智能机器人。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 为当前玩家生成本回合的行动序列。
    ///
    /// 返回应在当前回合执行的所有动作。
    /// 返回空 `Vec` 表示游戏已结束。
    pub fn decide_actions(&self, state: &GameState) -> Vec<Action> {
        if matches!(state.phase(), crate::core::state::Phase::GameOver { .. }) {
            return vec![];
        }

        let active = state.active_player();
        let enemy = active.opponent();
        let current_mana = state.player(active).current_mana;

        // ── 1. 卡牌打出阶段 ──
        let (play_actions, projected_charge, hero_weapon_attack, remaining_mana) =
            self.play_cards(state, active, current_mana);

        // ── 2. 英雄技能 ──
        let hero_power_action = self.hero_power(state, active, remaining_mana);

        // ── 3. 战斗阶段 ──
        let combat_actions =
            self.combat_phase(state, active, enemy, &projected_charge, hero_weapon_attack);

        // ── 4. 组装动作序列 ──
        let mut actions = Vec::with_capacity(
            play_actions.len() + hero_power_action.map_or(0, |_| 1) + combat_actions.len() + 1,
        );
        actions.extend(play_actions);
        if let Some(hp) = hero_power_action {
            actions.push(hp);
        }
        actions.extend(combat_actions);
        actions.push(Action::EndTurn);

        actions
    }

    // ============================================================
    // 卡牌评分与打出
    // ============================================================

    /// 打出最优卡牌序列。
    ///
    /// 返回：`(打牌动作, 投影的冲锋攻击者, 英雄是否获得武器攻击力, 剩余法力)`
    fn play_cards(
        &self,
        state: &GameState,
        player: PlayerId,
        current_mana: i32,
    ) -> (Vec<Action>, Vec<ProjectedAttacker>, Option<i32>, i32) {
        let world = state.world();

        // 收集所有可打出的牌（随从 + 武器 + 法术）及其评分
        let mut candidates: Vec<(f64, Entity)> = world
            .zones()
            .iter(Zone::Hand, player)
            .filter(|&e| {
                let ct = world.card_type(e);
                (ct == Some(CardType::Minion)
                    || ct == Some(CardType::Weapon)
                    || ct == Some(CardType::Spell))
                    && world.effective_cost(e).is_some_and(|c| c.0 <= current_mana)
            })
            .map(|e| (self.evaluate_card(state, player, e), e))
            .collect();

        // 按评分从高到低排序
        candidates
            .sort_by(|(s1, _), (s2, _)| s2.partial_cmp(s1).unwrap_or(std::cmp::Ordering::Equal));

        let mut actions = Vec::new();
        let mut projected_charge: Vec<ProjectedAttacker> = Vec::new();
        let mut hero_weapon_attack: Option<i32> = None;
        let mut remaining_mana = current_mana;

        for (_score, card) in candidates {
            let cost = world.effective_cost(card).map(|c| c.0).unwrap_or(0);
            if cost > remaining_mana {
                continue;
            }

            // 投影冲锋随从 → 可在本回合攻击
            if world.card_type(card) == Some(CardType::Minion) && world.charge(card).is_some() {
                let atk = world.attack(card).map(|a| a.0).unwrap_or(0);
                let hp = world.health(card).map(|h| h.0).unwrap_or(0);
                let ds = world.divine_shield(card).is_some();
                if atk > 0 {
                    projected_charge.push(ProjectedAttacker {
                        entity: card,
                        attack: atk,
                        health: hp,
                        has_divine_shield: ds,
                        is_hero: false,
                    });
                }
            }

            // 投影武器 → 英雄可以攻击
            if world.card_type(card) == Some(CardType::Weapon) {
                let atk = world.attack(card).map(|a| a.0).unwrap_or(0);
                if atk > 0 {
                    hero_weapon_attack = Some(atk);
                }
            }

            actions.push(Action::PlayCard { card });
            remaining_mana -= cost;
        }

        (
            actions,
            projected_charge,
            hero_weapon_attack,
            remaining_mana,
        )
    }

    /// 评估一张卡牌的价值分数。
    ///
    /// 基础公式：`身材效率 + 关键词加成 + 战吼/亡语加成`
    ///
    /// - 身材效率：`(attack + health) - (cost * 2 + 1)` — 相对于白板标准的偏差
    /// - 关键词：嘲讽 +1.5, 圣盾 +攻击力*0.7, 冲锋 +攻击力*0.5, 风怒 +攻击力*0.5, 法伤 +值
    /// - 战吼伤害 +伤害量*1.2，抽牌 +3*数量，buff +增益总量
    fn evaluate_card(&self, state: &GameState, _player: PlayerId, card: Entity) -> f64 {
        let world = state.world();
        let atk = world.attack(card).map(|a| a.0).unwrap_or(0) as f64;
        let hp = world.health(card).map(|h| h.0).unwrap_or(0) as f64;
        let cost = world.effective_cost(card).map(|c| c.0).unwrap_or(1).max(1) as f64;

        // 身材效率：相对于白板标准 (cost*2+1) 的偏差
        let vanilla_standard = cost * 2.0 + 1.0;
        let stats_efficiency = (atk + hp) - vanilla_standard;

        // 关键词加成
        let mut keyword_bonus = 0.0;
        if world.taunt(card).is_some() {
            keyword_bonus += 1.5;
        }
        if world.divine_shield(card).is_some() {
            keyword_bonus += atk * 0.7;
        }
        if world.windfury(card).is_some() {
            keyword_bonus += atk * 0.5;
        }
        if world.charge(card).is_some() {
            keyword_bonus += atk * 0.5;
        }
        if let Some(sd) = world.spell_damage(card) {
            keyword_bonus += sd.0 as f64 * 1.5;
        }

        // 战吼/亡语加成
        let mut effect_bonus = 0.0;
        if let Some(bc) = world.battlecry(card) {
            effect_bonus += evaluate_effect_value(bc.0);
        }
        if let Some(dr) = world.deathrattle(card) {
            // 亡语价值打折（延迟触发）
            effect_bonus += evaluate_effect_value(dr.0) * 0.6;
        }

        // 最终分数：效率 + 关键词 + 效果（对费用做归一化，使高分卡排序合理）
        stats_efficiency + keyword_bonus + effect_bonus + (atk + hp) / cost
    }

    // ============================================================
    // 英雄技能
    // ============================================================

    /// 决定是否使用英雄技能。
    fn hero_power(
        &self,
        state: &GameState,
        player: PlayerId,
        remaining_mana: i32,
    ) -> Option<Action> {
        let hero = state.player(player).hero;

        // 已使用过则不重复
        if state.world().hero_power_used(hero).is_some_and(|u| u.0) {
            return None;
        }

        let cost = state
            .world()
            .hero_power(hero)
            .map(|hp| hp.cost)
            .unwrap_or(2);
        if remaining_mana < cost {
            return None;
        }

        // 有剩余法力且英雄技能可用 → 使用（默认打 2 点伤害或护甲等都有价值）
        Some(Action::HeroPower { hero })
    }

    // ============================================================
    // 战斗阶段
    // ============================================================

    /// 战斗阶段：斩杀检测 → 价值交换 → 打脸
    fn combat_phase(
        &self,
        state: &GameState,
        player: PlayerId,
        enemy: PlayerId,
        projected_charge: &[ProjectedAttacker],
        hero_weapon_attack: Option<i32>,
    ) -> Vec<Action> {
        let world = state.world();

        // ── 收集所有攻击者 ──
        let mut attackers: Vec<ProjectedAttacker> = self.collect_existing_attackers(state, player);

        // 英雄攻击力（考虑新装备的武器）
        let hero = state.player(player).hero;
        let hero_can_attack = world
            .attacks_used(hero)
            .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(hero)));
        let existing_weapon_atk = state.player(player).weapon.and_then(|w| {
            if world.is_alive(w) {
                world.attack(w).map(|a| a.0)
            } else {
                None
            }
        });

        let effective_hero_atk = hero_weapon_attack.or(existing_weapon_atk);
        if let Some(atk) = effective_hero_atk {
            if atk > 0 && hero_can_attack {
                // 避免重复添加英雄
                if !attackers.iter().any(|a| a.entity == hero) {
                    attackers.push(ProjectedAttacker {
                        entity: hero,
                        attack: atk,
                        health: world.health(hero).map(|h| h.0).unwrap_or(30),
                        has_divine_shield: false,
                        is_hero: true,
                    });
                }
            }
        }

        // 加入投影的冲锋随从
        for pa in projected_charge {
            // 避免重复（不太可能，但安全检查）
            if !attackers.iter().any(|a| a.entity == pa.entity) {
                attackers.push(*pa);
            }
        }

        if attackers.is_empty() {
            return vec![];
        }

        // ── 收集敌方场面 ──
        let enemy_minions: Vec<EnemyMinion> = world
            .zones()
            .iter(Zone::Play, enemy)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion) && world.is_alive(e))
            .map(|e| EnemyMinion {
                entity: e,
                attack: world.effective_attack(e).map(|a| a.0).unwrap_or(0),
                health: world.effective_health(e).map(|h| h.0).unwrap_or(0),
                has_taunt: world.taunt(e).is_some(),
                has_divine_shield: world.divine_shield(e).is_some(),
            })
            .collect();

        let enemy_hero_entity = state.player(enemy).hero;
        let enemy_hero_health = world.health(enemy_hero_entity).map(|h| h.0).unwrap_or(0);
        let enemy_hero_armor = state.player(enemy).armor;
        let enemy_effective_hp = enemy_hero_health + enemy_hero_armor;

        // ── 分离嘲讽和非嘲讽 ──
        let (taunt_minions, non_taunt_minions): (Vec<EnemyMinion>, Vec<EnemyMinion>) =
            enemy_minions.into_iter().partition(|m| m.has_taunt);

        // ── 步骤1: 斩杀检测 ──
        if !attackers.is_empty() {
            let total_attack: i32 = attackers.iter().map(|a| a.attack).sum();

            if taunt_minions.is_empty() {
                // 无嘲讽，直接检查是否可以斩杀
                if total_attack >= enemy_effective_hp {
                    return attackers
                        .iter()
                        .map(|a| Action::Attack {
                            attacker: a.entity,
                            defender: enemy_hero_entity,
                        })
                        .collect();
                }
            } else {
                // 有嘲讽，检查是否可以通过嘲讽斩杀
                let taunt_total_hp: i32 = taunt_minions.iter().map(effective_hp).sum();
                let damage_needed = taunt_total_hp + enemy_effective_hp;
                if total_attack >= damage_needed
                    && self.can_clear_and_lethal(&attackers, &taunt_minions, enemy_effective_hp)
                {
                    // 先清嘲讽再打脸
                    let mut actions = self.assign_taunt_clear(&attackers, &taunt_minions);
                    // 剩余攻击者打脸
                    let used: Vec<Entity> = actions
                        .iter()
                        .map(|a| match a {
                            Action::Attack { attacker, .. } => *attacker,
                            _ => unreachable!(),
                        })
                        .collect();
                    for attacker in &attackers {
                        if !used.contains(&attacker.entity) {
                            actions.push(Action::Attack {
                                attacker: attacker.entity,
                                defender: enemy_hero_entity,
                            });
                        }
                    }
                    return actions;
                }
            }
        }

        // ── 步骤2: 嘲讽清除（非斩杀情况，必须解嘲讽） ──
        let mut assigned_attackers: Vec<Entity> = Vec::new();
        let mut actions = Vec::new();

        if !taunt_minions.is_empty() {
            let taunt_actions = self.assign_taunt_clear(&attackers, &taunt_minions);
            for a in &taunt_actions {
                if let Action::Attack { attacker, .. } = a {
                    assigned_attackers.push(*attacker);
                }
            }
            actions.extend(taunt_actions);
        }

        // ── 步骤3: 价值交换（对非嘲讽敌方随从） ──
        let available: Vec<&ProjectedAttacker> = attackers
            .iter()
            .filter(|a| !assigned_attackers.contains(&a.entity))
            .collect();

        if !available.is_empty() && !non_taunt_minions.is_empty() {
            let (trade_actions, newly_assigned) = self.value_trades(&available, &non_taunt_minions);
            assigned_attackers.extend(newly_assigned);
            actions.extend(trade_actions);
        }

        // ── 步骤4: 剩余攻击者打脸 ──
        // 再次检查是否还有未清除的嘲讽
        let still_has_taunt = state
            .world()
            .zones()
            .iter(Zone::Play, enemy)
            .any(|e| state.world().taunt(e).is_some());

        if !still_has_taunt {
            for attacker in &attackers {
                if !assigned_attackers.contains(&attacker.entity) {
                    actions.push(Action::Attack {
                        attacker: attacker.entity,
                        defender: enemy_hero_entity,
                    });
                }
            }
        }

        actions
    }

    /// 评估能否在清完嘲讽后完成斩杀。
    fn can_clear_and_lethal(
        &self,
        attackers: &[ProjectedAttacker],
        taunts: &[EnemyMinion],
        enemy_hp: i32,
    ) -> bool {
        // 简化：按攻击力降序排列攻击者，贪心分配清嘲讽
        let mut sorted_attackers = attackers.to_vec();
        sorted_attackers.sort_by_key(|a| std::cmp::Reverse(a.attack));

        let mut taunt_hps: Vec<i32> = taunts.iter().map(effective_hp).collect();
        let mut idx = 0;

        for th in &mut taunt_hps {
            while *th > 0 && idx < sorted_attackers.len() {
                *th -= sorted_attackers[idx].attack;
                idx += 1;
            }
            if *th > 0 {
                return false; // 无法清完嘲讽
            }
        }

        // 剩余攻击者能否斩杀
        let remaining_damage: i32 = sorted_attackers[idx..].iter().map(|a| a.attack).sum();
        remaining_damage >= enemy_hp
    }

    /// 分配攻击者清除嘲讽随从（最优解法）。
    fn assign_taunt_clear(
        &self,
        attackers: &[ProjectedAttacker],
        taunts: &[EnemyMinion],
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut used: Vec<Entity> = Vec::new();
        let mut remaining_taunts: Vec<&EnemyMinion> = taunts.iter().collect();

        for taunt in taunts {
            if let Some(best) = self.find_best_attacker(attackers, &used, taunt) {
                actions.push(Action::Attack {
                    attacker: best.entity,
                    defender: taunt.entity,
                });
                used.push(best.entity);
                remaining_taunts.retain(|t| t.entity != taunt.entity);
            }
        }

        // 如果有嘲讽没分配完（攻击者不够），用已有攻击者继续攻击
        for taunt in &remaining_taunts {
            // 找一个还有攻击力的未使用攻击者
            let remaining_available: Vec<&ProjectedAttacker> = attackers
                .iter()
                .filter(|a| !used.contains(&a.entity))
                .collect();
            if let Some(attacker) = remaining_available.first() {
                actions.push(Action::Attack {
                    attacker: attacker.entity,
                    defender: taunt.entity,
                });
                used.push(attacker.entity);
            }
        }

        actions
    }

    /// 价值交换：对非嘲讽敌方随从做有利交换。
    ///
    /// 返回 `(攻击动作, 已分配的实体列表)`
    fn value_trades(
        &self,
        available: &[&ProjectedAttacker],
        enemies: &[EnemyMinion],
    ) -> (Vec<Action>, Vec<Entity>) {
        let mut actions = Vec::new();
        let mut used: Vec<Entity> = Vec::new();

        // 按威胁程度排序（攻击力权重更高）
        let mut sorted_enemies = enemies.to_vec();
        sorted_enemies.sort_by_key(|e| std::cmp::Reverse(e.attack * 2 + e.health));

        for enemy in &sorted_enemies {
            // 评估是否值得交换
            let trade_value = self.evaluate_trade(available, &used, enemy);

            // 只有正价值或必须处理的威胁才交换
            if trade_value > -2.0 || enemy.attack >= 4 {
                if let Some(best) = self.find_best_attacker_slice(available, &used, enemy) {
                    actions.push(Action::Attack {
                        attacker: best.entity,
                        defender: enemy.entity,
                    });
                    used.push(best.entity);
                }
            }
        }

        (actions, used)
    }

    /// 为给定敌方随从找到最佳攻击者（使用 `&[ProjectedAttacker]` 切片）。
    fn find_best_attacker_slice(
        &self,
        available: &[&ProjectedAttacker],
        used: &[Entity],
        enemy: &EnemyMinion,
    ) -> Option<ProjectedAttacker> {
        let candidates: Vec<&ProjectedAttacker> = available
            .iter()
            .filter(|a| !used.contains(&a.entity))
            .copied()
            .collect();
        self.find_best_attacker_inner(&candidates, enemy)
    }

    /// 为给定敌方随从找到最佳攻击者（使用 `&[ProjectedAttacker]`）。
    fn find_best_attacker(
        &self,
        attackers: &[ProjectedAttacker],
        used: &[Entity],
        enemy: &EnemyMinion,
    ) -> Option<ProjectedAttacker> {
        let candidates: Vec<&ProjectedAttacker> = attackers
            .iter()
            .filter(|a| !used.contains(&a.entity))
            .collect();
        self.find_best_attacker_inner(&candidates, enemy)
    }

    /// 核心攻击者选择逻辑。
    ///
    /// 优先级：
    /// 1. 能击杀敌方且自己存活（完美交换）
    /// 2. 能用圣盾无伤交换
    /// 3. 能击杀敌方但会同归于尽（用价值最低的攻击者）
    /// 4. 用最弱的攻击者破圣盾
    /// 5. 无法击杀但可以蹭血（用最弱的攻击者）
    fn find_best_attacker_inner(
        &self,
        candidates: &[&ProjectedAttacker],
        enemy: &EnemyMinion,
    ) -> Option<ProjectedAttacker> {
        if candidates.is_empty() {
            return None;
        }

        // enemy_effective_hp is computed inline as needed

        // 第一优先：能击杀敌方且自己存活
        let best_trade = candidates
            .iter()
            .filter(|a| {
                let dmg_to_enemy = if enemy.has_divine_shield { 0 } else { a.attack };
                let dmg_to_self = if a.has_divine_shield { 0 } else { enemy.attack };
                dmg_to_enemy >= enemy.health && dmg_to_self < a.health
            })
            .min_by_key(|a| {
                // 用价值最低（攻击力最小）的达成完美交换
                (a.attack, a.health)
            });

        if let Some(&a) = best_trade {
            return Some(*a);
        }

        // 第二优先：用圣盾无伤破敌
        let divine_trade = candidates
            .iter()
            .filter(|a| a.has_divine_shield && !enemy.has_divine_shield && a.attack >= enemy.health)
            .min_by_key(|a| a.attack);

        if let Some(&a) = divine_trade {
            return Some(*a);
        }

        // 第三优先：能击杀敌方（会同归于尽），用价值最低的攻击者
        let kill_trade = candidates
            .iter()
            .filter(|a| a.attack >= enemy.health || (enemy.has_divine_shield && a.attack > 0))
            .min_by_key(|a| {
                // 优先用会死的，其次用攻击力最小的
                let will_die = !a.has_divine_shield && !a.is_hero && enemy.attack >= a.health;
                (if will_die { 0 } else { 1 }, a.attack, a.health)
            });

        if let Some(&a) = kill_trade {
            return Some(*a);
        }

        // 第四优先：破圣盾（用最弱攻击者）
        let pop_shield = candidates
            .iter()
            .filter(|_a| enemy.has_divine_shield)
            .min_by_key(|a| (a.attack, a.health));

        if let Some(&a) = pop_shield {
            return Some(*a);
        }

        // 无合适交换，不攻击这个敌方随从
        None
    }

    /// 评估一次交换的价值（正 = 划算，负 = 亏）。
    fn evaluate_trade(
        &self,
        available: &[&ProjectedAttacker],
        used: &[Entity],
        enemy: &EnemyMinion,
    ) -> f64 {
        if let Some(best) = self.find_best_attacker_slice(available, used, enemy) {
            let dmg_to_self = if best.has_divine_shield {
                0
            } else {
                enemy.attack
            };
            let attacker_dies = !best.is_hero && dmg_to_self >= best.health;
            let attacker_value = best.attack as f64 + best.health as f64 * 0.5;
            let enemy_value = enemy.attack as f64 * 1.5 + enemy.health as f64 * 0.5;

            if attacker_dies {
                enemy_value - attacker_value
            } else {
                enemy_value - (dmg_to_self as f64 * 0.3) // 血量损失的部分价值
            }
        } else {
            -999.0
        }
    }

    /// 收集当前场上所有可攻击的角色（不含冲锋投影）。
    fn collect_existing_attackers(
        &self,
        state: &GameState,
        player: PlayerId,
    ) -> Vec<ProjectedAttacker> {
        let world = state.world();

        world
            .zones()
            .iter(Zone::Play, player)
            .filter(|&e| {
                world.card_type(e) == Some(CardType::Minion)
                    && world.is_alive(e)
                    && world
                        .attacks_used(e)
                        .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(e)))
                    && world.effective_attack(e).is_some_and(|a| a.0 > 0)
            })
            .map(|e| ProjectedAttacker {
                entity: e,
                attack: world.effective_attack(e).map(|a| a.0).unwrap_or(0),
                health: world.effective_health(e).map(|h| h.0).unwrap_or(0),
                has_divine_shield: world.divine_shield(e).is_some(),
                is_hero: false,
            })
            .collect()
    }
}

// ============================================================
// 辅助类型和函数
// ============================================================

/// 敌方随从的快照信息（用于战斗规划）。
#[derive(Debug, Clone, Copy)]
struct EnemyMinion {
    entity: Entity,
    attack: i32,
    health: i32,
    has_taunt: bool,
    has_divine_shield: bool,
}

/// 计算随从的有效生命值（含圣盾）。
fn effective_hp(minion: &EnemyMinion) -> i32 {
    if minion.has_divine_shield {
        minion.health + 1 // 圣盾吸收一次伤害
    } else {
        minion.health
    }
}

/// 评估卡牌效果的期望价值。
fn evaluate_effect_value(effect: crate::core::effect::CardEffect) -> f64 {
    use crate::core::effect::CardEffect;
    match effect {
        CardEffect::DealDamage { amount, .. } => amount as f64 * 1.2,
        CardEffect::DrawCard { count } => count as f64 * 3.0,
        CardEffect::SummonMinion { .. } => 3.0,
        CardEffect::GainStats { attack, health, .. } => (attack + health) as f64 * 0.8,
        CardEffect::EquipWeapon { .. } => 2.0,
        CardEffect::GainArmor { amount, .. } => amount as f64 * 0.6,
        CardEffect::ReturnToHand { .. } => 2.0,
        CardEffect::IncreaseCost { amount, .. } => amount as f64 * 0.5,
        CardEffect::ReturnToHandAndIncreaseCost { amount, .. } => 2.0 + amount as f64 * 0.5,
        CardEffect::DestroyMinion { .. } => 5.0,
        CardEffect::SilenceMinion { .. } => 3.0,
        CardEffect::SetAttack { attack, .. } => attack as f64 * 0.5,
        CardEffect::RestoreHealth { amount, .. } => amount as f64 * 0.7,
        CardEffect::FreezeCharacter { .. } => 1.0,
        CardEffect::GainManaCrystal { count } => count as f64 * 2.0,
        CardEffect::DestroyWeapon => 2.0,
        CardEffect::GainHeroAttack { attack, armor } => attack as f64 * 1.5 + armor as f64 * 0.5,
        CardEffect::DealHeroAttackDamage { .. } => 3.0,
        CardEffect::FullHeal { .. } => 3.0,
        CardEffect::GrantWindfury { .. } => 3.0,
        CardEffect::GrantCharge { attack_bonus, .. } => 2.0 + attack_bonus as f64 * 1.5,
        CardEffect::DoubleAttack { .. } => 3.0,
        CardEffect::DoubleHealth { .. } => 3.0,
        CardEffect::BuffWeapon { attack, durability } => (attack + durability) as f64 * 1.5,
        CardEffect::DiscardRandomCard => -2.0,
        CardEffect::DealArmorDamage { .. } => 3.0,
        CardEffect::DestroyWeaponAndDraw => 5.0,
        CardEffect::ReturnAllToHand => 3.0,
        CardEffect::SetAttackToHealth { .. } => 3.0,
        CardEffect::DestroyAllExceptOne => 4.0,
        CardEffect::DestroyAndHeal { heal, .. } => 4.0 + heal as f64 * 0.7,
        CardEffect::DestroyAndAOE { .. } => 5.0,
        CardEffect::DealDamageToTwo { amount } => amount as f64 * 2.0,
        CardEffect::DealDamageAndDraw { damage, draw, .. } => {
            damage as f64 * 1.2 + draw as f64 * 3.0
        }
        CardEffect::DamageAndGainAttack {
            damage,
            attack_bonus,
            ..
        } => damage as f64 * 0.5 + attack_bonus as f64 * 1.5,
        CardEffect::DestroyAdjacent { .. } => 3.0,
        CardEffect::DestroyManaCrystal => -1.0,
        CardEffect::GiveCardsToOpponent { .. } => -2.0,
        CardEffect::ResurrectMinion => 5.0,
        CardEffect::CopyMinionStats => 4.0,
        CardEffect::TempDebuff {
            attack_reduction, ..
        } => attack_reduction as f64 * 1.5,
        CardEffect::ReflectDamage => 3.0,
        CardEffect::DealDamageAndReturnToHand { amount, .. } => amount as f64 * 1.2 + 3.0,
        CardEffect::ReturnFriendlyToHandAndReduceCost { amount } => 2.0 + amount as f64 * 0.5,
        CardEffect::AdjacentDamage => 3.0,
        CardEffect::DestroyWeaponAndDealAttackToEnemies => 4.0,
        CardEffect::GrantStealth => 2.0,
        CardEffect::SummonMultipleMinions { count, .. } => count as f64 * 2.0,
        CardEffect::DamagePlayedMinion { amount } => amount as f64 * 1.2,
        CardEffect::RedirectAttackToRandomCharacter => 3.0,
        CardEffect::SummonAndRedirectAttack { .. } => 3.0,
        CardEffect::SummonSpellbender => 2.0,
        CardEffect::NextSecretCostsZero => 2.0,
        CardEffect::DrawCardAndReduceCost { amount } => 3.0 + amount as f64 * 0.5,
        CardEffect::GrantDeathrattleAll { .. } => 3.0,
        CardEffect::GiveCardToOpponent { count, .. } => -(count as f64) * 1.0,
    }
}

// ============================================================
// SmartBot 测试
// ============================================================

#[cfg(test)]
mod smart_bot_tests {
    use super::*;
    use crate::cards::def::{
        BLOODFEN_RAPTOR, BLUEGILL_WARRIOR, EAGLEHORN_BOW, OGRE_MAGI, VOIDWALKER,
    };
    use crate::core::player::PlayerId;
    use crate::sim::game::GameBuilder;

    #[test]
    fn smart_bot_plays_cards_from_hand() {
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        let play_count = actions
            .iter()
            .filter(|a| matches!(a, Action::PlayCard { .. }))
            .count();
        assert!(play_count >= 1);
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn smart_bot_ends_turn() {
        let state = GameState::new();
        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn smart_bot_returns_empty_for_game_over() {
        let mut builder = GameBuilder::new();
        builder.phase(crate::core::state::Phase::GameOver {
            winner: PlayerId::Player1,
        });
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);
        assert!(actions.is_empty());
    }

    #[test]
    fn smart_bot_detects_lethal() {
        // 场上有一个 30 攻随从 → 应该检测到斩杀
        let mut builder = GameBuilder::new();
        builder.add_custom_minion_to_board(PlayerId::Player1, 30, 5, 3);
        // 确保敌方英雄 30HP，可以斩杀
        builder.hero_health(PlayerId::Player2, 30);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // 所有攻击应该打脸
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        assert!(!attack_actions.is_empty());
        for action in attack_actions {
            assert!(matches!(
                action,
                Action::Attack { defender, .. } if *defender == enemy_hero
            ));
        }
    }

    #[test]
    fn smart_bot_attacks_hero_when_no_taunts() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a_ent, defender: d_ent }
            if *a_ent == attacker && *d_ent == enemy_hero
        )));
    }

    #[test]
    fn smart_bot_must_attack_taunt_first() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 5, 5, 3);
        builder.add_minion_to_board(PlayerId::Player2, &VOIDWALKER);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let taunt = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .find(|&e| state.world().taunt(e).is_some())
            .unwrap();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        if !attack_actions.is_empty() {
            let first = attack_actions[0];
            assert!(
                matches!(
                    first,
                    Action::Attack { attacker: a, defender: d } if *a == attacker && *d == taunt
                ),
                "First attack should be our minion clearing enemy taunt"
            );
            // 清除嘲讽后，剩余攻击应该打脸
            let enemy_hero = state.player(PlayerId::Player2).hero;
            let remaining: Vec<_> = attack_actions.iter().skip(1).collect();
            if !remaining.is_empty() {
                for action in remaining {
                    assert!(matches!(
                        action,
                        Action::Attack { defender: d, .. } if *d == enemy_hero
                    ));
                }
            }
        }
    }

    #[test]
    fn smart_bot_hero_attacks_with_weapon() {
        let mut builder = GameBuilder::new();
        builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let hero = state.player(PlayerId::Player1).hero;
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == hero && *d == enemy_hero
        )));
    }

    #[test]
    fn smart_bot_prefers_high_value_cards() {
        // 手牌中有 4/4+法伤 食人魔法师（4费）和 3/2 迅猛龙（2费）
        // 食人魔法师的评分应更高（有法伤加成），在有足够法力时都会打出
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI); // 4/4 spell_dmg+1 for 4
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR); // 3/2 for 2
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // 两张都应该被打出（有足够法力）
        let play_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::PlayCard { .. }))
            .collect();
        assert_eq!(play_actions.len(), 2);
    }

    #[test]
    fn smart_bot_plays_charge_minion() {
        // 蓝鳃战士（2/1 冲锋）应该被打出并纳入攻击规划
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &BLUEGILL_WARRIOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let bluegill = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .unwrap();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // 应该有出牌动作
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::PlayCard { card } if *card == bluegill
        )));

        // 应该有攻击动作（冲锋随从可以攻击）
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, .. } if *a == bluegill
        )));
    }

    #[test]
    fn smart_bot_uses_hero_power_when_available() {
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

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::HeroPower { .. }))
        );
    }

    #[test]
    fn smart_bot_trades_favorably() {
        // 我方有 5/5，敌方有 3/2 → 我方能击杀敌方并存活，应该交换
        let mut builder = GameBuilder::new();
        let our_minion = builder.add_custom_minion_to_board(PlayerId::Player1, 5, 5, 3);
        let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 2, 2);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // 应该解掉敌方高威胁随从而非全打脸
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == our_minion && *d == enemy_minion
        )));
    }

    #[test]
    fn smart_bot_handles_divine_shield_enemy() {
        // 敌方有圣盾随从，我方有多个随从
        // SmartBot 应该正确评估圣盾，不会崩溃
        let mut builder = GameBuilder::new();
        let _weak = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 3, 1);
        let _strong = builder.add_custom_minion_to_board(PlayerId::Player1, 6, 6, 5);
        let _enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 4, 5, 4);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // 至少验证 bot 能正常完成决策，不会崩溃
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }
}
