//! Prepare definitions (M5-W1 — Escape from Violet Hold, JAIL_407 Vanessa
//! the Ringleader / JAIL_721 Tras'tath / JAIL_906 Moragg): the id-keyed
//! registry of the Prepare-keyword cards.
//!
//! Pinned semantics (verified from the three card texts and registered in
//! fidelity-debt §27): a Prepare card in hand may be dragged onto the deck
//! — the owner spends ALL remaining mana, the card's cost is permanently
//! reduced by (spent + 1), and the card stays in hand, unplayable (the
//! CantPlayNextTurn marker, which expires at the owner's next turn start).
//! Once per card, once per turn, never with 0 mana. The engine surfaces it
//! as `Action::Prepare` / `Event::PrepareCardExecuted` (validated at the
//! action site; the state changes run in the event handler).

use crate::core::entity::Entity;
use crate::core::state::GameState;

/// Whether the hand card carries the Prepare keyword (the action-validity
/// gate of `Action::Prepare`).
pub(crate) fn is_prepare_card(state: &GameState, card: Entity) -> bool {
    state
        .world()
        .card_id(card)
        .is_some_and(|c| matches!(c.0, "JAIL_407" | "JAIL_721" | "JAIL_906"))
}
