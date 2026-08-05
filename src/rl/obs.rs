//! Observation space — tensorizes `GameState` into a fixed-length float vector.
//!
//! Observations are encoded from the perspective of the given player (normalized to [0, 1]),
//! with a fixed layout and constant length, so they can be fed directly into a neural network.
//!
//! # Layout (`OBS_LEN` = 168)
//!
//! | Range | Content | Normalization |
//! |------|------|--------|
//! | 0..3   | Friendly hero: health, attack, armor | /30, /15, /30 |
//! | 3..5   | Friendly mana: total crystals, current mana | /10 |
//! | 5..8   | Enemy hero: health, attack, armor | /30, /15, /30 |
//! | 8..10  | Deck counts: friendly, enemy | /30 |
//! | 10..70 | Friendly hand (up to 10 cards): each [cost, attack, health, minion, spell, weapon] | /10, /15, /30, 0/1 |
//! | 70..119 | Friendly board (up to 7 minions): each [attack, health, taunt, divine shield, windfury, charge, stealth] | /15, /30, 0/1 |
//! | 119..168 | Enemy board (same as above, 7 slots) | |

use crate::core::component::CardType;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// Maximum hand size (number of encoding slots)
pub const MAX_HAND: usize = 10;
/// Maximum board minions (number of encoding slots)
pub const MAX_BOARD: usize = 7;
/// Number of features per hand card
pub const CARD_FEATURES: usize = 6;
/// Number of features per minion
pub const MINION_FEATURES: usize = 7;

/// Observation vector length — fixed value, for preallocation on the Python side.
pub const OBS_LEN: usize = 10 + MAX_HAND * CARD_FEATURES + 2 * MAX_BOARD * MINION_FEATURES;

/// Normalization helper: divides the value by the denominator and clamps to [0, 1].
fn norm(value: i32, max: f32) -> f32 {
    (value as f32 / max).clamp(0.0, 1.0)
}

fn normf(value: f32, max: f32) -> f32 {
    (value / max).clamp(0.0, 1.0)
}

/// Encodes the hero observation block for the given player (health, attack, armor).
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

/// Encodes the game state into a fixed-length observation vector (`player`'s perspective).
#[must_use]
pub fn encode_observation(state: &GameState, player: PlayerId) -> Vec<f32> {
    let world = state.world();
    let enemy = player.opponent();
    let mut obs = Vec::with_capacity(OBS_LEN);

    // Friendly hero + mana
    hero_block(state, player, &mut obs);
    let p = state.player(player);
    obs.push(norm(p.mana_crystals, 10.0));
    obs.push(norm(p.current_mana, 10.0));
    // Enemy hero
    hero_block(state, enemy, &mut obs);
    // Deck counts
    obs.push(norm(world.zones().len(Zone::Deck, player) as i32, 30.0));
    obs.push(norm(world.zones().len(Zone::Deck, enemy) as i32, 30.0));

    // Hand (encoded positionally, empty slots are 0)
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

    // Both boards (encoded positionally)
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
        // Hero 30 HP → 1.0, attack 0 → 0.0
        assert_eq!(obs[0], 1.0);
        assert_eq!(obs[1], 0.0);
        // Mana 0/0
        assert_eq!(obs[3], 0.0);
        assert_eq!(obs[4], 0.0);
        // Enemy hero 30 HP
        assert_eq!(obs[5], 1.0);
        // Empty hand / empty board are all 0
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
        // Hand slot 0: BLOODFEN_RAPTOR 2 cost 3/2 → [0.2, 0.2, 0.0667, 1, 0, 0]
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

        // Friendly board slot 0 (starting at 70)
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

        // Enemy board slot 0 (starting at 119)
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
        // After swapping perspectives, the hero blocks are swapped
        assert_eq!(obs1[0..3], obs2[5..8]);
        assert_eq!(obs1[5..8], obs2[0..3]);
    }
}
