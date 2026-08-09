//! Prepare definitions (M5-W1 + M5-W2 — Escape from Violet Hold): the
//! id-keyed registry of the Prepare-keyword cards. W1: JAIL_407 Vanessa
//! the Ringleader / JAIL_721 Tras'tath / JAIL_906 Moragg. W2 (the
//! closing wave, PR #162): JAIL_321 Tricksy Improviser, JAIL_326
//! Judgment, JAIL_395 Sewer Swimmer, JAIL_435 Rampaging Hound, JAIL_457
//! Hijacked Securitybot, JAIL_718 Black Market Auctioneer, JAIL_735 Code
//! Violet, JAIL_890 Captive Nathrezim, JAIL_909 Defias Wannabe, JAIL_912
//! Soothsayer, JAIL_913 Hold Them Off!, JAIL_998 Defias Smuggler.
//!
//! Pinned semantics (verified from the card texts and registered in
//! fidelity-debt §27/§28): a Prepare card in hand may be dragged onto the
//! deck — the owner spends ALL remaining mana, the card's cost is
//! permanently reduced by (spent + 1), and the card stays in hand,
//! unplayable (the CantPlayNextTurn marker, which expires at the owner's
//! next turn start). Once per card, once per turn, never with 0 mana. The
//! engine surfaces it as `Action::Prepare` / `Event::PrepareCardExecuted`
//! (validated at the action site; the state changes run in the event
//! handler).

use crate::core::entity::Entity;
use crate::core::state::GameState;

/// Whether the hand card carries the Prepare keyword (the action-validity
/// gate of `Action::Prepare`).
pub(crate) fn is_prepare_card(state: &GameState, card: Entity) -> bool {
    state.world().card_id(card).is_some_and(|c| {
        matches!(
            c.0,
            // M5-W1
            "JAIL_407"
                | "JAIL_721"
                | "JAIL_906"
                // M5-W2 — the Violet Hold closing wave
                | "JAIL_321"
                | "JAIL_326"
                | "JAIL_395"
                | "JAIL_435"
                | "JAIL_457"
                | "JAIL_718"
                | "JAIL_735"
                | "JAIL_890"
                | "JAIL_909"
                | "JAIL_912"
                | "JAIL_913"
                | "JAIL_998"
        )
    })
}
