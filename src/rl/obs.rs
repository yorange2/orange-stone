//! 观察空间 — 将 `GameState` 张量化为固定长度的浮点向量。
//!
//! 观察从指定玩家的视角编码（归一化到 [0, 1]），布局固定、长度恒定，
//! 便于直接送入神经网络。
//!
//! # 布局（`OBS_LEN` = 168）
//!
//! | 区间 | 内容 | 归一化 |
//! |------|------|--------|
//! | 0..3   | 己方英雄：生命、攻击、护甲 | /30, /15, /30 |
//! | 3..5   | 己方法力：总水晶、当前水晶 | /10 |
//! | 5..8   | 敌方英雄：生命、攻击、护甲 | /30, /15, /30 |
//! | 8..10  | 牌库数量：己方、敌方 | /30 |
//! | 10..70 | 己方手牌（最多 10 张）：每张 [费用, 攻击, 生命, 随从, 法术, 武器] | /10, /15, /30, 0/1 |
//! | 70..119 | 己方战场（最多 7 个随从）：每个 [攻击, 生命, 嘲讽, 圣盾, 风怒, 冲锋, 潜行] | /15, /30, 0/1 |
//! | 119..168 | 敌方战场（同上，7 个槽位） | |

use crate::core::component::CardType;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// 手牌上限（编码槽位数）
pub const MAX_HAND: usize = 10;
/// 战场随从上限（编码槽位数）
pub const MAX_BOARD: usize = 7;
/// 每张手牌的特征数
pub const CARD_FEATURES: usize = 6;
/// 每个随从的特征数
pub const MINION_FEATURES: usize = 7;

/// 观察向量长度 — 固定值，供 Python 侧预分配。
pub const OBS_LEN: usize = 10 + MAX_HAND * CARD_FEATURES + 2 * MAX_BOARD * MINION_FEATURES;

/// 归一化辅助：值除以分母并截断到 [0, 1]。
fn norm(value: i32, max: f32) -> f32 {
    (value as f32 / max).clamp(0.0, 1.0)
}

fn normf(value: f32, max: f32) -> f32 {
    (value / max).clamp(0.0, 1.0)
}

/// 编码指定玩家的英雄观察块（生命、攻击、护甲）。
fn hero_block(state: &GameState, player: PlayerId, out: &mut Vec<f32>) {
    let hero = state.player(player).hero;
    let world = state.world();
    out.push(norm(world.health(hero).map_or(0, |h| h.0), 30.0));
    out.push(normf(
        world.effective_attack(hero).map_or(0.0, |a| a.0 as f32),
        15.0,
    ));
    out.push(norm(state.player(player).armor, 30.0));
}

/// 将游戏状态编码为固定长度的观察向量（`player` 的视角）。
#[must_use]
pub fn encode_observation(state: &GameState, player: PlayerId) -> Vec<f32> {
    let world = state.world();
    let enemy = player.opponent();
    let mut obs = Vec::with_capacity(OBS_LEN);

    // 己方英雄 + 法力
    hero_block(state, player, &mut obs);
    let p = state.player(player);
    obs.push(norm(p.mana_crystals, 10.0));
    obs.push(norm(p.current_mana, 10.0));
    // 敌方英雄
    hero_block(state, enemy, &mut obs);
    // 牌库数量
    obs.push(norm(world.zones().len(Zone::Deck, player) as i32, 30.0));
    obs.push(norm(world.zones().len(Zone::Deck, enemy) as i32, 30.0));

    // 手牌（按位置编码，空槽位为 0）
    let hand: Vec<_> = world.zones().iter(Zone::Hand, player).collect();
    for i in 0..MAX_HAND {
        match hand.get(i) {
            Some(&card) => {
                let ct = world.card_type(card).unwrap_or(CardType::Minion);
                obs.push(norm(world.cost(card).map_or(0, |c| c.0), 10.0));
                obs.push(norm(world.attack(card).map_or(0, |a| a.0), 15.0));
                obs.push(norm(world.health(card).map_or(0, |h| h.0), 30.0));
                obs.push(i32::from(ct == CardType::Minion) as f32);
                obs.push(i32::from(ct == CardType::Spell) as f32);
                obs.push(i32::from(ct == CardType::Weapon) as f32);
            }
            None => {
                obs.extend_from_slice(&[0.0; CARD_FEATURES]);
            }
        }
    }

    // 双方战场（按位置编码）
    for side in [player, enemy] {
        let board: Vec<_> = world
            .zones()
            .iter(Zone::Play, side)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .collect();
        for i in 0..MAX_BOARD {
            match board.get(i) {
                Some(&m) => {
                    obs.push(normf(
                        world.effective_attack(m).map_or(0.0, |a| a.0 as f32),
                        15.0,
                    ));
                    obs.push(normf(
                        world.effective_health(m).map_or(0.0, |h| h.0 as f32),
                        30.0,
                    ));
                    obs.push(i32::from(world.taunt(m).is_some()) as f32);
                    obs.push(i32::from(world.divine_shield(m).is_some()) as f32);
                    obs.push(i32::from(world.windfury(m).is_some()) as f32);
                    obs.push(i32::from(world.charge(m).is_some()) as f32);
                    obs.push(i32::from(world.stealth(m).is_some()) as f32);
                }
                None => {
                    obs.extend_from_slice(&[0.0; MINION_FEATURES]);
                }
            }
        }
    }

    debug_assert_eq!(obs.len(), OBS_LEN, "observation length must be fixed");
    obs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::game::GameBuilder;

    #[test]
    fn fresh_state_observation_shape() {
        let state = GameState::new();
        let obs = encode_observation(&state, PlayerId::Player1);
        assert_eq!(obs.len(), OBS_LEN);
        assert_eq!(OBS_LEN, 168);
        // 英雄 30 HP → 1.0，攻击 0 → 0.0
        assert_eq!(obs[0], 1.0);
        assert_eq!(obs[1], 0.0);
        // 法力 0/0
        assert_eq!(obs[3], 0.0);
        assert_eq!(obs[4], 0.0);
        // 敌方英雄 30 HP
        assert_eq!(obs[5], 1.0);
        // 空手牌 / 空战场全为 0
        assert!(obs[10..168].iter().all(|&v| v == 0.0));
    }

    #[test]
    fn board_and_hand_are_encoded_positionally() {
        use crate::cards::def::BLOODFEN_RAPTOR;

        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player2, &BLOODFEN_RAPTOR);
        let state = builder.build();

        let obs = encode_observation(&state, PlayerId::Player1);
        // 手牌槽 0：BLOODFEN_RAPTOR 2 费 3/2 → [0.2, 0.2, 0.0667, 1, 0, 0]
        let hand0 = &obs[10..16];
        assert!((hand0[0] - 0.2).abs() < 1e-5, "cost 2/10: {}", hand0[0]);
        assert!(
            (hand0[1] - 3.0 / 15.0).abs() < 1e-5,
            "atk 3/15: {}",
            hand0[1]
        );
        assert!(
            (hand0[2] - 2.0 / 30.0).abs() < 1e-5,
            "hp 2/30: {}",
            hand0[2]
        );
        assert_eq!(hand0[3], 1.0, "is minion");
        assert_eq!(hand0[4], 0.0);
        assert_eq!(hand0[5], 0.0);

        // 己方战场槽 0（70 起）
        let own0 = &obs[70..77];
        assert!(
            (own0[0] - 3.0 / 15.0).abs() < 1e-5,
            "own board atk: {}",
            own0[0]
        );
        assert!(
            (own0[1] - 2.0 / 30.0).abs() < 1e-5,
            "own board hp: {}",
            own0[1]
        );

        // 敌方战场槽 0（119 起）
        let enemy0 = &obs[119..126];
        assert!(
            (enemy0[0] - 3.0 / 15.0).abs() < 1e-5,
            "enemy board atk: {}",
            enemy0[0]
        );
        assert!(
            (enemy0[1] - 2.0 / 30.0).abs() < 1e-5,
            "enemy board hp: {}",
            enemy0[1]
        );
    }

    #[test]
    fn perspective_changes_hero_order() {
        let state = GameState::new();
        let obs1 = encode_observation(&state, PlayerId::Player1);
        let obs2 = encode_observation(&state, PlayerId::Player2);
        // 视角互换后英雄块互换
        assert_eq!(obs1[0..3], obs2[5..8]);
        assert_eq!(obs1[5..8], obs2[0..3]);
    }
}
