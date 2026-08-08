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
    // Illidari Studies (Core Set W6): the next Outcast card costs less
    // (one-time, consumed on play — cleared after use)
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(crate::cards::def::has_outcast)
        && state.player(player).next_outcast_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_outcast_discount).max(0));
    }
    // Cult Neophyte (Core Set W4b): the opponent's spells cost more
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell) {
        let more = state.player(player).enemy_spell_cost_more;
        if more > 0 {
            cost = Cost(cost.0 + more);
        }
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
    // Glowroot Lure (2025–2026 expansions M1-W4a): costs (1) less for each
    // time the owner used their Hero Power this game
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "EDR_477")
    {
        let uses = state.player(player).hero_power_uses;
        cost = Cost((cost.0 - uses as i32).max(0));
    }
    // Everburning Phoenix (2025–2026 expansions M1-W5): costs (1) less for
    // each card the owner played this turn. The counter is bumped at the
    // CardPlayed path AFTER this discount computes, so the current card is
    // excluded — matching the official discount.
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "FIR_919")
    {
        let played = state.player(player).cards_played_this_turn;
        cost = Cost((cost.0 - played as i32).max(0));
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
    // Naralex, Herald of the Flights (2025–2026 expansions M1-W4b): while he
    // is on the board, the first Dragon the owner plays each turn costs (1)
    // — the per-turn play counter is Player::dragons_played_this_turn
    // (reset at the owner's turn start; incremented at the CardPlayed path).
    if state
        .world()
        .has_race(card, crate::core::component::Race::Dragon)
        && state.player(player).dragons_played_this_turn == 0
        && state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_844"))
    {
        cost = Cost(1);
    }
    // Agamaggan (2025–2026 expansions M1-W4b): the next card the owner plays
    // costs (0) — registered simplification (§14.4, the official effect sets
    // the cost to the opponent's Health). Applied after the Naralex set, so
    // Agamaggan's flag wins.
    if state.player(player).next_card_costs_zero {
        cost = Cost(0);
    }
    // Aviana, Elune's Chosen (2025–2026 expansions M1-W4b): all the owner's
    // cards cost (1) this game (registered simplification §14.4 — the real
    // lunar-cycle timing is approximated as immediate). "(1)" is a SET — the
    // card costs exactly 1 (official text: "Your cards cost (1)"), so a
    // 9-Cost minion is discounted to 1 and a (0) card is raised to 1.
    // Applied LAST so the one-time/set-to-value effects above keep their
    // lower costs only when they land at or below 1.
    if state.player(player).cards_cost_1 {
        cost = Cost(1);
    }
    cost
}
