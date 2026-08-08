//! Cost manager — the single composition point for card play costs (roadmap G5).
//!
//! The modifier stack lives in `World::effective_cost`: base + enchantment
//! deltas, then set-to-value / floor modifiers, then hand-only aura
//! reductions, floored at 0. This module adds the player-level modifiers that
//! the World cannot see (Kirin Tor Mage's one-time free secret) and is the
//! ONLY place a play cost is composed — validation, mana deduction, and bots
//! all read from here.

use crate::core::component::Cost;
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::GameState;

/// The cost of playing `card` for `player`: the entity cost stack plus
/// player-level modifiers (e.g. Kirin Tor Mage's one-time free secret).
#[must_use]
pub fn play_cost(state: &GameState, card: Entity, player: PlayerId) -> Cost {
    let mut cost = state.world().effective_cost(card).unwrap_or_default();
    // Kirin Tor Mage: the next secret costs 0 (one-time, consumed on play)
    let is_secret = state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.secret.is_some());
    if is_secret && state.player(player).next_secret_free {
        cost = Cost(0);
    }
    // Millhouse Manastorm: the opponent's spells cost 0 this turn
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell)
        && state.player(player).spells_cost_zero
    {
        cost = Cost(0);
    }
    // Preparation (W11): the next spell cast this turn costs `amount` less
    // (one-time — the flag is consumed by the first spell played)
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell) {
        let discount = state.player(player).next_spell_discount;
        if discount > 0 {
            cost = Cost((cost.0 - discount).max(0));
        }
    }
    // Raging Felscreamer (Core Set W4a): the next Demon costs less
    // (one-time, consumed on play — cleared after use)
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.race == Some(crate::core::component::Race::Demon))
        && state.player(player).next_demon_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_demon_discount).max(0));
    }
    // Foxy Fraud (Core Set W4a): the next Combo card costs less this turn
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.combo_effect.is_some())
        && state.player(player).next_combo_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_combo_discount).max(0));
    }
    // Dread Corsair (Core Set W3b): costs (1) less per Attack of the
    // owner's weapon
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CORE_NEW1_022")
    {
        let weapon_atk = state
            .player(player)
            .weapon
            .and_then(|w| state.world().effective_attack(w))
            .map_or(0, |a| a.0);
        cost = Cost((cost.0 - weapon_atk).max(0));
    }
    // Sea Giant (W11): costs (1) less for each minion on the battlefield
    // (both sides — the board-count rule composes here like Dread Corsair)
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "NEUTRAL_026")
    {
        let board_count = [player, player.opponent()]
            .iter()
            .map(|&p| {
                state
                    .world()
                    .zones()
                    .iter(crate::core::zone::Zone::Play, p)
                    .filter(|&e| {
                        state.world().card_type(e) == Some(crate::core::component::CardType::Minion)
                    })
                    .count()
            })
            .sum::<usize>();
        cost = Cost((cost.0 - board_count as i32).max(0));
    }
    // Dread Corsair: costs (1) less per Attack of your weapon
    let weapon_attack = state
        .player(player)
        .weapon
        .and_then(|w| state.world().attack(w))
        .map_or(0, |a| a.0);
    if weapon_attack > 0
        && state
            .world()
            .card_id(card)
            .is_some_and(|c| c.0 == "NEUTRAL_C13")
    {
        cost = Cost((cost.0 - weapon_attack).max(0));
    }
    // Pint-Sized Summoner: while its FirstMinionDiscount aura is on the board
    // (silencing the summoner removes the aura), the FIRST minion played this
    // turn costs `amount` less.
    if state.world().card_type(card) == Some(crate::core::component::CardType::Minion)
        && state.player(player).minions_played_this_turn == 0
    {
        let discount: i32 = state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .filter_map(|e| state.world().aura(e))
            .filter_map(|a| match a.effect {
                crate::core::component::AuraEffect::FirstMinionDiscount { amount } => Some(amount),
                _ => None,
            })
            .sum();
        cost = Cost((cost.0 - discount).max(0));
    }
    cost
}
