//! Structured observation/action views (roadmap M1-G3).
//!
//! Plain-data mirrors of the game state for the Python bindings and RL feature
//! engineering — the 168-dim tensor (`obs.rs`) and the `(index, description)`
//! action list stay, while these views expose the structured fields the
//! feature pipeline needs: entity ids, card ids, keywords, and action kinds.
//!
//! This module is PyO3-free: the pyclass wrappers live in `py_bind/views.rs`
//! and convert from these structs.

use crate::cards::card_by_id;
use crate::core::component::CardType;
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Step};
use crate::core::zone::Zone;
use crate::rl::env::{ActionInfo, legal_action_infos};

/// A hand card, minion, or other entity as seen from one perspective.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityView {
    /// Entity slot index — matches `ActionView::entity_id`/`target_id`
    pub entity_id: u32,
    /// Card ID (empty for generated entities without a CardId)
    pub card_id: String,
    /// Display name (from the card database; empty if unknown)
    pub name: String,
    /// Current (effective) cost
    pub cost: i32,
    /// Current (effective) attack
    pub attack: i32,
    /// Current (effective) health
    pub health: i32,
    /// Whether the entity can attack this turn
    pub can_attack: bool,
    /// Taunt keyword
    pub taunt: bool,
    /// Divine Shield keyword
    pub divine_shield: bool,
    /// Stealth keyword
    pub stealth: bool,
    /// Windfury keyword
    pub windfury: bool,
    /// Charge keyword
    pub charge: bool,
    /// Frozen this turn
    pub frozen: bool,
    /// Hand cards only: affordable with the owner's current mana
    pub playable: bool,
}

/// One player's view: hero, mana, zones, and entities.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerView {
    /// Hero remaining health
    pub hero_health: i32,
    /// Hero armor
    pub hero_armor: i32,
    /// Hero attack (weapon + buffs)
    pub hero_attack: i32,
    /// Remaining mana this turn
    pub remaining_mana: i32,
    /// Total mana crystals
    pub total_mana: i32,
    /// Cards in hand
    pub hand_count: usize,
    /// Cards in deck
    pub deck_count: usize,
    /// Equipped weapon attack (`0` without a weapon)
    pub weapon_attack: i32,
    /// Equipped weapon durability (`0` without a weapon)
    pub weapon_durability: i32,
    /// Whether the hero power can be used this turn (exists, not used, affordable)
    pub hero_power_usable: bool,
    /// Hero power cost (`0` without a hero power)
    pub hero_power_cost: i32,
    /// Minions on the battlefield (left to right)
    pub field: Vec<EntityView>,
    /// Cards in hand (only for the perspective player; the opponent's is empty)
    pub hand: Vec<EntityView>,
}

/// The full structured observation, from the perspective player's side.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    /// Current turn number (1-based)
    pub turn: u32,
    /// Whether it is the perspective player's turn
    pub my_turn: bool,
    /// Whether the game has ended
    pub done: bool,
    /// Winner: `0` = not over / draw, `1` = player 1, `2` = player 2
    pub winner: u8,
    /// Whether the engine is waiting for a choice (mulligan, choose-one, …)
    pub awaiting_choice: bool,
    /// The perspective player
    pub me: PlayerView,
    /// The opponent (hand hidden)
    pub opponent: PlayerView,
}

/// Structured metadata of one legal action (built from [`ActionInfo`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ActionView {
    /// Index into `legal_actions()` / `step(action_index)`
    pub index: usize,
    /// Action kind: `"end_turn" | "play" | "attack" | "hero_power" | "choose"`
    pub kind: &'static str,
    /// Hand index of the card for `play`, else `-1`
    pub card_index: i32,
    /// Entity id: card (`play`), attacker (`attack`), hero (`hero_power`), else `-1`
    pub entity_id: i32,
    /// Target entity id (play target / defender / hero power target), else `-1`
    pub target_id: i32,
    /// Readable description (kept for `play.py`)
    pub description: String,
}

/// Builds an [`EntityView`] for one entity.
#[must_use]
pub fn entity_view(state: &GameState, entity: Entity, is_hand: bool) -> EntityView {
    let world = state.world();
    let card_id = world
        .card_id(entity)
        .map(|c| c.0.to_string())
        .unwrap_or_default();
    let name = world
        .card_id(entity)
        .and_then(|c| card_by_id(c.0))
        .map_or_else(String::new, |c| c.name.to_string());
    let attack = world.effective_attack(entity).map_or(0, |a| a.0);
    let cost = world.effective_cost(entity).map_or(0, |c| c.0);

    EntityView {
        entity_id: entity.index,
        card_id,
        name,
        cost,
        attack,
        health: world.effective_health(entity).map_or(0, |h| h.0),
        can_attack: !is_hand
            && !world.cant_attack(entity).is_some()
            && attack > 0
            && world
                .attacks_used(entity)
                .is_none_or(|u| u.0 < world.max_attacks(entity)),
        taunt: world.taunt(entity).is_some(),
        divine_shield: world.divine_shield(entity).is_some(),
        stealth: world.stealth(entity).is_some(),
        windfury: world.windfury(entity).is_some(),
        charge: world.charge(entity).is_some(),
        frozen: world.freeze(entity).is_some(),
        playable: is_hand
            && world
                .player(entity)
                .is_some_and(|p| cost <= state.player(p).current_mana),
    }
}

/// Builds a [`PlayerView`]; `reveal_hand` is true only for the perspective player.
#[must_use]
pub fn player_view(state: &GameState, player: PlayerId, reveal_hand: bool) -> PlayerView {
    let world = state.world();
    let hero = state.player(player).hero;
    let (weapon_attack, weapon_durability) = match state.player(player).weapon {
        Some(w) => (
            world.effective_attack(w).map_or(0, |a| a.0),
            world.durability(w).map_or(0, |d| d.0),
        ),
        None => (0, 0),
    };
    let hero_power = world.hero_power(hero).map(|hp| hp.cost);

    PlayerView {
        hero_health: world.effective_health(hero).map_or(0, |h| h.0),
        hero_armor: state.player(player).armor,
        hero_attack: world.effective_attack(hero).map_or(0, |a| a.0),
        remaining_mana: state.player(player).current_mana,
        total_mana: state.player(player).mana_crystals,
        hand_count: world.zones().len(Zone::Hand, player),
        deck_count: world.zones().len(Zone::Deck, player),
        weapon_attack,
        weapon_durability,
        hero_power_usable: hero_power.is_some()
            && world.hero_power_used(hero).is_none()
            && hero_power.is_some_and(|c| c <= state.player(player).current_mana),
        hero_power_cost: hero_power.unwrap_or(0),
        field: world
            .zones()
            .iter(Zone::Play, player)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .map(|e| entity_view(state, e, false))
            .collect(),
        hand: if reveal_hand {
            world
                .zones()
                .iter(Zone::Hand, player)
                .map(|e| entity_view(state, e, true))
                .collect()
        } else {
            Vec::new()
        },
    }
}

/// Builds the structured observation from the perspective player's side.
#[must_use]
pub fn observation(state: &GameState, perspective: PlayerId) -> Observation {
    let winner = match state.step() {
        Step::GameOver { winner } => winner.index() as u8 + 1,
        _ => 0,
    };
    Observation {
        turn: state.turn(),
        my_turn: state.active_player() == perspective,
        done: matches!(state.step(), Step::GameOver { .. }),
        winner,
        awaiting_choice: state.pending_choice().is_some(),
        me: player_view(state, perspective, true),
        opponent: player_view(state, perspective.opponent(), false),
    }
}

/// Builds structured action views for the current player's legal actions.
#[must_use]
pub fn action_views(state: &GameState) -> Vec<ActionView> {
    legal_action_infos(state)
        .into_iter()
        .enumerate()
        .map(|(index, info)| action_view(index, &info))
        .collect()
}

/// Converts one [`ActionInfo`] into an [`ActionView`].
#[must_use]
pub fn action_view(index: usize, info: &ActionInfo) -> ActionView {
    ActionView {
        index,
        kind: info.kind,
        card_index: info.card_index,
        entity_id: info.entity_id,
        target_id: info.target_id,
        description: format!("{:?}", info.action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{Attack, AttacksUsed, Freeze};
    use crate::sim::game::GameBuilder;

    #[test]
    fn fresh_state_observation() {
        let state = GameState::new();
        let obs = observation(&state, PlayerId::Player1);
        assert!(obs.my_turn, "player 1 opens");
        assert!(!obs.done);
        assert_eq!(obs.winner, 0);
        assert_eq!(obs.turn, 1);
        assert_eq!(obs.me.hero_health, 30);
        assert_eq!(obs.me.hero_armor, 0);
        assert_eq!(obs.me.total_mana, 1);
        assert_eq!(obs.me.remaining_mana, 1);
        assert_eq!(obs.me.hand_count, 0);
        assert_eq!(obs.me.deck_count, 0);
        assert!(obs.me.field.is_empty());
        assert!(obs.me.hand.is_empty());
        assert!(obs.opponent.hand.is_empty(), "opponent hand is hidden");
        // Perspective swap: heroes swap, `my_turn` flips
        let obs2 = observation(&state, PlayerId::Player2);
        assert!(!obs2.my_turn);
        assert_eq!(obs2.me.hero_health, 30);
    }

    #[test]
    fn entity_view_exposes_components() {
        let mut builder = GameBuilder::new();
        let e = builder.add_custom_minion_to_board(PlayerId::Player1, 4, 5, 2);
        let world = builder.state_mut().world_mut();
        world.set_taunt(e, crate::core::component::Taunt);
        world.set_divine_shield(e, crate::core::component::DivineShield);
        world.set_stealth(e, crate::core::component::Stealth);
        world.set_windfury(e, crate::core::component::Windfury);
        world.set_charge(e, crate::core::component::Charge);
        world.set_freeze(e, Freeze);
        world.set_attack(e, Attack(4));
        world.set_attacks_used(e, AttacksUsed(0));
        let state = builder.build();

        let v = entity_view(&state, e, false);
        assert_eq!(v.attack, 4);
        assert_eq!(v.health, 5);
        assert_eq!(v.cost, 2);
        assert!(v.can_attack, "4-attack minion, 0 attacks used, windfury");
        assert!(v.taunt && v.divine_shield && v.stealth && v.windfury && v.charge && v.frozen);
        assert!(!v.playable, "board minion is not playable");

        // Used up attacks → can_attack false
        let mut builder = GameBuilder::new();
        let e2 = builder.add_custom_minion_to_board(PlayerId::Player1, 2, 2, 1);
        let world = builder.state_mut().world_mut();
        world.set_attacks_used(e2, AttacksUsed(1));
        let state = builder.build();
        let v2 = entity_view(&state, e2, false);
        assert!(!v2.can_attack, "attacks exhausted");
    }

    #[test]
    fn hand_cards_are_playable_by_mana() {
        let mut builder = GameBuilder::new();
        builder.set_mana(PlayerId::Player1, 3, 3);
        let cheap = builder.add_custom_minion_to_hand(PlayerId::Player1, 1, 1, 1);
        let expensive = builder.add_custom_minion_to_hand(PlayerId::Player1, 1, 1, 5);
        let state = builder.build();

        let v_cheap = entity_view(&state, cheap, true);
        let v_expensive = entity_view(&state, expensive, true);
        assert!(v_cheap.playable);
        assert!(!v_expensive.playable, "5-cost card with 3 mana");
    }

    #[test]
    fn game_over_winner_encoding() {
        use crate::core::state::Step;
        let mut builder = GameBuilder::new();
        builder.step(Step::GameOver {
            winner: PlayerId::Player2,
        });
        let state = builder.build();
        let obs = observation(&state, PlayerId::Player1);
        assert!(obs.done);
        assert_eq!(obs.winner, 2, "player 2 wins → winner=2");
    }

    #[test]
    fn action_views_cover_legal_actions() {
        use crate::rl::env::legal_actions;
        let state = GameState::new();
        let views = action_views(&state);
        let plain = legal_actions(&state);
        assert_eq!(views.len(), plain.len());
        assert_eq!(views[0].kind, "end_turn");
        for v in &views {
            assert_eq!(v.index as usize, views[v.index].index);
            assert!(!v.description.is_empty());
        }
    }
}
